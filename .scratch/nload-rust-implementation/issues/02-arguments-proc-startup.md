# Implement arguments, proc snapshots and startup ordering

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: arguments and proc snapshots)
Blocked by: 01

## Question

Can the Arguments and Proc sampler modules supply validated invocation outcomes, complete original-byte counter snapshots and deterministic startup selection under specification sections 2, 3.1, 4.1 and 8?

Implement the strict `args_os()` grammar, independent ceiling validation/conversion, Cargo-authoritative help/version outcomes, raw-byte identity, unbounded natural-digit comparison and selection validation. Implement fresh whole-file proc acquisition through EOF and complete checked snapshot parsing, preserving the distinction between global failure and absence in a valid snapshot. Keep terminal/clock work out of argument parsing and counter acquisition timestamps under the application/clock ownership specified by the handoff.

Provide meaningful deterministic coverage for applicable A01–A09, S01–S02 and raw-name/lifecycle acquisition cases. Initial startup control-flow/no-effect assertions are completed by the integration ticket, not assumed from parser tests. Document the caller-facing configuration and snapshot interfaces for the monitor/application consumers, and update the case-to-evidence record with precise coverage and remaining integration gaps. Resolution records implemented interfaces and passing appropriate locked tests.

## Comments

### Resolution — 2026-10-02 — alice

The Arguments and Proc sampler modules now supply the required validated
invocation, complete snapshot and pure startup-selection contracts. Detailed
caller ownership/invariants are in [Arguments and proc snapshot interfaces](../arguments-proc-interfaces.md);
actual commands, environment, case-to-test mappings and integration gaps are in
the [arguments and proc verification record](../verification.md#arguments-and-proc-run--2026-10-02).

- [Arguments](../../../src/arguments.rs): `parse(args_os().skip(1))` returns
  `Help`, `Version`, `Run(RunConfig)` or `ArgumentError`. It preserves exact OS
  name bytes, handles `--` and the strict one-name/separate-token grammar,
  validates every ceiling occurrence with checked decimal/conversion arithmetic,
  and applies independent defaults/zero resets/last-wins behavior. Help/version
  text uses Cargo identity/version metadata. No runtime config/effect I/O occurs.
- [Proc sampler](../../../src/proc_sampler.rs): `CounterSource` has production
  `ProcSampler` and test-only `ScriptedCounters` adapters. Production opens
  `/proc/net/dev` afresh, reads through EOF into a reusable growable buffer and
  returns only a fully validated `Snapshot` or global `AcquisitionError`.
  Header/row/name/field/duplicate validation and exact checked u64 RX/TX parsing
  prevent partial salvage. `counter_outcome` preserves readable/absent/read-failed
  distinctions for each retained name without emitting diagnostics.
- `Snapshot::startup` includes all discovered names, uses unbounded numeric-run
  natural sorting with full-byte ties, validates exact initial selection and
  distinguishes empty discovery from a missing requested name. It borrows the
  snapshot so discovery counters can supply monitor baselines without reacquiring.
  Completion timestamps and deadlines remain application/clock-owned.
- `cargo fmt --check`, `cargo test --locked` (**13 passed**) and
  `cargo build --release --locked` pass on rustc 1.98.1 / Cargo 1.98.0,
  `x86_64-unknown-linux-gnu`. The release build reports 27 dead-code warnings
  because these interfaces await the application caller. All 13 tests also pass
  with HOME unset/C locale and different HOME/en_US.utf8 locale. Source and
  supporting-document whitespace checks produced no diagnostics.
- The verification ledger covers applicable A01–A09/S01–S02 and acquisition-level
  A12, identifying remaining monitor/integration obligations precisely. The
  entry point is still the foundation diagnostic: application stream/status
  dispatch, no-effect assertions, startup effect ordering, completion timestamps,
  baseline/deadline reuse, navigation ceiling scope and nonfatal runtime reset/UI
  behavior are not inferred from these module tests.

No domain/specification conflict or newly specifiable fog surfaced. Existing
[Implement the pure monitor and statistics runs](03-monitor-statistics.md) and
[Integrate startup, sampling and the interactive application loop](07-application-integration.md)
own the remaining consumers/checks; no corrective child is needed. No dependency
changes or Git commit were made.
