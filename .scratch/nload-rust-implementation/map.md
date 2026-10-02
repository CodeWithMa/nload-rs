# Implement and verify the minimal Rust port of nload

Status: open
Labels: wayfinder:map
Assignee:

## Destination

A working Linux `nload-rs` application satisfying [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), with the required build, deterministic, Linux PTY, recorded emulator, live-smoke and artifact-review evidence. Completion means every required acceptance case has passing evidence, not merely that the binary builds.

## Notes

- **Execution override:** On 2026-10-02 the human requested: “yes Proceed to implementation with new implementation map”. This effort carries application implementation and verification into the map. Execution-oriented `wayfinder:task` tickets are intentional; their resolutions record code changes, checks and resulting facts. The earlier planning map's destination remains complete.
- Acceptance authority: [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), consolidated at repository commit `178b1d2`. Its linked resolutions retain rationale. Follow all sections, including the amendments in section 11; do not reopen accepted product or architecture choices without a concrete conflict and human agreement.
- Domain: Linux network traffic monitoring. Read [Network Traffic Monitor glossary](../../GLOSSARY.md) and [Domain docs](../../docs/agents/domain.md). Consult `codebase-design` when an implementation seam needs design work; consult `grilling` and `domain-modeling` when a new human decision is genuinely required. No domain changes are presumed.
- Tracker: [Local Markdown issue tracker](../../docs/agents/issue-tracker.md). Claim one open, unblocked, unassigned child by setting Assignee to `alice (OpenCode session: <purpose>)` before work. Work and resolve at most one non-research ticket per session. Concurrent sessions must re-read metadata before claiming or editing shared files.
- Charting creates this map and its tickets; it does not resolve an implementation ticket. Later sessions execute the frontier in numeric order unless the human names another available ticket.
- Keep the five established deep boundaries and thin application loop. Pure module tests observe monitor views and screen rows. Production and scripted counter/clock/terminal adapters are required; unsafe Linux calls remain in concrete adapters. Agree on shared interfaces through their owning tickets rather than editing another session's work.
- Record verification in `verification.md` alongside this map when checks begin. Map every acceptance case in specification section 10 to test names or recorded manual evidence. Record actual implementation revision, compiler/target/environment, command outcomes and dependency/artifact reviews. Unrun or failed checks remain pending or failed. A ticket resolution links evidence; the map only indexes its gist.
- Required final commands: `cargo fmt --check`, `cargo test --locked`, `cargo build --release --locked`, followed by direct Linux execution with `./target/release/nload-rs`. Run appropriate checks for each ticket; intermediate passes do not establish final application acceptance. Never commit unless explicitly requested. Track the required `Cargo.lock`; its Git commit remains part of the eventual authorized repository handoff.
- Initial environment observations: no Cargo project or application source exists; the worktree was clean. Available tools report Rust `1.98.1` (`48a229ceaefd4985c50990b14116b6d856af0985`, host `x86_64-unknown-linux-gnu`) and Cargo `1.98.0`. `TERM=alacritty` is an environment declaration, not observed visual verification. Record the actual tools and emulator used when verification runs.
- Human participation is required for recorded emulator observations. PTY output cannot substitute for them. Use no production test-only CLI flags and require no administrative privileges for live smoke.

## Decisions so far

<!-- One named link and short outcome per resolved child. Implementation evidence and detailed resolutions live in the tickets and their linked assets. -->

- [Establish the Rust project and delivery foundation](issues/01-project-foundation.md#resolution--2026-10-02--alice): Edition-2024 Cargo foundation, required locked baselines and GPL delivery files verified; foundation checks pass, with application acceptance still pending.
- [Implement arguments, proc snapshots and startup ordering](issues/02-arguments-proc-startup.md#resolution--2026-10-02--alice): Strict raw-byte invocation, complete proc snapshots and natural startup selection implemented; 13 deterministic tests and locked build pass, with monitor/application integration evidence pending.
- [Implement the pure monitor and statistics runs](issues/03-monitor-statistics.md#resolution--2026-10-02--alice): Fixed-list monitor, exact-delta windows, paired resets and retained off-screen histories implemented with a borrowed selected view; 15 new fixtures and all 28 tests pass, with screen/runtime evidence pending.
- [Implement the pure ASCII screen and formatting](issues/04-ascii-screen.md#resolution--2026-10-02--alice): Bounded owned ASCII rows, stacked graphs/statuses and exact widened formatting implemented; 12 new fixtures and all 40 tests pass, with terminal/application and visual evidence pending.
- [Implement staged terminal ownership and shutdown cleanup](issues/05-terminal-session.md#resolution--2026-10-02--alice): Owned raw/nonblocking terminal session, atomic signal pipe, exhaustive restoration and caught-panic/exit arbitration implemented; 19 new checks and all 59 tests pass, with runtime/application and product PTY evidence pending.
- [Implement incremental input and nonblocking runtime effects](issues/06-terminal-events.md#resolution--2026-10-02--alice): Bounded CSI/SS3 decoder, checked BOOTTIME/timerfd readiness and single-frame resumable nonblocking output implemented with shared scripted effects; 16 new checks and all 75 tests pass, with application fairness/sampling and product terminal evidence pending.
- [Integrate startup, sampling and the interactive application loop](issues/07-application-integration.md#resolution--2026-10-02--alice): Working binary, guarded startup and shutdown-first anchored sampling/redraw loop integrated; 18 new fixtures and all 93 tests/build gates pass, unblocking product PTY verification.
- [Verify the real binary through automated Linux PTY checks](issues/08-linux-pty.md#resolution--2026-10-02--alice): Eleven real-product PTY checks pass against debug/release, covering retained OS-state restoration and practical faults; all 104 tests/build gates pass, unblocking recorded emulator/live smoke.

## Not yet specified

- Integration and real-FD verification may expose concrete interactions among shutdown, partially emitted output, signal wakeups and staged restoration. The contracts are settled; any newly revealed corrective work should become a precise child ticket when evidence makes it specifiable.
- The final acceptance review may reveal uncovered contractual details or environment-specific gaps. Graduate concrete gaps into tickets rather than weakening acceptance or assuming a pass.

## Out of scope

- Replanning the confirmed product: the specification is the authority for this implementation effort. New features or changed behavior require an explicitly agreed new destination.
- Non-Linux support, multi-interface view, extra upstream options/modes, configuration files, address discovery, suspend/resume UI and a public extensibility library: excluded by the specification.
- Installation, publication and distribution packaging: this destination is a verified directly runnable binary and its source/artifacts.
- Recreating upstream quirks, screenshot equality, exact live-rate comparisons, privileged interface manipulation and a separate suspend test campaign: not required acceptance evidence.
