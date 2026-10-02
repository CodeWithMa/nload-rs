# Terminal runtime and clock interface

Owner: [Implement incremental input and nonblocking runtime effects](issues/06-terminal-events.md).
Authority: specification sections 4.2, 6 and 8. Session ownership/cleanup remains
described by [Terminal session and exit interface](terminal-session-interfaces.md).
All interfaces below are private application interfaces re-exported by
`src/terminal.rs`.

## Clock and lifetime

- `Clock::now() -> io::Result<u64>` returns integer BOOTTIME nanoseconds.
  `LinuxClock::new()` creates an owned nonblocking, close-on-exec BOOTTIME timerfd
  and validates an initial clock reading. Do this inside `guarded`, before initial
  counter acquisition. Read the clock again **after acquisition completes** and
  use that timestamp for the discovery baselines and first deadline anchor.
- Linux timespec fields, multiplication/addition and reverse timer conversion
  are checked. Negative/invalid/overflowing readings and backwards clock movement
  fail; equal readings are allowed. There is no wall-clock/`Instant` fallback.
- After `session.initialize()`, `session.runtime(&mut clock)` returns
  `Runtime<LinuxRuntime<'_>>`, borrowing both owners exclusively. No FD escapes to
  the application. The runtime must go out of scope before session cleanup; Rust
  enforces the borrowing lifetime. Its drop abandons frame/decoder state only;
  the Session performs applicable cancel-prefixed screen/OS restoration.
- `Runtime::new(effects)` is inert and is also the constructor used by scripted
  integration tests. `Runtime` implements `Clock`, so completion timestamps during
  interactive execution come from `runtime.now()`.

## Normalized effects and batches

- `runtime.notifications()` retains the existing drain-before-flags contract:
  returns sticky INT/TERM shutdown (INT wins if both are pending) and consumes
  resize independently. Tokens are wakes, never signal identities.
- `runtime.dimensions()` reads fresh `(width, height)`; tiny/zero values are valid.
- `runtime.wait(sample_deadline)` arms an **absolute one-shot** BOOTTIME timerfd
  and polls the session's input, signal pipe, timer and output descriptors.
  `Ready { input, signals, timer, output }` contains readiness hints, not commands
  or an acquisition count. Timer counts are consumed and ignored. Input/signal/
  timer monitoring continues under output backpressure; POLLOUT is requested only
  with pending output. Errors/hangups/invalid FDs on any of these are fatal even
  when output interest is disabled.
- Decoder inactivity is another wait deadline: remaining time determines the poll
  timeout, rounded upward to milliseconds (large values capped to an earlier
  harmless wake). An overdue decoder timeout requests a zero-time poll. An
  interrupted poll yields empty readiness so the loop can recheck shutdown/time.
- `runtime.input_batch(ready.input)` checks BOOTTIME and expires idle parser state,
  then performs **at most one nonblocking read of 256 bytes** if readable. Calling
  it with `false` still processes a timeout without reading input. It returns
  ordered `Event::{Forward, Backward, Shutdown(Shutdown)}` values; CSI/SS3 Right/
  Down map forward, Left/Up backward. Apply in order and stop at shutdown.
- q/Q quits only in plain input. Typed Ctrl+C selects Interrupt immediately,
  including during an incomplete sequence. Ctrl+Z, excluded aliases and modified
  keys are ignored. CSI/SS3 unknown finals and ordinary ESC sequences are consumed;
  OSC/DCS/SOS/PM/APC payloads are consumed through their terminators. No suffix is
  re-fed as a plain command.
- Decoder state stores **no sequence-byte buffer**: a capped length/validity bit
  and state recognize only unmodified three-byte arrows. Overlong sequences stay
  in discard mode until a terminator or **100 ms without additional input**. New
  fragments reset the idle deadline; at/exceeding the boundary expires first.
  Deadline overflow and clock regressions fail instead of silently changing time.
- EAGAIN/EINTR on a read/write yields immediately to the loop. Input EOF, zero
  writes, unrecoverable effects and invalid adapter byte counts are fatal I/O
  errors, suitable for propagation through `Failure` and `guarded` restoration.

## One resumable output frame

- `runtime.queue_frame(&rows, width, height) -> io::Result<bool>` validates bounded
  printable ASCII rows and owns their terminal encoding. `true` accepts/replaces
  an untouched frame. `false` means some byte of the existing frame was emitted:
  keep the application's latest redraw request dirty and try again after it
  finishes. No second/stale frame is retained inside the runtime.
- Encoding clears the screen and uses absolute row positions, trims trailing
  spaces and finishes at Home. It emits no CR/LF, does not toggle autowrap modes,
  and places no printable bytes beyond the supplied dimensions. Absolute positions
  cancel delayed right-margin wrap; emulator evidence remains separately required.
- `runtime.output_batch()` performs **at most one nonblocking write of 16 KiB**.
  It resumes the exact suffix after short writes, retains it after EAGAIN/EINTR,
  and never waits solely for output. `has_pending_output()` becomes false only
  after the complete frame has been emitted (or no frame was queued).
- Frame encoding/validation does not sample or render the monitor. Application
  redraw requests should coalesce to the latest monitor state. After a started
  frame finishes, render that latest state rather than caching intermediate views.

## Application connection

The loop must inspect notifications **before work**, process due sampling before
bounded input/resize batches, coalesce redraw requests, do bounded ready output,
and recheck shutdown/deadlines between batches. Inspect notifications after every
wake, including EINTR, independently of `Ready.signals`. For simultaneous wake
faults and signals, inspect pending shutdown before choosing a runtime-failure
exit. Read `now()` after counter acquisition completes. Compute and check the next
first-future 500-ms deadline in the application; readiness/expiration counts must
never manufacture observations. Timeout/resize/navigation never acquire counters.

Production construction inside the guarded closure:

```rust
let mut clock = terminal::LinuxClock::new()?;
// Initial acquisition; completion timestamp; startup selection and monitor.
let (width, height) = session.initialize()?;
let mut runtime = session.runtime(&mut clock);
// Thin loop uses runtime.now(), notifications(), wait(), input/output batches.
```

This sketch is the connection contract; application fairness, deadline advancement
and integrated counter/screen/exit assertions belong to
[Integrate startup, sampling and the interactive application loop](issues/07-application-integration.md).

## Scripted effects and evidence

`RuntimeEffects: Clock` is the effect seam: wait, notification inspection,
dimensions, read and write. LinuxRuntime supplies it with borrowed owners.
`terminal::scripted::{ScriptClock, ScriptedEffects, Call}` provides shared handles
for deterministic integration: explicit clock readings/errors, readiness/errors,
input chunks, output short-write/errors, notification/dimension results, effect
logs and actually emitted bytes. Clones share scripted external state, not
Runtime's decoder/frame internals. Missing input yields EAGAIN, missing output
results accept the full supplied batch, and missing waits yield empty readiness.

Passing test names and remaining evidence are indexed in
[Runtime verification](verification.md#terminal-runtime-run--2026-10-02).
