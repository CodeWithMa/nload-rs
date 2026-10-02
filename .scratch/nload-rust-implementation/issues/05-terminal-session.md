# Implement staged terminal ownership and shutdown cleanup

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 01

## Question

Can a direct Linux terminal session own support validation, pre-mutation signal notification, saved OS state, staged screen changes and best-effort exhaustive restoration under specification sections 6.1 and 7?

Implement terminal/TERM validation, saved termios and changed file flags, full raw mode, nonblocking ownership, atomic signal flags plus a nonblocking wake pipe, resize dimensions and unregister/FD lifetimes. Establish session ownership and notification before mutation. Record attempted alternate-screen/cursor operations even after partial writes; restore every applicable state despite individual errors, with a nonpanicking drop fallback. Keep unsafe calls in concrete adapters and expose narrow scripted operations for failure verification.

Provide the cleanup/exit arbitration and caught-panic boundary required by the specification, including restoring the prior panic hook and reporting only after applicable restoration. The integration ticket wires the boundary around initialization/execution. Cover applicable deterministic T01–T02, T06 and T09–T13, including partial initialization, simultaneous flags and multiple restoration failures. Document the session interface and evidence; real PTY checks are separately required.
