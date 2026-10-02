# Validate the single-interface screen and terminal behavior

Status: closed
Labels: wayfinder:prototype
Type: prototype
Mode: HITL
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: single-interface screen)
Blocked by: 03, 04

## Question

What should the default single-interface screen look like and how should it adapt to terminal constraints? Use a cheap throwaway mockup with representative statistics to get the human's reaction to the release-like layout, incoming/outgoing graphs, text fields, navigation feedback, resizing, and small-terminal behavior. Include absent-interface/read-error presentation as required by prior decisions. Decide the accepted screen contract and terminal cleanup expectations; link the prototype rather than building the application.

Use the confirmed [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice) for navigation feedback, retained unavailable entries, and established terminal/exit requirements. Decide presentation and remaining terminal-lifecycle details within that contract.

Use [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice) for statistics availability, graph samples/history/scaling and resize semantics. Choose the exact presentation of baseline versus unavailable states, graph glyphs and whether/how to label independently configurable ceilings, without inventing observations on redraw.

## Comments

### Prototype context — 2026-10-01 — alice

[Throwaway single-interface screen prototype](../screen-prototype.html) presents three synthetic terminal layouts: release-like stacked graphs, side-by-side directions, and statistics-first. Open the HTML file directly in a browser; `?variant=A`, `?variant=B`, and `?variant=C` select layouts. Controls exercise representative measured/idle/baseline/absent/read-error/saturation snapshots, interface selection, retained history on resize, graph glyphs and independent ceilings. Proposed presentation and size thresholds are not accepted decisions. Browser controls and debug state are outside the proposed terminal UI; cleanup is not simulated.

The asset is an uncommitted local scratch file. The repo has no commits; no prototype capture commit or production implementation is part of this planning session. Source-backed layout evidence comes from [release findings](../research/release-defaults.md#5-graphs-scale-glyphs-history-and-screen-geometry); the prototype deliberately makes alternatives available for human reaction rather than treating release quirks as requirements.

### Discussion — 2026-10-01 — first round

The human chose layout A (release-like stacked graphs with statistics alongside) but explicitly removed the proposed footer. The human accepted the other recommendations: an interface-name/position header without IPv4 discovery; ASCII `.`, `|`, `#` graph bars and visible independently formatted ceiling labels; distinct baseline/absent/read-error states, `--` placeholders rather than measured zeros, no stale totals while unavailable, and interface-local failure presentation without repeated stderr output; a warning below 60 columns or 16 rows while retaining sampling/navigation/quit and restoring retained history on growth, including startup at a small size; and an alternate-screen, hidden-cursor UI restoring the prior screen, cursor visibility and input mode on agreed exits and initialization/runtime failures, with best-effort cleanup before runtime diagnostics/status 1 and caught-panic reporting.

This is an interim discussion record, not a resolution. Exact geometry, text overflow/state placement, bar thresholds and remaining lifecycle details are being checked before complete-contract confirmation. The prototype was updated to remove the footer from the chosen layout and the small-terminal warning.

### Discussion — 2026-10-01 — second round

The human accepted all remaining recommendations: a header and separator/status line followed by incoming/outgoing sections, evenly split graph heights with the extra row assigned to outgoing, 28 statistics columns and a two-column gap with bottom-aligned fields, and unrounded ASCII partial-cell thresholds at 30%/70%/100%; baseline/unavailable messages replacing the separator, preserving the position indicator while shortening displayed names with `...`, escaping non-ASCII/control name bytes as `\xHH` without changing selection identity, and `[too wide]` rather than misleading numeric truncation; and caught-panic status 1 after best-effort restoration, ignoring Ctrl+Z with no suspend/resume feature. Cleanup guarantees apply to exits the application can handle.

All discussion branches are settled. The complete-contract summary awaits final human confirmation before resolution.

### Resolution — 2026-10-01 — alice

The human explicitly confirmed the complete screen and terminal contract after accepting both discussion rounds, choosing release-like layout A without the proposed footer. This resolution is the canonical decision; earlier comments preserve discussion history. The previously confirmed startup/navigation and statistics/sampling contracts remain in force.

#### Layout and navigation feedback

- Use the single-interface view with incoming above outgoing and a separate statistics block beside each graph. Each block lists `Curr`, `Avg`, `Min`, `Max`, `Ttl` in that order, using the agreed rate/total semantics, units and formatting.
- The first row is an interface-name and one-based position header, for example `Interface eth0 (1/3)`. Do not show an IPv4 field or introduce address discovery. There is no footer or persistent navigation-help line, including in the small-terminal view.
- The second row is a full-width `=` separator in the normal measured state. Replace it with the applicable baseline/unavailable status message below; keep the interface header visible.
- Follow with an incoming label, incoming graph/statistics area, outgoing label, and outgoing graph/statistics area. Each label includes its independently configured ceiling, using the agreed automatic binary-scaled rate formatting, for example `Incoming: (ceiling 10.00 MBit/s)`.
- Reserve 28 columns on the right for statistics and a two-column gap to their left. The graph takes the remaining `width - 30` columns. Bottom-align the five statistics rows in each graph area.
- After the header, separator/status row and two direction-label rows, divide the remaining `height - 4` rows evenly between graph areas. Incoming gets `floor((height - 4) / 2)` rows and outgoing gets the remainder. At 80 columns by 24 rows, each graph is 50 columns by 10 rows.
- Preserve prompt navigation redraw, the fixed name-based startup list and symmetric arrow-key wrapping from the startup/navigation contract. Header position changes with selection; navigation does not reset statistics or create observations.

#### Graph rendering

- Use release-like ASCII vertical bars, not Unicode blocks or connected lines. For a rate `r`, ceiling `c` and graph height `H`, the filled height is `H * min(r / c, 1)`, computed from the unrounded rate. Render from the bottom upwards: a cell with fill below 0.30 is blank, fill from 0.30 to below 0.70 is `.`, fill from 0.70 to below 1 is `|`, and fill at least 1 is `#`.
- Saturate rates at or above the ceiling to a full-height graph. Numeric rates remain uncapped; independently configured ceilings do not change the statistics.
- Use the statistics contract's actual retained graph samples: oldest fitting sample on the left, newest on the right, with unfilled space blank. A measured-zero sample occupies a history column but has no filled cells; it is not a fabricated sample. Baselines, redraws and ignored observations add no columns.
- Do not add scale axes, ticks, time labels or a footer. The direction labels expose the graph ceilings.

#### Baseline, unavailable and idle presentation

| State | Rates/extrema | Totals | Graphs | Second-row status |
| --- | --- | --- | --- | --- |
| Baseline | `--` | Available from the baseline observation | Empty | `Waiting for rate sample` |
| Interface absent | `--` | `--` | Empty | `Unavailable: interface absent` |
| Counter acquisition failed | `--` | `--` | Empty | `Unavailable: counter read failed` |
| Measured idle | Genuine measured zero values | Available | Actual zero-height sample columns | Normal `=` separator |

- A failure affecting either direction makes both directions unavailable, following the statistics contract. Do not show stale totals, synthetic zero rates or old graph history during unavailability.
- Keep failures within the selected interface's view, retaining selection and allowing navigation away. Do not repeatedly write acquisition-failure diagnostics to stderr while the UI is active. Continue attempting acquisition; a successful return establishes the baseline state before measured rates resume.

#### Terminal sizes, resize and text fit

- A full view requires at least 60 columns and 16 rows. Below either threshold, replace the normal layout with a terminal-too-small warning, current selection and status information as space permits. At extremely small sizes, show only what fits without wrapping or writing outside the screen.
- Starting below the size floor is not an initialization failure: show the same warning view. Continue sampling, arrow navigation and quit handling while small. Growing restores the normal layout and whatever actual history remains retained.
- Resize redraws do not create observations, reset statistics or change the five-minute retention policy. Shrinking hides older fitting history; growing reveals retained history. These are rendering changes, not sampling events.
- Preserve the interface position indicator when shortening an overlong displayed name, using `...` for truncation. Escape non-ASCII and control bytes in names as ASCII `\xHH`; display escaping does not alter the original name used for discovery, ordering, selection or acquisition.
- Keep statistics field labels visible. If a normally formatted numeric value does not fit its field, show `[too wide]` instead of truncating it into a misleading number. This exception affects presentation only; retain the original calculation and agreed normal formatting rules.

#### Terminal lifecycle and failures

- Use the alternate screen and hide the cursor for the interactive UI. Restore the prior screen, prior cursor visibility and input mode on handled exits and initialization/runtime failures, including state changed before initialization completes.
- Preserve `q`/`Q` status 0, Ctrl+C/SIGINT status 130, and SIGTERM status 143 from the startup/navigation decision.
- Runtime terminal/input/output failures trigger best-effort restoration, then a stderr diagnostic and status 1. Initialization failures retain the previously confirmed restoration/diagnostic/status-1 contract. Counter acquisition failures are interface-local unavailable states, not these fatal UI failures.
- Caught panics also attempt restoration before reporting failure to stderr and exiting 1. Cleanup guarantees apply to exits the application can handle.
- Ignore Ctrl+Z consistently with the other-key policy; do not add a suspend/resume feature. Mechanisms for input modes, signal handling and cleanup belong to the architecture decision.

#### Prototype and deliberate release departures

- [Throwaway single-interface screen prototype](../screen-prototype.html) is the supporting synthetic mockup. The revised A view demonstrates no footer, accepted graph geometry and separator-replacing status messages. B/C and the Unicode option preserve explored alternatives, not accepted requirements. The resolution above governs details the cheap browser mockup does not simulate, including name/numeric overflow and real terminal cleanup.
- The prototype remains an uncommitted scratch asset in this fresh repository, consistent with the effort's no-commit capture constraint. No application implementation was produced.
- Retain release-like stacked directions, field order and ASCII partial bars, but use defined graph geometry, visible ceiling labels, explicit baseline/unavailable states, small-terminal warnings, safe text limits and explicit alternate-screen/cursor restoration. Replace the upstream `Device`/optional-IP header with the glossary-consistent interface header. There is no footer, as in the release-like layout.

#### Remaining decisions and map review

- [Decide the port's architecture and test seams](07-module-boundaries.md) owns implementation mechanisms for this layout, safe rendering, terminal events, ignored Ctrl+Z, cleanup and panic handling within the engineering constraints. It also owns integration with independent sampling and retained histories.
- [Decide acceptance criteria and the specification handoff](08-acceptance-handoff.md) owns verifiable cases for normal geometry, glyph boundaries/saturation, state transitions, small-startup/shrink/grow, text overflow, and terminal restoration/statuses.
- No new question or dependency is required: these precise remaining decisions are already covered by live tickets. The screen-related fog is cleared; possible interactions exposed by architecture remain in the map. This session resolves only this ticket.

### Subsequent terminal-contract amendment — 2026-10-01

During [Decide the port's architecture and test seams](07-module-boundaries.md#discussion--2026-10-01--first-round), the human accepted ordinary xterm-compatible emulator sessions as the terminal target and showing the cursor on cleanup, rather than requiring restoration of an unknowable prior cursor visibility. That architecture ticket owns the amendment and its eventual complete mechanism/support contract. Alternate-screen and input-mode restoration remain required. This pointer prevents the earlier visibility promise being treated as an unconditional implementation requirement.

The complete amendment and mechanisms are now confirmed in [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice), including TERM checks, showing the cursor on cleanup and cleanup-error exit arbitration. Consult that resolution for those refinements rather than treating the original terminal promise as universal.
