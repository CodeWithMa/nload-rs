# Implement staged terminal ownership and shutdown cleanup

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: terminal ownership and shutdown cleanup)
Blocked by: 01

## Question

Can a direct Linux terminal session own support validation, pre-mutation signal notification, saved OS state, staged screen changes and best-effort exhaustive restoration under specification sections 6.1 and 7?

Implement terminal/TERM validation, saved termios and changed file flags, full raw mode, nonblocking ownership, atomic signal flags plus a nonblocking wake pipe, resize dimensions and unregister/FD lifetimes. Establish session ownership and notification before mutation. Record attempted alternate-screen/cursor operations even after partial writes; restore every applicable state despite individual errors, with a nonpanicking drop fallback. Keep unsafe calls in concrete adapters and expose narrow scripted operations for failure verification.

Provide the cleanup/exit arbitration and caught-panic boundary required by the specification, including restoring the prior panic hook and reporting only after applicable restoration. The integration ticket wires the boundary around initialization/execution. Cover applicable deterministic T01–T02, T06 and T09–T13, including partial initialization, simultaneous flags and multiple restoration failures. Document the session interface and evidence; real PTY checks are separately required.

## Comments

### Resolution — 2026-10-02 — alice

[Terminal/runtime](../../../src/terminal.rs) now exposes a staged `Session`,
inert `LinuxTerminal` adapter and `guarded` startup/execution boundary. The
implementation is in [Session ownership](../../../src/terminal/session.rs),
[Linux effects](../../../src/terminal/linux.rs), and
[scripted failure fixtures](../../../src/terminal/session/tests.rs).
The caller contract and downstream runtime connection are recorded in
[Terminal session and exit interface](../terminal-session-interfaces.md).
Actual revision/compiler, command outcomes, case-to-test mappings and remaining
application/PTY evidence are in the
[terminal session verification record](../verification.md#terminal-session-run--2026-10-02).

- Validates interactive stdin/stdout and raw TERM before snapshot/mutation;
  captures saved input termios and both original file-status flags before changes.
  Owns close-on-exec descriptor duplicates, full raw mode and nonblocking input/
  output. Zero/small sizes are valid; fresh dimensions are available for resize.
- Prepares owned atomic SIGINT/SIGTERM/SIGWINCH actions and a nonblocking pipe
  before terminal changes, with flag-before-wake registration order. Inspection
  drains tokens before flags, treats bytes only as wakes, gives simultaneous INT
  priority and consumes resize independently. Partial registration is owned for
  unregistration; signal/terminal descriptor lifetimes are explicit.
- Records raw/flag/screen/cursor attempts before effects, including short writes.
  Cleanup attempts cursor show, screen leave, termios, each changed flag and
  signal release despite earlier errors. Screen restoration cancels abandoned
  escape sequences and keeps finite nonblocking retries; it cannot indefinitely
  wait behind output backpressure. Drop is idempotent and nonpanicking, including
  a second adapter panic during unwinding.
- `guarded` suppresses default panic output during startup/execution/cleanup,
  catches unwinding, restores all applicable states and the previous hook, then
  returns diagnostics/status for application reporting. It represents Quit/INT/
  TERM as 0/130/143, pre-session selection errors as 2 and failures/panics as 1.
  Cleanup-only errors promote 0 to 1 while preserving an already chosen status.
- **19 new tests pass**, and all **59 tests** pass under `cargo test --locked`.
  `cargo fmt --check` and `cargo build --release --locked` pass. The native checks
  additionally verify an owned PTY's actual raw/flags/dimensions round trip, full
  wake-pipe behavior, and actual registered signals/unregistration/FD release in
  an isolated test subprocess. Release warnings are unused interfaces awaiting
  application/runtime wiring, not failed checks.

No specification conflict, domain change or concrete corrective ticket surfaced.
The caught-panic boundary still needs the application wiring owned by
[Integrate startup, sampling and the interactive application loop](07-application-integration.md).
Input decoding, BOOTTIME/timerfd, polling and resumable application frame output
remain with [Implement incremental input and nonblocking runtime effects](06-terminal-events.md),
now unblocked. Real integrated product PTY, emulator and live-smoke acceptance
remain required by their existing tickets. No Git commit was made.
