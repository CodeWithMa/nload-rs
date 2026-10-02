# Monitor and selected-view interface

Implemented by [Implement the pure monitor and statistics runs](issues/03-monitor-statistics.md).
Acceptance authority: [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), sections 3, 4 and 8.
These are crate-private caller contracts in [Monitor](../../src/monitor.rs).

## Construct, observe, navigate, read

```rust
Monitor::new(
    startup: StartupSelection,
    ceilings: Ceilings,
    at: u64,
    outcome: impl FnMut(&[u8]) -> CounterOutcome,
) -> Monitor

monitor.observe(at: u64, outcome: impl FnMut(&[u8]) -> CounterOutcome)
monitor.forward()
monitor.backward()
monitor.selected_view() -> SelectedView<'_>
```

- `startup` comes from `snapshot.startup(config.interface.as_deref())`. Its
  original-byte names must be a validated, nonempty ordered list with an in-range
  zero-based selection. The monitor consumes this list and never sorts, discovers,
  filters or changes membership. Invalid internal construction preconditions
  (empty list, out-of-range selection or zero ceiling) assert; user-facing
  discovery/selection/ceiling validation belongs to the established upstream
  modules. `Ceilings` is shared with Arguments, in positive bytes/s.
- `at` is **u64 BOOTTIME nanoseconds**, supplied by the clock adapter after
  acquisition completes. Clock conversion must be checked; a clock failure is
  fatal in application/runtime code, never a substitute timestamp or counter
  failure. The monitor has no clock reads, wall-time dependency or dimensions.
- Construction calls the lookup once for each ordered name, using the existing
  discovery snapshot. Each readable pair becomes a baseline. Absent/read-failed
  initial outcomes are also representable, without totals/rates/history. Normal
  application startup has already required successful nonempty discovery.
- Each accepted `observe` calls the lookup once for **every retained name**,
  including unavailable and never-displayed entries. The lookup reads an already
  acquired round; it must not perform per-name acquisition. New snapshot names
  never reach the monitor. The closure avoids exposing mutable entry lists or
  asking callers to match positional outcome arrays to private state.
- A round with `at <=` the latest accepted round time is ignored in its entirety,
  including decreased/read-failed/absent outcomes. The lookup is not called;
  totals, availability, intervals, extrema, history and accepted time are unchanged.
  Accepted absence/failure rounds also advance the round time. A successful return
  must occur in a later round and establishes only a baseline.
- Forward is Right/Down; backward is Left/Up. Each call performs one symmetric
  wrapping step; a singleton is a no-op. Decode arrows/ignored keys in Terminal,
  request a prompt redraw in the application. Navigation never calls the lookup
  or changes observations/statistics/history.
- `selected_view` is a borrowed read-only projection. Drop the view before
  mutating the monitor. Reads/redraws cannot expire history because they have
  neither mutable access nor a time parameter.

### Application connection

Using the established [Arguments and proc snapshot interfaces](arguments-proc-interfaces.md):

```rust
// After initial acquisition/completion timestamp and startup validation:
let startup = discovery.startup(config.interface.as_deref())?;
let initial_round = Ok(discovery);
let mut monitor = Monitor::new(startup, config.ceilings, completed_at, |name| {
    counter_outcome(&initial_round, name)
});

// Exactly one fresh bulk acquisition when due; timestamp completion, even Err.
let round = counter_source.acquire();
let completed_at = clock.now()?; // schematic: the clock ticket owns its interface
monitor.observe(completed_at, |name| counter_outcome(&round, name));
```

The connection above is schematic, not an implemented application loop. The
application owns deadline anchoring to discovery completion, BOOTTIME waiting,
missed-slot advancement, shutdown priority and redraw requests. It must not call
`observe` on navigation/resize/parser timeouts or fabricate catch-up observations.
Runtime counter failure remains a quiet, nonfatal retry round. No redundant
initialization acquisition is needed.

## Selected view for Screen

`SelectedView<'a>` provides:

| Field | Contract |
| --- | --- |
| `name: &'a [u8]` | Original identity bytes; Screen alone escapes/clips presentation. |
| `position: usize`, `count: usize` | One-based position and fixed startup count. |
| `state: ViewState` | `Baseline`, `Measured`, `Absent`, or `ReadFailed`. |
| `incoming`, `outgoing: DirectionView<'a>` | Independent directional values/history and shared-per-interface ceiling configuration. |

Each `DirectionView` provides:

- `total: Option<u64>`: exact latest successful cumulative counter, including
  baseline totals. `None` in either unavailable state; never stale totals.
- `rates: Option<Rates>`: unrounded f64 bytes/s fields `current`, `average`,
  `minimum`, `maximum`. `None` at baseline/unavailable. Measured idle is `Some`
  containing genuine zero rates, not an unavailable placeholder. Extrema compare
  successive current rates over the entire statistics run, independently per
  direction, and survive average/history expiry.
- `samples: &'a VecDeque<GraphSample>`: all retained graph samples in
  chronological order. Each contains integer completion `at` and unrounded f64
  current `rate` in bytes/s. Samples exactly on the latest accepted observation's
  `t - 300s` boundary are retained; older ones expire only when observing. Baseline
  and unavailable histories are empty. There is no fabricated zero padding.
- `ceiling: u64`: that direction's positive bytes/s graph ceiling. Statistics and
  samples remain uncapped even above this value; Screen alone saturates bars.

Screen takes the newest fitting suffix, e.g.
`samples.iter().skip(samples.len().saturating_sub(graph_width))`, rendering oldest
fitting sample left and newest right. Leave missing columns blank. A measured-zero
sample still consumes one column. Resize only changes this suffix; it never
changes the borrowed history. No interval queues or reset bookkeeping are exposed.

State invariants guaranteed by the monitor:

| State | Totals (both) | Rates (both) | Histories (both) |
| --- | --- | --- | --- |
| Baseline | `Some` | `None` | Empty |
| Measured | `Some` | `Some` | One actual column per accepted rate calculation, subject to expiry |
| Absent / ReadFailed | `None` | `None` | Empty |

Both directions reset together on a decrease in either counter or an unavailable
outcome. A readable decrease becomes an immediate new baseline; a return from
unavailability is baseline even when both counters are higher. Neither guessing
wraps nor bridging an unavailable gap occurs.

## Hidden calculations and checks

Counters, interval endpoints and checked deltas remain integer. Only integer
differences and derived contributions become f64. The monitor retains paired
intervals overlapping the oldest average-window boundary, including a crossing
interval, and weights overlaps by their actual durations. The one-second current
and 300-second average windows warm up without zero padding. Calculations,
extrema and stored graph samples are never display-rounded.

The monitor performs no I/O, diagnostics, unsafe operations, scheduling, scaling
or formatting. Work per round is proportional to retained names and overlapping
interval/sample histories; normal 500-ms cadence gives roughly 600 columns per
interface. There is no fixed sample-count limit that truncates actual five-minute
history or any additional public extensibility seam.

Passing fixtures and acceptance mappings are in the
[monitor verification record](verification.md#monitor-run--2026-10-02).
Actual screen rows, decoded events and application/clock behavior remain with
[Implement the pure ASCII screen and formatting](issues/04-ascii-screen.md) and
[Integrate startup, sampling and the interactive application loop](issues/07-application-integration.md).
