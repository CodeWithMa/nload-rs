# Prototype the Braille and color graph look

Status: closed
Labels: wayfinder:prototype
Type: prototype
Mode: HITL
Parent: [Find the way to a btop-style graph spec for nload-rs](../map.md)
Assignee: alice (OpenCode session: prototype graphs)
Blocked by: 01, 02

## Question

What should the replacement graphs look like to the human? Use a cheap throwaway mockup (HTML or terminal capture, not application code) showing representative histories in three forms: current ASCII `. | #`, same-geometry Braille + value gradient, and higher-resolution 2x4 Braille + value gradient. Include saturation, measured idle zeros, baseline/unavailable empty states, and incoming/outgoing sections at 80x24 and below 60x16. Get the human's reaction to dot density, gradient steps, and readability; link the prototype rather than building the application.

Use the confirmed btop facts from [Establish what btop actually renders](01-btop-rendering-facts.md) and feasibility limits from [Establish Braille and color terminal feasibility](02-terminal-color-feasibility.md). Decide only the accepted visual direction; exact thresholds, palette codes, and seam mechanics belong to the contract ticket.

## Comments

### Prototype ready — 2026-10-07 — alice (OpenCode session: prototype graphs)

Asset: [graph-look prototype](../prototype/graph-look.html) (throwaway, 18KB single file, no persistence).
Run: `python3 -m http.server -d .scratch/btop-graph-spec/prototype 8000` then open `http://localhost:8000/graph-look.html?variant=A` (flip A/B/C via bottom bar or ←/→ keys).
Covers: A=current ASCII, B=same-geometry Braille+gradient (1 sample/cell), C=2-sample/cell Braille+gradient (btop-faithful); scenarios ramp/saturated, idle zeros, baseline, absent; geometries 80x24, 60x16, 50x12 warning.
Evidence used: `braille_up` 25-entry table + positional per-row gradient (btop §1-3), 256-color ramp `46→…→226→…→196` with per-row SGR reset (feasibility §2-3), `no_zero` bottom-row dots vs blank zero.
Awaiting human reaction on dot density, gradient steps, readability before resolution.

### Resolution — 2026-10-07 — alice (OpenCode session: prototype graphs)

Resolved as prototype (HITL) with human. Accepted visual direction:

- Density: **C — 2 samples per cell** (higher-resolution 2×4 Braille, btop-faithful multiplexing). Confirms charting preference for higher subcell resolution over same-geometry swap. Same cell grid, 2× history per width; time-per-column halves.
- Hues: **per-direction hues** (btop-net indigo/magenta families), overturning the charting green→yellow→red value-gradient preference with human agreement. Positional per-row application stays (same color across whole row for height>1); exact families/steps belong to the contract ticket.
- Zeros: **true blank** for measured idle zeros (current blank-column look), rejecting btop `no_zero` faint `⣀` baseline. Baseline/absent/read-failed stay empty-history + status line; saturation stays full-height wall with uncapped numbers.
- Gradient steps/readability: deferred as “see Q2” — palette step count, SGR codes, stats-text contrast, and 60×16 legibility belong to the contract ticket under the per-direction-hue direction. Prototype showed 11-step `38;5` ramp with per-row reset and uncolored stats as the starting point.
- Small-terminal: warning path stays text-only (no Braille/color handling).

Asset: [graph-look prototype](../prototype/graph-look.html). No application code written. Handoff to [Decide the styled graph screen contract](04-screen-seam-contract.md) for exact 2-sample fill mapping, per-direction palettes, true-blank zero rule, seam accounting, and fixture migration.
