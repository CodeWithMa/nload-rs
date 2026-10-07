# Find the way to a btop-style graph spec for nload-rs

Status: closed
Labels: wayfinder:map
Assignee:

## Destination

An implementation-ready spec for replacing the ASCII `. | #` graph bars with btop-style higher-resolution Braille dots + per-direction positional color, emitted as styled cells with no ASCII fallback. The route is complete when no decisions remain before implementation.

## Notes

- Domain: Linux network traffic monitoring. Read [Network Traffic Monitor glossary](../../GLOSSARY.md) and [Domain docs](../../docs/agents/domain.md). Use glossary terms (interface, graph sample/history/ceiling, current rate) consistently; sharpen new terms (graph glyph, Braille cell, color grade) via `domain-modeling`, don't dilute GLOSSARY.md with implementation detail.
- Standing preferences from charting (2026-10-07): destination is a spec to hand off, not code; btop reference is Braille + color gradient; replace outright (no dual-mode); screen emits escapes (pure-ASCII-rows seam is explicitly relaxed); higher subcell resolution over same-geometry glyph swap; no fallback (assume UTF-8 + xterm-256color). The original green→yellow→red preference was superseded by the prototype decision.
- Visual direction and final contract are indexed in Decisions so far. The styled contract also corrects the earlier research/prototype time-per-column wording; consult its resolution for authoritative history placement and cadence interpretation.
- Support/seam/test/dependency concerns raised during charting are resolved by [Decide the styled graph screen contract](issues/04-screen-seam-contract.md). Support remains a declared assumption, not a universal capability guarantee.
- Skills used: `grilling`, `domain-modeling`, `prototype`, `research`, and `codebase-design`. Current handoff: [btop-style graph spec](spec.md), linking the styled contract and inherited [Minimal Rust port spec](../nload-rust-port/spec.md). The [ASCII screen resolution](../nload-rust-port/issues/05-single-interface-screen.md) and [screen source](../../src/screen.rs) describe the pre-change application, not the new contract.
- HITL decisions require the human's participation. Charting resolves nothing by hand. Claim a ticket before work (Assignee = `alice (OpenCode session: <purpose>)`); use the tracker conventions in `docs/agents/issue-tracker.md`.
- Research assets live in `research/` within this effort. Research uses isolated throwaway worktrees on `research/<name>` branches. Ticket comments carry branch/worktree context pointers and locally accessible asset links. Refer to maps and tickets by linked title, never bare numbers.
- Completed with human confirmation on 2026-10-07. All child tickets are closed; the route is clear and the handoff is ready.

## Decisions so far

<!-- Named links to closed decision tickets only. Charting scope is recorded in Notes. -->

- [Establish what btop actually renders](issues/01-btop-rendering-facts.md): pinned v1.4.7 is 2-samples-per-cell Braille with positional per-row gradient and per-direction net hues, not per-value green→yellow→red.
- [Establish Braille and color terminal feasibility](issues/02-terminal-color-feasibility.md): feasible with zero new deps; Braille 3 bytes/1 cell + `38;5` ramp, seam must go cell-aware with per-row SGR reset.
- [Prototype the Braille and color graph look](issues/03-graph-prototype.md): accepted C (2 samples/cell) with per-direction hues and true-blank zeros; palette steps deferred to contract.
- [Decide the styled graph screen contract](issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract): fixed half-cell placement, nearest-dot fill, six-grade direction palettes, opaque styled frame, support policy, and acceptance migration; implementation handoff complete.

## Not yet specified

None. The route is clear; all child tickets are closed.

## Out of scope

- Building or shipping the new graphs: this map ends at an implementation-ready spec; code changes belong to a later build effort.
- Non-Linux support, multi-interface view, extra upstream modes, configuration files, address discovery: excluded by the port spec authority.
- Green→yellow→red value gradient (CPU-style): ruled out by [Prototype the Braille and color graph look](issues/03-graph-prototype.md) in favor of per-direction hues with human agreement.
- ASCII dual-mode or fallback UI: ruled out by the standing preference and confirmed in [Decide the styled graph screen contract](issues/04-screen-seam-contract.md).
