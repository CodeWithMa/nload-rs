# Dynamic interface discovery — implementation record

Date: 2026-10-08
Branch: `dev`
Review baseline: `54ca32591cc83b1538b05d466bbf84f8bc948425` (confirmed by user)

Specification: [Dynamic interface discovery and one-time startup-request selection](spec.md).
Tickets: [Discover new interfaces while preserving selection](issues/01-discover-new-interfaces.md)
and [Honor an absent startup interface request once](issues/02-honor-startup-request-once.md).

## Delivery

- Complete validated snapshots supply discovery names to the existing pure
  monitor ingestion boundary as well as paired counter outcomes. Successful
  rounds insert unseen original-byte names with the shared natural comparator.
  Selection tracks the same entry across index changes. Existing entries and
  their statistics/history are retained; new entries begin with baselines.
- Startup selection carries only an absent exact-byte request into the monitor.
  The first accepted matching observation selects its entry and permanently
  consumes the request. Navigation, unrelated additions and acquisition failures
  leave a pending request intact; honored requests never rearm.
- README, help output and module contract comments describe dynamic discovery,
  retained unavailable entries and one-time delayed selection. The README links
  the superseding specification; historical specifications are preserved.

## Test seams and TDD evidence

Primary acceptance remains the approved scripted application-session seam in
`src/application/tests.rs`, using argument handling, startup and the real loop
with scripted acquisition, clock, input, output and terminal lifecycle effects.
Only test-side selected-view capture was extended (position and both directions'
totals, rates and graph history).

- Discovery tracer: the new navigation test failed with the old position/count
  `(1, 2)` instead of `(4, 8)`, then passed after runtime ingestion incorporated
  snapshot names.
- Delayed-request tracer: after ensuring the startup frame was flushed by the
  scripted output adapter, the test failed with `lo` selected instead of `wg0`,
  then passed after adding pending-request consumption.
- Acceptance covers multiple additions and row order, natural numeric ties,
  all four arrows and wrapping, zero-counter/loopback entries, statistics
  continuity, off-screen new-interface observations, both graph ceilings,
  absence/rename/empty-round/return lifecycles, exact-byte/case/escaped/leading-dash
  selection, silent fallback, pending failure recovery, all-or-nothing malformed
  snapshots, startup- and runtime-honored requests, repeated returns, input and
  shutdown ordering, slow acquisition scheduling, and empty-startup diagnostics.
- On 2026-10-08 the user additionally approved extending the existing pure
  monitor ignored-observation regression. Normal sessions reject backward clock
  reads before ingestion, so this existing seam verifies that equal/earlier
  timestamps cannot add entries, alter statistics or consume the request; a later
  accepted appearance must still fulfill it. Application-session coverage also
  verifies the runtime's existing backward-clock failure and restoration.

## Verification

| Command | Result |
| --- | --- |
| `cargo check --locked --all-targets` | Passed regularly during both slices and final verification. |
| `cargo test --locked application::tests` | Passed regularly; final focused run: 32 tests. |
| `cargo test --locked monitor::tests` | Passed: 15 tests. |
| `cargo test --locked proc_sampler::tests` | Passed: 8 tests. |
| `cargo fmt --check` | Passed. |
| `cargo clippy --locked --all-targets` | Passed with the six existing warning sites listed below. |
| `cargo test --locked` | Passed once at final integration: 125 unit tests and 16 Linux PTY integration tests. |
| `git diff --check` | Passed. |

The additional strict probe `cargo clippy --locked --all-targets -- -D warnings`
fails on unchanged code at these existing warning sites:

- `src/monitor.rs`: large `EntryState` enum variant.
- `src/screen.rs` and `src/terminal/linux.rs`: manual `is_multiple_of` patterns.
- `src/application/tests.rs`: existing `Tick::on_wait` callback type complexity.
- `src/terminal/runtime/tests.rs` and `tests/linux_pty.rs`: byte-character arrays.

## Review

Two independent review agents examined the complete working-tree implementation
against the confirmed baseline using `git diff 54ca325 --` (review precedes the
requested commit, so the three-dot committed comparison was initially empty).

### Standards

No documented-standard violations. One low-priority judgement-call finding:
two seven-field positional acceptance-test tables were hard to read (possible
Data Clumps). Replaced these tables with test-local named-field `Case` structs.
The standards reviewer confirmed resolution with no remaining or new findings.

### Spec

No findings: no missing/partial requirements, scope creep or incorrect behavior
identified against the specification and both tickets.

Summary: Standards — one low-priority finding resolved, zero remaining;
Spec — zero findings. After the test-only review refactor, all 32 application
tests, formatting, all-target typechecking, normal Clippy and whitespace checks
passed again. Production code is unchanged since the full-suite run.
