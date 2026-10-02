# Implement incremental input and nonblocking runtime effects

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 05

## Question

Can the terminal/runtime provide bounded incremental input decoding, checked BOOTTIME readings/timerfd waiting, normalized events and resumable frame output without hiding blocking work from the application loop, as required by sections 4.2, 6 and 8?

Implement fragmented CSI/SS3 input, unknown-sequence consumption, quit controls and bounded idle-timeout behavior; owned poll/timerfd/signal-pipe readiness; drain-before-flag signal inspection; checked integer nanosecond clock conversion; and pending-frame nonblocking partial writes. Only an untouched frame is replaceable; partially emitted output must finish before a later frame. EOF/hangup/unrecoverable I/O and clock failures are fatal effects, not counter unavailability.

Build on the terminal session interface and provide scripted runtime/clock/output effects for integration. Cover applicable deterministic S13, T03–T06 and T08 plus clock/FD error cases; test fragmentation, timeout boundaries and backpressure meaningfully. Application shutdown-first fairness, deadline advancement and sampling integration remain owned by the next ticket. Record interfaces and exact coverage in the verification record.
