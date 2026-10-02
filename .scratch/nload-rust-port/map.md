# Find the way to a minimal Rust port of nload

Status: closed
Labels: wayfinder:map
Assignee:

## Destination

An implementation-ready specification for a Linux Rust traffic monitor inspired by upstream nload's published `v0.7.4` release: launching `nload-rs` opens the single-interface view, an interface-name argument selects the initial interface, arrow keys navigate between interfaces, incoming/outgoing graph ceilings can be overridden by arguments, and `--help`/`--version` provide informational output. The route is complete when no decisions remain before implementation.

## Notes

- Upstream: <https://github.com/rolandriegel/nload>, using the release tag rather than current master. Research must pin the release commit and distinguish source behavior from documentation.
- This effort plans the port. Application implementation and production-ready deliverables belong to the subsequent build effort. Research notes and throwaway prototypes may support decisions.
- Scope confirmed by the human during charting: Linux only; release baseline; default single-interface view with arrow-key navigation; no multi-interface view.
- Scope clarified by the human during [Decide the minimal startup and navigation contract](issues/03-startup-navigation-contract.md): support `--help`, `--version`, and an interface-name argument for initial selection; the application is not restricted to no-argument invocation.
- Scope expanded by the human during [Decide traffic statistics and sampling behavior](issues/04-statistics-contract.md): require arguments for incoming/outgoing graph-ceiling overrides as well as fixed defaults. That ticket owns the extension's exact argument contract.
- Consult `grilling` and `domain-modeling` for decision conversations. Consult `prototype` for the screen ticket and `research` for research tickets. Consult `codebase-design` when deciding module boundaries.
- HITL decisions require the human's participation. Charting does not resolve them. Claim a ticket before work; use the tracker conventions in `docs/agents/issue-tracker.md`.
- Research assets live in `research/` within this effort. Research uses isolated throwaway worktrees on `research/<name>` branches; this fresh repository has no commits, so those branches can remain unborn until explicitly authorized to commit. Ticket comments carry branch/worktree context pointers and locally accessible asset links.

## Decisions so far

<!-- Named links to closed decision tickets only. Charting scope is recorded in Notes. -->

- [Establish the release's default single-interface behavior](issues/01-release-default-behavior.md): Pinned the release and established source-backed defaults, screen/statistic behavior, configuration influence, and navigation quirks.
- [Establish Linux interface and traffic-counter semantics](issues/02-linux-counter-semantics.md): Established counter APIs, identity and namespace limits, acquisition trade-offs, and rate/total constraints without selecting a backend.
- [Decide the minimal startup and navigation contract](issues/03-startup-navigation-contract.md#resolution--2026-10-01--alice): Confirmed natural ordering, startup selection/help/version arguments, fixed name-based discovery, symmetric arrows, configuration independence, and explicit exit/terminal behavior.
- [Decide traffic statistics and sampling behavior](issues/04-statistics-contract.md#resolution--2026-10-01--alice): Confirmed time-windowed statistics, discontinuity-safe baselines, sampling/history lifetimes, units and validated graph-ceiling arguments.
- [Validate the single-interface screen and terminal behavior](issues/05-single-interface-screen.md#resolution--2026-10-01--alice): Confirmed stacked ASCII graphs without a footer, visible ceilings, explicit availability states, size/overflow rules and terminal restoration.
- [Decide Rust and delivery constraints](issues/06-engineering-constraints.md#resolution--2026-10-01--alice): Confirmed `nload-rs` 0.1.0, stable Rust/edition 2024, small compatible dependency set, GPL-2.0-only with README inspiration credit, and locked Cargo builds for direct binary use.
- [Establish terminal, clock and dependency feasibility](issues/09-terminal-clock-library-facts.md): Verified compatible library/clock options and bulk proc acquisition; exposed terminal-dependent cursor restoration and crossterm input/wakeup limitations for the architecture decision.
- [Decide the port's architecture and test seams](issues/07-module-boundaries.md#resolution--2026-10-02--alice): Confirmed bulk proc sampling, pure monitoring/rendering modules, a direct Linux event loop and deterministic seams, with explicit terminal-support and cleanup refinements.
- [Decide acceptance criteria and the specification handoff](issues/08-acceptance-handoff.md#resolution--2026-10-02--alice): Confirmed acceptance authority, deterministic/PTY/emulator/live-Linux gates and specification handoff structure; final audit found no remaining decisions, completing the planning route.

## Not yet specified

<!-- Final acceptance/handoff audit found no remaining fog. All child tickets are closed; the planning route is complete. -->

## Out of scope

- Non-Linux support: the initial port targets Linux only.
- Multi-interface view: explicitly excluded by the human.
- Additional upstream modes and features beyond the single-interface view and scoped startup arguments: feature expansion is outside this destination. Help/version output, initial interface selection, graph-ceiling arguments, navigation, quit, and terminal-lifecycle behavior are in scope.
- Building, publishing, or packaging the application: this map ends at an implementation-ready specification; choosing delivery constraints is in scope.
