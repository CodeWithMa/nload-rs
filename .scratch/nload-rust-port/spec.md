# Minimal Rust port of nload — implementation specification

Consolidated 2026-10-02 from the completed [Find the way to a minimal Rust port of nload](map.md) planning map, recorded at repository commit `6b3ab01`. This document is the implementation handoff. Required verification is specified below; passing implementation results have not yet been recorded.

## 1. Scope, identity and reference baseline

Build an independently authored Linux Rust traffic monitor, **`nload-rs`**, inspired by [nload](https://github.com/rolandriegel/nload). Launching it opens a single-interface view. An optional interface-name argument selects the initial interface; all four arrow keys navigate the startup list. Incoming/outgoing graph ceilings have independent argument overrides. Standalone `--help` and `--version` provide informational output.

The Cargo package and executable are both `nload-rs`, initially version `0.1.0`. Cargo package-version metadata is the sole version authority. The distinct name permits coexistence with upstream; providing the human's personal `n` alias is not a requirement.

The behavioral reference is upstream **`v0.7.4`**, commit **`a5715a77eb2c11702a7ffc7664240b9250cafa4d`**. The confirmed port contracts, consolidated here, are the acceptance authority. Upstream source and observed behavior supply evidence, not a requirement to reproduce every quirk. Re-running upstream and comparing screenshots are optional.

Out of scope: non-Linux support; multi-interface view; extra upstream modes/options; configuration files; address discovery; suspend/resume UI; a public library/extensibility framework; installation, publication and distribution packaging. There is no older-MSRV promise, hard resource/latency budget, byte-identical-build requirement or separate suspend-specific test campaign.

Use the domain language in [Network Traffic Monitor glossary](../../GLOSSARY.md), particularly interface, current/average rate, cumulative total, graph ceiling/sample/history, baseline observation and statistics run.

### Decision provenance

These linked resolutions retain rationale and discussion history. Sections below consolidate their requirements; section 11 makes supersessions explicit.

- [Decide the minimal startup and navigation contract](issues/03-startup-navigation-contract.md#resolution--2026-10-01--alice).
- [Decide traffic statistics and sampling behavior](issues/04-statistics-contract.md#resolution--2026-10-01--alice), including ceiling-option scope expansion.
- [Validate the single-interface screen and terminal behavior](issues/05-single-interface-screen.md#resolution--2026-10-01--alice), as amended by the architecture decision.
- [Decide Rust and delivery constraints](issues/06-engineering-constraints.md#resolution--2026-10-01--alice).
- [Decide the port's architecture and test seams](issues/07-module-boundaries.md#resolution--2026-10-02--alice).
- [Decide acceptance criteria and the specification handoff](issues/08-acceptance-handoff.md#resolution--2026-10-02--alice).

Supporting evidence: [nload release findings](research/release-defaults.md), [Linux interface and counter semantics](research/linux-counters.md), [Terminal, clock and dependency feasibility](research/terminal-clock-dependencies.md), and the [Throwaway single-interface screen prototype](screen-prototype.html). Only the revised A layout is selected; B/C and Unicode glyphs are explored alternatives. The prototype does not establish real-terminal cleanup or numerical correctness.

## 2. CLI and startup contract

### 2.1 Invocation and argument handling

There are three invocation outcomes: Help, Version, or a validated run configuration. Parse OS argument tokens using `args_os()` and Unix raw-byte access; do not require UTF-8 or use lossy conversion.

- No arguments opens the UI with the first naturally sorted interface.
- At most one positional interface name is allowed, matched exactly and case-sensitively by original bytes. It selects the initial entry without filtering the navigation list.
- `--` terminates option parsing. Tokens after it are positional names, including names beginning with `-`; the one-name limit still applies.
- Standalone `--help` writes stdout and exits 0. Help describes the accepted invocation, ceiling flags, binary-scaled units, defaults and zero behavior.
- Standalone `--version` writes exactly `nload-rs 0.1.0` followed by a newline and exits 0. Obtain the version from Cargo package metadata at build time, not another version literal. Future version changes follow that authority.
- Help/version terminate before clock initialization, interface discovery, terminal checks or terminal mutation. They work with redirected standard streams.
- Help/version are standalone, not combinable with each other, an interface name or ceiling options. No short aliases are provided.
- Unknown flags, multiple names, malformed/missing ceiling values and unsupported combinations are syntactic argument errors: stderr diagnostic, status 2, no UI and no clock/discovery/terminal work.
- A requested name absent from a successfully discovered, nonempty startup list is a selection argument error: stderr diagnostic, status 2, no terminal mutation.
- Neither read nor write upstream configuration; defaults do not depend on `HOME`.

Diagnostic prose and the full help wording need not be byte-identical fixtures; their required contents, destinations and statuses are contractual. Version output and expressly specified screen text are exact.

### 2.2 Graph-ceiling options

Support separate-token `-i VALUE` for incoming and `-o VALUE` for outgoing. Options may appear before or after the optional interface name, until `--`.

| Rule | Contract |
| --- | --- |
| Units | VALUE is an integer number of binary-scaled kBit/s; multiply by 128 to obtain bytes/s. |
| Independent defaults | Each direction defaults to 10240, equivalent to 1,310,720 bytes/s or 10.00 MBit/s. |
| Zero | Restores the relevant direction's default. |
| Positive range | 1 through 144115188075855871 inclusive; the converted ceiling must fit u64. No clamp above 1. |
| Repetition | Last occurrence wins independently per direction. |
| Syntax | A nonempty whole token of ASCII decimal digits; leading zeros are decimal. Checked parsing and conversion must not overflow. |
| Rejection | Signs, fractions, whitespace, suffixes, missing values and attached forms such as `-i20480`/`-i=20480`; status 2 without UI. |
| Scope | Ceilings apply to every startup interface, not only initial selection. |

For example, `nload-rs -i 20480 -o 5120 eth0` selects `eth0` with ceilings of 2,621,440 incoming and 655,360 outgoing bytes/s. `nload-rs eth0 -i 20480 -o 5120` has the same configuration. `-i 0` restores incoming only; `-i 10 -i 20` uses 20. `--help -i 10` is an argument error.

### 2.3 Startup sequence and failures

Perform startup in this order:

1. Parse arguments and finish informational invocations.
2. Initialize the BOOTTIME clock, acquire a complete initial counter snapshot and timestamp its acquisition completion.
3. Naturally sort names; reject empty discovery and validate initial selection.
4. Validate terminal stdin/stdout and TERM before terminal mutation.
5. Prepare signal notification and staged session ownership before mutation; enter the UI using the discovery observations as baselines.

Do not acquire a redundant second snapshot just to initialize the monitor. Anchor the first 500-ms deadline to discovery completion, including when terminal initialization takes longer than one slot.

Empty discovery and failed enumeration are distinct stderr diagnostics, both status 1 before terminal initialization. Initial proc read/parse failure is enumeration failure. Clock or UI initialization failure is status 1; restore every applicable changed terminal state before reporting. A terminal below the size floor is usable in warning mode, not initialization failure. Section 7 specifies exit/error arbitration.

## 3. Interface ordering, navigation and lifecycle

### 3.1 Fixed startup membership and ordering

Include all discovered interfaces, including loopback, inactive and addressless interfaces. Discover membership once, sort once and retain that ordered list for the process lifetime.

Natural ordering compares digit runs by numeric value without requiring conversion into a bounded integer. Compare non-digit bytes case-sensitively and independently of locale. If natural comparisons tie, use the complete original byte name as the tie-breaker. Thus `eth1`, `eth2`, `eth3`, `eth10` have numeric order, and `eth02` sorts before `eth2` when those compare naturally equal.

Original name bytes are the identity used for ordering, selection and acquisition. Presentation escaping never alters identity.

### 3.2 Navigation

- Right/Down advances one entry; Left/Up retreats one entry. Wrap symmetrically at either end, applying each decoded key event in order.
- One interface makes navigation a no-op.
- Redraw selection promptly without waiting for the next sample; the header's one-based position reflects selection.
- Navigation neither acquires counters nor creates samples, resets statistics or discards history.
- PageUp/PageDown, Tab and Enter are not aliases. Other/modified keys are ignored except the explicit quit controls in sections 6–7.

### 3.3 Removal, rename, return and name reuse

Keep an original entry and current selection if its name disappears. Show it as unavailable and permit navigation away; do not automatically select another interface. Newly added names and rename targets are ignored until restart.

Retry every retained startup name each acquisition round. A returning original name begins a fresh statistics run with a baseline, even if its counters are higher than before disappearance. This is a name-based contract, not continuity of the underlying interface incarnation. A remove/recreate cycle entirely between successful observations cannot reliably be distinguished from continuity if no observable discontinuity occurs.

## 4. Acquisition, timing and statistics

### 4.1 Whole-list proc acquisition

Use `/proc/net/dev` as the single bulk backend. Reopen or rewind for every fresh acquisition; read bytes through EOF. Do not assume a fixed buffer, one read, atomic RX/TX accounting or simultaneous all-interface collection. Reuse buffers where practical; collection is one bulk traversal per round rather than per-interface sysfs reads. Collection cost must remain modest, without a measured performance claim or numerical budget.

Validate the complete snapshot. Remove proc padding while preserving exact interface-name bytes; parse paired RX/TX cumulative byte counters with checked u64 arithmetic. RX bytes is numeric field 1 and TX bytes field 9 after the colon. Kernel-valid names cannot contain the delimiter or whitespace but need not be UTF-8. Reject malformed/incomplete snapshots rather than salvaging rows.

For each retained name, a round supplies one of:

| Outcome | Meaning and effect |
| --- | --- |
| Readable paired counters | Exact incoming/outgoing cumulative u64 byte counts. |
| Absent | Name is missing from a complete valid snapshot. |
| Read failed | Global file-read or snapshot-parse failure; every retained name gets this outcome for the round. |

A failed traversal is not evidence of absence or zero traffic. Filter successful runtime snapshots against the startup list and ignore new names. Runtime acquisition failure is nonfatal, even when all retained entries are affected: reset affected statistics and show the selected entry's unavailable state without repeated stderr diagnostics.

These are interface-accounted bytes, not application payload, unique host-wide traffic or physical-medium occupancy. Exported u64 width does not establish the driver's counter width, freshness, epoch or atomicity. Reading the calling process's accessible proc view does not promise host-wide visibility across network namespaces.

### 4.2 Time and cadence

- Use Linux `CLOCK_BOOTTIME` semantics: monotonic, suspend-inclusive elapsed time. Wall-clock changes do not affect statistics. No separate suspend detector/tracker is needed.
- The clock seam returns an integer nanosecond representation, with checked timespec conversion and differences. Failure is fatal initialization/runtime failure; do not substitute wall time or `Instant`.
- Timestamp acquisition completion. A bulk snapshot may use a shared completion timestamp; this is not a claim that kernel accounting was simultaneous or freshly updated.
- Sample every retained interface, including off-screen entries, on 500-ms deadlines anchored to initial acquisition completion.
- Use BOOTTIME timerfd in the event loop. If initialization/processing misses slots, acquire once and advance to the first future slot. Expiration counts never manufacture catch-up observations.
- Calculate with full actual elapsed intervals, including delay/suspend. Ignore nonpositive-duration observations without advancing statistics/history.
- Navigation, resize and parser-timeout processing are redraw/input operations, not acquisitions.

### 4.3 Statistics runs, baseline, totals and resets

Store exact counters as u64 and calculate checked deltas before converting derived values to f64. Integer timestamps/endpoints and interval byte deltas remain exact; ordinary f64 derived rates are accepted. Keep calculations, extrema and graph scaling unrounded.

A baseline observation exposes the incoming/outgoing cumulative totals but no current/average rate, extrema or graph column. The first positive-duration interval initializes current, average, minimum and maximum rates independently for each direction.

`Ttl` is the latest successful interface-accounted cumulative counter for that direction. It is not a session byte sum or a guaranteed since-boot total, and has no wrap correction.

A decrease in either counter, absence, or failed acquisition of either direction resets **both directions'** rates, extrema and graph history. A successful readable decrease immediately becomes the new baseline; after absence/failure, the next successful paired reading is baseline only. Never bridge unavailable gaps, guess wraps or turn failures into measured zeros. Do not display stale totals while unavailable.

### 4.4 Current and average windows

`Curr` is the actual one-second window ending at the latest accepted observation. `Avg` is the actual 300-second window. During warm-up use the shorter available run history without zero padding.

Estimate uniform traffic within each observed interval. For horizon `H`, baseline time `b`, latest accepted time `t`, and intervals `j` with byte delta `d_j` and positive duration `dt_j`:

```text
a = max(b, t - H)
rate_H = sum(d_j * overlap_duration(interval_j, [a, t]) / dt_j) / (t - a)
```

There is no rate when `t <= b`. Retain the interval crossing the oldest five-minute calculation boundary even if its earlier endpoint precedes the window. Partial contributions estimate packet timing; they do not recover it.

Minimum/maximum compare successive unrounded current rates over the whole statistics run, separately per direction. They do not roll out with the average window or reset on navigation. A measured idle interval may establish a genuine zero minimum.

### 4.5 Graph history and ceilings

Each accepted rate calculation adds one timestamped current-rate graph sample per direction for that interface. Baselines, ignored observations and redraws add none; a delayed acquisition adds one column, not catch-up columns.

Keep five minutes of actual history for every startup interface, including never-displayed entries. While processing observations, expire samples outside the five-minute window ending at the latest accepted observation; retain samples exactly on the boundary. Resets clear both histories. Expiry belongs to the monitor, not rendering.

Show the newest retained samples that fit: oldest fitting sample on the left, newest on the right. Unfilled space is blank, not a measured zero. A measured-zero sample occupies a column but has zero height. Shrink/grow hides/reveals retained history without changing it. Horizontal spacing represents observations, not an exact time axis.

Ceilings are fixed and independently configurable. Graph samples at/above a ceiling saturate; numeric rates remain uncapped. Ceilings do not alter statistics.

### 4.6 Numerical examples

Times below are seconds; rates are bytes/s before display conversion. Counter examples describe incoming; apply the same independent calculations to outgoing.

| Scenario | Observations/action | Expected result |
| --- | --- | --- |
| Warm-up | Counters 1000 at 0, 1100 at 0.5, 1400 at 1. | No baseline rate; then current/average 200; then current/average 400. Minimum 200, maximum 400, total 1400 rather than session delta 400. |
| Partial current boundary | Counters 0 at 0, 100 at 0.5, 400 at 1.25. | Current `(100 * 0.25/0.5 + 300) / 1 = 350`; average `400/1.25 = 320`. |
| Five-minute boundary | Counters 0 at 0, 600 at 2, 90800 at 302. | Average `(90800 - 600)/300`; the first interval is excluded. |
| Partial average boundary | Counters 0 at 0, 600 at 2, 91200 at 301. | Window begins at 1; average `(600 * 1/2 + 90600)/300 = 303`. Retain the interval crossing that boundary. |
| Delay | An additional 3000 bytes over one three-second interval. | Interval rate 1000, not 6000 from nominal 0.5-second division; one graph column. |
| Readable decrease | RX decreases from 4000000000 to 10000, then rises another 500 over 0.5 seconds. | Both directions reset at the decrease; total 10000 and no rate/column at new baseline; next RX current/minimum/maximum 1000. |
| Failure and return | One direction fails or name disappears, then counters return higher. | Both unavailable/reset; return is baseline only, without a delta across the gap. |

The partial-average example is a derived application of the already confirmed formula, not a new statistics policy.

## 5. Screen, formatting and graph rules

### 5.1 Full-size geometry

The pure screen module produces ASCII rows from the selected monitor view and dimensions. It does not mutate the monitor or produce terminal escape sequences. The terminal adapter owns positioning, clearing and output. Clip safely so rendering neither wraps nor writes outside the supplied screen.

Full view requires **width >= 60 and height >= 16**. Its rows are:

1. Interface-name and one-based position header, for example `Interface eth0 (1/3)`; no IPv4 field.
2. Full-width `=` separator in measured state, or the applicable status text in section 5.3.
3. Incoming direction label, including its formatted ceiling.
4. Incoming graph and adjacent statistics area.
5. Outgoing direction label, including its formatted ceiling.
6. Outgoing graph and adjacent statistics area.

There is **no footer**, persistent help line, scale axis, tick or time label. Direction labels use the form `Incoming: (ceiling 10.00 MBit/s)` and `Outgoing: (ceiling 10.00 MBit/s)` with their own ceiling values.

Reserve 28 columns at the right for each statistics block and a two-column gap. Graph width is `width - 30`. List `Curr`, `Avg`, `Min`, `Max`, `Ttl` in that order, bottom-aligned in each graph area.

Split `height - 4` remaining rows: incoming height `floor((height - 4)/2)`, outgoing gets the remainder. At 80x24 each graph is 50x10: incoming label row 2, graph rows 3–12, outgoing label row 13, graph rows 14–23 (zero-based); statistic rows 8–12 and 19–23. At odd available height the extra row belongs to outgoing.

### 5.2 ASCII bars

For unrounded rate `r`, positive ceiling `c` and graph height `H`, filled height is `H * min(r/c, 1)`. Render upward from the bottom. The portion of fill in each cell selects:

| Cell fill | Glyph |
| --- | --- |
| Below 0.30 | Space |
| 0.30 to below 0.70 | `.` |
| 0.70 to below 1 | ASCII vertical bar (`\|`) |
| At least 1 | `#` |

At/above ceiling all graph cells are full; numeric statistics still show the actual rate. Use section 4.5's fitting history suffix, with blank unfilled space and no synthetic columns.

### 5.3 Availability states

Keep the selected header visible. Replace the separator row rather than appending status to the full-size header.

| State | Rates/extrema | Totals | Graphs | Second-row text |
| --- | --- | --- | --- | --- |
| Baseline | `--` | Baseline cumulative counters | Empty | `Waiting for rate sample` |
| Absent | `--` | `--` | Empty | `Unavailable: interface absent` |
| Read failed | `--` | `--` | Empty | `Unavailable: counter read failed` |
| Measured idle | Genuine measured zero values | Latest counters | Actual zero-height samples | Normal full-width `=` separator |

Unavailability preserves selection/navigation and acquisition retries, not stale values. A successful return first renders the baseline state, then measured rates resume after a positive interval.

### 5.4 Small sizes, resize and text overflow

Below either size threshold replace the full view with a terminal-too-small warning, current selection and status as space permits. At extremely small sizes show only what fits. Starting small is valid. Continue sampling, navigation and quit handling while small; growth restores full layout and actual retained history.

Shorten overlong displayed names with `...`, preserving the position indicator when space permits. Escape non-ASCII/control name bytes as ASCII `\xHH`. Selection/acquisition still use the original bytes. Keep statistic field labels visible; if the normally formatted value does not fit, use `[too wide]` rather than misleading numeric truncation. This is presentation only, not changed calculations.

### 5.5 Units and rounding

- Rates use `Bit/s`, `kBit/s`, `MBit/s`, `GBit/s`; totals use `Byte`, `kByte`, `MByte`, `GByte`. Prefix factors are 1024, not 1000.
- Choose each value's prefix independently **before rounding**: the smallest prefix with a value below 1024, capped at GBit/GByte. Larger values remain in the highest unit.
- Display two fractional digits, nearest with halfway values upward. A value just below a unit boundary may display `1024.00` in the smaller unit.
- Derived rates remain unrounded f64 until explicit formatting. Do not rely on an unspecified formatter tie rule or truncate to integer bytes/s early.
- Format cumulative totals with widened integer arithmetic, preserving exact u64 accounting and avoiding overflowing intermediate bit conversion.

Examples: 128 bytes/s displays `1.00 kBit/s`; 1024 cumulative bytes displays `1.00 kByte`; measured zero displays `0.00 Bit/s`, distinct from `--`. An exactly halfway 0.125 Bit/s displays `0.13 Bit/s`; 127.9995 bytes/s displays `1024.00 Bit/s` because its pre-rounding bit rate is below 1024.

## 6. Terminal support, input and event scheduling

### 6.1 Support and session setup

Target ordinary xterm-compatible emulator sessions. Require terminal stdin and stdout; stderr may be redirected. Reject unset/empty TERM and exact values `dumb` and `linux` before mutation with stderr diagnostic/status 1. Informational invocations are exempt. Other TERM values declare assumed compatibility; this is not a successful capability probe or universal terminal guarantee.

Use a direct Linux terminal adapter. Save input termios and all original file-status flags changed for nonblocking input/output. Full raw mode delivers typed Ctrl+C/Ctrl+Z as bytes rather than terminal-generated signals. Use conventional xterm-compatible alternate-screen and cursor-hide sequences. Cleanup shows the cursor and leaves the alternate screen; it does not preserve unknown prior cursor visibility.

Register SIGINT, SIGTERM and resize notification before mutation, using atomic flags and a nonblocking signal pipe. Signal handlers do not allocate, draw or restore state. Pipe bytes are wake tokens, not signal identities: drain wake tokens before inspecting flags. Read dimensions through the Linux terminal adapter at startup/resize.

### 6.2 Incremental input

Decode ordinary unmodified CSI (`ESC [ A/B/C/D`) and SS3 (`ESC O A/B/C/D`) arrows, including fragmented and repeated sequences. Each complete sequence is one navigation event. Plain q/Q quits; Ctrl+C selects status 130. Ignore Ctrl+Z, other keys and modified keys.

Consume unknown escape sequences as sequences rather than interpreting their trailing bytes as plain commands. Bound parser buffering to 64 bytes; discard incomplete sequences after 100 ms without additional input. Timeout is a wait deadline, not a blocking read. There is no suspend/resume feature.

### 6.3 Scheduling and output

Use one single-threaded poll loop over owned terminal, signal-pipe and BOOTTIME timerfd FDs:

1. Check shutdown first. Pending shutdown prevents more sampling/rendering work.
2. Process due acquisition before input/resize batches.
3. Process a bounded input/resize batch, applying navigation events in order.
4. Coalesce redraw requests and perform ready output.
5. Recheck shutdown/acquisition deadlines between batches so sustained input cannot starve sampling.

Keep one owned pending output frame with resumable nonblocking partial writes. Replace a pending frame only before any byte is emitted. Finish a partially emitted frame, then render the latest requested state. Backpressure must not stop signal, input or sampling handling. EOF, hangup and unrecoverable input/output errors are fatal UI failures.

## 7. Failures, cleanup and exit statuses

### 7.1 Error classification

| Condition | Result |
| --- | --- |
| Standalone help/version | stdout, status 0; no clock/discovery/terminal setup. |
| Syntactic argument error or absent requested name in successful nonempty discovery | stderr diagnostic, status 2; no UI. |
| Empty discovery or startup acquisition failure | Distinct stderr diagnostics, status 1; no terminal mutation. |
| Non-terminal interactive stdin/stdout or rejected TERM | stderr diagnostic, status 1; no mutation. |
| Too-small dimensions | Continue interactive warning view; not fatal. |
| Runtime counter failure/absence | Selected interface unavailable; both directions reset; continue UI/retries, no repeated stderr output. |
| Clock/session initialization/runtime failure, fatal terminal input/output failure or caught panic | Best-effort applicable restoration, then stderr diagnostic, status 1. |

### 7.2 Staged restoration and panic handling

One session owner snapshots state before mutation and records attempted screen/cursor changes, including commands whose partial output may already have changed terminal state. Explicit cleanup attempts **every applicable restoration even if another fails**: saved termios, changed file flags, cursor show and alternate-screen leave. Drop is a nonpanicking best-effort fallback. Cleanup must not panic.

Install a temporary nonprinting panic hook and catch unwinding around initialization/interactive execution. Restore before reporting a caught panic to stderr; restore the prior hook. Return the chosen `ExitCode` normally rather than calling `process::exit` before cleanup. Guarantees apply to exits the application can handle; an unwritable terminal or unhandleable termination cannot carry a universal restoration guarantee.

### 7.3 Exit arbitration

| Exit cause | Status |
| --- | --- |
| Successful q/Q | 0 |
| Typed Ctrl+C or external SIGINT | 130 |
| External SIGTERM | 143 |
| Both SIGINT/SIGTERM pending when inspected | SIGINT wins: 130; otherwise do not reconstruct signal arrival order. |
| Initialization/runtime failure or caught panic | 1 |
| Cleanup-only failure after otherwise successful q/Q | 1 |
| Cleanup failure after an already chosen signal/failure status | Preserve that status; diagnose cleanup failure after restoration attempts. |

## 8. Module ownership and caller-facing contracts

These are internal deep boundaries and narrow test seams; file subdivision and private implementation details may be chosen during implementation. Unsafe Linux calls remain inside concrete adapters.

| Module | Caller-facing contract | Ownership and hidden invariants |
| --- | --- | --- |
| Arguments | OS tokens to Help, Version, validated run configuration, or argument diagnostic. | Strict grammar, ceilings/defaults/conversion, optional original-byte name, help/version text and Cargo version authority. No clock, discovery or terminal setup. |
| Proc sampler | Acquire a complete name/paired-counter snapshot or acquisition failure. | Byte reads through EOF, padding/name preservation, complete validation, checked exact u64 RX/TX. First snapshot supplies discovery/baselines; later snapshots do not change membership. |
| Monitor | Construct from ordered names, initial selection, ceilings and baseline outcomes; observe a completion time/outcomes for retained names; navigate forward/backward; return selected read-only view. | Fixed names/selection, availability, statistics runs/resets, paired exact interval queues, boundary-crossing retention, derived rates/extrema, timestamped graph queues and observation-only expiry. No I/O, clock reads or dimensions. |
| Screen | Selected view and dimensions to ASCII rows. | Geometry, safe escaping/clipping, status presentation, unit formatting, glyphs and small-terminal mode. No monitor mutation or terminal escape sequences. |
| Terminal/runtime | Clock/deadline waiting, normalized input/resize/shutdown events, dimensions, frame output and explicit session cleanup. | FDs/raw mode/file flags, poll/timerfd/signal pipe, incremental decoder, partial writes, terminal protocols and staged restoration. |

A thin application loop connects the modules and owns sampling scheduling/redraw requests. Its counter, clock and terminal effect seams each have a production adapter and scripted test adapter; do not add abstractions for hypothetical future backends.

The selected monitor view contains original name, one-based position/count, baseline/measured/absent/read-failed state, applicable directional statistics/totals, directional graph samples and ceilings. It exposes neither private interval queues nor reset bookkeeping. It may borrow samples instead of copying histories. Tests observe monitor/view and screen-row interfaces rather than reaching into internal queues.

## 9. Dependencies, licensing and delivery

Use stable Rust, edition 2024. Direct dependency baselines are **libc 0.2.189** and **signal-hook 0.4.4**, with signal-hook default features disabled. Use atomic notifications and its Unix signal pipe, not an iterator thread. No terminal framework, async runtime or argument-parser crate is needed.

Keep the dependency set small without a numerical cap and compatible with GPL-2.0-only. The examined graph offers MIT license routes, including signal-hook-registry/errno transitives; verify the actual resolved target/feature graph in Cargo.lock and applicable notices during implementation. Research is feasibility evidence, not a generated lockfile or build certification.

License the independently written application **GPL-2.0-only**; declare this in Cargo metadata and include the GPL version 2 license text. Upstream is inspiration/reference, not source to translate or adapt. Include this README credit:

> Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.

Additional upstream release/commit attribution in the product README is not required; planning/research retains its pinned references.

Commit Cargo.lock and record the exact compiler used to verify the build. Reproducible means repeatable with recorded compiler/locked dependency versions, not byte-identical output. Document and verify:

```text
cargo fmt --check
cargo test --locked
cargo build --release --locked
./target/release/nload-rs
```

The first three commands are acceptance gates; the last is direct Linux execution used for terminal/live checks. Clippy is optional, not another gate. Packaging, Cargo installation, crates.io/Git publication and downloadable binaries are not delivery requirements.

## 10. Acceptance matrix and verification procedure

### 10.1 Evidence and comparison rules

Each case below supplies setup, action, expected result and evidence type. The matrix is a minimum contractual coverage plan, not a demand for one test per row; parameterized cases may share fixtures. Map implemented test names/manual records back to these cases, and cover every requirement in sections 1–9.

- **D — Deterministic automated:** pure modules and scripted effect adapters, including application-loop integration. No private-queue assertions or real-time sleeps as a numerical oracle.
- **P — Automated Linux PTY:** exercise the real compiled binary and Linux terminal/FD/signal behavior. Observe restoration sequences and saved OS state; a PTY does not prove visual emulator behavior.
- **E — Recorded emulator check:** one ordinary xterm-compatible emulator, recording its identity and observed visual/cleanup behavior.
- **L — Nonprivileged live Linux smoke:** actual proc discovery/counters and ordinary generated traffic; exact live rates are not an oracle.
- **B — Build/artifact review:** required commands and checks of module boundaries, metadata, dependency graph and delivery files.

Where a row lists multiple types, collect each indicated evidence for its applicable aspects: deterministic fixtures establish exact logic, PTYs establish OS/input/restoration behavior, and the emulator establishes visible behavior. This does not require every fixture variant to be recreated in every environment. Use harness-controlled real failures where practical; scripted adapters cover failures a PTY cannot reliably induce. No production test-only CLI flags or particular injection mechanism is required by this specification.

Compare exact counters, states, sample counts, mandated text and rendered fixtures exactly. For derived rates in bytes/s, accept `abs(actual - expected) <= 1e-9 * max(1, abs(expected))`. Formatted text and glyph-threshold decisions are exact checks, not tolerance checks. Diagnostic/help prose is checked for its required semantics/destinations/statuses rather than arbitrary byte-for-byte wording.

### 10.2 Arguments, startup, ordering and lifecycle

| Case / contract | Setup | Action | Expected outcome | Evidence |
| --- | --- | --- | --- | --- |
| A01 — Identity/version (§§1, 2.1, 9) | Package metadata at 0.1.0; scripted clock/discovery/terminal adapters that reject calls. | Invoke standalone `--version`, including redirected streams. | Exactly `nload-rs 0.1.0\n`, stdout/status 0; no adapter calls; executable/package identity and version authority match. | D, B |
| A02 — Help (§2.1) | Same no-work adapters and redirected streams. | Invoke standalone `--help`. | stdout/status 0; documented invocation, flags, units/defaults/zero; no clock/discovery/terminal work. | D |
| A03 — Grammar errors (§2.1) | No-work adapters. | Unknown option, `-h`, `-V`, two names, help+version, help/version+name or ceiling flag. | stderr/status 2; no UI or adapters; no short informational aliases. | D |
| A04 — Positional identity (§§2.1, 3) | Startup list includes case variants, raw non-UTF-8 names and a name beginning with `-`. | Select exact name; use `--` for leading dash; try absent/case-mismatched name. | Original-byte matching; full list retained; missing selection status 2 before mutation in successful nonempty discovery. | D |
| A05 — Ceiling defaults/options (§2.2) | Multi-entry startup list. | No options; options before/after name; `-i 20480 -o 5120`; repeated flags; leading zeros; `-i 0`. | Independent defaults; 2621440/655360 bytes/s override example; last wins; zero resets only its direction; same ceilings after navigation. | D |
| A06 — Ceiling limits/errors (§2.2) | No-work adapters for invalid syntax. | Values 1, 144115188075855871, 144115188075855872, very long overflow token, empty/missing, signed, fractional, whitespace, suffix or attached form. | Valid converted ceilings 128 and 18446744073709551488; other listed invalid inputs status 2 without UI/work; no wrap/clamp. | D |
| A07 — Configuration independence (§2.1) | Set/unset HOME; hypothetical conflicting upstream config. | Start with identical arguments/discovery. | Identical defaults/list/view; no upstream config read/write. | D, B |
| A08 — Startup discovery outcomes (§2.3) | Empty complete snapshot, failed read/parse, or nonempty snapshot missing requested name. | Start each variant. | Empty versus enumeration-failure diagnostics/status 1; requested missing name in nonempty snapshot status 2; no terminal mutation. | D |
| A09 — Natural order (§3.1) | Unsorted `eth10`, `eth2`, `eth02`, `eth1`, `eth3`, loopback/inactive names, case/raw-byte variants; comparator fixtures with very long digit runs. | Construct startup list under differing locale settings. | Natural order/tie-break from §3.1; no bounded-run overflow or locale dependence; all discovery entries included; default first selected. | D |
| A10 — Startup ordering/baseline (§§2.3, 4.2) | Instrumented acquisition/clock/session calls; slow UI initialization. | Start and then process an overdue first deadline. | Parse → initial clock/snapshot/completion timestamp → list/selection → terminal validation → notifications/ownership → mutation; discovery data reused; first deadline completion+500 ms; one fresh overdue acquisition. | D |
| A11 — Navigation (§3.2) | Three ordered entries and a one-entry variant. | All four arrows, repeated events, reverse/forward wrapping, excluded aliases. | One symmetric step per event in order; one-entry no-op; aliases ignored; prompt requested redraw with zero acquisition/sample/reset effects. | D; real input/visual evidence in T03/T14 |
| A12 — Lifecycle (§3.3) | Startup entries A/B; later valid snapshots remove/rename A, add C, then return/reuse A. | Acquire and navigate through transitions. | Fixed ordered A/B membership; selection retained; unavailable A remains navigable; rename target/C ignored; returning A is baseline, not bridged delta or guaranteed incarnation continuity. | D |

### 10.3 Acquisition, timing, statistics and history

| Case / contract | Setup | Action | Expected outcome | Evidence |
| --- | --- | --- | --- | --- |
| S01 — Proc snapshot (§4.1) | Valid proc bytes with padding, raw names, zero and maximum-u64 counters; chunked reads. | Acquire through EOF; acquire again. | Complete original-byte name/paired-counter snapshot; checked RX field 1/TX field 9; fresh reopen/rewind rather than EOF reuse; no one-read/fixed-buffer assumption. | D |
| S02 — Failure versus absence (§4.1) | Complete snapshot missing one retained name; separate malformed/truncated/overflowing snapshots and read failure. | Acquire runtime rounds. | Valid missing name is absent; any global read/parse failure makes all retained entries read failed; no partial salvage, zeros or fatal UI exit; retry without repeated stderr diagnostics. | D |
| S03 — Warm-up/totals (§§4.3–4.6) | RX 1000@0, 1100@0.5, 1400@1; independently different TX. | Observe each pair. | Baseline totals only; rates 200 then 400; min 200/max 400/total 1400; TX independent; no baseline graph column. | D |
| S04 — Partial current (§4.4) | RX 0@0, 100@0.5, 400@1.25. | Observe. | Current 350 and average 320; unrounded interval contributions. | D |
| S05 — Average boundaries (§§4.4, 4.6) | Exact and partial 300-second-boundary examples in §4.6. | Observe final point. | Exact `(90800-600)/300`; partial 303; crossing interval retained and only overlap counted. | D |
| S06 — Time/warm-up (§4.2) | Explicit BOOTTIME integers; changed wall-clock fixture; same/earlier observation times; interval spanning suspend-duration equivalent. | Observe/advance scheduling. | Wall-clock irrelevant; shorter warm-up unpadded; nonpositive time does not advance statistics/history; suspend-inclusive actual elapsed denominator. | D, B |
| S07 — Delayed deadlines (§4.2) | Baseline deadline anchor; missed slots/large timerfd expiration count; 3000 bytes over 3 seconds. | Process due acquisition. | One fresh acquisition/sample, interval 1000 bytes/s, no synthetic catch-up; next deadline first future slot. | D |
| S08 — Precision/extrema (§4.3–4.4) | Fractional rates, equal rendered rates with different unrounded values, measured idle, extrema older than 300 seconds. | Observe; navigate; advance beyond averaging retention. | Checked exact deltas before f64 conversion; min/max use unrounded current rates and survive navigation/window expiry; genuine idle may set min 0. | D |
| S09 — Counter decrease (§4.3) | RX 4000000000 → 10000; TX still increasing; then RX +500 in 0.5 seconds. | Observe. | Both runs/history reset; immediate new baseline and total 10000 without wrap/column; next RX current/min/max 1000. Repeat decrease on TX. | D |
| S10 — Failure/return (§4.3) | Measured entries; one counter invalid, global failed snapshot, or absent entry, followed by higher readable counts. | Observe each transition. | Both directions reset/unavailable without stale totals/zero interval; next paired success baseline only; new run thereafter. | D |
| S11 — Off-screen history (§4.5) | Multiple interfaces, one never viewed; regular and delayed observations. | Sample all then select unseen interface. | Actual statistics and timestamped history accumulated off-screen; one column per rate calculation; navigation adds none. | D |
| S12 — Retention/resize (§4.5) | Samples before, exactly at and after t−300; enough history to exceed width. | Process observation; shrink/grow screen; redraw without observation. | Older samples expire, boundary retained; newest fitting suffix oldest-left/newest-right; width hides/reveals retained samples; redraw neither expires nor adds samples. | D |
| S13 — Backend/clock shape (§§4.1–4.2, 8) | Production adapter implementation and scripted clock failure/conversion overflow. | Review acquisition/clock boundaries and run failure fixtures. | One bulk proc traversal per round, completion timestamps, owned BOOTTIME/timerfd, checked integer time conversion; fatal clock failure without alternate-clock fallback; unsafe confined to adapters. | D, B |

### 10.4 Screen and formatting

| Case / contract | Setup | Action | Expected outcome | Evidence |
| --- | --- | --- | --- | --- |
| V01 — Geometry (§5.1) | Measured selected view, 80x24; 60x16 and odd-height variants. | Render rows. | Header/separator/labels and 50x10 graphs at 80x24; fields/order/right-hand 28-column blocks/two-column gaps/bottom alignment; outgoing gets extra row; no footer/IP/axes/ticks/time labels; ASCII bounded rows. | D; visual evidence in T14 |
| V02 — Bar boundaries (§5.2) | Rates giving per-cell fill just below/at/above 0.30, 0.70 and 1; at/above ceilings. | Render both graphs. | Exact space/dot/vertical-bar/hash thresholds; bottom-up bars; full saturation; numeric rates uncapped. | D |
| V03 — History placement (§§4.5, 5.2) | Short history, measured zero column, delayed sample and fitting suffix. | Render and resize. | Blank missing space; zero occupies a column with no fill; newest right; no manufactured columns or mutation. | D |
| V04 — States (§5.3) | Baseline, absent, read-failed, measured idle, recovered baseline views. | Render each. | Exact status/placeholder table; baseline totals available; unavailable totals/history absent; measured zero numeric rather than `--`; header retained. | D |
| V05 — Small terminals (§5.4) | 59x16, 60x15, 60x16 and extreme tiny dimensions; retained histories. | Start small; sample/navigate/quit; shrink/grow. | Below either threshold warning/selection/status only as fits, no footer/wrap/out-of-bounds; sampling/navigation/quit continue; growth recovers layout/history. | D, P, E |
| V06 — Names/values (§5.4) | Overlong displayed/escaped control/non-ASCII name; large formatted value. | Render normal and tiny sizes; select by original bytes. | ASCII `\xHH`, ellipsis and position as fits; identity unchanged; field label retained; `[too wide]` instead of misleading numeric clipping. | D |
| V07 — Units/rounding (§5.5) | Zero, 128 bytes/s, 1024 total bytes, 0.125 Bit/s, 127.9995 bytes/s and values at both sides of every binary prefix boundary. | Format each independently. | `0.00 Bit/s`, `1.00 kBit/s`, `1.00 kByte`, `0.13 Bit/s`, `1024.00 Bit/s`; prefixes chosen before halfway-up rounding; two decimals; independent prefixes capped at G. | D |
| V08 — Large totals/ceilings (§§2.2, 5.5) | Exact maximum-u64 totals, largest valid ceilings, different incoming/outgoing ceilings. | Format/render. | No integer bit-conversion overflow or lost accounting; appropriate G-unit/overflow presentation; independently correct ceiling labels and graph saturation. | D |

### 10.5 Terminal, scheduling, failure and cleanup

| Case / contract | Setup | Action | Expected outcome | Evidence |
| --- | --- | --- | --- | --- |
| T01 — Support validation (§6.1) | Terminal/nonterminal stdin/stdout independently; redirected stderr; TERM unset/empty/dumb/linux and xterm-compatible declaration. | Start interactive or informational mode. | Interactive rejected variants stderr/status 1 before mutation; stderr redirection permitted; other declarations assumed compatible; help/version exempt. | D, P |
| T02 — Raw mode/flags (§6.1) | Save real PTY termios/file flags; scripted setup operations. | Enter UI and type Ctrl+C/Ctrl+Z. | Full raw mode, nonblocking input/output with saved flags; Ctrl+C byte selects 130, Ctrl+Z ignored rather than suspend; notification/session ownership before mutation. | D, P |
| T03 — Incremental arrows (§6.2) | CSI/SS3 bytes split at every boundary, repeated sequences and multiple events per read. | Feed decoder; exercise fragmented arrows through PTY. | Exactly one ordered navigation event per complete unmodified sequence; no blocking wait for remaining bytes. | D, P |
| T04 — Other/unknown input (§6.2) | Modified keys, excluded aliases, unknown escape sequences with command-looking suffixes, Ctrl+Z. | Feed sequences and plain q/Q controls. | Noncommands ignored/consumed; suffixes do not become quit/navigation; genuine plain q/Q quits. | D, P |
| T05 — Parser bounds (§6.2) | Incomplete input; overlong escape sequence; gaps below and at/above 100-ms idle bound. | Advance scripted input/clock. | Buffer bounded to 64 bytes; incomplete sequence discarded after idle timeout; timeout is wait deadline and causes no acquisition. | D |
| T06 — Signal wakeup/arbitration (§§6.1, 7.3) | Idle wait, collated/full-pipe wake-token scenarios, INT/TERM flags. | Deliver external signals and inspect simultaneous pending flags. | Wake ordinary flow; drain tokens before flags; handlers no allocation/drawing/cleanup; statuses 130/143, both pending INT wins; no pipe-byte signal identity. | D, P, B |
| T07 — Fairness/shutdown (§6.3) | Sustained input/resize batches and due sample deadlines; pending shutdown variant. | Drive application loop. | Shutdown first and stops further sampling/rendering; due sample before bounded batches; ordered navigation/coalesced redraw; rechecks prevent sampling starvation; no redraw-triggered samples. | D |
| T08 — Partial output (§6.3) | Would-block and short-write scripted terminal; newer requested views. | Start frames before/after some bytes emitted; keep feeding events/deadlines. | Only untouched frame replaceable; emitted frame resumed/finished before latest state; signals/input/sampling continue under backpressure. | D |
| T09 — Normal exits (§7.3) | Live UI; saved PTY state and observable session sequences. | q, Q, typed Ctrl+C, external SIGINT and SIGTERM in separate runs. | Status 0/0/130/130/143; applicable termios/flag restoration and show-cursor/leave-alternate output. | D, P; visual cleanup evidence in T14 |
| T10 — Initialization failures (§§2.3, 7.2) | Script failures before/after each applicable mutation, including partial session-sequence writes. | Initialize and fail. | Applicable restorations attempted even after partial writes; stderr after attempts/status 1; earlier untouched state not needlessly claimed changed; drop fallback nonpanicking. | D, P where practical |
| T11 — Runtime faults (§7.1) | Input EOF/hangup/unrecoverable I/O, clock failure and output failure. | Inject during active session; induce real harness faults where practical. | Fatal status 1, all applicable restoration attempts then diagnostic; counter acquisition failures remain nonfatal. | D, P where practical |
| T12 — Panic (§7.2) | Scripted unwinding panic during initialization/interactive execution; saved previous hook. | Catch/cleanup/report. | No default hook output before restoration; caught failure status 1; prior hook restored; cleanup/drop do not panic; no early process exit. | D, B |
| T13 — Cleanup failures (§7.3) | One/multiple restoration failures after q, SIGINT, SIGTERM or failure. | Finish session. | Attempt all applicable restorations; q success becomes 1; selected 130/143/1 preserved; cleanup diagnostic after attempts. | D |
| T14 — Visual protocols (§§5, 6.1) | One recorded xterm-compatible emulator; normal/small dimensions and traffic/history. | Navigate, start small, shrink/grow; separately q/Ctrl+C/SIGTERM. | Visible stacked screen/prompt selection/graph recovery; prior screen returns and cursor visible on each handled exit. PTY streams alone are insufficient evidence. | E |

### 10.6 Delivery, live smoke and verification records

| Case / contract | Setup | Action | Expected outcome | Evidence |
| --- | --- | --- | --- | --- |
| D01 — Build (§9) | Stable compiler, edition-2024 project and committed lockfile. | Run `cargo fmt --check`, `cargo test --locked`, `cargo build --release --locked`; record compiler. | All pass with locked versions; repeatable recorded-compiler build; no byte-identical-output claim. | B |
| D02 — Dependencies/modules (§§8–9) | Actual resolved target/feature graph and code. | Review direct baselines, signal-hook disabled defaults, transitives/licenses and ownership/seams. | Small GPL-2.0-only-compatible graph; no iterator thread/async runtime/terminal framework/parser crate required; module responsibilities, pure view/rows and scripted effect adapters implemented; unsafe stays in adapters. | B |
| D03 — Product files (§§1, 9) | Cargo metadata, executable, license and README. | Inspect identity/version authority/license text/credit/build instructions. | nload-rs/0.1.0, GPL-2.0-only metadata and GPL v2 text, exact agreed inspiration credit, Cargo.lock and compiler record, documented locked build/direct binary use. | B |
| D04 — Live Linux (§§2–4, 9) | Direct release binary, accessible proc, ordinary emulator and no administrative privileges. | Discover/select/navigate real interfaces; override ceilings; generate ordinary traffic on a monitored interface. | Real startup list/selection, correct independent ceiling labels, advancing totals and rate/graph activity; exact live rates not an oracle. | L |
| D05 — Reference/departures (§§1, 11) | Pinned research/source and this spec. | Review retained behaviors, explicit differences and amendments. | Source-backed retained stacked directions/field order/units/default ceilings/nominal cadence; no requirement to recreate quirks or screenshots; no unresolved superseded promise. | B |

Implementation acceptance requires the build gates, deterministic coverage, automated Linux PTY checks, recorded emulator checks and nonprivileged live smoke. Interface creation/removal/rename checks use deterministic fixtures; administrative privileges and exact live-rate comparisons are not required.

Record the implementation revision, exact compiler, target/environment and dependency graph review; required command outcomes; automated case-to-test mappings; PTY outcomes; emulator identity/observations; live smoke setup/observations; and artifact-review outcomes. Unrun cases remain pending, not passing. A harness limitation must identify the affected failure case and its scripted evidence where the agreed PTY-practicality rule applies. No implementation verification is claimed by this specification.

## 11. Accepted upstream departures and supersession notes

| Area | Retained/reference behavior | Accepted port departure |
| --- | --- | --- |
| Startup/list | Single-interface startup, all interfaces including loopback/inactive, discovery once. | Natural numeric order/raw-byte ties; initial-name selection without filtering; fixed defaults independent of config/HOME; defined discovery/argument failures. |
| Navigation | Four arrow directions and quit controls. | Symmetric per-event wrapping and prompt selection redraw; no PageUp/PageDown/Tab/Enter aliases or extra modes. |
| Timing/rates | Nominal 500-ms cadence; current/average/run extrema/cumulative-total roles. | BOOTTIME deadlines and exact elapsed windows/overlap instead of wall clock/frame counts; fractional calculations; safe baseline/reset/failure behavior; exact latest totals without guessed wraps. |
| History | Current-rate graph columns, newest right, saturating fixed scales. | Five-minute actual off-screen history from startup; timestamped observation expiry; no first-display/resize zero padding or fabricated catch-up. |
| Ceilings/units | `-i`/`-o`, independent 10240 binary-kBit/s defaults, binary-scaled units. | Strict checked whole-token arguments and status-2 rejection; explicit halfway-up rounding, widened exact-total formatting and no premature rate truncation/overflow. |
| Screen | Stacked directions, Curr/Avg/Min/Max/Ttl, ASCII partial bars, no footer. | Interface rather than Device/IP header; defined geometry, visible ceilings, exact availability distinctions, safe overflow and small-terminal warning. |
| Terminal/delivery | Interactive traffic monitoring and handled quit cleanup. | Owned direct Linux loop; xterm-compatible support declarations; alternate-screen/cursor handling; explicit signal/failure statuses and staged cleanup; distinct Rust package/version/GPL-2.0-only identity. |

Two cross-ticket amendments are already incorporated, not alternatives left for implementation:

1. [Decide traffic statistics and sampling behavior](issues/04-statistics-contract.md#resolution--2026-10-01--alice) extends [Decide the minimal startup and navigation contract](issues/03-startup-navigation-contract.md#resolution--2026-10-01--alice) with ceiling options and their grammar. All other startup/navigation requirements remain.
2. [Decide the port's architecture and test seams](issues/07-module-boundaries.md#resolution--2026-10-02--alice) amends [Validate the single-interface screen and terminal behavior](issues/05-single-interface-screen.md#resolution--2026-10-01--alice): target xterm-compatible emulator sessions, reject the specified TERM declarations, **show the cursor on cleanup rather than restore unknown prior visibility**, and apply cleanup-error/simultaneous-signal exit arbitration. Saved input mode/file flags and leaving the alternate screen remain required.

The planning route is complete. Implementation may choose private types, file subdivision and practical test-harness mechanics within these contracts; those are implementation details rather than open product decisions.
