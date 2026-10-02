# Decide the port's architecture and test seams

Status: closed
Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: architecture/test seams)
Blocked by: 02, 03, 04, 05, 06, 09

## Question

Which Linux sampling backend, terminal approach, and module interfaces best implement the agreed narrow behavior within the engineering constraints? Decide ownership of interface identity, sample timing/history, navigation state, graph data, terminal events and cleanup, and errors. Specify deep boundaries and deterministic test seams for counters, time, and terminal behavior without exposing unnecessary internals. Consult `codebase-design`, `grilling`, and `domain-modeling`; additional crate/API research belongs in newly surfaced research tickets if needed. Produce enough interface detail for implementation, not application code.

Include argument handling and deterministic name ordering, and implement the name-based lifecycle, input/redraw and terminal/exit contract confirmed in [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice). Keep prompt selection redraw distinct from sample acquisition.

Use [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice) for the complete timing/statistics/history and graph-ceiling argument contract. Account for efficient sampling of all startup interfaces, boundary-crossing intervals, clock choice, unrounded calculations and safe formatting/conversion. The human accepts off-screen sampling provided collection work is modest and suspend-aware timing as a straightforward clock choice; no hard resource budget was established.

Use [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md#resolution--2026-10-01--alice) for the confirmed geometry, ASCII bars, ceiling labels, availability states, small-terminal behavior, safe name/value presentation and alternate-screen/cursor/input restoration. Decide rendering/event/cleanup mechanisms, including ignoring Ctrl+Z and caught-panic status 1, without reopening accepted presentation decisions.

Use [Decide Rust and delivery constraints](06-engineering-constraints.md#resolution--2026-10-01--alice) for package/executable naming, stable Rust/edition 2024, dependency/license constraints, version authority and locked-build delivery. Check selected libraries against those constraints.

## Comments

### Discussion — 2026-10-01 — first round

The human accepted all five recommendations: whole-list `/proc/net/dev` acquisition preserving raw-byte names and checked counters; Arguments/Monitor/Proc sampler/Screen/Terminal-runtime ownership with a pure monitor accepting explicit times/outcomes and a thin application loop; a dedicated `args_os()` parser for the strict accepted grammar; and exact u64 counters/integer nanosecond timestamps with f64 derived rates, integer total formatting and explicit halfway-up display rounding.

The human also accepted targeting ordinary xterm-compatible emulator sessions and showing the cursor on cleanup rather than restoring an unknowable prior visibility. This explicitly amends the prior-cursor-visibility requirement in [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md#resolution--2026-10-01--alice); alternate-screen and input-mode restoration remain required. Exact terminal support checks, library/event-loop mechanisms, error treatment and test interfaces remain under discussion. This comment records accepted choices, not final resolution.

### Discussion — 2026-10-02 — second round

The human accepted all five recommendations: a single-threaded direct Linux terminal/runtime using libc 0.2.189 and signal-hook 0.4.4 with default features disabled, saved termios/full raw mode, incremental input decoding, signal notification pipe, CLOCK_BOOTTIME timerfd and one poll loop; rejecting unset/empty TERM and TERM=dumb/linux before mutation while otherwise assuming xterm-compatible protocols; all-or-nothing bulk snapshot parsing with global read/parse failures distinct from absent names in a complete snapshot; shutdown-first scheduling, due sampling ahead of bounded input/resize batches, coalesced rendering and first-future-deadline advancement without catch-up observations; and a pure screen returning ASCII rows with terminal positioning/clearing/output owned by the terminal adapter. Input edges, concrete interfaces, startup ordering and complete cleanup/error rules remain to be confirmed.

### Discussion — 2026-10-02 — third round

The human accepted all five recommendations: incremental ordinary CSI/SS3 arrows, plain q/Q and Ctrl+C, ignoring other/modified keys including Ctrl+Z, consuming unknown escape sequences rather than treating their suffixes as commands, and 64-byte/100-ms-idle parser bounds; reusing the complete timestamped discovery snapshot as initial baselines, with the first 500-ms deadline anchored there and argument/discovery/selection validation before terminal mutation; a Monitor interface consisting of construction, observation, navigation and a selected read-only view, with private exact-delta interval queues, boundary-crossing retention and timestamped graph queues expired only while processing observations; nonblocking terminal output with resumable pending frames, replacement only before any frame byte is emitted, continued event/sampling handling under backpressure and scripted counter/clock/terminal adapters; and staged explicit cleanup with nonpanicking drop fallback, a temporary nonprinting panic hook/catch-unwind, SIGINT precedence over simultaneously pending SIGTERM, cleanup-only errors converting successful q to status 1, and preservation of already chosen signal/failure statuses. The complete-contract summary awaits final human confirmation before resolution.

### Resolution — 2026-10-02 — alice

The human accepted all recommendations in three live rounds and explicitly confirmed the complete-contract summary. This resolution is the canonical architecture decision; discussion comments preserve its history. The accepted startup, statistics and screen contracts remain authoritative except for the explicit terminal/cleanup amendments below.

#### Evidence and dependency choice

- Use bulk `/proc/net/dev` acquisition, a direct Linux terminal implementation, and a single-threaded application loop.
- Direct dependency baselines are **libc 0.2.189** and **signal-hook 0.4.4**, with signal-hook's default features disabled. Use its atomic notifications and Unix signal pipe, not an iterator thread. No terminal framework, async runtime or argument-parser crate is needed for this design.
- [Establish terminal, clock and dependency feasibility](09-terminal-clock-library-facts.md#resolution--2026-10-01--alice) owns the supporting factual investigation and links [Terminal, clock and dependency feasibility](../research/terminal-clock-dependencies.md). The selected direct graph offers GPL-2.0-only-compatible MIT license routes, including the examined registry/errno transitives. Stable Rust/edition 2024 is feasible. Implementation must verify the resolved target/feature graph in its committed Cargo.lock and record the actual tested compiler, as required by [Decide Rust and delivery constraints](06-engineering-constraints.md#resolution--2026-10-01--alice).
- The direct approach trades an existing keyboard decoder for locality of input, wakeups, error handling and terminal ownership. Crossterm's inspected synchronous reader has private polling and fault/fragmentation limitations; a direct owned FD loop makes the agreed behavior explicit. Unsafe Linux calls stay inside the concrete adapters.

#### Module interfaces and ownership

These are internal modules and narrow seams, not a public library or extensibility framework. File subdivision is an implementation detail; the ownership and interface contracts are not.

| Module | Caller-facing interface | Hidden implementation and invariants |
| --- | --- | --- |
| Arguments | Parse OS argument tokens into Help, Version or validated run configuration, or an argument diagnostic. | Own the strict grammar, ceiling defaults/conversion, optional raw-byte initial name and informational text. Do not discover interfaces or initialize terminals. Use Cargo package metadata as the version authority. |
| Proc sampler | Acquire a complete name/counter snapshot, or an acquisition failure. | Own byte reads, snapshot validation and checked RX/TX parsing. Return original name bytes and exact u64 counters. Discovery uses the first snapshot; later acquisition does not alter the navigation list. |
| Monitor | Construct from ordered startup names, initial selection, ceilings and baseline outcomes; observe a completion time and outcomes for every retained name; navigate forward/backward; provide the selected read-only view. | Own fixed names, selection, availability, statistics runs, resets, rate windows, extrema and graph history. No external I/O, clock reads or terminal dimensions. Observation outcomes are readable paired counters, absent or read failed. |
| Screen | Produce ASCII rows from the selected view and supplied terminal dimensions. | Own settled layout, status presentation, escaping, clipping, graph glyphs, numeric formatting and small-terminal view. No mutation of monitoring state or terminal escape sequences. |
| Terminal/runtime | Provide clock/deadline waiting, normalized input/resize/shutdown events, dimensions, frame output and explicit terminal-session cleanup. | Own FDs, raw mode, poll/timerfd/signal pipe, incremental decoding, partial writes, terminal protocols and staged restoration. |

A thin application loop connects these modules and owns sampling scheduling and redraw requests. Its effect seams are counter acquisition, clock readings and terminal events/output. Each has a production adapter and a scripted test adapter; introduce no extra abstraction for hypothetical future backends.

A monitor view carries the selected original name and position/count, one of baseline/measured/absent/read-failed, applicable directional statistics/totals, directional graph samples and configured ceilings. It exposes neither interval queues nor reset bookkeeping. A view may borrow retained samples rather than copy entire histories. Baseline totals and unavailable placeholders follow [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md#resolution--2026-10-01--alice).

#### Arguments and deterministic names

- Use `args_os()` and Unix raw-byte access, not a lossy UTF-8 conversion. Preserve exact identity for matching, discovery and acquisition; screen escaping is presentation only.
- Implement the startup grammar from [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice) plus the ceiling extension in [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice). Parse and convert decimal ceiling values with checked arithmetic; validated ceilings are positive bytes/s after zero has restored the default.
- Sort startup names naturally and deterministically once. Compare digit runs by value without requiring a bounded integer conversion; compare non-digit bytes case-sensitively and independently of locale; use complete raw-name order to break natural ties. Include loopback/inactive names. The initially requested name selects an index without filtering the list.
- Help/version terminate before clock, discovery, terminal checks or terminal mutation. Invalid syntactic arguments produce status 2 before those operations.

#### Proc acquisition and failure semantics

- Reopen or rewind for each fresh acquisition, read the file as bytes through EOF, and parse a complete snapshot rather than assuming a fixed buffer, one read or atomic kernel accounting. Reuse buffers where practical. The cost is one bulk traversal per round, not separate sysfs reads for each interface; no hard latency/resource budget or measured performance claim was established.
- Preserve original name bytes, remove proc padding, and parse RX/TX byte counters with checked u64 conversion. Kernel-valid names cannot contain the delimiter or whitespace, but need not be UTF-8. Reject malformed/incomplete snapshots rather than salvaging rows.
- In a complete valid snapshot, a retained name with counters is readable and a missing retained name is absent. Any file read or snapshot parse failure makes every retained entry read failed for that round. A failed traversal is never evidence of absent names or zero traffic.
- Filter successful runtime snapshots against the startup list. Ignore new names, retain unavailable entries and retry all startup names each round. Original names that return establish new statistics runs under the existing reset policy.
- A startup read/parse failure is an enumeration failure, reported before UI initialization with status 1. Runtime acquisition failure is nonfatal and interface-local in presentation, even when the bulk failure affects every entry; it resets both directions and is not repeatedly reported to stderr.

#### Startup, clock and sampling

1. Parse arguments and handle informational output.
2. Initialize the BOOTTIME clock, acquire the initial complete snapshot and timestamp acquisition completion.
3. Naturally sort names, reject an empty discovery and validate the requested initial selection under the existing status contract.
4. Validate terminal stdin/stdout and TERM before changing terminal state.
5. Prepare signal notification and session ownership before mutations, then enter the UI with the discovery observations as baselines.

The clock seam returns an integer nanosecond representation of Linux `CLOCK_BOOTTIME`; use checked `timespec` conversion and differences. Read it at acquisition completion, with one shared completion timestamp for a bulk snapshot. Clock failure is a fatal runtime/initialization failure, not a reason to silently substitute wall time or `Instant`.

Anchor the first deadline 500 ms after initial acquisition completion. Use a BOOTTIME timerfd in the owned poll loop. If initialization or processing passes a deadline, acquire once, then advance to the first future slot. A timer expiration count never becomes that many fabricated observations. Long intervals, including suspend, use actual elapsed time. Navigation, resize and parser timeouts do not acquire counters.

#### Statistics, history and numerical representation

- Store counters as u64 and calculate checked deltas before converting to f64. Store interval endpoints as integer times and byte deltas exactly. Derived current/average/extrema/graph rates are unrounded f64 values; ordinary double precision is accepted rather than introducing arbitrary-precision arithmetic.
- Implement the interval-overlap formula in [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice), including warm-up and actual delayed intervals. Keep private queues of paired directional deltas/endpoints. Retain the interval crossing the oldest five-minute calculation boundary, not just intervals beginning inside it.
- Keep run extrema separately from window retention. Preserve the existing baseline, nonpositive-time, counter-decrease and failed/absent-observation rules; either-direction discontinuity resets both directions.
- Keep timestamped directional graph samples for every startup interface, one per accepted rate calculation. Expire graph samples outside the five-minute window ending at the latest accepted observation while processing observations; keep samples on the boundary. Observation-based expiry and resets are monitor operations, never redraw operations. Screen width only selects a fitting suffix of retained history.
- Convert/format totals with widened integer arithmetic, retaining exact u64 accounting without overflowing a multiply-by-eight conversion. Choose prefixes before rounding, capped at the agreed G units. Implement halfway-up, two-decimal formatting explicitly for derived rates rather than relying on an unspecified formatter tie rule. Apply ceilings to unrounded graph values; do not cap numeric rates. Screen overflow uses the accepted `[too wide]` presentation.

#### Terminal protocols, input and event loop

- The supported interactive target is an ordinary xterm-compatible emulator session. Require terminal stdin/stdout and reject unset/empty TERM and exact TERM values `dumb` and `linux` before mutation with status 1. Help/version are exempt. Other TERM values are treated as a compatibility declaration, not a successful capability probe.
- Snapshot input termios, use full raw mode, and retain every original file-status flag that nonblocking input/output changes. Full raw mode delivers typed Ctrl+C/Ctrl+Z as bytes; external SIGINT/SIGTERM still require notification.
- Use conventional xterm-compatible alternate-screen and cursor-hide sequences. **Show the cursor on cleanup**, even if it had been hidden before launch. This explicitly supersedes the prior-visibility requirement in [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md#resolution--2026-10-01--alice); restoring input mode and leaving the alternate screen remain required. There is no visibility-query handshake or universal terminal-capability guarantee.
- Notify ordinary control flow of SIGINT, SIGTERM and resize through atomic flags and a nonblocking signal pipe. Signal handlers do not allocate, draw or restore the terminal. Register notifications before mutation; drain wake tokens before inspecting flags, preserving signal identity in flags rather than treating pipe bytes as identities. Obtain dimensions on startup/resize through the Linux terminal adapter.
- Decode ordinary unmodified CSI (`ESC [ A/B/C/D`) and SS3 (`ESC O A/B/C/D`) arrows incrementally. Each sequence is one event, including repeated sequences. Plain q/Q quits, Ctrl+C selects status 130, Ctrl+Z and other/modified input are ignored. Unknown escape sequences are consumed as sequences, not split into ordinary trailing commands. Bound buffering to 64 bytes and discard incomplete sequences after 100 ms without additional input; parser timeout is a wait deadline, never a blocking read.
- Check shutdown first, then due acquisition, then a bounded batch of input/resize events, followed by coalesced redraw/output. Apply every decoded navigation event in order. Recheck shutdown and acquisition deadlines between batches so sustained input does not starve sampling. With one retained interface navigation is a no-op.
- The terminal adapter positions/clears the screen and encodes the ASCII rows. Keep one owned pending frame and resume partial writes on readiness. A frame may be replaced only before any byte has been emitted; finish a partially emitted frame and then render the latest requested state. Continue checking signals, input and sampling during output backpressure. EOF, hangup and unrecoverable input/output errors are fatal UI failures.

#### Cleanup, panic and exit arbitration

- One staged session owner snapshots state before mutation and records attempted screen/cursor changes, including operations whose partial writes may already have changed the terminal. Explicit cleanup attempts every applicable restoration even if another fails; Drop is a nonpanicking best-effort fallback. Restore saved termios/file flags, show the cursor and leave the alternate screen as applicable.
- Install a temporary nonprinting panic hook and catch unwinding around initialization and interactive execution. Restore before reporting a caught panic to stderr with status 1; restore the prior hook. Cleanup must not panic. Return the selected ExitCode normally rather than invoking `process::exit` before cleanup.
- Successful q/Q uses status 0, Ctrl+C/SIGINT 130 and SIGTERM 143. Pending shutdown prevents further sampling/rendering work. If both SIGINT and SIGTERM are pending when inspected, choose SIGINT/130; signal ordering is otherwise not reconstructed.
- Runtime/initialization failures and caught panics use status 1. A cleanup-only failure changes an otherwise successful q exit to 1. An already selected signal or failure status remains unchanged, with cleanup failures diagnosed after the restoration attempts. This is the explicit cleanup-error refinement of the earlier exit contract.
- Guarantees remain limited to exits the application can handle, including best-effort behavior if the terminal becomes unwritable. No suspend/resume feature is introduced.

#### Deterministic verification and map review

Counter acquisition, clock and terminal seams admit scripted adapters returning snapshots, explicit times, normalized events and output outcomes. Test through the monitor/view and screen-row interfaces instead of exposing internal queues. Parser fragmentation/timeout, deadline scheduling, navigation without acquisition, partial output, shutdown wakeups, staged initialization failures and cleanup arbitration can be exercised deterministically. Real Linux PTY tests are still needed for raw-mode bytes, FD behavior, signals and restoration; emulator checks cover assumed screen protocols.

[Decide acceptance criteria and the specification handoff](08-acceptance-handoff.md) owns the final acceptance coverage and handoff audit, including these mechanism-specific cases and the terminal amendments. It is now unblocked. No new decision ticket is needed: the possible terminal/sampling/history interactions recorded in the map's fog have been resolved here, and the remaining coverage/handoff question already has a ticket. This session resolves only this HITL ticket and the supporting research ticket; it produces no application implementation or consolidated specification.
