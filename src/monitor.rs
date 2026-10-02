// SPDX-License-Identifier: GPL-2.0-only

//! Pure traffic statistics, fixed interface membership and navigation.
//!
//! Owns statistics runs, availability, exact deltas, windows, extrema and history.
//! Exposes selected read-only views; performs no I/O, clock reads or sizing.

use std::collections::VecDeque;

use crate::arguments::Ceilings;
use crate::proc_sampler::{CounterOutcome, Counters, StartupSelection};

const SECOND_NS: u64 = 1_000_000_000;
const HISTORY_NS: u64 = 300 * SECOND_NS;

/// Availability of the selected entry. Baseline has totals but no measured rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewState {
    Baseline,
    Measured,
    Absent,
    ReadFailed,
}

/// Unrounded bytes/s; extrema belong to the entire uninterrupted statistics run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Rates {
    pub(crate) current: f64,
    pub(crate) average: f64,
    pub(crate) minimum: f64,
    pub(crate) maximum: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GraphSample {
    /// Integer BOOTTIME nanoseconds at acquisition completion.
    pub(crate) at: u64,
    pub(crate) rate: f64,
}

/// History is chronological, including real zero-height samples. Rendering takes
/// the newest fitting suffix without modifying it or manufacturing blank samples.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DirectionView<'a> {
    pub(crate) total: Option<u64>,
    pub(crate) rates: Option<Rates>,
    pub(crate) samples: &'a VecDeque<GraphSample>,
    pub(crate) ceiling: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SelectedView<'a> {
    pub(crate) name: &'a [u8],
    /// One-based header position in the fixed startup list.
    pub(crate) position: usize,
    pub(crate) count: usize,
    pub(crate) state: ViewState,
    pub(crate) incoming: DirectionView<'a>,
    pub(crate) outgoing: DirectionView<'a>,
}

/// One bulk round calls the supplied outcome lookup for every retained name.
/// Neither construction nor observation knows about proc, clocks or dimensions.
pub(crate) struct Monitor {
    entries: Vec<Entry>,
    selected: usize,
    ceilings: Ceilings,
    latest_at: u64,
}

impl Monitor {
    /// `startup` is the validated nonempty ordered list/zero-based selection from
    /// discovery; ceilings must be positive (Arguments already guarantees this).
    /// Reuse the discovery snapshot via the lookup, timestamped at completion.
    pub(crate) fn new(
        startup: StartupSelection,
        ceilings: Ceilings,
        at: u64,
        mut outcome: impl FnMut(&[u8]) -> CounterOutcome,
    ) -> Self {
        assert!(!startup.names.is_empty() && startup.selected < startup.names.len());
        assert!(ceilings.incoming > 0 && ceilings.outgoing > 0);
        Self {
            entries: startup
                .names
                .into_iter()
                .map(|name| {
                    let state = EntryState::from_outcome(at, outcome(&name));
                    Entry { name, state }
                })
                .collect(),
            selected: startup.selected,
            ceilings,
            latest_at: at,
        }
    }

    /// Accept one completion-timestamped round. Equal/earlier times are ignored
    /// entirely (including failure/decrease outcomes); no lookup or state change.
    /// The lookup should use `proc_sampler::counter_outcome(&round, name)`.
    pub(crate) fn observe(&mut self, at: u64, mut outcome: impl FnMut(&[u8]) -> CounterOutcome) {
        if at <= self.latest_at {
            return;
        }
        for entry in &mut self.entries {
            match (&mut entry.state, outcome(&entry.name)) {
                (EntryState::Run(run), CounterOutcome::Readable(counters)) => {
                    run.observe(at, counters);
                }
                (state, outcome) => *state = EntryState::from_outcome(at, outcome),
            }
        }
        self.latest_at = at;
    }

    /// Right/Down, one decoded event at a time. Does not sample or reset.
    pub(crate) fn forward(&mut self) {
        self.selected = if self.selected + 1 == self.entries.len() {
            0
        } else {
            self.selected + 1
        };
    }

    /// Left/Up, with symmetric wrapping. Does not sample or reset.
    pub(crate) fn backward(&mut self) {
        self.selected = if self.selected == 0 {
            self.entries.len() - 1
        } else {
            self.selected - 1
        };
    }

    /// No time/dimensions parameter: a read/redraw can never expire history.
    pub(crate) fn selected_view(&self) -> SelectedView<'_> {
        let entry = &self.entries[self.selected];
        let (state, totals, incoming, outgoing) = match &entry.state {
            EntryState::Run(run) => (
                if run.incoming.rates.is_some() {
                    ViewState::Measured
                } else {
                    ViewState::Baseline
                },
                Some(run.totals),
                &run.incoming,
                &run.outgoing,
            ),
            EntryState::Absent => (ViewState::Absent, None, &EMPTY_DIRECTION, &EMPTY_DIRECTION),
            EntryState::ReadFailed => (
                ViewState::ReadFailed,
                None,
                &EMPTY_DIRECTION,
                &EMPTY_DIRECTION,
            ),
        };
        SelectedView {
            name: &entry.name,
            position: self.selected + 1,
            count: self.entries.len(),
            state,
            incoming: incoming.view(totals.map(|pair| pair.incoming), self.ceilings.incoming),
            outgoing: outgoing.view(totals.map(|pair| pair.outgoing), self.ceilings.outgoing),
        }
    }
}

struct Entry {
    name: Vec<u8>,
    state: EntryState,
}

enum EntryState {
    Run(Run),
    Absent,
    ReadFailed,
}

impl EntryState {
    fn from_outcome(at: u64, outcome: CounterOutcome) -> Self {
        match outcome {
            CounterOutcome::Readable(counters) => Self::Run(Run::new(at, counters)),
            CounterOutcome::Absent => Self::Absent,
            CounterOutcome::ReadFailed => Self::ReadFailed,
        }
    }
}

struct Run {
    baseline_at: u64,
    latest_at: u64,
    totals: Counters,
    intervals: VecDeque<Interval>,
    incoming: Direction,
    outgoing: Direction,
}

struct Interval {
    start: u64,
    end: u64,
    delta: Counters,
}

impl Run {
    fn new(at: u64, totals: Counters) -> Self {
        Self {
            baseline_at: at,
            latest_at: at,
            totals,
            intervals: VecDeque::new(),
            incoming: Direction::default(),
            outgoing: Direction::default(),
        }
    }

    fn observe(&mut self, at: u64, totals: Counters) {
        let delta = match (
            totals.incoming.checked_sub(self.totals.incoming),
            totals.outgoing.checked_sub(self.totals.outgoing),
        ) {
            (Some(incoming), Some(outgoing)) => Counters { incoming, outgoing },
            _ => {
                // A decrease in either direction starts a *paired* new baseline.
                *self = Self::new(at, totals);
                return;
            }
        };
        self.intervals.push_back(Interval {
            start: self.latest_at,
            end: at,
            delta,
        });
        self.latest_at = at;
        self.totals = totals;

        let oldest = at.saturating_sub(HISTORY_NS);
        // Keep the interval crossing the boundary; an interval ending exactly
        // there contributes nothing to either current or average calculations.
        while self.intervals.front().is_some_and(|j| j.end <= oldest) {
            self.intervals.pop_front();
        }
        let (current_in, current_out) = self.window(SECOND_NS);
        let (average_in, average_out) = self.window(HISTORY_NS);
        self.incoming.record(at, current_in, average_in);
        self.outgoing.record(at, current_out, average_out);
    }

    fn window(&self, horizon: u64) -> (f64, f64) {
        let start = self.baseline_at.max(self.latest_at.saturating_sub(horizon));
        let (mut incoming, mut outgoing) = (0.0, 0.0);
        for interval in &self.intervals {
            if interval.end <= start {
                continue;
            }
            // Subtract exact integer endpoints *before* conversion; absolute
            // BOOTTIME and cumulative counters may exceed f64 integer precision.
            let overlap = interval.end - interval.start.max(start);
            let fraction = overlap as f64 / (interval.end - interval.start) as f64;
            incoming += interval.delta.incoming as f64 * fraction;
            outgoing += interval.delta.outgoing as f64 * fraction;
        }
        let scale = SECOND_NS as f64 / (self.latest_at - start) as f64;
        (incoming * scale, outgoing * scale)
    }
}

#[derive(Default)]
struct Direction {
    rates: Option<Rates>,
    samples: VecDeque<GraphSample>,
}

static EMPTY_DIRECTION: Direction = Direction {
    rates: None,
    samples: VecDeque::new(),
};

impl Direction {
    fn record(&mut self, at: u64, current: f64, average: f64) {
        self.rates = Some(Rates {
            current,
            average,
            minimum: self.rates.map_or(current, |r| r.minimum.min(current)),
            maximum: self.rates.map_or(current, |r| r.maximum.max(current)),
        });
        let oldest = at.saturating_sub(HISTORY_NS);
        while self
            .samples
            .front()
            .is_some_and(|sample| sample.at < oldest)
        {
            self.samples.pop_front();
        }
        self.samples.push_back(GraphSample { at, rate: current });
    }

    fn view(&self, total: Option<u64>, ceiling: u64) -> DirectionView<'_> {
        DirectionView {
            total,
            rates: self.rates,
            samples: &self.samples,
            ceiling,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proc_sampler::{Snapshot, counter_outcome};

    fn readable(incoming: u64, outgoing: u64) -> CounterOutcome {
        CounterOutcome::Readable(Counters { incoming, outgoing })
    }

    fn single(at: u64, incoming: u64, outgoing: u64) -> Monitor {
        Monitor::new(
            StartupSelection {
                names: vec![b"eth0".to_vec()],
                selected: 0,
            },
            Ceilings::default(),
            at,
            |_| readable(incoming, outgoing),
        )
    }

    fn observe(monitor: &mut Monitor, at: u64, incoming: u64, outgoing: u64) {
        monitor.observe(at, |_| readable(incoming, outgoing));
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1e-9 * expected.abs().max(1.0),
            "actual {actual:?}, expected {expected:?}"
        );
    }

    fn rates(direction: DirectionView<'_>, expected: [f64; 4]) {
        let actual = direction.rates.unwrap();
        for (actual, expected) in [
            actual.current,
            actual.average,
            actual.minimum,
            actual.maximum,
        ]
        .into_iter()
        .zip(expected)
        {
            close(actual, expected);
        }
        assert_eq!(direction.samples.back().unwrap().rate, actual.current);
    }

    fn baseline(view: SelectedView<'_>, incoming: u64, outgoing: u64) {
        assert_eq!(view.state, ViewState::Baseline);
        assert_eq!(view.incoming.total, Some(incoming));
        assert_eq!(view.outgoing.total, Some(outgoing));
        for direction in [view.incoming, view.outgoing] {
            assert_eq!(direction.rates, None);
            assert!(direction.samples.is_empty());
        }
    }

    fn unavailable(view: SelectedView<'_>, state: ViewState) {
        assert_eq!(view.state, state);
        for direction in [view.incoming, view.outgoing] {
            assert_eq!(direction.total, None);
            assert_eq!(direction.rates, None);
            assert!(direction.samples.is_empty());
        }
    }

    // An owned copy of only the caller-facing view, never monitor bookkeeping.
    fn saved(view: SelectedView<'_>) -> String {
        format!("{view:?}")
    }

    #[test]
    fn warm_up_exposes_exact_totals_and_independent_directional_rates() {
        let mut monitor = single(0, 1000, 9000);
        baseline(monitor.selected_view(), 1000, 9000);
        observe(&mut monitor, SECOND_NS / 2, 1100, 9200);
        let view = monitor.selected_view();
        assert_eq!(view.state, ViewState::Measured);
        rates(view.incoming, [200.0; 4]);
        rates(view.outgoing, [400.0; 4]);
        assert_eq!(view.incoming.samples.len(), 1);
        assert_eq!(view.outgoing.samples.len(), 1);
        observe(&mut monitor, SECOND_NS, 1400, 9200);
        let view = monitor.selected_view();
        rates(view.incoming, [400.0, 400.0, 200.0, 400.0]);
        rates(view.outgoing, [200.0, 200.0, 200.0, 400.0]);
        assert_eq!(view.incoming.total, Some(1400));
        assert_eq!(view.outgoing.total, Some(9200));
        assert_eq!(view.incoming.samples.len(), 2);
        assert_eq!(view.incoming.samples[0].at, SECOND_NS / 2);
        assert_eq!(view.incoming.samples[1].at, SECOND_NS);
    }

    #[test]
    fn partial_current_window_weights_only_interval_overlap() {
        let mut monitor = single(0, 0, 0);
        observe(&mut monitor, SECOND_NS / 2, 100, 200);
        observe(&mut monitor, SECOND_NS * 5 / 4, 400, 800);
        let view = monitor.selected_view();
        rates(view.incoming, [350.0, 320.0, 200.0, 350.0]);
        rates(view.outgoing, [700.0, 640.0, 400.0, 700.0]);
    }

    #[test]
    fn exact_and_partial_average_boundaries_preserve_crossing_intervals() {
        for (seconds, total, expected) in
            [(302, 90800, (90800.0 - 600.0) / 300.0), (301, 91200, 303.0)]
        {
            let mut monitor = single(0, 0, 0);
            observe(&mut monitor, 2 * SECOND_NS, 600, 1200);
            observe(&mut monitor, seconds * SECOND_NS, total, total * 2);
            let view = monitor.selected_view();
            close(view.incoming.rates.unwrap().average, expected);
            close(view.outgoing.rates.unwrap().average, 2.0 * expected);
            close(
                view.incoming.rates.unwrap().current,
                (total - 600) as f64 / (seconds - 2) as f64,
            );
            assert_eq!(view.incoming.total, Some(total));
        }
    }

    #[test]
    fn delayed_and_suspend_length_intervals_use_actual_elapsed_time_once() {
        let mut monitor = single(17, 400, 800);
        observe(&mut monitor, 17 + 3 * SECOND_NS, 3400, 6800);
        let view = monitor.selected_view();
        rates(view.incoming, [1000.0; 4]);
        rates(view.outgoing, [2000.0; 4]);
        assert_eq!(view.incoming.samples.len(), 1);
        assert_eq!(view.incoming.samples[0].at, 17 + 3 * SECOND_NS);
        // A long suspend-equivalent interval still contributes only one column.
        observe(&mut monitor, 17 + 603 * SECOND_NS, 63400, 126800);
        let view = monitor.selected_view();
        rates(view.incoming, [100.0, 100.0, 100.0, 1000.0]);
        rates(view.outgoing, [200.0, 200.0, 200.0, 2000.0]);
        assert_eq!(view.incoming.samples.len(), 1); // old column expired
    }

    #[test]
    fn nonpositive_times_ignore_entire_outcomes_without_advancing_any_view() {
        let mut monitor = single(10 * SECOND_NS, 1000, 2000);
        for at in [10 * SECOND_NS, 9 * SECOND_NS] {
            monitor.observe(at, |_| panic!("ignored baseline round was consulted"));
            baseline(monitor.selected_view(), 1000, 2000);
        }
        observe(&mut monitor, 11 * SECOND_NS, 1100, 2200);
        let before = saved(monitor.selected_view());
        for at in [11 * SECOND_NS, 10 * SECOND_NS, 0] {
            for outcome in [
                readable(1, 1),
                CounterOutcome::Absent,
                CounterOutcome::ReadFailed,
            ] {
                monitor.observe(at, |_| outcome);
                assert_eq!(saved(monitor.selected_view()), before);
            }
        }
        observe(&mut monitor, 12 * SECOND_NS, 1400, 2800);
        let view = monitor.selected_view();
        rates(view.incoming, [300.0, 200.0, 100.0, 300.0]);
        assert_eq!(view.incoming.samples.len(), 2);
        monitor.observe(13 * SECOND_NS, |_| CounterOutcome::Absent);
        monitor.observe(13 * SECOND_NS, |_| readable(5000, 6000));
        unavailable(monitor.selected_view(), ViewState::Absent);
        observe(&mut monitor, 14 * SECOND_NS, 5000, 6000);
        baseline(monitor.selected_view(), 5000, 6000);
    }

    #[test]
    fn exact_small_deltas_and_nanosecond_differences_survive_large_values() {
        // At these magnitudes, converting counters/endpoints before subtraction
        // would erase the one-byte/one-nanosecond differences.
        let origin = u64::MAX - 10;
        let mut monitor = single(origin, u64::MAX - 3, u64::MAX - 6);
        observe(&mut monitor, origin + 1, u64::MAX - 2, u64::MAX - 4);
        let view = monitor.selected_view();
        rates(view.incoming, [1_000_000_000.0; 4]);
        rates(view.outgoing, [2_000_000_000.0; 4]);
        assert_eq!(view.incoming.total, Some(u64::MAX - 2));
        observe(&mut monitor, origin + 2, u64::MAX, u64::MAX);
        let view = monitor.selected_view();
        rates(
            view.incoming,
            [
                1_500_000_000.0,
                1_500_000_000.0,
                1_000_000_000.0,
                1_500_000_000.0,
            ],
        );
        rates(
            view.outgoing,
            [
                3_000_000_000.0,
                3_000_000_000.0,
                2_000_000_000.0,
                3_000_000_000.0,
            ],
        );
        assert_eq!(view.incoming.total, Some(u64::MAX));
        assert_eq!(view.outgoing.total, Some(u64::MAX));
        assert_eq!(view.incoming.samples[1].at, origin + 2);
    }

    #[test]
    fn full_u64_interval_delta_is_checked_before_derived_conversion() {
        let mut monitor = single(0, 0, 0);
        observe(&mut monitor, SECOND_NS, u64::MAX, u64::MAX - 1);
        let view = monitor.selected_view();
        rates(view.incoming, [u64::MAX as f64; 4]);
        assert_eq!(view.incoming.total, Some(u64::MAX));
        assert_eq!(view.outgoing.total, Some(u64::MAX - 1));
    }

    #[test]
    fn unrounded_run_extrema_survive_average_expiry_and_measured_idle() {
        let mut monitor = single(0, 0, 0);
        observe(&mut monitor, SECOND_NS, 1, 2);
        observe(&mut monitor, 2 * SECOND_NS + 1, 2, 4);
        let fractional = SECOND_NS as f64 / (SECOND_NS + 1) as f64;
        let view = monitor.selected_view();
        let incoming = view.incoming.rates.unwrap();
        assert!(incoming.current < 1.0);
        assert_eq!(incoming.minimum, incoming.current);
        assert_eq!(incoming.maximum, 1.0);
        close(incoming.current, fractional);
        observe(&mut monitor, 302 * SECOND_NS + 1, 602, 904);
        let view = monitor.selected_view();
        rates(view.incoming, [2.0, 2.0, fractional, 2.0]);
        assert_eq!(view.incoming.samples.len(), 2); // t=2+1 ns boundary retained
        // Average/history expiry does not expire the older minimum.
        observe(&mut monitor, 303 * SECOND_NS + 1, 602, 904);
        let view = monitor.selected_view();
        assert_eq!(view.state, ViewState::Measured);
        assert_eq!(view.incoming.rates.unwrap().current, 0.0);
        assert_eq!(view.incoming.rates.unwrap().minimum, 0.0);
        assert_eq!(view.incoming.rates.unwrap().maximum, 2.0);
        assert_eq!(view.outgoing.rates.unwrap().minimum, 0.0);
        assert_eq!(view.outgoing.rates.unwrap().maximum, 3.0);
        assert_eq!(view.incoming.samples.back().unwrap().rate, 0.0);
        assert_eq!(view.incoming.total, Some(602));
    }

    #[test]
    fn decrease_in_either_counter_resets_both_directions_to_new_baseline() {
        for decrease_incoming in [true, false] {
            let mut monitor = single(0, 4_000_000_000, 8_000_000_000);
            observe(&mut monitor, SECOND_NS / 2, 4_000_000_500, 8_000_001_000);
            let (incoming, outgoing) = if decrease_incoming {
                (10000, 8_000_002_000)
            } else {
                (4_000_001_000, 10000)
            };
            observe(&mut monitor, SECOND_NS, incoming, outgoing);
            baseline(monitor.selected_view(), incoming, outgoing);
            observe(
                &mut monitor,
                SECOND_NS * 3 / 2,
                incoming + 500,
                outgoing + 1000,
            );
            let view = monitor.selected_view();
            rates(view.incoming, [1000.0; 4]);
            rates(view.outgoing, [2000.0; 4]);
            assert_eq!(view.incoming.samples.len(), 1);
            assert_eq!(view.outgoing.samples.len(), 1);
        }
    }

    #[test]
    fn absence_and_read_failure_hide_totals_and_never_bridge_return_gaps() {
        for (outcome, state) in [
            (CounterOutcome::Absent, ViewState::Absent),
            (CounterOutcome::ReadFailed, ViewState::ReadFailed),
        ] {
            let mut monitor = single(0, 1000, 2000);
            observe(&mut monitor, SECOND_NS, 1100, 2200);
            monitor.observe(2 * SECOND_NS, |_| outcome);
            unavailable(monitor.selected_view(), state);
            monitor.observe(3 * SECOND_NS, |_| outcome);
            unavailable(monitor.selected_view(), state);
            observe(&mut monitor, 100 * SECOND_NS, 50000, 90000);
            baseline(monitor.selected_view(), 50000, 90000);
            observe(&mut monitor, 100 * SECOND_NS + SECOND_NS / 2, 50100, 90050);
            let view = monitor.selected_view();
            rates(view.incoming, [200.0; 4]);
            rates(view.outgoing, [100.0; 4]);
            assert_eq!(view.incoming.samples.len(), 1);
        }
    }

    #[test]
    fn initial_unavailable_outcomes_and_reason_changes_remain_explicit() {
        let mut monitor = Monitor::new(
            StartupSelection {
                names: vec![b"A".to_vec(), b"B".to_vec()],
                selected: 0,
            },
            Ceilings::default(),
            0,
            |name| {
                if name == b"A" {
                    CounterOutcome::Absent
                } else {
                    CounterOutcome::ReadFailed
                }
            },
        );
        unavailable(monitor.selected_view(), ViewState::Absent);
        monitor.forward();
        unavailable(monitor.selected_view(), ViewState::ReadFailed);
        monitor.observe(SECOND_NS, |_| CounterOutcome::Absent);
        unavailable(monitor.selected_view(), ViewState::Absent);
        monitor.observe(2 * SECOND_NS, |_| readable(1, 2));
        baseline(monitor.selected_view(), 1, 2);
        monitor.backward();
        baseline(monitor.selected_view(), 1, 2);
    }

    #[test]
    fn navigation_wraps_symmetrically_preserving_views_and_global_ceilings() {
        let ceilings = Ceilings {
            incoming: 2_621_440,
            outgoing: 655_360,
        };
        let names = vec![b"eth1".to_vec(), b"eth02".to_vec(), b"e\xff2".to_vec()];
        let mut monitor = Monitor::new(
            StartupSelection {
                names: names.clone(),
                selected: 1,
            },
            ceilings,
            0,
            |_| readable(0, 0),
        );
        monitor.observe(SECOND_NS, |_| readable(5_242_880, 1_310_720));
        let mut expected = Vec::new();
        for index in [1, 2, 0] {
            let view = monitor.selected_view();
            assert_eq!(view.name, names[index]);
            assert_eq!(view.position, index + 1);
            assert_eq!(view.count, 3);
            assert_eq!(view.incoming.ceiling, ceilings.incoming);
            assert_eq!(view.outgoing.ceiling, ceilings.outgoing);
            // Numeric rates stay above graph ceilings, never capped.
            assert_eq!(view.incoming.rates.unwrap().current, 5_242_880.0);
            assert_eq!(view.outgoing.rates.unwrap().current, 1_310_720.0);
            expected.push(saved(view));
            monitor.forward();
        }
        for index in [0, 2, 1, 0, 2, 1] {
            monitor.backward();
            let view = monitor.selected_view();
            assert_eq!(view.name, names[index]);
            let slot = match index {
                1 => 0,
                2 => 1,
                _ => 2,
            };
            assert_eq!(saved(view), expected[slot]);
        }
        let mut one = single(0, 1, 2);
        observe(&mut one, SECOND_NS, 3, 4);
        let before = saved(one.selected_view());
        for _ in 0..4 {
            one.forward();
            one.backward();
            assert_eq!(saved(one.selected_view()), before);
        }
    }

    fn snapshot(rows: &[(&[u8], u64, u64)]) -> Snapshot {
        let mut bytes = b"Inter-|Receive|Transmit\nface|bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n".to_vec();
        for (name, incoming, outgoing) in rows {
            bytes.extend_from_slice(name);
            bytes.extend_from_slice(
                format!(": {incoming} 0 0 0 0 0 0 0 {outgoing} 0 0 0 0 0 0 0\n").as_bytes(),
            );
        }
        Snapshot::parse(&bytes).unwrap()
    }

    #[test]
    fn snapshot_lifecycle_retains_raw_membership_selection_and_retries_all_names() {
        let raw = b"A\xff".as_slice();
        let initial = snapshot(&[(raw, 100, 200), (b"B", 1000, 2000)]);
        let startup = initial.startup(Some(raw)).unwrap();
        let initial = Ok(initial);
        let mut monitor = Monitor::new(startup, Ceilings::default(), 0, |name| {
            counter_outcome(&initial, name)
        });
        let next = Ok(snapshot(&[(raw, 200, 400), (b"B", 1100, 2200)]));
        monitor.observe(SECOND_NS, |name| counter_outcome(&next, name));
        let removed = Ok(snapshot(&[
            (b"renamed", 90000, 90000),
            (b"B", 1200, 2400),
            (b"C", 5, 6),
        ]));
        let mut looked_up = Vec::new();
        monitor.observe(2 * SECOND_NS, |name| {
            looked_up.push(name.to_vec());
            counter_outcome(&removed, name)
        });
        assert_eq!(looked_up, [raw.to_vec(), b"B".to_vec()]);
        let view = monitor.selected_view();
        assert_eq!(view.name, raw);
        assert_eq!((view.position, view.count), (1, 2));
        unavailable(view, ViewState::Absent);
        monitor.forward();
        assert_eq!(monitor.selected_view().name, b"B");
        assert_eq!(monitor.selected_view().incoming.samples.len(), 2);
        monitor.forward();
        assert_eq!(monitor.selected_view().name, raw);
        // A global parse failure affects even the successfully measured B.
        let failed =
            Snapshot::parse(b"malformed\n").map_err(crate::proc_sampler::AcquisitionError::Parse);
        monitor.observe(3 * SECOND_NS, |name| counter_outcome(&failed, name));
        unavailable(monitor.selected_view(), ViewState::ReadFailed);
        monitor.forward();
        unavailable(monitor.selected_view(), ViewState::ReadFailed);
        let returned = Ok(snapshot(&[
            (raw, 50000, 90000),
            (b"B", 2000, 3000),
            (b"C", 7, 8),
        ]));
        monitor.observe(4 * SECOND_NS, |name| counter_outcome(&returned, name));
        baseline(monitor.selected_view(), 2000, 3000);
        monitor.backward();
        baseline(monitor.selected_view(), 50000, 90000);
        assert_eq!(monitor.selected_view().name, raw);
        assert_eq!(monitor.selected_view().count, 2);
    }

    #[test]
    fn never_selected_interfaces_accumulate_real_history_at_every_round() {
        let mut monitor = Monitor::new(
            StartupSelection {
                names: vec![b"A".to_vec(), b"B".to_vec()],
                selected: 0,
            },
            Ceilings::default(),
            0,
            |_| readable(0, 0),
        );
        for (at, a, b) in [
            (SECOND_NS / 2, 50, 100),
            (SECOND_NS, 100, 300),
            (4 * SECOND_NS, 400, 3300),
        ] {
            monitor.observe(at, |name| {
                if name == b"A" {
                    readable(a, a * 2)
                } else {
                    readable(b, b * 2)
                }
            });
        }
        monitor.forward();
        let view = monitor.selected_view();
        assert_eq!(view.name, b"B");
        rates(view.incoming, [1000.0, 825.0, 200.0, 1000.0]);
        rates(view.outgoing, [2000.0, 1650.0, 400.0, 2000.0]);
        assert_eq!(
            view.incoming
                .samples
                .iter()
                .map(|s| (s.at, s.rate))
                .collect::<Vec<_>>(),
            [
                (SECOND_NS / 2, 200.0),
                (SECOND_NS, 300.0),
                (4 * SECOND_NS, 1000.0)
            ]
        );
        let before = saved(view);
        monitor.backward();
        monitor.forward();
        assert_eq!(saved(monitor.selected_view()), before);
    }

    #[test]
    fn history_expiry_retains_exact_boundary_and_reads_only_hide_fitting_suffixes() {
        let mut monitor = single(0, 0, 0);
        observe(&mut monitor, SECOND_NS - 1, 0, 0);
        observe(&mut monitor, SECOND_NS, 0, 0);
        observe(&mut monitor, SECOND_NS + 1, 0, 0);
        // Populate more history than a normal 50-column graph can show.
        for second in 2..=301 {
            observe(&mut monitor, second * SECOND_NS, second * 10, second * 20);
        }
        let view = monitor.selected_view();
        let retained: Vec<_> = view.incoming.samples.iter().copied().collect();
        assert_eq!(retained.len(), 302);
        assert_eq!(retained[0].at, SECOND_NS); // t-300 exactly retained
        assert_eq!(retained[0].rate, 0.0); // genuine zero occupies a column
        assert_eq!(retained[1].at, SECOND_NS + 1);
        assert_eq!(retained.last().unwrap().at, 301 * SECOND_NS);
        let before = saved(view);
        // Consumer-side fitting demonstrates shrink/grow cannot mutate/expire:
        // no dimensions or redraw clock are accepted by the monitor interface.
        for width in [50, 1, 0, 400, 50] {
            let samples = monitor.selected_view().incoming.samples;
            let fitting: Vec<_> = samples
                .iter()
                .skip(samples.len().saturating_sub(width))
                .copied()
                .collect();
            assert_eq!(fitting, retained[retained.len().saturating_sub(width)..]);
            assert_eq!(saved(monitor.selected_view()), before);
        }
        observe(&mut monitor, 301 * SECOND_NS + 1, 3010, 6020);
        let view = monitor.selected_view();
        assert_eq!(view.incoming.samples.front().unwrap().at, SECOND_NS + 1);
        assert_eq!(view.incoming.samples.len(), 302);
        assert_eq!(view.outgoing.samples.len(), 302);
    }
}
