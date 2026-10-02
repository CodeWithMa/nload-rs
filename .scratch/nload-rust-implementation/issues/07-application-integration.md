# Integrate startup, sampling and the interactive application loop

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: application integration)
Blocked by: 02, 03, 04, 05, 06

## Question

Can the compiled application connect the completed modules through a thin single-threaded loop and production/scripted counter, clock and terminal adapters, satisfying startup order, sampling cadence, redraw fairness, backpressure, shutdown and error arbitration in sections 2–8?

Wire the actual binary entry point, informational/error streams and exit statuses, discovery snapshot/baseline reuse, fixed startup membership and completion timestamps. Anchor 500-ms deadlines to discovery completion and acquire once when overdue, advancing to the first future slot. Enforce shutdown before further work, due acquisition before bounded input/resize batches and between-batch rechecks. Navigation/resize/parser timeouts only request redraw; output backpressure must not prevent sampling or shutdown. Wire explicit cleanup, caught-panic handling and ordinary `ExitCode` return.

Use scripted counter/clock/terminal adapters to cover remaining A01–A12, S02/S06–S07/S13 and T01/T05–T13 integration requirements, including no-effect informational/argument cases, initialization ordering, long initialization, sustained input, simultaneous shutdown, partial output and failure/cleanup paths. Run all three required build commands and update evidence mappings. A runnable binary and passing scripted coverage unblock real PTY/emulator checks; they do not establish final acceptance by themselves.

## Comments

### Resolution — 2026-10-02 — alice

Implemented the actual binary startup and single-threaded application loop in
[Application](../../../src/application.rs), connected from
[Main](../../../src/main.rs). Standalone help/version and syntax failures finish
without clock/discovery/terminal work; interactive startup and execution use the
existing guarded session owner, with diagnostics emitted after restoration and
ordinary `ExitCode` return.

Discovery supplies the fixed naturally sorted membership and original baseline
observations, timestamped at completion. The loop anchors 500-ms slots to that
completion, acquires once when overdue and advances to the first future slot.
Shutdown/deadline checkpoints surround bounded input/resize/redraw work, including
inspection after acquisition; navigation and parser/resize work do not sample.
Dirty redraws coalesce through Runtime's single nonblocking frame, preserving
partially emitted frames while acquisition/input/shutdown remain available.

[Application integration fixtures](../../../src/application/tests.rs) add 18
passing tests using scripted counters, clocks, runtime effects and the new
[Scripted session adapter](../../../src/terminal/session/scripted.rs). A shared
test mutex serializes process-global panic-hook guards. Fixtures observe selected
monitor views, requested effects, emitted frames and diagnostics after cleanup,
not private statistics queues. README now documents the working invocation.

Verification: `cargo fmt --check`, `cargo test --locked` (**93 passed**) and
`cargo build --release --locked` all pass. Direct release subprocess checks pass
for redirected help/version, invalid arguments and nonterminal interactive
rejection. Startup/default/list/view checks also pass under set/unset HOME and
different locales. Exact base revision, tested source blob identities, compiler,
commands and A/S/V/T case mappings are recorded in
[Application integration run](../verification.md#application-integration-run--2026-10-02).

No new contractual decision, corrective blocker or specifiable fog surfaced.
This resolves application integration and unblocks
[Verify the real binary through automated Linux PTY checks](08-linux-pty.md).
Product PTY, recorded emulator, live-smoke and final acceptance remain pending in
their existing tickets. No commit was made.
