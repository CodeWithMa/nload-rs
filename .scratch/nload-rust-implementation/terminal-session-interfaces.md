# Terminal session and exit interface

Owner: [Implement staged terminal ownership and shutdown cleanup](issues/05-terminal-session.md).
Authority: specification sections 2.3, 6.1, 7 and 8. Implementation is under
`src/terminal/`, re-exported by `src/terminal.rs`. These are private application
interfaces, not a public library.

## Caller-facing interface

- `LinuxTerminal::new()` creates an inert effect adapter: no descriptor,
  environment, clock or terminal operations. `Session::new(adapter)` establishes
  the owner before any fallible initialization.
- `guarded(&mut session, execute)` runs the startup/interactive closure with a
  temporary nonprinting panic hook, catches unwinding, performs exhaustive
  session cleanup, restores the previous hook, and returns an `ExitReport`.
  Use it once from the single-threaded entry point; hooks are process-global.
- The closure returns `Result<Shutdown, Failure>`.
  `Shutdown::{Quit, Interrupt, Terminate}` choose 0, 130 and 143.
  `Failure::argument(String)` chooses 2 for startup selection errors;
  `From<String>` and `From<io::Error>` choose 1 for initialization/runtime faults.
- `Session::initialize() -> io::Result<(usize, usize)>` runs validation, state
  capture, notification preparation, raw/nonblocking mutation, dimensions and
  alternate-screen/cursor setup in that order. The pair is `(width, height)`.
  It is single-use, including failed initialization; a second call is an error.
- `Session::dimensions()` reads fresh Linux dimensions. Small or zero dimensions
  are valid data for Screen, not session failures.
- `Session::notifications()` drains nonblocking wake tokens *before* reading
  sticky INT/TERM flags and consuming the resize flag. It returns
  `Notifications { shutdown: Option<Shutdown>, resized: bool }`. Both shutdown
  flags pending selects Interrupt, independently of byte contents/count/arrival
  order. Call before work and after wakeups; a stale/spurious wake is harmless.
- `Session::cleanup() -> Vec<String>` explicitly attempts every applicable
  restoration and returns errors. It is nonpanicking and idempotent: each attempt
  consumes its stage even if it fails. `guarded` calls it automatically; Drop
  supplies the same best-effort fallback if explicit cleanup was skipped.
- `ExitReport { status: u8, diagnostics: Vec<String> }` is available only after
  cleanup and prior-hook restoration. The application writes diagnostics to
  stderr then returns `ExitCode::from(report.status)` normally. Cleanup-only
  failure promotes successful Quit to 1; otherwise it preserves the chosen
  status and adds cleanup diagnostics after the primary error.

## Application ordering

Parse OS arguments and finish Help/Version before clock/discovery/terminal work.
Construct the inert session owner, then put fallible clock initialization,
discovery, selection, session initialization and execution inside `guarded`.
Initialize the session only after successful nonempty discovery/selection:

```rust
let mut session = terminal::Session::new(terminal::LinuxTerminal::new());
let report = terminal::guarded(&mut session, |session| {
    // Initialize BOOTTIME, acquire/timestamp discovery and select startup list.
    // Map a missing requested name to Failure::argument; other failures to 1.
    let (width, height) = session.initialize()?;
    // Construct/use the monitor with the discovery baselines, then run the loop.
    // Runtime returns Quit/Interrupt/Terminate; fatal effects return Failure.
    let _ = (width, height);
    Ok(terminal::Shutdown::Quit)
});
// Report diagnostics to stderr here and return ExitCode normally.
```

The sketch is an ordering illustration, not application integration evidence.
No diagnostic or default panic output is emitted by the session module.

## Hidden implementation invariants

- Validate terminal stdin and stdout independently. Stderr is never tested or
  modified. TERM is raw OS data: unset/empty/exact `dumb`/exact `linux` reject;
  other values are declarations of assumed compatibility, not probes.
- Duplicate stdin/stdout with `F_DUPFD_CLOEXEC`; retain their owned descriptor
  lifetimes. Duplicates share original open-file descriptions, so snapshot both
  original file-status flags **before either is changed**. Already-nonblocking
  descriptions are preserved without claiming a flag mutation.
- Capture input termios; use `cfmakeraw`, `VMIN=1`, `VTIME=0`, `TCSANOW` and
  nonblocking input/output. ISIG is disabled so Ctrl+C/Ctrl+Z are input bytes.
- Store signal-pipe ownership before any registrations; register each atomic
  flag action before its wake action for SIGINT/SIGTERM/SIGWINCH. Both pipe ends
  are nonblocking and close-on-exec at creation. The three signal-hook Unix pipe
  writers own duplicated FDs. Registration errors retain earlier registrations
  for cleanup. Handlers only set atomics and write nonblocking wake tokens.
- Record mutation attempts before syscalls and before each screen/cursor write.
  Enter uses `ESC[?1049h`; hide uses `ESC[?25l`. Short/interrupted writes resume
  exactly; setup does not block waiting for output capacity. A would-block/zero
  or unrecoverable setup write fails initialization and triggers cleanup.
- Cleanup attempts cursor show, alternate-screen leave, saved termios, changed
  input flags, changed output flags, and notification release. Commands start
  with CAN to cancel any abandoned partial control sequence. Screen writes stay
  nonblocking, with at most 100 ms of best-effort retry time **per command**;
  timeout/error does not prevent subsequent restorations. This is a private
  cleanup bound, not a new product latency requirement. `Instant` measures only
  this finite restoration budget, never observations or sampling deadlines.
- No blocking `tcdrain`, no early `process::exit`, and no terminal output from
  handlers. Notification release unregisters every owned action before pipe
  descriptors close. Terminal duplicates close when the session owner drops.
  Signal-hook unregisters actions but does not restore prior OS signal handlers;
  the normally exiting application does not reuse that process for another UI.
- Cleanup catches even an adapter panic per operation, continues other attempts
  and reports it. Unknown panic payloads may have panicking destructors, so
  caught payloads are deliberately forgotten rather than dropped during cleanup.
  Unwritable terminals still have only best-effort screen restoration.

## Internal seam and next runtime ticket

`session::SessionOps` is an internal effect seam used by the Linux adapter and
the failure-script adapter. State capture, raw/flags, notification setup/release,
write/writability, dimensions and inspection are effect operations; callers use
the staged Session interface instead of coordinating them. Its types remain
inside the private session module.

The sibling runtime implementation can use `Session::effects_mut()` and the
Linux adapter's lifetime-borrowed input/output/signal FDs. No raw descriptor is
handed to the application. Add event/clock/output behavior through the owning
[Implement incremental input and nonblocking runtime effects](issues/06-terminal-events.md)
ticket. Setup/cleanup sequence writes are separate from its resumable application
frame handling; this implementation does not establish frame scheduling,
BOOTTIME/timerfd, input decoding, fairness or product PTY acceptance.

## Verification

Actual tests/commands and acceptance gaps are recorded in
[Terminal session run](verification.md#terminal-session-run--2026-10-02).
The scripted tests cross Session initialization/cleanup and `guarded`; Linux
adapter checks use a private PTY and an isolated signal-test subprocess. Neither
substitutes for running the integrated product through its required PTY and
human-recorded emulator campaigns.
