# Terminal color + Braille feasibility (research)

Ticket: [Establish Braille and color terminal feasibility](../issues/02-terminal-color-feasibility.md).
Parent map: [Find the way to a btop-style graph spec for nload-rs](../map.md).
Branch/worktree: `research/terminal-color` at `/tmp/opencode/nload-rs-research-terminal-color`
(base commit `4d4e8ac` on `dev`); this asset is mirrored there and hash-verified (see ticket comment).
No application code was written; no user-visible behavior was changed.

## Verdict

Feasible under the standing constraints with **zero new dependencies**, provided the
screen/terminal seam stops counting `String::len()` bytes as columns and treats SGR
escapes as zero-width. Every other risk (NO_COLOR, non-UTF-8 locale, under-declared
`TERM`) is a *policy* question for the seam/spec tickets, not a technical blocker:
the bytes work, the width contract does not survive them unchanged.

## 1. Braille: encoding, bytes vs cells

- Block **Braille Patterns U+2800–U+28FF** (256 patterns). Code point =
  `U+2800 + bitmask`, dots 1–8 mapping to bits 0–7 (dots 1–6 in the top 3 rows,
  dots 7–8 in the bottom row).
  Source: The Unicode Standard, ch. 7 (European Alphabetic Scripts — Braille
  Patterns) and code chart U2800.pdf; `unicodedata.name()` locally confirms
  e.g. U+283F = "BRAILLE PATTERN DOTS-123456", category So.
- **UTF-8: 3 bytes per Braille cell, always.** U+0800–U+FFFF encodes as
  `1110xxxx 10xxxxxx 10xxxxxx` (RFC 3629 §4); locally verified:
  U+2800 → `e2 a0 80`, U+283F → `e2 a0 bf`, U+28FF → `e2 a3 bf`.
- **Display width: 1 cell.** Braille has East Asian Width Neutral (Unicode
  Standard Annex #11); `wcwidth` = 1 on conforming implementations. It is *not*
  zero-width and *not* double-width — one cell per pattern, including U+2800
  BLANK (which advances the cursor unlike space only in that it carries no ink).
- Consequence: a graph row holding W Braille cells is `3W` bytes but W columns.
  Any assertion of the form `row.len() == width` fails by a factor of 3 before
  color is even considered (local probe: `'⣿'.encode('utf-8')` = 3 bytes).

## 2. Gradient color: SGR 256-color codes

- Foreground color is `ESC [ 38 ; 5 ; n m` (SGR "extended foreground"), reset with
  `ESC [ 0 m`. Source: ECMA-48 §8.3.117 (SGR); local terminfo confirms the
  spelling: `setaf=\E[...38;5;%p1%d...m` in `infocmp xterm-256color`
  (`colors#0x100` = 256 colors; `setab` is the `48;5` background twin).
- Green→yellow→red path along the top edge of the 6×6×6 cube
  (`code = 16 + 36·r + 6·g + b`, verified against the xterm palette):
  `46 → 82 → 118 → 154 → 190 → 226 → 220 → 214 → 208 → 202 → 196`
  (r ramps 0→5 at g=5,b=0, then g ramps 5→0 at r=5,b=0).
  Midpoints 226 (yellow) and 196 (pure red) match the btop-style gradient poles;
  the 11-step ramp can be subsampled to fewer grades by the palette ticket.
- Each SGR sequence is **10–12 bytes, zero display cells** (local probe:
  `ESC[38;5;46m` = 10 bytes, `ESC[0m` = 4 bytes). Per-cell coloring therefore
  costs ~14 bytes/cell on the wire; per-column coloring (one SGR pair per column
  run) is cheaper but is a seam-ticket decision, not a feasibility question.
- Truecolor-capable terminals accept 256-color SGR (it is a subset of the xterm
  control-sequence grammar; source: XTerm Control Sequences, "SGR" section), so
  emitting `38;5;n` degrades gracefully upward. No capability probe is needed to
  *emit*; whether to *probe* is §4 policy.

## 3. Impact on the bounded-rows width contract (`src/screen.rs`, `src/terminal/runtime.rs`)

- Today's contract (`src/screen.rs:17,21,43-46,257-263`): `render()` returns
  exactly `height` rows of exactly `width` **bytes**, printable ASCII only
  (`0x20..=0x7E`), enforced by the `bounded()` test helper and by
  `String::from_utf8(row).expect("screen rows contain only ASCII")`.
- `Runtime::queue_frame` (`src/terminal/runtime.rs:120-150`) re-enforces it from
  the other side: rejects any row with `row.len() > width` or any byte outside
  `0x20..=0x7E`, then emits with per-row absolute cursor positioning
  (`ESC[{i};1H`) and `trim_end_matches(' ')`.
- Three breakages, all in *accounting*, none in terminal capability:
  1. Braille breaks `len() == width` (3 bytes/cell) and the ASCII-only rule.
  2. SGR breaks `len() <= width` (10–14 invisible bytes/cell) and the ASCII-only
     rule (`ESC` is `0x1B`).
  3. `trim_end_matches(' ')` is escape-unsafe: trailing spaces before a reset
     sequence would leave `ESC[0m` dangling at line end, and trimming must never
     split a multi-byte glyph or an escape. Each row needs its color reset
     *before* the cursor-move to the next row or color bleeds into statistics
     text and the `=` separator.
- Required seam shape (evidence only; design belongs to the seam ticket): width
  accounting in **display cells** with escapes counted as zero-width, glyph
  slicing on char/sequence boundaries, and a per-row "reset SGR" invariant.
  The pure styled-cell model itself stays deterministic and unit-testable (see §6).

## 4. NO_COLOR / dumb / non-UTF-8 exposure

- **NO_COLOR: currently ignored.** `grep NO_COLOR src/` returns zero hits, so
  under replace-outright the gradient would emit for NO_COLOR users too.
  Source of the convention: <https://no-color.org/> (env `NO_COLOR`, any value
  incl. empty, suppresses color). Honoring it is a one-`env::var_os` check, but
  *whether* to honor it despite the no-fallback preference is a human/spec
  decision — flagged as the one policy blocker (§7).
- **dumb/linux TERM: already rejected.** `src/terminal/session.rs:197-210`
  (`validate_support`) rejects unset/empty/`dumb`/`linux` before mutation; the
  `linux` console is 16-color-only so rejecting it is consistent with a
  256-color assumption. No change needed.
- **Under-declared TERM (e.g. `xterm` without `-256color`): assumed compatible
  today.** Session tests state it explicitly
  (`src/terminal/session/tests.rs:399`: "Other declarations, even non-UTF-8, are
  declarations, not capability probes"; port spec §6.1: "Other TERM values
  declare assumed compatibility"). 256-color SGR on an 8-color terminal is
  typically rendered as wrong-but-legible color or ignored — cosmetic, never a
  crash — but it is the largest real-world rendering risk of the no-fallback
  stance and should be named in the spec.
- **Non-UTF-8 locale: mojibake, no crash path in our code.** There is no
  locale/encoding check anywhere in `src/` today. A 3-byte Braille cell in a
  non-UTF-8 locale renders as 3 garbage cells (or 6 with `C` locale fallbacks
  in some emulators) and breaks column alignment visually. Bytes still flow;
  `libc::write` has no encoding opinion. Again policy, not feasibility.

## 5. xterm-256color assumption risks

- The assumption is already the project's posture (port spec §6.1: ordinary
  xterm-compatible sessions; map Notes: assume UTF-8 + xterm-256color). What
  changes is only that Braille + `38;5` make the assumption *load-bearing*:
  | Risk | Severity |
  |---|---|
  | `TERM=xterm-256color` (or `-direct`, tmux/screen-256color): `colors#256` present, `setaf` = `38;5` | none |
  | `TERM=xterm`, `screen`, `vt100` | wrong colors possible, layout intact (SGR is zero-width either way) |
  | `TERM=dumb`/`linux`/unset | rejected pre-mutation today — safe |
  | multiplexers translating 256↔truecolor | cosmetic shade shift only |
- No runtime terminfo query (`tput`/`setupterm`) is needed: hardcoding the two
  SGR byte shapes matches the existing hardcoded `ENTER_SCREEN`/`HIDE_CURSOR`
  approach (`src/terminal/session.rs:8-12`) and avoids a terminfo dependency.

## 6. Dependencies, licensing, deterministic tests

- **Zero new dependencies required.** Direct deps today: `libc =0.2.189`,
  `signal-hook =0.4.4` (default-features off), locked in `Cargo.lock`
  (verified via `cargo tree`: only `signal-hook-registry`/`errno` transitives).
  Braille needs only `char`/bitmask arithmetic; SGR needs only byte literals.
  Nothing in this ticket justifies `unicode-width`, `termcolor`, `crossterm`,
  or `owo-colors`.
- **GPL-2.0-only impact: none if no dep is added.** Crate license is
  `GPL-2.0-only` (`Cargo.toml:6`); port spec §9/D02 requires any addition to be
  GPL-2.0-compatible with lockfile review. The no-new-dep path sidesteps the
  question entirely — recorded here so the seam ticket does not reopen it.
- **Deterministic tests survive with fixture migration.** The pure
  view→styled-cells function remains fully deterministic (no I/O, clock, or
  locale dependence — same seam as today's `render()`); byte-exact assertions
  stay exact, but the 12 screen fixtures and V01–V08 cases that assert ASCII
  bytes/`bounded()` must migrate to cell-aware assertions. PTY cases
  (T01–T13) are structurally unaffected; T14 (recorded emulator check) must
  re-record Braille/color rendering on one named emulator. Toolchain pinned:
  `rustc 1.98.1`, stable, edition 2024 — no unstable features needed
  (`format!` inline args + byte literals suffice).

## 7. 60×16 small-terminal interaction

- Geometry is unchanged by Braille: at 60×16, graph width = `60 − 28 − 2 = 30`
  cells, `(16−4)/2 = 6` rows per direction (`src/screen.rs:38-41,69`). Braille
  keeps the same cell grid while giving 8 subcell steps per cell instead of the
  current 3-threshold (`0.30/0.70/1.0`) per cell — a pure resolution win *if* the
  cell grid is kept. Packing two data-columns per Braille cell (using the 2-wide
  dot matrix) would double horizontal samples to 60 but halves per-column cell
  independence for color; that trade-off belongs to the prototype/seam tickets.
- The too-small warning path (`width < 60 || height < 16`) is text-only and
  needs no Braille/color handling; its `bounded()` assertions are the only
  touchpoint.

## 8. Handoff: what the seam/prototype tickets still need

1. Styled-cell width-accounting contract (cells vs bytes, zero-width escapes,
   escape-safe trim/split, per-row SGR reset) — seam ticket.
2. Per-cell vs per-column (vs per-dot) color application + palette step count
   along the §2 ramp — prototype + palette decision.
3. NO_COLOR / non-UTF-8 guard policy under replace-outright (human decision;
   technical cost is ~one env check).

## Sources

- Primary: The Unicode Standard — §7 Braille Patterns (U+2800 block,
  bitmask layout), UAX #11 (Neutral width); code chart verified via
  `unicodedata` (category So, names DOTS-*).
- Primary: RFC 3629 §4 (UTF-8 3-byte form for U+0800–U+FFFF).
- Primary: ECMA-48 §8.3.117 (SGR); XTerm Control Sequences (SGR/`38;5`/`48;5`
  extended color); `infocmp xterm-256color` on this machine
  (`colors#0x100`, `setaf`/`setab` with `38;5`/`48;5`).
- Primary: <https://no-color.org/> (NO_COLOR convention).
- Local: `src/screen.rs` (bounded-rows contract, thresholds), `src/terminal/runtime.rs`
  (`queue_frame` validation/emission), `src/terminal/session.rs`
  (`validate_support`, ENTER/HIDE/SHOW/LEAVE literals), `src/terminal/linux.rs`
  (raw `write`, no encoding layer), `Cargo.toml`/`Cargo.lock`/`cargo tree`
  (dep graph), port spec §5–§6/§9, GLOSSARY.md (graph sample/history/ceiling).
- Local probes (this machine, `LANG=en_US.UTF-8`, `TERM=alacritty`,
  `tput colors` → 256): Braille UTF-8 bytes (`e2a080`/`e2a0bf`/`e2a3bf`),
  SGR byte counts (10/4), cube-code ramp arithmetic, `grep` absence of NO_COLOR.
