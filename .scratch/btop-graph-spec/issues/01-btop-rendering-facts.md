# Establish what btop actually renders

Status: open
Labels: wayfinder:research
Type: research
Mode: AFK
Parent: [Find the way to a btop-style graph spec for nload-rs](../map.md)
Assignee: alice (research agent: btop rendering)
Blocked by:

## Question

What exactly does btop render for its Braille-dot + value-gradient graphs that this effort means by "how btop displays it"? Pin a concrete btop version/commit, then establish from primary source and observed behavior: which Braille characters are used, how samples map to 2x4 subcells, how the green->yellow->red gradient is computed (per-dot, per-cell, per-column; by fill fraction or rate), how saturation/zero/idle history is drawn, and what geometry trade-offs a higher-resolution graph implies versus the current H x W ASCII columns with 0.30/0.70/1.0 thresholds.

The answer supplies evidence for later spec decisions; it does not choose the nload-rs mapping. Link a single research Markdown asset citing primary sources with a branch/worktree context pointer. Do not write application code or change agreed user-visible behavior.

## Comments

### Research context — 2026-10-07 — alice (research agent: btop rendering)

Findings: [btop rendering facts](../research/btop-rendering.md) (pinned btop
v1.4.7, commit `6e39144aaf5a6bc01b9f795010b0914431067183`).
Context: throwaway branch `research/btop-rendering` (no commit, HEAD 4d4e8ac),
isolated worktree `/tmp/opencode/nload-rs-research-btop-rendering` with
byte-identical asset mirror (sha256
`2513d73061bd5ef089a3ef9c56afe50b248849a64849a434f6dcc800d421d20f`).
Status stays open for the charting session's resolution indexing; no
application code written.
