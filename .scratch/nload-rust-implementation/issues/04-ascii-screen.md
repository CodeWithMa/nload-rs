# Implement the pure ASCII screen and formatting

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: ASCII screen and formatting)
Blocked by: 03

## Question

Can the Screen module turn the selected monitor view and supplied dimensions into bounded ASCII rows satisfying every geometry, status, graph, escaping and numeric-formatting contract in specification section 5?

Implement the settled stacked layout, independent ceiling labels, newest-right fitting history, ASCII fractional-cell thresholds, safe small-terminal behavior, byte escaping/ellipsis and value overflow presentation. Implement binary prefix selection before explicit halfway-up rounding, with widened exact-total arithmetic. Rendering must not mutate/expire monitor state or emit terminal escape sequences.

Cover V01–V08 through exact row/format/glyph checks, including 80x24, minimum/odd/tiny dimensions, rate-boundary fixtures, maximum-u64 totals, baseline/absence/read-failure/idle states and shrink/grow history selection. Update verification mappings and record the stable rows interface for terminal output. Visible emulator behavior remains pending until its dedicated HITL ticket.

## Comments

### Resolution — 2026-10-02 — alice

[Screen](../../../src/screen.rs) now implements the pure selected-view renderer
required by specification section 5. Its stable entry point is
`screen::render(view, width, height) -> Vec<String>`: owned, printable ASCII rows,
exactly the supplied dimensions, with no terminal protocols or monitor mutation.
The full contract and terminal-output connection are in
[Screen rows interface](../screen-interfaces.md). Actual revision/environment,
command outcomes, case-to-test mappings and pending effect/visual evidence are in
the [screen verification record](../verification.md#screen-run--2026-10-02).

- Implements the stacked incoming/outgoing layout, odd-height split, independent
  ceiling labels, bottom-aligned 28-column statistics and two-column gap. Full
  80x24 rows, minimum/odd layouts, state text and placeholders are checked exactly.
- Uses right-aligned newest fitting history, preserves real zero columns, renders
  bottom-up ASCII fractional bars and saturates only graphs. Exact threshold
  fixtures found and corrected a floating-point scaling artifact at 1.70 cells;
  no glyph tolerance was added. Shrink/grow tests include a distinctive history
  tail and verify identical recovered rows and unchanged actual Monitor views.
- Below 60x16, renders bounded warning/selection/status rows, including zero and
  extremely tiny dimensions. Pure sampling/navigation while small restores actual
  off-screen history on growth. Name escaping uses uppercase `\xHH`, shortening
  preserves complete escape chunks and the position when it fits, and numeric
  overflow uses `[too wide]` while retaining statistic labels.
- Selects each binary prefix before explicit halfway-up rounding. Rates retain
  fractional f64 input; totals and ceiling-to-bit conversion use u128 arithmetic.
  Checks cover all prefix boundaries, halfway examples, independent units/ceilings,
  largest valid ceiling, maximum-u64 totals and adjacent large exact totals.
- **12 new screen tests pass**; all **40 tests** pass under `cargo test --locked`.
  `cargo fmt --check` and `cargo build --release --locked` pass. The release build
  reports 61 dead-code warnings while modules await application wiring.

No specification conflict, domain change or newly specifiable corrective ticket
surfaced. Terminal positioning/right-margin/output, application effects and quit
continuity, Linux PTY, recorded emulator and live evidence remain with existing
owning tickets. No Git commit was made. This closes only this implementation
ticket and supplies the owned rows contract to
[Implement incremental input and nonblocking runtime effects](06-terminal-events.md)
and [Integrate startup, sampling and the interactive application loop](07-application-integration.md).
