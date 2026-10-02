# Screen rows interface

Implemented by [Implement the pure ASCII screen and formatting](issues/04-ascii-screen.md).
Acceptance authority: [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), sections 5 and 8.
The crate-private caller contract lives in [Screen](../../src/screen.rs).

## Render selected state

```rust
screen::render(
    view: SelectedView<'_>,
    width: usize,
    height: usize,
) -> Vec<String>

let rows = screen::render(monitor.selected_view(), width, height);
// Terminal/runtime positions and emits an owned frame made from these rows.
```

- Supply the borrowed [selected monitor view](monitor-interfaces.md#selected-view-for-screen)
  and the current terminal dimensions in columns/rows. The monitor guarantees
  positive directional ceilings, finite nonnegative unrounded bytes/s, exact
  optional u64 totals and paired availability invariants.
- The result owns exactly `height` strings, each exactly `width` bytes, including
  trailing blank cells. Every byte is printable ASCII (`0x20`–`0x7E`). There are no
  newlines, controls, terminal escape sequences or implicit frame delimiters.
  Zero width produces empty rows; zero height produces no rows.
- Terminal/runtime owns positioning, clearing, alternate-screen protocols and
  resumable frame output. Position rows explicitly and handle the right margin
  without wrapping/scrolling beyond the supplied screen. Row padding makes the
  desired cell contents explicit; simply joining rows with line feeds is not the
  terminal-output contract. Full output includes the bottom screen row.
- Render takes no mutable state, clock or effect adapter. It neither samples nor
  expires/reset statistics/history. Once rows are returned, their contents do not
  borrow Monitor and can be owned by a pending output frame. Application redraw
  requests and partially emitted-frame replacement rules belong to runtime.

## Presentation owned by Screen

- Full view starts at 60 columns by 16 rows. Header and separator/status occupy
  rows 0–1. Incoming/outgoing labels and stacked graph areas consume the rest;
  outgoing receives the extra row at odd available heights. Statistics reserve
  the rightmost 28 columns, separated by two blank columns, with `Curr`, `Avg`,
  `Min`, `Max`, `Ttl` bottom-aligned and their labels retained.
- The newest fitting history suffix is right-aligned, chronological, with blank
  missing columns. Real zero samples consume columns. Saturation affects bars
  only; numeric rates stay uncapped. Fractional cells use the settled ASCII
  space/dot/vertical-bar/hash thresholds, tested exactly at the boundaries.
- Baseline and unavailable views use their specified second-row text and `--`
  placeholders. Baseline totals remain visible; unavailable totals are absent.
  Measured idle shows numeric zeros and the full-width separator.
- Smaller dimensions show `Terminal too small (need 60x16)` on row 0, the selected
  header on row 1 and status on row 2, clipping each as space permits. Measured
  status is `Measured`; the other states retain their full-size status text.
  Remaining rows are blank. At extreme sizes only existing cells/rows are used.
- Printable name bytes stay literal. Other bytes become uppercase ASCII `\xHH`.
  Shortening never splits an escape chunk, uses `...` when at least three name
  cells fit, and preserves the position indicator when it fits. Original name
  bytes remain the monitor/acquisition identity.
- Values choose their own binary prefix before explicit halfway-up hundredth
  rounding. G is the capped prefix. Rates keep fractional f64 input through
  formatting; totals and ceilings use widened integer arithmetic, including exact
  ceiling-to-bit conversion. A value exceeding its statistics field uses
  `[too wide]`, with the field label intact, instead of numeric clipping.

The [screen verification record](verification.md#screen-run--2026-10-02) maps
passing exact fixtures to V01–V08 and S12. Application/terminal integration,
Linux PTY and recorded visible-emulator evidence remain with their owning tickets.
