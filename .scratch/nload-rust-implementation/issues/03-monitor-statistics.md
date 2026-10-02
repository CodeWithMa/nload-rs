# Implement the pure monitor and statistics runs

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: monitor statistics)
Blocked by: 01

## Question

Can a pure Monitor module implement the fixed startup list, navigation, paired observation/reset semantics, exact-delta rate windows and five-minute graph histories specified in sections 3, 4 and 8, exposing only the selected read-only view?

Implement construction from ordered names, selection, ceilings and initial baselines; explicit-time observations for retained names; forward/backward navigation; and the selected view contract. Keep counters/endpoints/deltas exact, window calculations and extrema unrounded, baseline/unavailable states explicit, and histories observation-owned. Preserve boundary-crossing intervals, reset both directions on discontinuity, reject nonpositive observation intervals without advancing statistics/history, and never fabricate catch-up columns or bridge unavailable gaps.

Cover applicable A11–A12 and S03–S12 using explicit-time fixtures through public-to-the-application monitor/view operations rather than private queues. Include the specification's numerical examples, paired reset cases, off-screen history, boundary expiry and resize-independent history. Document the view interface for screen/runtime integration and link test mappings/results in the verification record. Clock/deadline effect behavior belongs to integration, not this pure module.

## Comments

### Resolution — 2026-10-02 — alice

[Monitor](../../../src/monitor.rs) now implements the pure fixed-list statistics
module and borrowed selected view required by specification sections 3, 4 and 8.
The stable caller contract, state invariants and application/screen connection are
documented in [Monitor and selected-view interface](../monitor-interfaces.md).
Actual command outcomes, environment/base revision, case-to-test mappings and
remaining effect/rendering evidence are in the
[monitor verification record](../verification.md#monitor-run--2026-10-02).

- Construction consumes the validated ordered startup list, selection and shared
  positive directional ceilings. Initial outcomes reuse discovery data as
  baselines; later completion-timestamped rounds look up every retained original
  byte name, including off-screen/unavailable entries, without accepting additions.
- Forward/backward navigation wraps symmetrically and does not acquire, reset or
  change history. Views expose original name, one-based position/count, explicit
  baseline/measured/absent/read-failed state, independent exact totals/unrounded
  rates, chronological timestamped graph samples and ceilings.
- Exact paired u64 deltas and integer nanosecond endpoint differences are
  calculated before f64 conversion. The one-second/300-second windows weight
  interval overlap, retain the crossing interval and warm up without padding.
  Independent extrema span the entire statistics run and are never rounded.
- A decrease in either counter resets both directions to the new readable
  baseline. Absence/failure clears both rates/extrema/histories and hides totals;
  return is baseline only, never a gap delta. Equal/earlier round times leave the
  entire view unchanged. Baselines/ignored rounds add no graph samples; each
  accepted calculation adds one, regardless of delay.
- Five-minute history accumulates for never-displayed entries, expires only on
  observations and retains samples exactly at the cutoff. Read-only fitting
  suffixes hide/reveal retained history without expiring or manufacturing columns.
  Numeric rates and samples are uncapped by graph ceilings.
- **15 new explicit-time monitor tests pass**, including all specification
  numerical examples, independent directional calculations, maximum-u64 counters,
  one-byte/one-nanosecond differences above f64 precision, both decrease variants,
  absence/failure/return, actual snapshot lifecycle, navigation, off-screen history
  and exact-boundary retention. Full `cargo test --locked` passes **28 tests**;
  `cargo fmt --check` and `cargo build --release --locked` also pass. The release
  build reports 45 dead-code warnings while the foundation entry point awaits
  application wiring; no warning suppression or entry-point change was introduced.

No specification conflict, domain change or newly specifiable corrective work
surfaced. Screen rows/resize placement, decoded arrows/prompt redraw and
clock/deadline/application effects remain owned by existing tickets; pure fixture
passes do not claim complete application, PTY, emulator or live acceptance. No Git
commit was made. This closes only this implementation ticket and makes its
selected-view contract available to
[Implement the pure ASCII screen and formatting](04-ascii-screen.md).
