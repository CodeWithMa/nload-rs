# Decide acceptance criteria and the specification handoff

Status: closed
Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: acceptance/handoff)
Blocked by: 03, 04, 05, 06, 07

## Question

What verifiable acceptance criteria and specification structure make the agreed port implementation-ready? Decide reference comparisons against the pinned release, deterministic counter/timing cases, keyboard and terminal scenarios, and required Linux smoke checks. Identify which observable departures are accepted and what an implementer must know about module contracts, failure handling, and delivery. Audit for unresolved decisions and graduate remaining fog before declaring the route clear. This ticket records the handoff decision; writing the consolidated specification and implementing the port happen after the planning map is complete.

Include CLI/informational-output and exit-status scenarios, natural-ordering/tie cases, and name-based interface lifecycle cases from the confirmed [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice), alongside keyboard and terminal scenarios.

Use the confirmed [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice) and its concrete examples for window-boundary/warm-up calculations, extrema/totals, delays, resets/returns, off-screen history/resize, unit thresholds and graph-ceiling argument validation. Its ceiling-option extension must be included alongside the earlier startup contract.

Use [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md#resolution--2026-10-01--alice) for cases covering footer-free stacked layout and 80x24 geometry, ASCII cell thresholds and saturation, ceiling labels, baseline/absent/read-error/idle distinctions, small-terminal startup/shrink/grow with active sampling/navigation, safe name/value overflow, ignored Ctrl+Z, and terminal restoration/statuses on handled exits, initialization/runtime failures and caught panics. The synthetic browser prototype is supporting evidence, not a real-terminal cleanup test.

Use [Decide Rust and delivery constraints](06-engineering-constraints.md#resolution--2026-10-01--alice) for acceptance of the package/executable identity, version authority/output, license and README credit, and reproducible locked Cargo build with direct execution of the compiled binary.

Use [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) for bulk snapshot failure/absence distinctions, discovery-baseline reuse, BOOTTIME deadlines, incremental input limits/fragmentation, nonblocking partial output, event fairness, scripted counter/clock/terminal adapters and staged cleanup. Include real Linux PTY checks in the coverage decision. Its explicitly confirmed amendments target xterm-compatible emulator sessions, reject unsupported TERM declarations, show the cursor on cleanup rather than preserve unknown prior visibility, and define cleanup-error and simultaneous-signal exit arbitration. The screen's earlier prior-visibility promise is superseded to that extent. Decide final coverage and handoff structure without reopening accepted architecture choices.

## Comments

### Resolution — 2026-10-02 — alice

The human explicitly answered "accept all" to the five live recommendations covering acceptance authority, deterministic coverage, real-terminal verification, delivery gates and specification handoff. This resolution is the canonical acceptance/handoff decision. It completes planning; it does not assert that implementation or verification has happened.

#### Acceptance authority and reference comparisons

- The confirmed ticket contracts are authoritative. Upstream nload `v0.7.4`, pinned to commit `a5715a77eb2c11702a7ffc7664240b9250cafa4d`, supplies reference evidence, not a second acceptance oracle.
- Reference comparisons cite the pinned source/research for retained behavior: stacked incoming/outgoing directions, statistic field order, binary-scaled units, default ceilings and nominal cadence. Record accepted departures explicitly.
- Running upstream again is optional. Screenshot equality and reproduction of release quirks are not gates. The browser screen prototype is supporting visual evidence, not a real-terminal cleanup test.
- Preserve the graph-ceiling argument extension from [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice) when consolidating the earlier startup contract.
- The explicit terminal/cleanup amendments in [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) govern: target ordinary xterm-compatible emulator sessions; reject unset/empty TERM and exact `dumb`/`linux`; show the cursor on cleanup rather than promise preservation of unknown prior visibility; apply the confirmed cleanup-error and simultaneous-signal exit arbitration. Normalize the screen ticket's superseded visibility promise in the handoff.

#### Deterministic acceptance coverage

Require a coverage matrix linking every contractual requirement to an automated test or explicit manual check. Use the agreed module/view/row interfaces and scripted counter, clock and terminal adapters rather than exposing private queues or bookkeeping. The following automated coverage is mandatory; canonical source contracts supply the exact expected outcomes:

| Area | Required scenarios | Canonical contract |
| --- | --- | --- |
| Arguments and identity | No-argument startup; initial exact-name selection without filtering; `--` handling; standalone help/version without clock/discovery/terminal work; exact `nload-rs 0.1.0\n` output sourced from package metadata; unsupported combinations/flags and multiple/unknown names; informational stdout, diagnostic stderr and agreed statuses; no short help/version aliases; configuration/HOME independence. | [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice), [Decide Rust and delivery constraints](06-engineering-constraints.md#resolution--2026-10-01--alice), [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) |
| Ceiling options | Independent defaults and conversion; options before/after the initial name until `--`; zero reset; leading zeros; repeated last-wins values; valid maximum and overflow boundary; missing values, signs, fractions, whitespace, suffixes and attached forms; help/version combinations; help description of units/defaults/zero. | [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice) |
| Ordering and navigation | Natural numeric ordering including unbounded digit-run comparison, raw-name ties and case/locale independence; raw-byte identity; loopback/inactive inclusion; all four arrows with symmetric wrapping and ordered per-event steps; one-interface no-op; ignored aliases; prompt redraw without acquisition. | [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice), [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) |
| Acquisition and lifecycle | Complete byte snapshots with checked paired u64 counters; malformed/incomplete snapshots and read failures versus absence in a valid snapshot; startup empty discovery versus enumeration failure; discovery snapshot reused as baselines; runtime bulk failure affects every retained entry without fatal UI failure or repeated stderr diagnostics; fixed startup membership; removal/rename/new names/return/name reuse; either-direction discontinuity resets both directions. | [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice), [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice), [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) |
| Statistics and history | Confirmed concrete numerical examples; baseline with totals but no synthetic rates/extrema/columns; one-second/300-second overlap windows, warm-up and fractional boundary intervals; actual elapsed delays; ignored nonpositive times; exact cumulative totals; independent run extrema and measured zero; decreases/failures/returns without guessed wraps or gap deltas; off-screen history; five-minute boundary retention/expiry; resize/navigation without observations or resets. | [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice), [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) |
| Screen and formatting | Footer-free stacked layout and field order; 80x24 geometry; minimum-size boundaries and odd-height split; independent ceiling labels; ASCII cell thresholds at 0.30/0.70/1 and saturation with uncapped numbers; newest-right fitting history with blank unfilled space; baseline/absent/read-failed/idle distinctions; small startup/shrink/grow with active sampling/navigation; escaped/truncated names with preserved position/identity; `[too wide]`; binary prefix boundaries, pre-rounding prefix selection, halfway-up two decimals, capped G units and overflow-safe exact totals. | [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md#resolution--2026-10-01--alice), [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice), [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) |
| Input and scheduling | Fragmented/repeated CSI and SS3 arrows; q/Q and Ctrl+C; ignored Ctrl+Z/other/modified input; unknown escape sequences do not become suffix commands; 64-byte buffering and 100-ms idle timeout; BOOTTIME deadlines anchored to discovery completion; due acquisition and missed slots without catch-up observations; input/resize fairness; coalesced redraws and no redraw-triggered acquisition. | [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) |
| Output, failure and shutdown | Nonblocking partial-frame output; replacement only before any byte is emitted; continued event/sampling handling during backpressure; shutdown priority/wakeups; clock/UI initialization/runtime failures; EOF/hangup/unrecoverable I/O errors; staged restoration including partial initialization, all applicable restoration attempts despite errors and nonpanicking fallback; caught panic restoration before diagnostics; q/Q 0, Ctrl+C/SIGINT 130, SIGTERM 143; simultaneous pending SIGINT precedence; cleanup-only failure changes successful q to 1 without replacing an already selected signal/failure status. | [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice) |

- Compare counters, states, sample counts, mandated text and rendered fixtures exactly. Diagnostic/help prose is checked against its contractual contents; the existing contracts do not mandate every prose byte.
- For derived rates, accept an absolute error no greater than `1e-9 * max(1, abs(expected_rate))` bytes/s. This combines an absolute floor with relative tolerance for the accepted f64 calculations.
- Formatted text and graph-threshold decisions remain exact checks, not tolerance-based checks. Cover values on either side of and at the contractual boundaries.

#### Real Linux terminal verification

Both automated Linux PTY checks and a recorded manual check in one xterm-compatible emulator are required.

**Automated PTY coverage:**

- Exercise the real binary's raw keyboard input, fragmented arrows, q/Q, typed Ctrl+C, external SIGINT/SIGTERM, ignored Ctrl+Z, resize and EOF/hangup handling.
- Exercise supported/rejected TERM declarations and pre-mutation rejection behavior.
- Verify restoration of saved termios/file flags on handled exits; observe applicable cursor-show and alternate-screen-exit output.
- Verify initialization/runtime failure paths with harness-controlled failures where practical. Scripted adapters cover failures a PTY cannot reliably induce, including cleanup arbitration and caught-panic/staged-cleanup paths.

**Recorded emulator coverage:**

- Confirm visible stacked layout, prompt navigation, small-terminal startup/shrink/grow, and graph recovery after growth.
- Confirm return to the prior screen and a visible cursor after q, Ctrl+C and SIGTERM.

PTY checks establish Linux OS/FD behavior; the emulator check establishes visible behavior of the assumed screen protocols. No new hard redraw-latency budget or universal emulator-support guarantee is introduced. Restoration remains best effort where an exit or terminal failure cannot be fully handled.

#### Build, live smoke and delivery gates

Require the following commands to pass during the subsequent implementation acceptance:

```text
cargo fmt --check
cargo test --locked
cargo build --release --locked
```

- Record the exact tested compiler version and verify the resolved target/feature dependency graph and license compatibility. Use the selected stable Rust/edition 2024 and dependency baselines from the confirmed engineering/architecture decisions.
- Run `./target/release/nload-rs` directly on Linux. A nonprivileged live smoke check confirms real interface discovery, initial selection/navigation, independent ceiling overrides, advancing cumulative totals and rate/graph activity under ordinary generated traffic.
- Exact live rates are not an oracle. Privileged interface creation/removal/rename scenarios remain deterministic fixture cases; administrative privileges are not a smoke-check requirement.
- Delivery includes the agreed package/executable identity and Cargo-authoritative version, committed `Cargo.lock`, GPL version 2 license text and metadata, the agreed README inspiration credit, and documented locked-build/direct-run commands.
- Clippy may be used but is not an additional acceptance gate. No hard resource budget, suspend-specific test campaign, publication, installation or distribution-packaging requirement is added.

#### Specification handoff

After this planning map closes, consolidate the confirmed decisions into [Consolidated Rust-port specification](../spec.md). That file is the next handoff artifact; it does not exist at resolution time. Use this structure:

1. Scope, identity and reference baseline.
2. CLI/startup contract.
3. Interface ordering, navigation and lifecycle.
4. Acquisition, timing and statistics.
5. Screen, formatting and graph rules.
6. Terminal support, input and event scheduling.
7. Failures, cleanup and exit statuses.
8. Module ownership and caller-facing contracts.
9. Dependencies, licensing and delivery.
10. Acceptance matrix and verification procedure.
11. Accepted upstream departures and explicit supersession notes.

Each acceptance case identifies setup, action, expected outcome and evidence type. Link to canonical ticket resolutions for rationale. Consolidate the actual module ownership/interface contracts and failure semantics, not only topic headings. Normalize superseded promises instead of leaving contradictory requirements for the implementer to adjudicate. Record the required verification evidence when implementation acceptance happens.

Planning completion means the decisions are confirmed and no product choices remain unresolved. Implementation acceptance means the required checks later pass with their evidence recorded. Specification consolidation is the next activity after planning, followed by implementation.

#### Final audit and map completion

- Audited the confirmed startup, statistics, screen, engineering and architecture resolutions. Their remaining-decision pointers terminate in this coverage/handoff decision; the graph-ceiling and terminal amendments have explicit owners and precedence.
- Every other child ticket is closed. No remaining fog or newly specifiable decision was found, and no scope change or new ticket is needed. The route is clear and the planning map can close.
- The application and consolidated specification have not been built or written in this session. The acceptance commands and terminal/live checks above are future implementation gates, not claimed results.
- This session resolves only this ticket. Existing domain terminology is sufficient; no new glossary term or ADR is needed.
