// SPDX-License-Identifier: GPL-2.0-only

use super::*;
use crate::monitor::ViewState;
use crate::proc_sampler::{AcquisitionError, Snapshot};
use crate::terminal::scripted::{Call, ScriptClock, ScriptedEffects};
use crate::terminal::scripted_session::ScriptSession;
use crate::terminal::{HOOK_LOCK, Notifications};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::os::unix::ffi::OsStringExt;
use std::rc::Rc;

const SECOND: u64 = 1_000_000_000;
type Log = Rc<RefCell<Vec<&'static str>>>;

fn snapshot(entries: &[(&[u8], u64)]) -> Snapshot {
    let mut bytes = b"Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n".to_vec();
    for (name, total) in entries {
        bytes.extend_from_slice(name);
        bytes.extend_from_slice(
            format!(": {total} 0 0 0 0 0 0 0 {total} 0 0 0 0 0 0 0\n").as_bytes(),
        );
    }
    Snapshot::parse(&bytes).unwrap()
}

struct Counters {
    rounds: VecDeque<Result<Snapshot, AcquisitionError>>,
    durations: VecDeque<u64>,
    clock: ScriptClock,
    log: Log,
    calls: usize,
    panic_at: Option<usize>,
    on_acquire: Option<Box<dyn FnMut(usize)>>,
}

impl CounterSource for Counters {
    fn acquire(&mut self) -> Result<Snapshot, AcquisitionError> {
        self.log.borrow_mut().push("acquire");
        self.calls += 1;
        assert_ne!(self.panic_at, Some(self.calls), "counter panic");
        self.clock.0.borrow_mut().time += self.durations.pop_front().unwrap_or(0);
        if let Some(callback) = &mut self.on_acquire {
            callback(self.calls);
        }
        self.rounds.pop_front().expect("unexpected acquisition")
    }
}

#[derive(Clone)]
struct TracedClock {
    clock: ScriptClock,
    log: Log,
}
impl Clock for TracedClock {
    fn now(&mut self) -> io::Result<u64> {
        self.log.borrow_mut().push("clock-now");
        self.clock.now()
    }
}

struct Tick {
    at: u64,
    ready: Ready,
    input: Vec<u8>,
    notifications: Notifications,
    after_read: Option<u64>,
    dimensions: Option<(usize, usize)>,
}

impl Tick {
    fn input(at: u64, bytes: &[u8]) -> Self {
        Self {
            at,
            ready: Ready {
                input: true,
                output: true,
                ..Ready::default()
            },
            input: bytes.to_vec(),
            notifications: Notifications::default(),
            after_read: None,
            dimensions: None,
        }
    }
    fn signal(at: u64, shutdown: Shutdown) -> Self {
        Self {
            notifications: Notifications {
                shutdown: Some(shutdown),
                resized: true,
            },
            ready: Ready {
                signals: true,
                timer: true,
                input: true,
                output: true,
            },
            ..Self::input(at, b"\x1b[C")
        }
    }
}

struct Effects {
    base: ScriptedEffects,
    clock: TracedClock,
    ticks: VecDeque<Tick>,
    pending: Notifications,
    after_read: Option<u64>,
    log: Log,
}

impl Clock for Effects {
    fn now(&mut self) -> io::Result<u64> {
        self.clock.now()
    }
}

impl RuntimeEffects for Effects {
    fn wait(&mut self, deadline: u64, idle: Option<u64>, output: bool) -> io::Result<Ready> {
        self.log.borrow_mut().push("wait");
        self.base.wait(deadline, idle, output)?;
        let tick = self
            .ticks
            .pop_front()
            .ok_or_else(|| io::Error::other("script exhausted"))?;
        self.base.clock.0.borrow_mut().time = tick.at;
        self.pending = tick.notifications;
        self.after_read = tick.after_read;
        if tick.ready.input {
            self.base
                .script
                .borrow_mut()
                .reads
                .push_back(Ok(tick.input));
        }
        if let Some(dimensions) = tick.dimensions {
            self.base
                .script
                .borrow_mut()
                .dimensions
                .push_back(Ok(dimensions));
        }
        Ok(tick.ready)
    }
    fn notifications(&mut self) -> io::Result<Notifications> {
        self.log.borrow_mut().push("notifications");
        let scripted = self.base.notifications()?;
        let pending = Notifications {
            shutdown: self.pending.shutdown.or(scripted.shutdown),
            resized: std::mem::take(&mut self.pending.resized) || scripted.resized,
        };
        Ok(pending)
    }
    fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        self.log.borrow_mut().push("resize");
        self.base.dimensions()
    }
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.log.borrow_mut().push("read");
        let result = self.base.read(bytes);
        if let Some(time) = self.after_read.take() {
            self.base.clock.0.borrow_mut().time = time;
        }
        result
    }
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.log.borrow_mut().push("frame-write");
        self.base.write(bytes)
    }
}

#[derive(Debug)]
struct View {
    name: Vec<u8>,
    count: usize,
    state: ViewState,
    total: Option<u64>,
    samples: Vec<(u64, f64)>,
    ceilings: (u64, u64),
}

struct Harness {
    counters: Counters,
    session: ScriptSession,
    effects: Effects,
    clock: TracedClock,
    init_error: bool,
    view: Option<View>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Harness {
    fn new(entries: &[(&[u8], u64)], ticks: Vec<Tick>) -> Self {
        let clock = ScriptClock::default();
        let log = Log::default();
        let traced = TracedClock {
            clock: clock.clone(),
            log: log.clone(),
        };
        let session = ScriptSession {
            clock: clock.clone(),
            ..ScriptSession::default()
        };
        session.state.borrow_mut().log = Some(log.clone());
        Self {
            counters: Counters {
                rounds: VecDeque::from([Ok(snapshot(entries))]),
                durations: VecDeque::new(),
                clock: clock.clone(),
                log: log.clone(),
                calls: 0,
                panic_at: None,
                on_acquire: None,
            },
            session,
            effects: Effects {
                base: ScriptedEffects {
                    clock,
                    ..ScriptedEffects::default()
                },
                clock: traced.clone(),
                ticks: ticks.into(),
                pending: Notifications::default(),
                after_read: None,
                log,
            },
            clock: traced,
            init_error: false,
            view: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    fn run(&mut self, args: &[OsString]) -> u8 {
        let _lock = HOOK_LOCK.lock().unwrap();
        invoke(
            args.iter().cloned(),
            &mut self.stdout,
            &mut AfterCleanupWriter {
                bytes: &mut self.stderr,
                session: self.session.clone(),
            },
            |config| {
                let mut session = Session::new(self.session.clone());
                interactive(
                    config,
                    &mut self.counters,
                    &mut session,
                    || {
                        self.clock.log.borrow_mut().push("clock-init");
                        if self.init_error {
                            return Err(io::Error::other("clock init failure"));
                        }
                        let mut clock = self.clock.clone();
                        clock.now()?;
                        Ok(clock)
                    },
                    |_, _, monitor, counters, deadline, dimensions| {
                        let effects = Effects {
                            base: self.effects.base.clone(),
                            clock: self.effects.clock.clone(),
                            ticks: std::mem::take(&mut self.effects.ticks),
                            pending: self.effects.pending,
                            after_read: None,
                            log: self.effects.log.clone(),
                        };
                        let result = drive(
                            monitor,
                            counters,
                            &mut Runtime::new(effects),
                            deadline,
                            dimensions,
                        );
                        let view = monitor.selected_view();
                        self.view = Some(View {
                            name: view.name.to_vec(),
                            count: view.count,
                            state: view.state,
                            total: view.incoming.total,
                            samples: view
                                .incoming
                                .samples
                                .iter()
                                .map(|sample| (sample.at, sample.rate))
                                .collect(),
                            ceilings: (view.incoming.ceiling, view.outgoing.ceiling),
                        });
                        result.map_err(Failure::from)
                    },
                )
            },
        )
    }

    fn plain(&mut self) -> u8 {
        self.run(&[])
    }
    fn emitted(&self) -> String {
        String::from_utf8(self.effects.base.script.borrow().emitted.clone()).unwrap()
    }
    fn waits(&self) -> Vec<(u64, Option<u64>, bool)> {
        self.effects
            .base
            .script
            .borrow()
            .calls
            .iter()
            .filter_map(|call| match call {
                Call::Wait {
                    deadline,
                    idle_ns,
                    output,
                } => Some((*deadline, *idle_ns, *output)),
                _ => None,
            })
            .collect()
    }
    fn restored(&self) {
        assert!(self.session.state.borrow().calls.ends_with(&[
            "show",
            "leave",
            "restore-termios",
            "restore-input",
            "restore-output",
            "release"
        ]));
    }
}

struct AfterCleanupWriter<'a> {
    bytes: &'a mut Vec<u8>,
    session: ScriptSession,
}

impl Write for AfterCleanupWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let state = self.session.state.borrow();
        if state.calls.contains(&"prepare") {
            assert_eq!(
                state.calls.last(),
                Some(&"release"),
                "diagnostic emitted before exhaustive cleanup"
            );
        }
        self.bytes.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn args(tokens: &[&str]) -> Vec<OsString> {
    tokens.iter().map(OsString::from).collect()
}

#[test]
fn informational_and_syntax_paths_have_correct_streams_status_and_zero_effects() {
    for (tokens, status) in [
        (vec!["--version"], 0),
        (vec!["--help"], 0),
        (vec!["--unknown"], 2),
        (vec!["-h"], 2),
        (vec!["-V"], 2),
        (vec!["a", "b"], 2),
        (vec!["--help", "--version"], 2),
        (vec!["--help", "a"], 2),
        (vec!["--version", "a"], 2),
        (vec!["--help", "-i", "10"], 2),
        (vec!["--version", "-o", "10"], 2),
        (vec!["-i"], 2),
        (vec!["-i", ""], 2),
        (vec!["-i", "+1"], 2),
        (vec!["-i", "-1"], 2),
        (vec!["-i", "1.2"], 2),
        (vec!["-i", " 1"], 2),
        (vec!["-i", "1k"], 2),
        (vec!["-i20480"], 2),
        (vec!["-i=20480"], 2),
        (vec!["-o", "144115188075855872"], 2),
        (vec!["-o", "999999999999999999999999999"], 2),
    ] {
        let mut harness = Harness::new(&[], vec![]);
        assert_eq!(harness.run(&args(&tokens)), status);
        assert_eq!(harness.counters.calls, 0);
        assert!(harness.clock.log.borrow().is_empty());
        assert!(harness.session.state.borrow().calls.is_empty());
        if status == 2 {
            assert!(harness.stdout.is_empty());
            assert!(!harness.stderr.is_empty());
        } else {
            assert!(harness.stderr.is_empty());
            if tokens == ["--version"] {
                assert_eq!(harness.stdout, arguments::version_text().as_bytes());
            } else {
                assert_eq!(harness.stdout, arguments::help_text().as_bytes());
            }
        }
    }
}

#[test]
fn startup_errors_stop_before_terminal_validation_and_remain_distinct() {
    for variant in 0..6 {
        let mut h = Harness::new(&[(b"eth0", 10)], vec![]);
        let tokens = if variant == 0 {
            args(&["ETH0"])
        } else {
            vec![]
        };
        match variant {
            1 => h.counters.rounds = VecDeque::from([Ok(snapshot(&[]))]),
            2 => {
                h.counters.rounds = VecDeque::from([Err(AcquisitionError::Read(io::Error::other(
                    "read failure",
                )))])
            }
            3 => {
                h.counters.rounds = VecDeque::from([Err(AcquisitionError::Parse(
                    Snapshot::parse(b"bad\n").unwrap_err(),
                ))])
            }
            4 => h.init_error = true,
            5 => {
                h.clock.clock.0.borrow_mut().readings =
                    VecDeque::from([Ok(0), Err(io::Error::other("completion clock failure"))])
            }
            _ => {}
        }
        assert_eq!(h.run(&tokens), if variant == 0 { 2 } else { 1 });
        assert!(h.stdout.is_empty());
        let error = String::from_utf8(h.stderr).unwrap();
        assert!(error.contains(
            [
                "requested interface",
                "no interfaces",
                "enumeration",
                "enumeration",
                "clock init",
                "completion clock"
            ][variant]
        ));
        assert!(h.session.state.borrow().calls.is_empty());
    }
}

#[test]
fn startup_reuses_discovery_and_orders_clock_before_terminal_mutation() {
    let mut h = Harness::new(
        &[(b"eth10", 100), (b"eth2", 20), (b"eth02", 2)],
        vec![Tick::input(10, b"q")],
    );
    h.counters.durations.push_back(7);
    assert_eq!(h.plain(), 0);
    assert_eq!(
        &h.clock.log.borrow()[..4],
        ["clock-init", "clock-now", "acquire", "clock-now"]
    );
    assert_eq!(
        &h.session.state.borrow().calls[..9],
        [
            "support",
            "snapshot",
            "prepare",
            "raw",
            "input",
            "output",
            "dimensions",
            "enter",
            "hide"
        ]
    );
    assert_eq!(h.counters.calls, 1);
    assert_eq!(
        &h.clock.log.borrow()[..13],
        [
            "clock-init",
            "clock-now",
            "acquire",
            "clock-now",
            "support",
            "snapshot",
            "prepare",
            "raw",
            "input",
            "output",
            "dimensions",
            "enter",
            "hide"
        ]
    );
    let view = h.view.as_ref().unwrap();
    assert_eq!(view.name, b"eth02");
    assert_eq!(view.count, 3);
    assert_eq!(view.state, ViewState::Baseline);
    assert_eq!(view.total, Some(2));
    assert!(view.samples.is_empty());
    assert_eq!(h.waits()[0].0, 7 + SAMPLE_NS);
    h.restored();
    assert!(h.stderr.is_empty());
}

#[test]
fn raw_selection_and_ceiling_options_reach_the_fixed_monitor() {
    for tokens in [
        vec![
            OsString::from_vec(vec![0xff]),
            "-i".into(),
            "20480".into(),
            "-o".into(),
            "5120".into(),
        ],
        vec![
            "-i".into(),
            "20480".into(),
            "-o".into(),
            "5120".into(),
            OsString::from_vec(vec![0xff]),
        ],
    ] {
        let mut h = Harness::new(
            &[(b"-name", 1), (&[0xff], 2)],
            vec![Tick::input(1, b"\x1b[C"), Tick::input(2, b"q")],
        );
        assert_eq!(h.run(&tokens), 0);
        let view = h.view.unwrap();
        assert_eq!(view.name, b"-name");
        assert_eq!(view.count, 2);
        assert_eq!(view.ceilings, (2_621_440, 655_360));
        assert_eq!(h.counters.calls, 1);
    }
    let mut h = Harness::new(&[(b"-name", 1), (b"other", 2)], vec![Tick::input(1, b"Q")]);
    assert_eq!(
        h.run(&args(&[
            "-i", "10", "-i", "00020", "-o", "5120", "-i", "0", "--", "-name"
        ])),
        0
    );
    assert_eq!(
        h.view.unwrap().ceilings,
        (arguments::DEFAULT_CEILING_BYTES, 655_360)
    );
    for value in ["1", "144115188075855871"] {
        let mut h = Harness::new(&[(b"a", 0)], vec![Tick::input(1, b"q")]);
        assert_eq!(h.run(&args(&["-i", value])), 0);
        assert_eq!(
            h.view.unwrap().ceilings.0,
            value.parse::<u64>().unwrap() * 128
        );
    }
}

#[test]
fn slow_initialization_and_slow_acquisition_skip_slots_use_completion_and_actual_time() {
    let mut h = Harness::new(&[(b"a", 0)], vec![Tick::input(3 * SECOND + 1, b"q")]);
    h.session.state.borrow_mut().init_delay = 2 * SECOND;
    h.counters.rounds.push_back(Ok(snapshot(&[(b"a", 3000)])));
    h.counters.durations = VecDeque::from([0, SECOND]);
    assert_eq!(h.plain(), 0);
    assert_eq!(h.counters.calls, 2);
    assert_eq!(h.waits()[0].0, 3 * SECOND + SAMPLE_NS);
    assert_eq!(h.view.unwrap().samples, [(3 * SECOND, 1000.0)]);
}

#[test]
fn navigation_resize_and_parser_timeout_redraw_without_acquisition() {
    let mut resize = Tick::input(50_000_000, b"\x1b[C\x1b[B\x1b[D\x1b[A\x1bOD");
    resize.notifications.resized = true;
    resize.dimensions = Some((10, 2));
    let timeout = Tick {
        ready: Ready {
            output: true,
            ..Ready::default()
        },
        ..Tick::input(150_000_000, b"")
    };
    let mut grow = Tick::input(160_000_000, b"\x1bOB\t\n\x1b[5~\x1b[6~\x1b[1;2C\x1a");
    grow.notifications.resized = true;
    grow.dimensions = Some((80, 24));
    let mut h = Harness::new(
        &[(b"a", 10), (b"b", 20), (b"c", 30)],
        vec![
            resize,
            Tick::input(50_000_000, b"\x1b["),
            timeout,
            grow,
            Tick::input(170_000_000, b"q"),
        ],
    );
    h.session.state.borrow_mut().dimensions = Some((1, 1));
    assert_eq!(h.plain(), 0);
    assert_eq!(h.counters.calls, 1);
    assert!(h.view.as_ref().unwrap().samples.is_empty());
    assert_eq!(h.view.as_ref().unwrap().name, b"a");
    assert!(h.emitted().contains("Interface a (1/3)"));
    assert!(
        h.waits()
            .iter()
            .any(|(_, idle, _)| *idle == Some(100_000_000))
    );
}

#[test]
fn sustained_bounded_input_rechecks_due_acquisition_before_next_batch() {
    let mut first = Tick::input(100_000_000, &b"\x1b[C".repeat(85));
    first.after_read = Some(SAMPLE_NS);
    let mut h = Harness::new(
        &[(b"a", 0), (b"b", 0), (b"c", 0)],
        vec![
            first,
            Tick::input(SECOND, &b"\x1b[D".repeat(85)),
            Tick::input(SECOND + 1, b"q"),
        ],
    );
    h.counters.rounds.extend([
        Ok(snapshot(&[(b"a", 50), (b"b", 100), (b"c", 150)])),
        Ok(snapshot(&[(b"a", 100), (b"b", 200), (b"c", 300)])),
    ]);
    assert_eq!(h.plain(), 0);
    assert_eq!(h.counters.calls, 3);
    let log = h.clock.log.borrow();
    let first_read = log.iter().position(|call| *call == "read").unwrap();
    let sample = log
        .iter()
        .enumerate()
        .skip(first_read + 1)
        .find(|(_, call)| **call == "acquire")
        .unwrap()
        .0;
    let second_read = log
        .iter()
        .enumerate()
        .skip(first_read + 1)
        .find(|(_, call)| **call == "read")
        .unwrap()
        .0;
    assert!(sample < second_read);
    assert_eq!(h.view.as_ref().unwrap().samples.len(), 2);
    assert!(
        h.effects
            .base
            .script
            .borrow()
            .calls
            .iter()
            .filter_map(|call| match call {
                Call::Read(size) => Some(size),
                _ => None,
            })
            .all(|size| *size == 256)
    );
}

#[test]
fn shutdown_preempts_due_sampling_input_resize_and_rendering() {
    for shutdown in [Shutdown::Interrupt, Shutdown::Terminate] {
        let mut h = Harness::new(&[(b"a", 0)], vec![]);
        h.effects.pending = Notifications {
            shutdown: Some(shutdown),
            resized: true,
        };
        h.session.state.borrow_mut().init_delay = SECOND;
        assert_eq!(h.plain(), shutdown.status());
        assert_eq!(h.counters.calls, 1);
        assert!(h.effects.base.script.borrow().emitted.is_empty());
        assert!(
            h.effects
                .base
                .script
                .borrow()
                .calls
                .iter()
                .all(|call| matches!(call, Call::Notifications))
        );
        h.restored();
        let mut h = Harness::new(&[(b"a", 0)], vec![Tick::signal(SAMPLE_NS, shutdown)]);
        assert_eq!(h.plain(), shutdown.status());
        assert_eq!(h.counters.calls, 1);
        assert!(
            !h.effects
                .base
                .script
                .borrow()
                .calls
                .iter()
                .any(|call| matches!(call, Call::Read(_) | Call::Dimensions | Call::Write(_)))
        );
    }
}

#[test]
fn runtime_failure_absence_addition_and_return_are_quiet_nonfatal_fixed_membership() {
    let mut h = Harness::new(
        &[(b"a", 0), (b"b", 0)],
        vec![
            Tick::input(SAMPLE_NS, b""),
            Tick::input(SECOND, b""),
            Tick::input(SECOND + SAMPLE_NS, b""),
            Tick::input(2 * SECOND, b""),
            Tick::input(2 * SECOND + 1, b"q"),
        ],
    );
    // Timeout-only wakes must not be modeled as readable EOF.
    for tick in h.effects.ticks.iter_mut().take(4) {
        tick.ready.input = false;
    }
    h.counters.rounds.extend([
        Ok(snapshot(&[(b"b", 100), (b"renamed-a", 500), (b"c", 500)])),
        Err(AcquisitionError::Parse(
            Snapshot::parse(b"bad\n").unwrap_err(),
        )),
        Err(AcquisitionError::Read(io::Error::other("proc failure"))),
        Ok(snapshot(&[(b"a", 10000), (b"b", 20000), (b"c", 30000)])),
    ]);
    assert_eq!(h.plain(), 0);
    assert!(h.stderr.is_empty());
    assert_eq!(h.counters.calls, 5);
    assert!(h.emitted().contains("Unavailable: interface absent"));
    assert!(h.emitted().contains("Unavailable: counter read failed"));
    let view = h.view.unwrap();
    assert_eq!(view.count, 2);
    assert_eq!(view.name, b"a");
    assert_eq!(view.state, ViewState::Baseline);
    assert_eq!(view.total, Some(10000));
    assert!(view.samples.is_empty());
}

#[test]
fn backpressure_and_partial_frames_preserve_sampling_and_latest_redraw() {
    let mut h = Harness::new(
        &[(b"a", 0), (b"b", 0), (b"c", 0)],
        vec![
            Tick::input(1, b""),
            Tick::input(2, b"\x1b[C"),
            Tick::input(SAMPLE_NS, b"\x1b[C"),
            Tick::input(SAMPLE_NS + 1, b""),
            Tick::input(SAMPLE_NS + 2, b""),
            Tick::input(SAMPLE_NS + 3, b"q"),
        ],
    );
    for index in [0, 3, 4] {
        h.effects.ticks[index].ready.input = false;
    }
    h.counters
        .rounds
        .push_back(Ok(snapshot(&[(b"a", 50), (b"b", 100), (b"c", 150)])));
    h.effects.base.script.borrow_mut().writes = VecDeque::from([
        Err(io::ErrorKind::WouldBlock.into()),
        Ok(4),
        Err(io::ErrorKind::WouldBlock.into()),
    ]);
    assert_eq!(h.plain(), 0);
    assert_eq!(h.counters.calls, 2);
    let emitted = h.emitted();
    assert!(!emitted.contains("Interface a")); // untouched baseline replaced
    let old = emitted.find("Interface b").unwrap();
    let latest = emitted.find("Interface c").unwrap();
    assert!(old < latest);
    assert_eq!(emitted.matches("\x1b[H\x1b[2J").count(), 2);
    assert_eq!(h.view.as_ref().unwrap().samples, [(SAMPLE_NS, 300.0)]);
    h.restored();
}

#[test]
fn blocked_output_still_handles_shutdown_without_an_extra_sample_or_write() {
    let mut first = Tick::input(1, b"");
    first.ready.input = false;
    let mut h = Harness::new(
        &[(b"a", 0)],
        vec![first, Tick::signal(SAMPLE_NS, Shutdown::Interrupt)],
    );
    h.effects
        .base
        .script
        .borrow_mut()
        .writes
        .push_back(Err(io::ErrorKind::WouldBlock.into()));
    assert_eq!(h.plain(), 130);
    assert_eq!(h.counters.calls, 1);
    assert_eq!(
        h.effects
            .base
            .script
            .borrow()
            .calls
            .iter()
            .filter(|call| matches!(call, Call::Write(_)))
            .count(),
        1
    );
    assert!(h.emitted().is_empty());
    h.restored();
}

#[test]
fn shutdown_arriving_during_acquisition_stops_before_input_or_frame_work() {
    let mut h = Harness::new(&[(b"a", 0)], vec![Tick::input(SAMPLE_NS, b"\x1b[Cq")]);
    h.counters.rounds.push_back(Ok(snapshot(&[(b"a", 50)])));
    let script = h.effects.base.script.clone();
    h.counters.on_acquire = Some(Box::new(move |call| {
        if call == 2 {
            script
                .borrow_mut()
                .notifications
                .push_back(Ok(Notifications {
                    shutdown: Some(Shutdown::Interrupt),
                    resized: true,
                }));
        }
    }));
    assert_eq!(h.plain(), 130);
    assert_eq!(h.counters.calls, 2);
    assert!(
        !h.effects
            .base
            .script
            .borrow()
            .calls
            .iter()
            .any(|call| matches!(call, Call::Read(_) | Call::Dimensions | Call::Write(_)))
    );
    h.restored();
}

#[test]
fn small_session_keeps_offscreen_sampling_and_recovers_real_history_on_growth() {
    let mut grow = Tick::input(SECOND + 1, b"\x1b[C");
    grow.notifications.resized = true;
    grow.dimensions = Some((80, 24));
    let mut h = Harness::new(
        &[(b"a", 0), (b"b", 0)],
        vec![
            Tick::input(SAMPLE_NS, b"\x1b[C"),
            Tick::input(SECOND, b"\x1b[D"),
            grow,
            Tick::input(SECOND + 2, b"q"),
        ],
    );
    h.session.state.borrow_mut().dimensions = Some((10, 3));
    h.counters.rounds.extend([
        Ok(snapshot(&[(b"a", 50), (b"b", 100)])),
        Ok(snapshot(&[(b"a", 100), (b"b", 200)])),
    ]);
    assert_eq!(h.plain(), 0);
    assert_eq!(h.counters.calls, 3);
    let view = h.view.as_ref().unwrap();
    assert_eq!(view.name, b"b");
    assert_eq!(view.state, ViewState::Measured);
    assert_eq!(view.samples, [(SAMPLE_NS, 200.0), (SECOND, 200.0)]);
    assert!(h.emitted().contains("Interface b (2/2)"));
    assert!(h.emitted().contains("Curr: 1.56 kBit/s"));
    h.restored();
}

#[test]
fn actual_loop_runtime_faults_restore_before_reporting() {
    for variant in 0..6 {
        let mut tick = Tick::input(1, b"x");
        if variant == 0 {
            tick.input.clear();
        }
        if variant == 4 {
            tick.notifications.resized = true;
        }
        let mut h = Harness::new(&[(b"a", 0)], vec![tick]);
        match variant {
            1 => h
                .effects
                .base
                .script
                .borrow_mut()
                .waits
                .push_back(Err(io::Error::other("poll failure"))),
            2 => h
                .effects
                .base
                .script
                .borrow_mut()
                .writes
                .push_back(Err(io::ErrorKind::BrokenPipe.into())),
            3 => {
                h.clock.clock.0.borrow_mut().readings =
                    VecDeque::from([Ok(0), Ok(0), Err(io::Error::other("runtime clock failure"))])
            }
            4 => h
                .effects
                .base
                .script
                .borrow_mut()
                .dimensions
                .push_back(Err(io::Error::other("resize failure"))),
            5 => h
                .effects
                .base
                .script
                .borrow_mut()
                .notifications
                .push_back(Err(io::Error::other("signal failure"))),
            _ => {}
        }
        assert_eq!(h.plain(), 1);
        assert!(!h.stderr.is_empty());
        assert!(h.stdout.is_empty());
        h.restored();
    }
}

#[test]
fn initialization_and_caught_startup_runtime_panics_use_the_guarded_application_path() {
    for name in [
        "snapshot",
        "prepare",
        "raw",
        "input",
        "output",
        "dimensions",
        "enter",
        "hide",
    ] {
        let mut h = Harness::new(&[(b"a", 0)], vec![]);
        h.session.state.borrow_mut().faults.insert(name, false);
        assert_eq!(h.plain(), 1);
        assert!(String::from_utf8_lossy(&h.stderr).contains(name));
        assert_eq!(
            h.session.state.borrow().calls.contains(&"show"),
            name == "hide"
        );
    }
    for variant in 0..3 {
        let mut h = Harness::new(&[(b"a", 0)], vec![Tick::input(SAMPLE_NS, b"q")]);
        match variant {
            0 => h.counters.panic_at = Some(1),
            1 => {
                h.session.state.borrow_mut().faults.insert("hide", true);
            }
            2 => h.counters.panic_at = Some(2),
            _ => unreachable!(),
        }
        assert_eq!(h.plain(), 1);
        assert!(String::from_utf8_lossy(&h.stderr).contains("caught panic"));
        if variant > 0 {
            h.restored();
        }
    }
}

#[test]
fn real_loop_quit_interrupt_term_and_cleanup_fault_arbitration() {
    for (tick, status) in [
        (Tick::input(1, b"q"), 0),
        (Tick::input(1, b"Q"), 0),
        (Tick::input(1, b"\x03"), 130),
        (Tick::signal(1, Shutdown::Interrupt), 130),
        (Tick::signal(1, Shutdown::Terminate), 143),
    ] {
        let mut h = Harness::new(&[(b"a", 0)], vec![tick]);
        for name in [
            "show",
            "leave",
            "restore-termios",
            "restore-input",
            "restore-output",
        ] {
            h.session.state.borrow_mut().faults.insert(name, false);
        }
        assert_eq!(h.plain(), if status == 0 { 1 } else { status });
        h.restored();
        assert_eq!(
            String::from_utf8_lossy(&h.stderr)
                .matches("terminal cleanup:")
                .count(),
            5
        );
    }
}

#[test]
fn unsupported_interactive_session_is_rejected_after_discovery_before_mutation() {
    let mut h = Harness::new(&[(b"a", 0)], vec![]);
    h.session.state.borrow_mut().unsupported = true;
    assert_eq!(h.plain(), 1);
    assert_eq!(h.counters.calls, 1);
    assert_eq!(h.session.state.borrow().calls, ["support"]);
    assert!(String::from_utf8_lossy(&h.stderr).contains("must be terminals"));
}

#[test]
fn deadline_overflow_is_fatal_before_mutation_or_after_one_delayed_acquisition() {
    let mut h = Harness::new(&[(b"a", 0)], vec![]);
    h.clock.clock.0.borrow_mut().time = u64::MAX;
    assert_eq!(h.plain(), 1);
    assert!(h.session.state.borrow().calls.is_empty());
    let mut h = Harness::new(&[(b"a", 0)], vec![Tick::input(u64::MAX, b"q")]);
    h.counters.rounds.push_back(Ok(snapshot(&[(b"a", 100)])));
    assert_eq!(h.plain(), 1);
    assert_eq!(h.counters.calls, 2);
    h.restored();
}
