# Establish Braille and color terminal feasibility

Status: open
Labels: wayfinder:research
Type: research
Mode: AFK
Parent: [Find the way to a btop-style graph spec for nload-rs](../map.md)
Assignee: alice (research agent: terminal color)
Blocked by:

## Question

Can nload-rs emit higher-resolution Braille dots + ANSI value-gradient color under its standing constraints (Linux-only, stable Rust 2024, GPL-2.0-only deps, deterministic tests, 60x16 small-terminal mode) while assuming UTF-8 + xterm-256color with no ASCII fallback? Establish from primary sources and local probes: Unicode Braille width/encoding behavior, ANSI color code choices for the green->yellow->red gradient, how multi-byte glyphs and zero-width escapes affect the bounded-rows width contract in `src/screen.rs`, NO_COLOR/dumb-terminal/non-UTF8 exposure, and whether any new dependency is needed.

The answer supplies evidence for the seam decision; it does not choose fallback policy or palette. Link a single research Markdown asset citing primary sources with a branch/worktree context pointer. Do not write application code or change agreed user-visible behavior.

## Comments

- 2026-10-07 alice (research agent: terminal color): findings linked, ticket left open. Asset: `.scratch/btop-graph-spec/research/terminal-color-feasibility.md` (sha256 `f9f64eb4…5fe375`, mirrored identically). Context: throwaway branch `research/terminal-color` (base `4d4e8ac` on `dev`, uncommitted by design) with isolated worktree `/tmp/opencode/nload-rs-research-terminal-color` holding the mirrored asset. Gist: feasible with zero new deps — Braille is 3 bytes/1 cell (U+2800–U+28FF, RFC 3629 + UAX #11), gradient ramp is hardcoded SGR `38;5` codes `46→…→226→…→196` (ECMA-48 + local terminfo `colors#256`), but the `String::len()`-bytes bounded-rows contract in `src/screen.rs` + `queue_frame` validation must become cell-aware with zero-width escapes and per-row SGR reset; NO_COLOR is currently ignored and non-UTF-8/under-declared-TERM behavior is policy for the seam ticket, not a technical blocker. No application code written.
