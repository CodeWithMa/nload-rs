# Decide the styled graph screen contract

Status: closed
Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Parent: [Find the way to a btop-style graph spec for nload-rs](../map.md)
Assignee: alice (OpenCode session: styled graph contract)
Blocked by: 01, 02, 03

## Question

What is the implementation-ready contract for styled graphs that replaces ASCII bars outright with no fallback? Decide with the human, using the btop facts, terminal feasibility, and accepted prototype direction (C — 2 samples per cell, per-direction hues, true-blank zeros): screen-owns-escapes seam (new styled-cell/rows interface replacing pure ASCII rows), exact 2-sample-per-cell 2x4 Braille fill mapping replacing 0.30/0.70/1.0 thresholds, per-direction positional color application and saturation rule, width accounting with multi-byte + zero-width escapes, 60x16 small-terminal and baseline/absent/read-failed/idle presentation with true-blank zeros, statistics-text interaction, and migration of the 12 screen fixtures plus V01–V08 acceptance cases. Record sharpened glossary terms (graph glyph, Braille cell, color grade) without diluting GLOSSARY.md.

Use [Establish what btop actually renders](01-btop-rendering-facts.md), [Establish Braille and color terminal feasibility](02-terminal-color-feasibility.md), and [Prototype the Braille and color graph look](03-graph-prototype.md). Consult `grilling`, `domain-modeling`, and `codebase-design` for the seam. The resolution closes the map only if no fog remains; otherwise graduate newly specifiable tickets and clear them from Not yet specified.

## Comments

### Resume and factual inspection — 2026-10-07 — alice (OpenCode session: styled graph contract)

Claimed the next frontier ticket; no contract decisions resolved yet. Consulted `grilling`, `domain-modeling`, and `codebase-design`; read-only code inspection supplied the following facts for the live decision rounds:

- Existing graph grids are 30×6 at 60×16 and 50×10 at 80×24. At accepted density these display 60 and 100 samples respectively. History is already time-retained for 300 seconds, with one sample per accepted measured observation; doubling visible samples needs no sampling/storage change.
- Correction to the prototype resolution and rendering research's wording: with unchanged 500-ms cadence, two samples per terminal cell mean approximately one second per cell, twice the elapsed history per cell, not half. No cadence change was agreed.
- Odd-count pairing was not decided by the visual acceptance. The prototype duplicates an unmatched final sample; the contract must choose deliberately rather than treating this incidental mockup behavior as authority.
- `screen::render` currently returns ASCII-only `Vec<String>` with byte-based width. `terminal::Runtime::queue_frame` separately rejects Unicode and ESC. Both need a defined styled-frame interface and cell-aware validation; the PTY decoder also assumes ASCII.
- Session cleanup currently has no SGR reset. Partial writes resume by byte offset. Styling must define reset behavior for both complete rows and interrupted output.
- Existing terminal checks reject missing/empty TERM and exact `dumb`/`linux`; no production locale, NO_COLOR, or 256-color probe exists.

Pending human decisions: half-cell anchoring/odd counts, dot quantization, structured styled-frame seam and SGR ownership, exact palettes/text contrast, environment policy, and deterministic test/acceptance supersession. Ticket remains open during the live exchange.

### Confirmed decision round Q1–Q5 — 2026-10-07 — alice

Human response: “Q1–Q5 as recommended”. These are confirmed partial decisions, not the final ticket resolution:

1. **Horizontal placement:** select the newest fitting suffix of at most twice the graph width in actual samples, right-align at half-cell resolution, and place the newest sample in the right half of the rightmost cell. Older is left, newer is right. An odd-length suffix has a blank left half beside the unmatched oldest sample. Never duplicate samples. New observations shift presentation left by one half-cell; rendering/resize do not mutate history or add samples.
2. **Vertical quantization:** for graph height H, finite nonnegative rate r, and positive ceiling c, clamp r/c to [0,1], multiply by 4H, and round to nearest whole dot count with ties upward. Fill bottom-up independently for each sample. Zero stays blank; sufficiently small positive rates also round to blank. At/above ceiling all 4H dots of that sample fill, independently of its neighbor; numeric rates remain uncapped. This replaces ASCII 0.30/0.70/1.0 thresholds and does not copy btop's biased rounding.
3. **Seam:** an opaque styled frame replaces ASCII rows. Screen builds bounded cells and serializes Braille plus foreground-color escapes; terminal handles positioning and writes. Width means terminal cells, not UTF-8 bytes. Callers neither clip encoded strings nor parse arbitrary ANSI. Reset before statistics text, at each row end, and in terminal cleanup. Exact construction/encoding/reset guarantees remain for the next round.
4. **Environment:** retain existing interactive checks for terminal stdin/stdout and TERM (reject missing/empty, `dumb`, `linux`). Accepted sessions assume UTF-8, Braille font coverage, and xterm-256color. Always emit colored Braille; no locale/capability probes, NO_COLOR handling, or fallback. Informational invocations retain their exemption.
5. **Visual contrast:** incoming indigo/lavender, outgoing magenta; use legible medium-dark bottom shades rising to pale top shades, rather than btop's darkest endpoints. Color only graph dots, one shade per row. Statistics, labels, status, and small-terminal warning use terminal-default foreground/background. Exact palette codes/row selection remain for the next round.

Sharpened [glossary](../../../GLOSSARY.md) terms: graph sample no longer equates to a terminal column; graph glyph combines adjacent samples within a graph row; color grade is selected by graph-row position. “Braille cell” remains a presentation-format term in this contract (a two-column/four-row dot glyph), not a general Unicode implementation entry in the domain glossary.

Next frontier: exact palette/row grades; opaque frame invariants and Unicode dot mapping; acceptance/test migration; handoff and map-completion confirmation. Ticket remains open.

### Resolution — 2026-10-07 — alice (OpenCode session: styled graph contract)

Resolved with the human. The human accepted “Q1–Q5 as recommended” and then “Q6–Q9 as recommended”, including confirmation that this completes the shared understanding and the route. The following is the canonical implementation-ready styled graph contract; earlier comments record the conversation's progression, not outstanding work.

#### Authority and scope

This resolution supersedes the affected ASCII graph/screen/terminal promises in the [Minimal Rust port spec](../../nload-rust-port/spec.md), as indexed in the [btop-style graph spec handoff](../spec.md). The port spec supplies the rest of the application contract. [Prototype the Braille and color graph look](03-graph-prototype.md) and [Establish what btop actually renders](01-btop-rendering-facts.md) supply context, not alternative algorithms.

Replace the ASCII graph outright with two-samples-per-cell, bottom-filled Braille and per-direction positional color. No ASCII mode or fallback. This effort delivers the planning contract, not an implementation. No application-code, dependency, cadence, monitor-retention, rate-calculation, CLI, or configuration change is part of this resolution.

#### Geometry and presentation states

Use the port spec's existing layout and text rules:

- Full view begins at width >= 60 and height >= 16. Reserve 28 terminal columns for statistics and a two-column gap. Graph width W = terminal width − 30.
- Incoming graph height is floor((terminal height − 4)/2); outgoing gets the remainder. Keep the stacked labels, selected-interface header, status/separator row, and bottom-aligned Curr/Avg/Min/Max/Ttl blocks. No new footer, axis, ticks, or time labels.
- At 60×16, each graph is 30×6 and fits 60 samples. At 80×24, each is 50×10 and fits 100 samples. Odd available height still gives outgoing the extra row.
- Baseline: empty graph history, `Waiting for rate sample`, unavailable rates/extrema, available baseline totals.
- Absent: empty graphs, unavailable rates/extrema/totals, `Unavailable: interface absent`.
- Read failed: empty graphs, unavailable rates/extrema/totals, `Unavailable: counter read failed`.
- Measured idle: genuine numeric zero rates, actual zero-height samples, latest totals, normal `=` separator. Existing nonzero history can remain visible as newer zero samples enter; idle does not erase history.
- Recovery remains baseline first. Below either size threshold, show only the existing warning/selection/status as fits. Sampling, navigation, quit, and retained-history recovery continue. Zero dimensions remain valid.
- Statistics, labels, header, separator/status, and warning text use terminal-default foreground/background. Only graph glyphs receive the direction's foreground grade; no background styling.
- Preserve original-byte interface identity and printable-ASCII `\xHH` display escaping, safe ellipsis, position reservation, units/rounding, and `[too wide]` numeric overflow behavior.

#### Horizontal sample placement

Take the newest min(retained sample count, 2W) samples and place them chronologically into the rightmost half-cell positions of a 2W-position graph. Older adjacent sample is left, newer is right. The newest sample always occupies the right half of the rightmost terminal cell.

Unfilled positions are blank presentation space, not fabricated zero measurements. For an odd-length suffix, its unmatched oldest sample occupies the right half of its cell with a blank left half. Thus a three-sample suffix `[a,b,c]` ends in `(blank,a) (b,c)`. A one-sample suffix is `(blank,a)`. Never duplicate an unmatched sample.

Every new accepted observation shifts the visible history left by one half-cell. Pairing is recomputed from the newest fitting suffix rather than a persistent frame phase. Shrink/grow, pruning of older history, repeated redraws, navigation, and small-terminal rendering must not manufacture, duplicate, or mutate samples. A measured zero consumes a real half-cell position despite having no dots.

The monitor still adds one current-rate sample per accepted measured observation, retains the existing five-minute history, and alone owns observation-driven expiry/resets. No width-based history cap or cadence change. Two samples per terminal cell double the displayed observation history for the same width. At regular 500-ms cadence this is nominally one second per terminal cell; delayed observations retain observation-based spacing, not an exact time axis. This corrects the earlier prototype/research “time-per-column halves” wording.

#### Vertical quantization and Braille mapping

For each sample's finite nonnegative unrounded rate r, its positive direction-specific ceiling c, and graph height H:

1. Normalize f = clamp(r/c, 0, 1).
2. Dot count D = floor(f × 4H + 0.5), clamped to [0, 4H]. This is nearest-dot rounding with halfway values upward; no btop percentage-band rounding or extra rounding bias.
3. For row k counted from the bottom (0 through H−1), fill clamp(D − 4k, 0, 4) dots in that sample's half-cell, bottom-up.

Zero is truly blank. Positive rates below half of the first dot step also round to blank; there is no forced minimum dot. Each sample at/above ceiling fills its own full height independently of its neighbor. Numeric rates remain actual uncapped rates, and graph ceilings do not change statistics.

For example, H = 6 gives 24 dot steps: zero has no dots, r = c/48 is the first halfway boundary and fills one bottom dot, r = c/2 fills the bottom three rows, and r >= c fills all six rows of that half-cell. Exact threshold decisions use the unrounded rate calculation; do not apply a fuzzy tolerance to glyph selection.

A Braille cell is a two-column/four-row dot glyph. Standard Unicode dot numbering is left top-to-bottom 1,2,3,7 and right top-to-bottom 4,5,6,8. Set bit (dot number − 1) for each filled dot and encode U+2800 + mask. Use ordinary ASCII space for a zero mask, not U+2800. For independently saturated halves, left-only is `⡇`, right-only is `⢸`, and both are `⣿`.

Implement this mapping independently from the standard dot arrangement; do not copy btop's lookup-table implementation. Every allowed Braille glyph occupies one terminal cell under the supported-terminal assumptions.

#### Exact per-direction palettes

Hardcode the following xterm-256color foreground grades, bottom to top. RGB values describe the standard xterm cube; the emitted representation is the numeric index, not truecolor or dynamic interpolation.

| Grade | Incoming index / reference RGB | Outgoing index / reference RGB |
| --- | --- | --- |
| 0 | 60 / `#5f5f87` | 96 / `#875f87` |
| 1 | 61 / `#5f5faf` | 132 / `#af5f87` |
| 2 | 98 / `#875fd7` | 133 / `#af5faf` |
| 3 | 140 / `#af87d7` | 170 / `#d75fd7` |
| 4 | 146 / `#afafd7` | 176 / `#d787d7` |
| 5 | 183 / `#d7afff` | 219 / `#ffafff` |

For row k from the bottom, grade = round_half_up(5k/(H−1)). Full-view graphs always have H >= 6, so this denominator is nonzero. Evaluate each direction against its own height, including odd-height layouts.

Six-row graphs use grades 0,1,2,3,4,5; ten-row graphs use 0,1,1,2,2,3,3,4,4,5 (bottom to top). Color is positional: all nonblank graph glyphs on a row share its grade, regardless of the rates or the halves' individual fill levels. A lower-rate sample simply reaches fewer rows; color is not a separate per-value scale.

Incoming is indigo/lavender, outgoing magenta, with medium-dark bottoms and pale tops. No terminal-default dark-background override, palette configuration, or exact btop-theme reproduction. Target-emulator acceptance verifies the actual appearance and statistics contrast.

#### Deep screen/terminal seam and encoding guarantees

Replace ASCII-row output with an opaque styled frame owning dimensions and private rows. The pure screen module consumes the selected read-only monitor view and dimensions, builds bounded cells, and encodes glyphs/foreground styles. Construction stays inside the screen module; callers cannot supply arbitrary encoded strings or inconsistent dimensions. A narrow read-only frame interface exposes dimensions and encoded rows to the terminal and tests. A generic public styled-cell builder or ANSI parser is unnecessary.

Frame guarantees:

- Exactly the supplied logical row count, each with exactly the supplied terminal-cell width. Zero-width/zero-height frames emit no display characters outside those dimensions.
- Display characters are printable ASCII or nonblank Unicode Braille. Original names remain escaped ASCII. No embedded LF, CR, arbitrary Unicode, raw control characters, or user-supplied ESC.
- Screen-generated styling contains only `ESC[38;5;Nm` for the agreed direction palette and `ESC[0m` resets. SGR is zero-width. Screen emits no positioning/clearing/session commands.
- Empty masks and missing positions are spaces. Plain text and blank cells have default styling; only nonblank graph glyphs need foreground styling. Consecutive equally styled glyphs may be encoded as runs.
- Clip text and place graph glyphs while working with cells, never by slicing UTF-8/escape bytes. Trim trailing blank cells before encoding; retain the logical dimensions even when encoded rows omit trailing blanks, which the terminal's frame clear supplies.
- Reset at each emitted row's start/end and before the statistics area. Resets are protocol hygiene, not warning/status coloration; those paths have no palette foreground codes. Do not trim serialized rows or accidentally remove their final reset.

Terminal/runtime owns frame clearing, absolute row positioning, final cursor placement, and resumable nonblocking writes. It consumes only screen-constructed frames, not arbitrary ANSI strings, and uses frame-owned dimensions instead of separately supplied dimensions that can disagree. Frame clearing must happen in default styling; foreground selection remains screen-owned.

Keep the existing no-LF/CR output protocol and right-margin bounds. A pending frame can be replaced only before any byte is emitted; once started, finish it before the latest requested view. Resume from the exact byte offset even when a write splits a UTF-8 glyph or SGR sequence. Do not drop the suffix, interleave frames, or delay input/signals/sampling while waiting for output.

Session cleanup cancels any incomplete escape sequence and attempts `ESC[0m` before cursor-show/alternate-screen-leave. Include reset in staged explicit cleanup and nonpanicking fallback; attempt all applicable restoration steps even after another fails. Existing status arbitration remains authoritative. Interrupted/unwritable output is best-effort cleanup, not a claim that reset always reaches the emulator.

This moves screen formatting, glyph selection, cell bounds, and encoding into one deep module; terminal callers do not need Braille masks, palette logic, display-width scans, or escape-safe clipping. Use the frame interface as the test surface rather than opening private monitor queues or adding hypothetical backend seams.

#### Terminal declaration policy

Retain terminal stdin/stdout requirements, redirected-stderr support, and rejection before mutation of missing/empty TERM and exact `dumb`/`linux`. Help/version remain exempt. Other TERM values are declarations of assumed support, not capability verification.

Document UTF-8 output, Braille font coverage, and xterm-256color as interactive requirements. Always emit colored Braille for accepted full-view sessions. No locale checks, color-capability probe, NO_COLOR handling, ASCII mode, or fallback. Small-terminal mode is the existing text-only presentation, not a compatibility fallback.

#### Acceptance and test migration

Migrate all twelve current screen test functions by coverage, not merely replacing text snapshots or insisting on twelve replacement functions. Screen and terminal tests cross the new frame interface; ASCII byte-length helpers and byte-offset graph assertions must become terminal-cell/structured-frame assertions.

| Port case | Required styled-graph coverage |
| --- | --- |
| V01 — Geometry | Exact logical cell bounds/layout at 80×24, 60×16, odd heights/widths; statistics gaps/alignment; exact direction palettes and row grades. |
| V02 — Bar boundaries | Replace ASCII thresholds with nearest-dot boundaries just below/at/above half steps, standard Braille masks, true zero, very small positive values, independent half-cell/direction saturation, uncapped numeric rates. |
| V03 — History placement | Newest 2W-sample suffix, chronological half-cell order, one/odd/even counts, real zero positions, no sample duplication, delayed samples, shrink/grow/redraw without history mutation. |
| V04 — States | Existing baseline/absent/read-failed/idle/recovery status and statistics distinctions, blank unavailable histories, and existing nonzero history scrolling into measured idle. |
| V05 — Small terminals | Existing 59×16, 60×15, 60×16 and extreme dimensions; warning with no palette codes; continued acquisition/navigation/quit; growth restores retained half-cell history. |
| V06 — Names/values | Original-byte identity and escaped control/non-ASCII names; ellipsis/position/overflow guarantees; no raw escape injection; default-style text despite adjacent colored glyphs. |
| V07 — Units/rounding | Preserve all existing rate/total unit and exact halfway-up text cases, independent of graph quantization and color. |
| V08 — Large totals/ceilings | Exact maximum-u64/large-ceiling behavior, independent ceilings and labels, no accounting/conversion overflow, independently saturated graph halves. |

Other affected acceptance:

- S03/S07/S09/S11/S12 retain their sampling/history semantics; “column” means an actual sample position, not necessarily a terminal cell. Check doubled visible capacity without changing retained data or observation counts.
- T01 documents the UTF-8/Braille/xterm-256color assumption while preserving rejection/exemption checks and declaration-based support. Verify always-colored output under NO_COLOR and locale declarations without adding guards.
- T08 expands partial-output fixtures to split inside multibyte glyphs and every relevant SGR fragment, with exact byte resumption, untouched-frame replacement, and fairness under backpressure.
- T09–T13 include reset/cancel behavior in normal, signal, initialization/runtime/panic and cleanup-failure paths while preserving existing staged restoration/status rules.
- Update the real Linux PTY decoder to consume UTF-8 Braille and the permitted SGR subset alongside existing frame/session commands. PTYs verify protocol and OS behavior, not visible palette/font quality.
- T14/emulator evidence records emulator identity, font/UTF-8/256-color setup, both palettes at 60×16 and 80×24, statistics contrast, true-blank zero and empty states, selection/resize recovery, and default styling/cursor/prior-screen restoration after handled exits. Existing practical scripted evidence remains available for states a live PTY cannot reliably induce.
- Retain nonprivileged live Linux smoke, independent ceilings, and build/artifact review. Update module/README support language and case-to-test mappings so no affected ASCII promise remains silently authoritative.

Required implementation gates remain `cargo fmt --check`, `cargo test --locked`, and `cargo build --release --locked`, plus applicable deterministic, Linux PTY, recorded emulator, live-smoke, and artifact evidence from the port spec. Glyph/grade/text decisions are exact comparisons; numerical-rate tolerances do not excuse glyph differences. Unrun evidence remains pending.

#### Dependencies, licensing, and cost

Linux-only, stable Rust edition 2024, GPL-2.0-only application and compatible locked dependencies remain required. Add no dependency for Braille, display-cell accounting, SGR, or palette selection. Write the algorithm independently; use standard Unicode/ANSI facts and the pinned btop research as inspiration, not copied upstream implementation code.

Render only the fitting history suffix and construct bounded logical cells; rendering/encoding cost scales with screen area and visible samples, not copying every interface's five-minute history. Fixed palette lookup and bounded per-cell dot work need no external width library, general ANSI parser, or new timing benchmark gate. Partial-output batching and loop responsiveness keep their existing acceptance obligations.

#### Handoff and map completion

The human confirmed this completes the route. All previous fog is covered above, no additional decision tickets are needed, and implementation belongs to a later effort. The [handoff spec index](../spec.md) links this contract and the inherited port authority without duplicating decision detail. The parent map is closed with a named context pointer to this resolution. No implementation verification is claimed by these planning artifacts.
