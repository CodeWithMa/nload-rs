// SPDX-License-Identifier: GPL-2.0-only

//! Whole-list acquisition of original-byte interface names and paired counters.
//!
//! Owns fresh `/proc/net/dev` reads through EOF and complete snapshot validation.
//! Runtime snapshots do not change the monitor's fixed startup membership.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Counters {
    pub(crate) incoming: u64,
    pub(crate) outgoing: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CounterOutcome {
    Readable(Counters),
    Absent,
    ReadFailed,
}

/// A validated complete snapshot. No partially parsed rows can escape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Snapshot {
    counters: BTreeMap<Vec<u8>, Counters>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SnapshotError(&'static str);

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for SnapshotError {}

#[derive(Debug)]
pub(crate) enum AcquisitionError {
    Read(io::Error),
    Parse(SnapshotError),
}

impl fmt::Display for AcquisitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(f, "proc counter read failed: {error}"),
            Self::Parse(error) => write!(f, "proc counter snapshot invalid: {error}"),
        }
    }
}

impl std::error::Error for AcquisitionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) => Some(error),
            Self::Parse(error) => Some(error),
        }
    }
}

/// The application owns the completion timestamp after this operation returns.
/// On Err, the *entire round* failed, never an empty/partially salvaged snapshot.
pub(crate) trait CounterSource {
    fn acquire(&mut self) -> Result<Snapshot, AcquisitionError>;
}

/// Production counter adapter: one fresh open and bulk traversal per round.
#[derive(Default)]
pub(crate) struct ProcSampler {
    buffer: Vec<u8>,
}

impl CounterSource for ProcSampler {
    fn acquire(&mut self) -> Result<Snapshot, AcquisitionError> {
        self.acquire_with(|| File::open("/proc/net/dev"))
    }
}

impl ProcSampler {
    // Private I/O seam for chunking/failure tests, shared by the real adapter.
    fn acquire_with<R: Read>(
        &mut self,
        open: impl FnOnce() -> io::Result<R>,
    ) -> Result<Snapshot, AcquisitionError> {
        self.buffer.clear();
        let mut reader = open().map_err(AcquisitionError::Read)?;
        reader
            .read_to_end(&mut self.buffer)
            .map_err(AcquisitionError::Read)?;
        Snapshot::parse(&self.buffer).map_err(AcquisitionError::Parse)
    }
}

impl Snapshot {
    /// Validate the two kernel headers, newline-complete rows, exact names and
    /// all 16 unsigned fields. Padding alone is removed; names need not be UTF-8.
    /// Empty discovery (headers only) is valid; duplicates/partial rows are not.
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, SnapshotError> {
        if !bytes.ends_with(b"\n") {
            return Err(SnapshotError("missing final newline (incomplete snapshot)"));
        }
        let mut lines = bytes[..bytes.len() - 1].split(|byte| *byte == b'\n');
        let first = lines.next().ok_or(SnapshotError("missing proc headers"))?;
        let second = lines.next().ok_or(SnapshotError("missing proc headers"))?;
        if !header_matches(first, [b"Inter-", b"Receive", b"Transmit"])
            || !header_matches(
                second,
                [
                    b"face",
                    b"bytes packets errs drop fifo frame compressed multicast",
                    b"bytes packets errs drop fifo colls carrier compressed",
                ],
            )
        {
            return Err(SnapshotError("malformed proc headers"));
        }

        let mut counters = BTreeMap::new();
        for line in lines {
            let colon = line
                .iter()
                .position(|byte| *byte == b':')
                .ok_or(SnapshotError("missing interface delimiter"))?;
            let name = line[..colon].trim_ascii();
            if name.is_empty()
                || name
                    .iter()
                    .any(|byte| byte.is_ascii_whitespace() || *byte == 0)
            {
                return Err(SnapshotError("invalid interface name"));
            }
            let fields: Vec<_> = words(&line[colon + 1..]).collect();
            if fields.len() != 16 {
                return Err(SnapshotError(
                    "expected exactly 16 interface counter fields",
                ));
            }
            let mut values = [0; 16];
            for (value, field) in values.iter_mut().zip(fields) {
                *value = decimal_counter(field)?;
            }
            if counters
                .insert(
                    name.to_vec(),
                    Counters {
                        incoming: values[0],
                        outgoing: values[8],
                    },
                )
                .is_some()
            {
                return Err(SnapshotError("duplicate interface name"));
            }
        }
        Ok(Self { counters })
    }

    pub(crate) fn get(&self, name: &[u8]) -> Option<Counters> {
        self.counters.get(name).copied()
    }

    /// Compute startup membership/selection without consuming the snapshot:
    /// the same counters must supply the monitor's initial baselines.
    /// Call only after successful acquisition; acquisition errors remain distinct.
    pub(crate) fn startup(
        &self,
        requested: Option<&[u8]>,
    ) -> Result<StartupSelection, StartupError> {
        if self.counters.is_empty() {
            return Err(StartupError::EmptyDiscovery);
        }
        let mut names: Vec<_> = self.counters.keys().cloned().collect();
        names.sort_by(|left, right| natural_order(left, right));
        let selected = match requested {
            None => 0,
            Some(name) => names
                .iter()
                .position(|entry| entry == name)
                .ok_or(StartupError::MissingSelection)?,
        };
        Ok(StartupSelection { names, selected })
    }
}

fn words(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes
        .split(u8::is_ascii_whitespace)
        .filter(|word| !word.is_empty())
}

fn header_matches(line: &[u8], expected: [&[u8]; 3]) -> bool {
    let parts: Vec<_> = line.split(|byte| *byte == b'|').collect();
    parts.len() == 3
        && parts
            .iter()
            .zip(expected)
            .all(|(part, expected)| words(part).eq(words(expected)))
}

fn decimal_counter(field: &[u8]) -> Result<u64, SnapshotError> {
    if field.is_empty() || !field.iter().all(u8::is_ascii_digit) {
        return Err(SnapshotError("counter is not ASCII unsigned decimal"));
    }
    field
        .iter()
        .try_fold(0_u64, |value, digit| {
            value.checked_mul(10)?.checked_add(u64::from(digit - b'0'))
        })
        .ok_or(SnapshotError("counter exceeds u64"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StartupSelection {
    /// Naturally sorted, complete fixed membership; retain for the whole run.
    pub(crate) names: Vec<Vec<u8>>,
    /// Zero-based initial selection, always within the nonempty names list.
    pub(crate) selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartupError {
    /// Application diagnostic/status 1, distinct from acquisition failure.
    EmptyDiscovery,
    /// Successful nonempty discovery but missing exact requested bytes: status 2.
    MissingSelection,
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyDiscovery => "no interfaces discovered",
            Self::MissingSelection => "requested interface is absent from startup discovery",
        })
    }
}

impl std::error::Error for StartupError {}

/// Translate one global round for a retained name. Iterating the startup list,
/// rather than snapshot names, ignores additions and retries every original name.
pub(crate) fn counter_outcome(
    round: &Result<Snapshot, AcquisitionError>,
    name: &[u8],
) -> CounterOutcome {
    match round {
        Ok(snapshot) => snapshot
            .get(name)
            .map_or(CounterOutcome::Absent, CounterOutcome::Readable),
        Err(_) => CounterOutcome::ReadFailed,
    }
}

fn natural_order(left: &[u8], right: &[u8]) -> Ordering {
    let (mut a, mut b) = (left, right);
    while let (Some(&x), Some(&y)) = (a.first(), b.first()) {
        let order = if x.is_ascii_digit() && y.is_ascii_digit() {
            let a_len = a.iter().take_while(|byte| byte.is_ascii_digit()).count();
            let b_len = b.iter().take_while(|byte| byte.is_ascii_digit()).count();
            let a_digits = significant_digits(&a[..a_len]);
            let b_digits = significant_digits(&b[..b_len]);
            a = &a[a_len..];
            b = &b[b_len..];
            a_digits
                .len()
                .cmp(&b_digits.len())
                .then_with(|| a_digits.cmp(b_digits))
        } else {
            a = &a[1..];
            b = &b[1..];
            x.cmp(&y)
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    a.len().cmp(&b.len()).then_with(|| left.cmp(right))
}

fn significant_digits(digits: &[u8]) -> &[u8] {
    let zeros = digits.iter().take_while(|byte| **byte == b'0').count();
    &digits[zeros..]
}

/// Scripted counter adapter for deterministic application-loop tests. Exhaustion
/// is a test failure, not an invented runtime outcome; no clock/terminal effects.
#[cfg(test)]
pub(crate) struct ScriptedCounters {
    rounds: std::collections::VecDeque<Result<Snapshot, AcquisitionError>>,
    pub(crate) calls: usize,
}

#[cfg(test)]
impl ScriptedCounters {
    pub(crate) fn new(
        rounds: impl IntoIterator<Item = Result<Snapshot, AcquisitionError>>,
    ) -> Self {
        Self {
            rounds: rounds.into_iter().collect(),
            calls: 0,
        }
    }
}

#[cfg(test)]
impl CounterSource for ScriptedCounters {
    fn acquire(&mut self) -> Result<Snapshot, AcquisitionError> {
        self.calls += 1;
        self.rounds
            .pop_front()
            .expect("unexpected counter acquisition")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const HEADERS: &[u8] = b"Inter-|   Receive                                                |  Transmit\n face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n";

    fn fixture(rows: &[(&[u8], u64, u64)]) -> Vec<u8> {
        let mut bytes = HEADERS.to_vec();
        for (name, incoming, outgoing) in rows {
            bytes.extend_from_slice(b"   ");
            bytes.extend_from_slice(name);
            bytes.extend_from_slice(
                format!(": {incoming} 1 2 3 4 5 6 7 {outgoing} 9 10 11 12 13 14 15\n").as_bytes(),
            );
        }
        bytes
    }

    #[test]
    fn snapshot_preserves_padding_raw_names_and_exact_paired_fields() {
        let bytes = fixture(&[
            (b"lo", 0, u64::MAX),
            (b"e\xff2", u64::MAX, 17),
            (b"ETH0", 3, 4),
        ]);
        let snapshot = Snapshot::parse(&bytes).unwrap();
        assert_eq!(
            snapshot.get(b"lo"),
            Some(Counters {
                incoming: 0,
                outgoing: u64::MAX
            })
        );
        assert_eq!(
            snapshot.get(b"e\xff2"),
            Some(Counters {
                incoming: u64::MAX,
                outgoing: 17
            })
        );
        assert_eq!(
            snapshot.get(b"ETH0"),
            Some(Counters {
                incoming: 3,
                outgoing: 4
            })
        );
        assert_eq!(snapshot.get(b"eth0"), None);
        assert_eq!(snapshot.get(b"e\\xFF2"), None);
        let padded = [
            HEADERS,
            b"\t eth0 \t: \t0001 1 2 3 4 5 6 7 00002 9 10 11 12 13 14 15 \t\n",
        ]
        .concat();
        assert_eq!(
            Snapshot::parse(&padded).unwrap().get(b"eth0"),
            Some(Counters {
                incoming: 1,
                outgoing: 2
            })
        );
    }

    struct Chunked {
        cursor: Cursor<Vec<u8>>,
        chunk: usize,
        interrupted: bool,
        reads: usize,
        eof: bool,
    }

    impl Read for Chunked {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.reads += 1;
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            let limit = buffer.len().min(self.chunk);
            let count = self.cursor.read(&mut buffer[..limit])?;
            self.eof |= count == 0;
            Ok(count)
        }
    }

    #[test]
    fn acquisition_reads_through_eof_and_opens_fresh_each_round() {
        let names: Vec<_> = (0..3000)
            .map(|index| format!("eth{index}").into_bytes())
            .collect();
        let rows: Vec<_> = names
            .iter()
            .map(|name| (name.as_slice(), 123, 456))
            .collect();
        let bytes = fixture(&rows);
        assert!(bytes.len() > 128 * 1024);
        let mut sampler = ProcSampler::default();
        let mut opens = 0;
        for (round_bytes, expected) in [(&bytes, 3000), (&fixture(&[(b"only", 8, 9)]), 1)] {
            let mut reader = Chunked {
                cursor: Cursor::new(round_bytes.clone()),
                chunk: 7,
                interrupted: false,
                reads: 0,
                eof: false,
            };
            let snapshot = sampler
                .acquire_with(|| {
                    opens += 1;
                    Ok(&mut reader)
                })
                .unwrap();
            assert!(reader.eof);
            assert!(reader.reads > 2);
            assert_eq!(snapshot.startup(None).unwrap().names.len(), expected);
            if expected == 1 {
                assert_eq!(
                    snapshot.get(b"only"),
                    Some(Counters {
                        incoming: 8,
                        outgoing: 9
                    })
                );
                assert_eq!(snapshot.get(b"eth0"), None); // no previous buffer/EOF reuse
            }
        }
        assert_eq!(opens, 2);
    }

    #[test]
    fn rejects_malformed_headers_rows_fields_overflow_and_duplicates() {
        let valid = fixture(&[(b"good", 1, 2)]);
        let invalid_rows: &[&[u8]] = &[
            b"\n",
            b"eth0 1 2\n",
            b": 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            b"bad name: 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            b"bad\0name: 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            b"eth0: 1 2 3 4 5 6 7 8 9\n", // paired fields alone are insufficient
            b"eth0: 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            b"eth0: -1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            b"eth0: 0 0 0 0 0 0 0 0 +1 0 0 0 0 0 0 0\n",
            b"eth0: 0 0 0 0 0 0 0 0 1.5 0 0 0 0 0 0 0\n",
            b"eth0: 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 nope\n",
            b"eth0: 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 \xff\n",
            b"eth0: 18446744073709551616 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            b"eth0: 0 0 0 0 0 0 0 0 18446744073709551616 0 0 0 0 0 0 0\n",
            b"eth0: 0 9999999999999999999999999999999 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n",
            b"eth0: 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0: \n",
        ];
        for row in invalid_rows {
            let bad = [valid.as_slice(), row].concat();
            assert!(Snapshot::parse(&bad).is_err(), "salvaged row {row:?}");
        }
        assert!(Snapshot::parse(&fixture(&[(b"same", 1, 2), (b"same", 3, 4)])).is_err());
        for bad in [
            b"".as_slice(),
            b"\n",
            b"Inter-|Receive|Transmit\n",
            b"wrong\nheaders\n",
            b"Inter-|Receive|Transmit|extra\nface|bytes|bytes\n",
        ] {
            assert!(Snapshot::parse(bad).is_err());
        }
        for end in [valid.len() - 1, valid.len() - 10, HEADERS.len() - 1] {
            assert!(Snapshot::parse(&valid[..end]).is_err());
        }
        let reordered = String::from_utf8(HEADERS.to_vec()).unwrap().replacen(
            "bytes    packets",
            "packets bytes",
            1,
        );
        assert!(Snapshot::parse(reordered.as_bytes()).is_err());
    }

    #[test]
    fn failed_acquisition_is_global_not_absence_and_retries_cleanly() {
        struct FailsAfterData(Cursor<Vec<u8>>);
        impl Read for FailsAfterData {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                let count = self.0.read(buffer)?;
                if count == 0 {
                    Err(io::Error::other("scripted read failure"))
                } else {
                    Ok(count)
                }
            }
        }
        let mut sampler = ProcSampler::default();
        let valid = fixture(&[(b"A", 1, 2)]);
        let absent = sampler.acquire_with(|| Ok(Cursor::new(valid.clone())));
        assert_eq!(counter_outcome(&absent, b"B"), CounterOutcome::Absent);
        assert_eq!(
            counter_outcome(&absent, b"A"),
            CounterOutcome::Readable(Counters {
                incoming: 1,
                outgoing: 2
            })
        );
        let open_failure = sampler.acquire_with(|| {
            Err::<Cursor<Vec<u8>>, _>(io::Error::from(io::ErrorKind::PermissionDenied))
        });
        let read_failure = sampler.acquire_with(|| Ok(FailsAfterData(Cursor::new(valid.clone()))));
        let parse_failure =
            sampler.acquire_with(|| Ok(Cursor::new([valid.as_slice(), b"bad row\n"].concat())));
        assert!(matches!(open_failure, Err(AcquisitionError::Read(_))));
        assert!(matches!(read_failure, Err(AcquisitionError::Read(_))));
        assert!(matches!(parse_failure, Err(AcquisitionError::Parse(_))));
        for failed in [&open_failure, &read_failure, &parse_failure] {
            for name in [b"A", b"B"] {
                assert_eq!(counter_outcome(failed, name), CounterOutcome::ReadFailed);
            }
        }
        let retry = sampler.acquire_with(|| Ok(Cursor::new(fixture(&[(b"B", 5, 6)]))));
        assert_eq!(counter_outcome(&retry, b"A"), CounterOutcome::Absent);
        assert_eq!(
            counter_outcome(&retry, b"B"),
            CounterOutcome::Readable(Counters {
                incoming: 5,
                outgoing: 6
            })
        );
    }

    #[test]
    fn startup_sorts_all_names_and_selects_exact_original_bytes() {
        let rows = [
            (b"eth10".as_slice(), 0, 0),
            (b"eth2", 0, 0),
            (b"eth02", 0, 0),
            (b"eth1", 0, 0),
            (b"eth3", 0, 0),
            (b"lo", 0, 0),
            (b"inactive", 0, 0),
            (b"Eth2", 0, 0),
            (b"e\xff1", 0, 0),
            (b"e\xfe2", 0, 0),
            (b"-first", 0, 0),
        ];
        let snapshot = Snapshot::parse(&fixture(&rows)).unwrap();
        let expected = [
            b"-first".as_slice(),
            b"Eth2",
            b"eth1",
            b"eth02",
            b"eth2",
            b"eth3",
            b"eth10",
            b"e\xfe2",
            b"e\xff1",
            b"inactive",
            b"lo",
        ]
        .map(<[u8]>::to_vec)
        .to_vec();
        let default = snapshot.startup(None).unwrap();
        assert_eq!(default.names, expected);
        assert_eq!(default.selected, 0);
        for (index, name) in expected.iter().enumerate() {
            let selection = snapshot.startup(Some(name)).unwrap();
            assert_eq!(selection.names, expected);
            assert_eq!(selection.selected, index);
            assert!(snapshot.get(name).is_some()); // initial baselines remain available
        }
        for missing in [b"ETH2".as_slice(), b"missing", b"e\\xFF1", b""] {
            assert_eq!(
                snapshot.startup(Some(missing)),
                Err(StartupError::MissingSelection)
            );
        }
        let empty = Snapshot::parse(HEADERS).unwrap();
        assert_eq!(empty.startup(None), Err(StartupError::EmptyDiscovery));
        assert_eq!(
            empty.startup(Some(b"missing")),
            Err(StartupError::EmptyDiscovery)
        );
        assert_ne!(
            StartupError::EmptyDiscovery.to_string(),
            StartupError::MissingSelection.to_string()
        );
    }

    #[test]
    fn natural_order_handles_unbounded_runs_and_full_byte_ties() {
        let large = format!("eth1{}", "0".repeat(1000)).into_bytes();
        let smaller = format!("eth{}", "9".repeat(1000)).into_bytes();
        let padded = format!("eth{}2", "0".repeat(1500)).into_bytes();
        let names = [
            b"eth2".to_vec(),
            large.clone(),
            b"eth02a10".to_vec(),
            b"eth2a2".to_vec(),
            b"eth00".to_vec(),
            smaller.clone(),
            padded.clone(),
            b"eth0".to_vec(),
            b"eth000".to_vec(),
            b"eth".to_vec(),
            b"eth02a2".to_vec(),
            b"eth2a10".to_vec(),
        ];
        let rows: Vec<_> = names.iter().map(|name| (name.as_slice(), 0, 0)).collect();
        assert_eq!(
            Snapshot::parse(&fixture(&rows))
                .unwrap()
                .startup(None)
                .unwrap()
                .names,
            vec![
                b"eth".to_vec(),
                b"eth0".to_vec(),
                b"eth00".to_vec(),
                b"eth000".to_vec(),
                padded,
                b"eth2".to_vec(),
                b"eth02a2".to_vec(),
                b"eth2a2".to_vec(),
                b"eth02a10".to_vec(),
                b"eth2a10".to_vec(),
                smaller,
                large
            ]
        );
        // Later natural differences take precedence over earlier zero padding,
        // before the complete-byte tie-break is applied.
    }

    #[test]
    fn os_invocation_to_startup_preserves_identity_and_full_membership() {
        use crate::arguments::{self, Invocation};
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let snapshot = Snapshot::parse(&fixture(&[
            (b"e\xff2", 1, 2),
            (b"-eth0", 3, 4),
            (b"lo", 0, 0),
        ]))
        .unwrap();
        for name in [b"e\xff2".as_slice(), b"-eth0"] {
            let Invocation::Run(config) = arguments::parse([
                OsString::from("-i"),
                OsString::from("20480"),
                OsString::from("--"),
                OsString::from_vec(name.to_vec()),
            ])
            .unwrap() else {
                panic!("expected run");
            };
            let startup = snapshot.startup(config.interface.as_deref()).unwrap();
            assert_eq!(startup.names[startup.selected], name);
            assert_eq!(
                startup.names,
                [b"-eth0".to_vec(), b"e\xff2".to_vec(), b"lo".to_vec()]
            );
            assert_eq!(config.ceilings.incoming, 2_621_440);
            assert_eq!(config.ceilings.outgoing, 1_310_720);
        }
    }

    #[test]
    fn retained_name_rounds_ignore_additions_and_recognize_raw_name_return() {
        let raw = b"A\xff".as_slice();
        let initial = Snapshot::parse(&fixture(&[(raw, 1, 2), (b"B", 3, 4)])).unwrap();
        let membership = initial.startup(Some(raw)).unwrap();
        let removed = Snapshot::parse(&fixture(&[
            (b"renamed", 10, 20),
            (b"B", 5, 6),
            (b"C", 1, 1),
        ]))
        .unwrap();
        let returned = Snapshot::parse(&fixture(&[(raw, 100, 200), (b"B", 7, 8)])).unwrap();
        let mut source = ScriptedCounters::new([Ok(initial), Ok(removed), Ok(returned)]);
        let baseline = source.acquire();
        assert_eq!(
            counter_outcome(&baseline, raw),
            CounterOutcome::Readable(Counters {
                incoming: 1,
                outgoing: 2
            })
        );
        let round = source.acquire();
        let outcomes: Vec<_> = membership
            .names
            .iter()
            .map(|name| counter_outcome(&round, name))
            .collect();
        assert_eq!(
            outcomes,
            [
                CounterOutcome::Absent,
                CounterOutcome::Readable(Counters {
                    incoming: 5,
                    outgoing: 6
                })
            ]
        );
        let round = source.acquire();
        assert_eq!(
            counter_outcome(&round, raw),
            CounterOutcome::Readable(Counters {
                incoming: 100,
                outgoing: 200
            })
        );
        assert_eq!(membership.names, [raw.to_vec(), b"B".to_vec()]);
        assert_eq!(membership.selected, 0);
        assert_eq!(source.calls, 3);
        // Monitor tests, rather than acquisition, establish the new-run reset.
    }
}
