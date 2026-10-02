# Integrate startup, sampling and the interactive application loop

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 02, 03, 04, 05, 06

## Question

Can the compiled application connect the completed modules through a thin single-threaded loop and production/scripted counter, clock and terminal adapters, satisfying startup order, sampling cadence, redraw fairness, backpressure, shutdown and error arbitration in sections 2–8?

Wire the actual binary entry point, informational/error streams and exit statuses, discovery snapshot/baseline reuse, fixed startup membership and completion timestamps. Anchor 500-ms deadlines to discovery completion and acquire once when overdue, advancing to the first future slot. Enforce shutdown before further work, due acquisition before bounded input/resize batches and between-batch rechecks. Navigation/resize/parser timeouts only request redraw; output backpressure must not prevent sampling or shutdown. Wire explicit cleanup, caught-panic handling and ordinary `ExitCode` return.

Use scripted counter/clock/terminal adapters to cover remaining A01–A12, S02/S06–S07/S13 and T01/T05–T13 integration requirements, including no-effect informational/argument cases, initialization ordering, long initialization, sustained input, simultaneous shutdown, partial output and failure/cleanup paths. Run all three required build commands and update evidence mappings. A runnable binary and passing scripted coverage unblock real PTY/emulator checks; they do not establish final acceptance by themselves.
