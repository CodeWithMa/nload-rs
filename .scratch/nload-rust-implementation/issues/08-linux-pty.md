# Verify the real binary through automated Linux PTY checks

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: product Linux PTY verification)
Blocked by: 07

## Question

Does the real compiled binary satisfy the Linux terminal/FD/input/signal/restoration aspects of the acceptance matrix when exercised by an automated PTY harness?

Implement and run repeatable Linux PTY tests for applicable V05 and T01–T04/T06/T09–T11. Observe saved termios and file flags, emitted session/restoration sequences, fragmented/repeated arrows, typed versus external controls, signal statuses, resize/small startup and EOF/hangup/fault behavior where practical. Test independent stdin/stdout support and rejected TERM declarations before mutation. Preserve an observer for shared file-status flags where needed; do not mistake a closed child FD for proof of restored flags.

Exercise real harness-controlled failures where practical and document specific limitations with the corresponding deterministic evidence. No production test-only CLI flags are authorized. Correct concrete implementation defects exposed by these checks; if corrective work exceeds this ticket, create a precise blocker and retain a pending status until the required checks pass. Integrate repeatable automated checks with the locked test gate, record environment/test names/outcomes, and distinguish PTY protocol evidence from still-pending visible emulator checks.

## Comments

### Resolution — 2026-10-02 — alice

Added [Linux product PTY integration tests](../../../tests/linux_pty.rs): eleven
repeatable Linux-only tests included automatically in `cargo test --locked`,
spawning the actual Cargo-built executable. The test-harness-only binary override
also ran the complete suite against `./target/release/nload-rs` directly.

The tests observe full raw mode, input/output nonblocking flags, retained shared
OFD flags after child exit, defined saved termios fields, actual bounded ASCII
row-address protocol, alternate-screen/cursor entry and cleanup sequences. They
exercise independent stdin/stdout terminals and their independent rejection,
shared-terminal/OFD and already-nonblocking variants, rejected TERM and exempt
help/version invocations, fragmented/repeated CSI/SS3 arrows and ignored keys,
small startup/resize/sampling/navigation/quit, normal typed/external exits and
queued INT/TERM arbitration.

Practical real faults cover read-only terminal stdout (EBADF), flow-stopped setup
output (EAGAIN), input/output master hangups separately, canonical line-discipline
EOF, and large-frame output backpressure without draining. Failed restoration on
disconnected/unwritable terminals is diagnosed; other applicable restorations
still occur. Cleanup-only failure changes q to status 1 while INT/TERM retain
130/143. No production defect or corrective blocker surfaced.

`cargo fmt --check`, `cargo test --locked` (**104 passed: 93 existing + 11 PTY**),
and `cargo build --release --locked` pass. The direct release PTY repetition also
passes **11/11**. The existing Session convenience-method dead-code warning is
unchanged. README documents the integrated gate and release repetition.

[Product Linux PTY run](../verification.md#product-linux-pty-run--2026-10-02)
records the exact base revision, tested harness/source and executable identities,
compiler/kernel/environment, commands, individual case mappings and harness
development corrections. It explicitly maps impractical partial-setup/stage,
clock/additional syscall, internal full-signal-pipe and inter-stream diagnostic
ordering cases to passing deterministic/native evidence under the agreed
practicality rule.

This resolves the required practical P evidence for V05 and T01–T04/T06/T09–T11.
No new decision or specifiable fog emerged. It unblocks
[Record emulator behavior and nonprivileged live traffic smoke](09-emulator-live-smoke.md),
whose human-recorded V05/T14 emulator observations and D04 live traffic evidence
remain pending, along with final artifact/revision acceptance. PTY streams do not
establish visible screen/cursor behavior. No commit was made.
