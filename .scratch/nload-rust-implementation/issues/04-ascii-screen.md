# Implement the pure ASCII screen and formatting

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 03

## Question

Can the Screen module turn the selected monitor view and supplied dimensions into bounded ASCII rows satisfying every geometry, status, graph, escaping and numeric-formatting contract in specification section 5?

Implement the settled stacked layout, independent ceiling labels, newest-right fitting history, ASCII fractional-cell thresholds, safe small-terminal behavior, byte escaping/ellipsis and value overflow presentation. Implement binary prefix selection before explicit halfway-up rounding, with widened exact-total arithmetic. Rendering must not mutate/expire monitor state or emit terminal escape sequences.

Cover V01–V08 through exact row/format/glyph checks, including 80x24, minimum/odd/tiny dimensions, rate-boundary fixtures, maximum-u64 totals, baseline/absence/read-failure/idle states and shrink/grow history selection. Update verification mappings and record the stable rows interface for terminal output. Visible emulator behavior remains pending until its dedicated HITL ticket.
