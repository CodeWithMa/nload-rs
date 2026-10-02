# Establish the release's default single-interface behavior

Status: closed
Labels: wayfinder:research
Type: research
Mode: AFK
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (research agent: release defaults)
Blocked by:

## Question

What exactly does upstream `v0.7.4` do on Linux when invoked as `nload` without arguments? Pin the release commit and investigate the default screen, displayed fields, units, graph scale and history, refresh and averaging periods, initial interface selection and ordering, arrow-key behavior, configuration-file influence, and traffic-statistic lifetimes. Trace both docs and source and distinguish discrepancies, bugs, and unknowns from established facts. Identify only the release behavior relevant to the scoped port, not master-only additions.

The answer supplies evidence for decisions; it does not choose which deviations the Rust port should make. Link a single research Markdown asset citing primary sources.

## Comments

### Research context

Claimed for delegated research during charting. Throwaway branch: `research/nload-release-defaults`; isolated worktree: `/tmp/nix-shell.TfI7CV/opencode/nload-release-defaults`. The branch is unborn because this repository has no commits. Findings will be linked from `../research/release-defaults.md` and mirrored in the research worktree; no commit is authorized.

### Resolution — 2026-10-01 — alice

Completed [nload v0.7.4: Linux no-argument single-interface behavior](../research/release-defaults.md), with immutable primary-source citations, exact defaults/calculations/graph thresholds, configuration influence, lifetime/error behavior, labeled uncertainty, and implications for later decisions. The same single logical asset is mirrored at `/tmp/nix-shell.TfI7CV/opencode/nload-release-defaults/research/release-defaults.md`; byte identity was verified.

Verified `v0.7.4` resolves to `a5715a77eb2c11702a7ffc7664240b9250cafa4d`. The author's release archive matches its published SHA-256; C++ sources, headers and man-page template match the tag. Built the unmodified archive in the temporary worktree and checked its 80×24 single-interface screen, four-arrow navigation and normal quit in a pseudo-terminal.

Factual gist: configuration-free defaults use automatic bits/s and byte totals, a 500 ms requested sleep interval, a 300 s setting implemented as 600 retained frames (599 intervals), and fixed 10240 kBit/s graphs. Candidates are initially name-sorted, include loopback/inactive interfaces, and are discovered once. Right/Down past the last returns to first; Left/Up at first stays there. All handlers keep sampling, but never-viewed graphs retain only one sample until first display. The man-page unit defaults are stale; config files can change no-argument behavior. Outstanding kernel guarantees and port choices fit the existing Linux-counter, startup/navigation, statistics, screen, engineering, architecture, and acceptance tickets; no new ticket is warranted by this research.
