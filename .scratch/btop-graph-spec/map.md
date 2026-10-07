# Find the way to a btop-style graph spec for nload-rs

Status: open
Labels: wayfinder:map
Assignee:

## Destination

An implementation-ready spec for replacing the ASCII `. | #` graph bars with btop-style higher-resolution Braille dots + value-gradient color, emitted as styled cells with no ASCII fallback. The route is complete when no decisions remain before implementation.

## Notes

- Domain: Linux network traffic monitoring. Read [Network Traffic Monitor glossary](../../GLOSSARY.md) and [Domain docs](../../docs/agents/domain.md). Use glossary terms (interface, graph sample/history/ceiling, current rate) consistently; sharpen new terms (graph glyph, Braille cell, color grade) via `domain-modeling`, don't dilute GLOSSARY.md with implementation detail.
- Standing preferences from charting (2026-10-07): destination is a spec to hand off, not code; btop reference is Braille + color gradient; replace outright (no dual-mode); screen emits escapes (pure-ASCII-rows seam is explicitly relaxed); higher subcell resolution over same-geometry glyph swap; no fallback (assume UTF-8 + xterm-256color); color encodes value gradient (green->yellow->red by fill).
- Tension to carry: replace-outright + no-fallback conflicts with preserved 60x16 small-terminal, deterministic-test, GPL-2.0-only, Linux-only, stable-Rust-2024 guarantees until the seam/terminal tickets resolve it. Do not assume universal Braille/color support without evidence.
- Skills: consult `grilling` and `domain-modeling` for decision conversations; `prototype` for the visual ticket; `research` for research tickets; `codebase-design` when the screen/terminal seam needs design work. Current authority: [Minimal Rust port spec](../nload-rust-port/spec.md) §5, [ASCII screen resolution](../nload-rust-port/issues/05-single-interface-screen.md), and [pure screen source](../../src/screen.rs) (ASCII rows, no escapes, 0.30/0.70/1.0 thresholds, saturation with uncapped numbers).
- HITL decisions require the human's participation. Charting resolves nothing by hand. Claim a ticket before work (Assignee = `alice (OpenCode session: <purpose>)`); use the tracker conventions in `docs/agents/issue-tracker.md`.
- Research assets live in `research/` within this effort. Research uses isolated throwaway worktrees on `research/<name>` branches. Ticket comments carry branch/worktree context pointers and locally accessible asset links. Refer to maps and tickets by linked title, never bare numbers.

## Decisions so far

<!-- Named links to closed decision tickets only. Charting scope is recorded in Notes. -->

## Not yet specified

<!-- In-scope fog: too coarse to ticket yet; graduates as the frontier advances. -->

- Exact Braille-dot mapping: how current 0.30/0.70/1.0 per-cell thresholds translate to 2x4 subcell fills, per-column vs per-cell color application, and saturation behavior at/above ceiling.
- Exact color palette and encoding: which ANSI codes/256-color steps form the green->yellow->red gradient, how they interact with small-terminal warning, baseline/unavailable/idle states, and statistics text.
- Screen/terminal seam and test fallout: new styled-cell interface shape, width accounting with multi-byte Braille + zero-width escapes, migration of the 12 screen fixtures and V01–V08 acceptance cases, performance and GPL-dependency impact.
- Rollout shape of replace-outright under no-fallback: whether TERM/locale checks, NO_COLOR, or dumb-terminal behavior need any guard despite the standing no-fallback preference.

## Out of scope

- Building or shipping the new graphs: this map ends at an implementation-ready spec; code changes belong to a later build effort.
- Non-Linux support, multi-interface view, extra upstream modes, configuration files, address discovery: excluded by the port spec authority.
- Per-direction hues (incoming vs outgoing colors): ruled out during charting in favor of the value gradient.
- ASCII dual-mode or fallback UI: ruled out by the standing no-fallback preference unless a frontier ticket overturns it with evidence and human agreement.
