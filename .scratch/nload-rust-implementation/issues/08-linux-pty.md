# Verify the real binary through automated Linux PTY checks

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 07

## Question

Does the real compiled binary satisfy the Linux terminal/FD/input/signal/restoration aspects of the acceptance matrix when exercised by an automated PTY harness?

Implement and run repeatable Linux PTY tests for applicable V05 and T01–T04/T06/T09–T11. Observe saved termios and file flags, emitted session/restoration sequences, fragmented/repeated arrows, typed versus external controls, signal statuses, resize/small startup and EOF/hangup/fault behavior where practical. Test independent stdin/stdout support and rejected TERM declarations before mutation. Preserve an observer for shared file-status flags where needed; do not mistake a closed child FD for proof of restored flags.

Exercise real harness-controlled failures where practical and document specific limitations with the corresponding deterministic evidence. No production test-only CLI flags are authorized. Correct concrete implementation defects exposed by these checks; if corrective work exceeds this ticket, create a precise blocker and retain a pending status until the required checks pass. Integrate repeatable automated checks with the locked test gate, record environment/test names/outcomes, and distinguish PTY protocol evidence from still-pending visible emulator checks.
