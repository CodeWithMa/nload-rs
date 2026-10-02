# Implement incremental input and nonblocking runtime effects

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: incremental input and nonblocking runtime)
Blocked by: 05

## Question

Can the terminal/runtime provide bounded incremental input decoding, checked BOOTTIME readings/timerfd waiting, normalized events and resumable frame output without hiding blocking work from the application loop, as required by sections 4.2, 6 and 8?

Implement fragmented CSI/SS3 input, unknown-sequence consumption, quit controls and bounded idle-timeout behavior; owned poll/timerfd/signal-pipe readiness; drain-before-flag signal inspection; checked integer nanosecond clock conversion; and pending-frame nonblocking partial writes. Only an untouched frame is replaceable; partially emitted output must finish before a later frame. EOF/hangup/unrecoverable I/O and clock failures are fatal effects, not counter unavailability.

Build on the terminal session interface and provide scripted runtime/clock/output effects for integration. Cover applicable deterministic S13, T03–T06 and T08 plus clock/FD error cases; test fragmentation, timeout boundaries and backpressure meaningfully. Application shutdown-first fairness, deadline advancement and sampling integration remain owned by the next ticket. Record interfaces and exact coverage in the verification record.

## Comments

### Resolution — 2026-10-02 — alice

[Terminal/runtime](../../../src/terminal.rs) now exposes `Clock`, `LinuxClock`,
normalized `Event`/`Ready`, `Runtime<RuntimeEffects>` and
`session.runtime(&mut clock)`. The implementation is in
[Incremental input](../../../src/terminal/input.rs),
[Runtime and frame ownership](../../../src/terminal/runtime.rs) and
[Concrete Linux clock/readiness effects](../../../src/terminal/linux/runtime.rs).
Shared [scripted clock/runtime effects](../../../src/terminal/runtime/scripted.rs)
are available for application integration.

Caller construction, lifetime/ordering requirements and bounded-batch semantics
are recorded in [Terminal runtime and clock interface](../terminal-runtime-interfaces.md).
The [runtime verification record](../verification.md#terminal-runtime-run--2026-10-02)
records the actual base revision/compiler, command outcomes, all passing test
names, exact acceptance coverage and remaining integrated evidence.

- Decodes all unmodified CSI/SS3 arrows incrementally and in order. Consumes
  modified/unknown escape/control-string sequences, ignores excluded aliases and
  Ctrl+Z, and normalizes plain q/Q and immediate raw Ctrl+C shutdown controls.
  Parser state stores no sequence-byte buffer, caps length tracking, discards
  overlong sequences and expires at exactly 100 ms of idle time using a wait
  deadline. Reads are one nonblocking 256-byte batch, never a sequence-completion
  wait.
- Owns nonblocking/CLOEXEC BOOTTIME timerfd and checked integer timespec conversion;
  rejects clock failures/regression/overflow without fallback. Poll borrows owned
  session input/output/signal FDs, requests output only when pending, treats timer
  counts as wakes, and preserves drain-before-flag signal inspection. EOF, hangup,
  invalid descriptors and unrecoverable effects are fatal results.
- Owns exactly one frame. EAGAIN leaves untouched frames replaceable; emitted
  frames reject replacement and resume exact suffixes through short writes/EINTR.
  Output is one nonblocking syscall of at most 16 KiB per batch; newer state stays
  as the application's redraw request until the current frame finishes. Bounded
  ASCII frame encoding positions rows without CR/LF or autowrap-mode changes.
- **16 new checks pass** (10 deterministic runtime fixtures and 6 native Linux
  adapter checks); **all 75 tests pass** under `cargo test --locked`.
  `cargo fmt --check`, `cargo build --release --locked` and `git diff --check` pass.
  Native checks cover actual timerfd readiness, nonblocking output backpressure,
  simultaneous input/signal wake, EOF, FD hangup/errors and conversion boundaries.
  Compiler unused-interface warnings reflect pending application wiring.

No new product/domain decision, specification conflict or concrete corrective
ticket surfaced. The now-unblocked
[Integrate startup, sampling and the interactive application loop](07-application-integration.md)
owns shutdown-first fairness, acquisition completion timestamps, deadline
advancement, redraw coalescing and sampling under backpressure. Product PTY,
recorded emulator/live smoke and final acceptance remain required by their
existing tickets. No Git commit was made.
