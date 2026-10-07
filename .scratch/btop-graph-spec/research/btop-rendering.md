# btop rendering facts (pinned: v1.4.7)

Pinned reference: **btop v1.4.7**, tag `v1.4.7`, commit
`6e39144aaf5a6bc01b9f795010b0914431067183`, released 2026-05-01.
Release page notes 107 commits to main since release at time of reading.

Primary sources (all pinned to `v1.4.7` unless noted):

- Release: <https://github.com/aristocratos/btop/releases/tag/v1.4.7>
- Commit: <https://github.com/aristocratos/btop/commit/6e39144aaf5a6bc01b9f795010b0914431067183>
- Graph + symbols: <https://raw.githubusercontent.com/aristocratos/btop/v1.4.7/src/btop_draw.cpp>
  (`Symbols::graph_symbols`, `Draw::Graph::_create`, `Draw::Graph` ctor, `Net::draw`)
- Gradients: <https://raw.githubusercontent.com/aristocratos/btop/v1.4.7/src/btop_theme.cpp>
  (`Default_theme`, `TTY_theme`, `generateGradients()`, `generateTTYColors()`)
- Config defaults: <https://raw.githubusercontent.com/aristocratos/btop/v1.4.7/src/btop_config.cpp>
  (`graph_symbol*`, `net_auto`, `net_sync`, `tty_mode`, `lowcolor`)
- Net collection / autoscale: <https://raw.githubusercontent.com/aristocratos/btop/v1.4.7/src/linux/btop_collect.cpp>
  (`Net::collect`, `graph_max`, `max_count`, `bandwidth` history cap)
- Terminal requirements: <https://github.com/aristocratos/btop> README
  "Prerequisites" (truecolor, UTF-8 locale, Braille Patterns U+2800–U+28FF font).

Scope: what btop actually renders. No nload-rs mapping is chosen here.

## 1. Braille character sets

`Symbols::graph_symbols` in `btop_draw.cpp` defines six 25-entry tables
indexed `left * 5 + right` (see §2). The braille tables are:

- `braille_up` (bottom-anchored fill):
  `[" ", "⢀", "⢠", "⢰", "⢸", "⡀", "⣀", "⣠", "⣰", "⣸",`
  `"⡄", "⣄", "⣤", "⣴", "⣼", "⡆", "⣆", "⣦", "⣶", "⣾",`
  `"⡇", "⣇", "⣧", "⣷", "⣿"]`
- `braille_down` (top-anchored fill):
  `[" ", "⠈", "⠘", "⠸", "⢸", "⠁", "⠉", "⠙", "⠹", "⢹",`
  `"⠃", "⠋", "⠛", "⠻", "⢻", "⠇", "⠏", "⠟", "⠿", "⢿",`
  `"⡇", "⡏", "⡟", "⡿", "⣿"]`

Parallel `block_up` / `block_down` (U+25A0–U+25FF geometric shapes) and
`tty_up` / `tty_down` (`" "`, `"░"`, `"▒"`, `"█"`) tables exist for the
`block` and `tty` symbol modes. Default config is `graph_symbol = "braille"`
with per-box overrides (`graph_symbol_cpu/net/mem/gpu/proc`) defaulting to
`"default"` (inherit global); `tty_mode` forces `"tty"`.

Requirement context (README Prerequisites): best experience needs truecolor,
UTF-8 locale, and a font covering Braille Patterns U+2800–U+28FF (not needed
in TTY mode or `block`/`tty` graph modes); 256-color is reached by 24-bit to
256-color conversion when `lowcolor` is set; 16-color TTY mode activates on a
real TTY or with `-t/--tty`.

## 2. Sample-to-subcell mapping (2 samples per cell, 5 levels per half-cell)

Constructor (`Draw::Graph`, non-TTY path):

- `value_width = ceil(data.size() / 2)`; `data_offset` drops oldest samples so
  at most `width * 2` samples are drawn; odd counts are evened (`data_offset--`).
- Two alternating string buffers (`graphs[2]`, `current` toggled each frame)
  provide incremental scroll: `operator()(data, data_same)` shifts one cell and
  calls `_create(data, data.size() - 1)` unless `data_same`.
- Leading unfilled width is blank-padded (`" " * (width - value_width)`, or
  `Mv::r(1)` cursor-right for `height == 1`).

`_create(data, data_offset)` core loop:

- Horizontal: one terminal column per **two adjacent samples**,
  `{last, data_value}` (`last` = previous sample, seeded from
  `data[data_offset - 1]` when multiplexing). `current = not current`
  alternates which half-cell each new sample occupies.
- Vertical: per graph row `horizon` in `0..height`:
  `cur_high = round(100 * (height - horizon) / height)`,
  `cur_low = round(100 * (height - (horizon + 1)) / height)`
  (for `height == 1`: `cur_high = 100`, `cur_low = 0`).
- Each of the two values is normalized to 0–100 first (see §4), then quantized
  per row to **0–4**:
  `4` if `value >= cur_high`, `clamp_min` if `value <= cur_low`, else
  `clamp(round((value - cur_low) * 4 / (cur_high - cur_low) + mod), clamp_min, 4)`
  with `mod = 0.3` for `height == 1`, else `0.1`.
- Glyph = `graph_symbol[result[0] * 5 + result[1]]` where `result[0]` is the
  left (older) half-cell level and `result[1]` the right (newer) level.
- `invert` selects the `_down` table and reverses row emission order.

TTY path differs: `value_width = data.size()` (one sample per cell), no
half-cell multiplexing. Net graphs use the multiplexed path under default
settings.

## 3. Gradient computation (positional, not per-dot by value)

`Theme::generateGradients()` builds one 101-entry (`0..100`) escape-code array
per `*_start` color family by linear RGB interpolation: `start -> end`
(`0..100`), or `start -> mid -> end` split `0..50` / `50..100` when a `_mid`
exists. `Default_theme` anchors relevant to graphs:

- `cpu_start #77ca9b` (green), `cpu_mid #cbc06c` (yellow-ish),
  `cpu_end #dc4c4c` (red) — the green→yellow→red family.
- `download_start #291f75`, `download_mid #4f43a3`, `download_end #b0a9de`
  (dark indigo → pale lavender).
- `upload_start #620665`, `upload_mid #7d4180`, `upload_end #dcafde`
  (plum/magenta family).
- `temp_*`, `used_*`, `free_*`, `cached_*`, `available_*`, `process_*`
  families follow the same mechanism.

Application in `Graph::_create`:

- `height > 1`: **one color per row by vertical position**, same color for the
  whole row string:
  normal: `g.at(100 - ((i - 1) * 100 / height))`;
  `invert`: `g.at(i * 100 / height)`. The color does **not** depend on the
  sample value in that cell.
- `height == 1`: **one color per column by value**:
  `g.at(clamp(max(last, data_value), 0, 100))`; all-zero emits `Mv::r(1)`
  (blank, no glyph).
- `Meter::operator()` (CPU/GPU meters, not graphs) *is* per-segment by value;
  do not conflate it with graph coloring.

Net-box wiring (`Net::draw`): `graphs["download"]` uses gradient `"download"`,
`graphs["upload"]` uses `"upload"` — i.e. **per-direction hues**, not the
green→yellow→red value scale. The green→yellow→red `"cpu"` gradient is used by
CPU-box graphs/meters (and GPU graphs reuse `"cpu"`). The map's standing note
"color encodes value gradient (green→yellow→red by fill)" therefore matches
btop's **CPU** convention positionally, not its **network** convention; the
spec must decide which "btop-style" the nload-rs graph adopts.

TTY/lowcolor: `generateTTYColors()` replaces gradients with stepped 16-color
ANSI sequences (`cpu: 92m/93m/91m`-style; download/upload two-step blue/magenta
families); `lowcolor` converts 24-bit through `truecolor_to_256`.

## 4. Saturation, zero, idle, disconnected

- Normalization + clamp: when `max_value > 0`,
  `value = clamp((raw + offset) * 100 / max_value, 0, 100)`. At/above max every
  row quantizes to 4 → full-height column. Numeric labels (`graph_text` max
  label, speed/top/total text) are drawn from unclamped stats.
- Net `max_value`: `down_max`/`up_max` = autoscale `graph_max[dir]` when
  `net_auto` (default true) else `(net_download|net_upload << 20) / 8` bytes/s.
  Autoscale: `bandwidth` history capped at `width * 2` samples; `graph_max`
  floor is `10 << 10` (10 KiB); grow/shrink decided by hysteresis counters
  (`max_count[dir][0|1] >= 5`) from the last-5 average (`1.3x` on grow,
  `3.0x` on shrink path); `net_sync` (default true) copies the larger
  direction's scale to the other. Fixed mode saturates against the configured
  MiBit value instead.
- `no_zero = true` for both net graphs (and CPU/GPU graphs): bottom-row
  `clamp_min = 1` (except the very first half-sample of a fresh draw), so an
  idle/zero column still paints the minimal dot step on the bottom row rather
  than a fully blank cell. `height == 1` zero columns are the exception (blank
  cursor-step). This differs from nload-rs today, where a measured-zero sample
  occupies a column with zero height (all spaces).
- Empty/disconnected: `Net::draw` early-returns the box frame when either
  direction's bandwidth history is empty; per-frame call passes
  `redraw or data_same or not net.connected`, freezing the graph while
  disconnected. Baselines add samples only on accepted observations
  ( Carroll: `bandwidth.push_back(speed)` per collected interval, capped);
  redraws add none.

## 5. Geometry implications vs current H×W ASCII columns

Current (`src/screen.rs`, port spec §5): graph width `width - 30` columns hold
at most one sample per column (newest fitting suffix, oldest-left); graph
height `H` rows hold one of 4 states per cell via whole-bar thresholds
`0.30 / 0.70 / 1.0` (`' '`, `'.'`, `'|'`, `'#'`), bottom-up, saturating at the
fixed per-direction ceiling with uncapped numbers.

btop-equivalent over the same terminal footprint:

- Horizontal density 2×: the same `width - 30` columns carry up to
  `2 * (width - 30)` samples (e.g. 50 cols → 100 samples at 80×24).
  Time-per-column halves; shrink/grow reveals twice the history per redraw,
  or the spec keeps 1-sample-per-column and wastes half the Braille
  horizontal resolution.
- Vertical resolution ~4× (5 quantized levels per row): the same `H` rows
  carry `H` Braille rows × 4 dots with levels 0–4 per half-cell, vs 4 coarse
  ASCII states per cell today. Threshold semantics change from absolute
  whole-bar fractions (0.30/0.70/1.0) to per-row normalized 0–100 bands with
  `+0.1` rounding bias (`+0.3` for single-row graphs).
- Cell cost: each Braille glyph is 3 bytes UTF-8 (vs 1 byte ASCII) and each
  colored row adds variable-length escapes around the whole row string
  (multi-row) or per column (single-row), breaking the current
  "exactly `width` bytes, ASCII-only, no escapes" contract — owned by the
  screen/terminal seam ticket.
- Zero/saturation appearance changes: `no_zero` bottom-row minimum vs current
  blank zero column; autoscale-vs-fixed-ceiling changes what "full" means
  (btop net autoscales by default; nload-rs ceilings are fixed and saturate).
- Font/locale dependency: Braille requires UTF-8 + U+2800–U+28FF coverage;
  `block`/`tty` tables are the upstream fallback, but the standing
  no-fallback preference rules those out pending the seam/terminal tickets.

## 6. What the spec still must decide (evidence handoff, no choices made)

- Which gradient semantic: positional green→yellow→red (`"cpu"`-style) vs
  btop-net per-direction indigo/magenta hues, and per-row vs per-value
  application (multi-row btop is per-row).
- Horizontal packing: 2-samples-per-cell (btop-faithful, 2× history) vs
  1-sample-per-cell (nload-rs-faithful, half the Braille width wasted).
- Zero/bottom-row rule (`no_zero` minimum dots vs current blank zero column)
  and saturation rule (fixed ceiling vs autoscale, and what the max label shows).
- Seam accounting for 3-byte glyphs + escapes (width, clipping, fixtures,
  determinism) — for the screen-seam and terminal-color tickets.
- License hygiene: btop is Apache-2.0; nload-rs is GPL-2.0-only. This note
  records observed facts and pinned links only — no upstream code is copied.
