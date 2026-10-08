# Ticket 04 — runnable styled-graph emulator checklist

**Completed — human observations recorded.** The
[actual report and follow-up](04-human-observations.md) complete the T14,
V05-emulator and ordinary-emulator D04 observations; see
[current evidence mappings](04-evidence.md#current-acceptance-and-applicability--2026-10-08)
and the [ticket resolution](issues/04-emulator-live-traffic.md#resolution--2026-10-08--implementer-04-opencode).
The procedure remains runnable for repeat checks, using the
[reusable report template](04-human-report-template.md). Its prescribed actions
and expected statuses are instructions, not additional observations.

Recorded full sizes were 60×16 and 80×24. Run A's actual small q size was **56×17**,
substituting for the suggested 59×16; Run C's below-floor startup/growth was
confirmed without an exact small-size transcript. Exact 59×16/60×15 boundaries
use the automated evidence mapped in §5 and the completion record. Configured
font/client OS/theme name were unavailable; clean single-cell Braille and the
listed color/text/idle/history/exit behaviors were explicitly confirmed.

## 1. Identify the tested source, executable and emulator

Use two terminal windows/tabs, A for the UI and B for traffic/size queries,
connected to the **same Linux network namespace**. Use the permanent checkout
`/mnt/programming/rust/nload-rs` on `integration/btop-styled-graphs` in both.
Commands below are relative to its root. The corrected release artifact is
`target/btop-review-fixes/release/nload-rs`; it is already built at this durable
path from the source revision identified below.
Use an ordinary interactive shell so nonzero exit statuses can be recorded.

In A, record:

```sh
pwd
git branch --show-current
git rev-parse HEAD
git status --short
git diff 5e4661164f7afda85c266b2a45ce730276c71bad -- src tests Cargo.toml Cargo.lock
./target/btop-review-fixes/release/nload-rs --version
sha256sum ./target/btop-review-fixes/release/nload-rs
rustc --version --verbose
python3 -c 'import os; print("uid/euid:", os.getuid(), os.geteuid()); print({k: os.environ.get(k) for k in ("TERM", "COLORTERM", "TERM_PROGRAM", "TERM_PROGRAM_VERSION", "LANG", "LC_ALL", "NO_COLOR", "TMUX", "STY", "SSH_TTY")}); print("network namespace:", os.readlink("/proc/self/ns/net"))'
tty
stty size
```

The agent's tested source is `5e4661164f7afda85c266b2a45ce730276c71bad`
(`implement/btop-review-fixes`); version `nload-rs 0.1.1`; artifact SHA-256
`d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`.
Integration now contains the correction. The human report identified checkout
`3765b31` with an empty executable-source diff from the reviewed source and this
artifact hash. Closure review at integration `2ee6d8d` found the same source inputs;
both `target/release/nload-rs` and `target/btop-review-fixes/release/nload-rs` had
the hash above and passed a byte comparison. Commands below retain the durable
path actually identified in the human report. For a repeat run, record source
and artifact identity first; rebuild if needed with:

```sh
cargo build --release --locked --target-dir target/btop-review-fixes
sha256sum ./target/btop-review-fixes/release/nload-rs
```

The original ticket-04 source `48584d0`, artifact
`50d9a750…`, and temporary worktree in [historical live evidence](04-evidence.md)
are provenance, not current corrected-artifact validation. Fresh agent release,
PTY and live checks are in [Review fixes — Standards and Spec evidence](review-fixes-evidence.md).
If your HEAD, executable delta or artifact differs, include that fact in the report
so applicability can be reviewed. A later documentation-only HEAD is compatible
when executable inputs and artifact are unchanged. Rebuilds need not be identical.

On the **client machine actually running the emulator**, record name/version
(e.g. `alacritty --version`, `kitty --version`, or the About dialog), client OS,
font name/size and any fallback font, background/theme, SSH and multiplexer layers.
Keep the emulator's real TERM declaration; supported operation assumes UTF-8,
one-cell Braille font coverage and xterm-256color support. Missing/empty TERM and
exact `dumb`/`linux` are rejected; TERM/locale alone are not visual evidence.

Record how UTF-8 and 256-color support are configured. Inspect actual Braille in
the application for missing boxes, overlapping glyphs or unexpected two-cell width.
This optional font probe gives independently recognizable full/half glyphs:

```sh
python3 -c 'print("Braille font probe: |⡇|⢸|⣿|  each glyph should occupy one cell")'
```

### Dimensions and discovery

`stty size` returns **rows columns**: `16 60` means **60×16**. Resize the actual
emulator window to the exact targets; changing just kernel `stty rows/cols` does
not demonstrate visible behavior. If exact sizing is difficult, report the actual
size and leave the exact-size criterion pending. In B, measure A during a run:

```sh
stty -F /dev/pts/N size
```

Replace `/dev/pts/N` with the path returned by A's `tty`. Query at each transition
and record it. In B, confirm namespace and current interfaces:

```sh
python3 -c 'import os; from pathlib import Path; print("uid/euid:", os.getuid(), os.geteuid()); print("network namespace:", os.readlink("/proc/self/ns/net")); print(Path("/proc/net/dev").read_text())'
```

Agent-discovered interfaces: `docker0`, `enp1s0`, `lo` in natural startup order,
positions 1/3, 2/3, 3/3. If current membership differs, record actual names/positions
and adapt navigation accordingly. The ordinary local generator below monitors
`lo`; do not assume it exercises `enp1s0` or `docker0`.

## 2. Run A — exact 60×16, discovery, palettes, live traffic, q

1. Set A to **60×16** and record `stty size`=`16 60`. Leave recognizable prior
   shell text with this marker, then start the **direct release executable**:

   ```sh
   python3 -c 'print("TICKET04 RUN A prior-screen marker — q")'
   ./target/btop-review-fixes/release/nload-rs -i 2048 -o 4096
   ```

2. Record the initial header, ideally baseline status (`Waiting for rate sample`,
   rates `--`, totals available, graphs empty). Baseline may be too brief to see;
   report that honestly and use the scripted state evidence in §5. With today's
   list the initial selection is `Interface docker0 (1/3)`.
3. Press **Right, Down, Right, Up, Left, Down**, separately. Today's expected
   sequence is `enp1s0 (2/3)`, `lo (3/3)`, `docker0 (1/3)`, `lo (3/3)`,
   `enp1s0 (2/3)`, `lo (3/3)`. Record all actual headers and whether changes look
   prompt. Confirm these exact independent ceiling labels throughout navigation:
   **Incoming: (ceiling 2.00 MBit/s)**; **Outgoing: (ceiling 4.00 MBit/s)**.
4. Before traffic, note lo incoming/outgoing `Ttl` and `Curr` separately. In B:

   ```sh
   python3 .scratch/nload-rust-implementation/loopback-traffic.py
   ```

   Keep the generator output. It runs six ten-second low/high/idle phases, uses
   nonprivileged ordinary local UDP, and prints exact proc counters/deltas. Payload
   pacing is approximately 256 KiB/s / 1 MiB/s, not an exact UI-rate expectation.
5. During both low and high phases observe **both** stacked 30×6 graphs, their
   older-left/newer-right half-cell appearance, and no wrapping/stray rows.
   Incoming is indigo/lavender; outgoing magenta. Bottom grades should be legible
   medium-dark shades and top grades pale. High traffic normally saturates both,
   revealing the complete palettes. If it does not reach the top, report that and
   repeat with `-i 1 -o 2 lo` and traffic; labels then become 1.00/2.00 kBit/s.
6. Verify default-style header, separator, labels, two-column gap, and adjacent
   right-hand `Curr/Avg/Min/Max/Ttl` fields. Record their contrast beside the colored
   dots. No background coloring or persistent graph color should leak into text.
7. Record actual incoming/outgoing rates and totals at low/high/idle checkpoints.
   At the 2/4 MBit/s ceilings high **numeric** rates may exceed both labels while
   graphs saturate. During idle look for genuine `0.00 Bit/s`, blank new positions
   on the right and older nonzero history scrolling left, rather than being erased.
   If other lo traffic prevents measured zero, record it and cite §5's scripted
   zero evidence. Totals should advance during traffic, remaining cumulative rather
   than starting at zero for this session.
8. After recording the full-size appearance, shrink A to **59×16**, measure it
   from B and inspect default-style warning/selection/status, with no full graphs
   or wrap. Press **q while small**. Immediately in A:

   ```sh
   smoke_status=$?; printf 'Run A q status: %s\n' "$smoke_status"
   stty size
   ```

   Expected status **0**. Record **each**: visible cursor, default foreground AND
   background, normal shell typing/echo/editing, and prior shell screen/marker
   returned. Type a normal harmless command such as `pwd` to check shell interaction.

## 3. Run B — exact 80×24, width recovery, small sampling, typed Ctrl+C

1. Set A to **80×24** (`24 80`); leave a new marker and explicitly select lo:

   ```sh
   python3 -c 'print("TICKET04 RUN B prior-screen marker — typed Ctrl+C")'
   ./target/btop-review-fixes/release/nload-rs -i 2048 -o 4096 lo
   ```

2. In B rerun the **same** default 60-second generator command from Run A. At
   80×24 verify stacked **50×10** graphs, both complete palettes through high
   traffic, pale tops/medium-dark bottoms, default-style text contrast and exact
   independent 2.00/4.00 MBit/s labels. Record before/after separate totals and
   low/high/idle rates again; Run A does not supply observations for this size.
3. At about **35 seconds** into traffic, note a recognizable earlier low/high/idle
   history pattern on lo. Shrink to the still-full **60×16** and measure it. Older
   positions can be hidden because capacity drops from 100 to 60 samples. Then
   shrink to **59×16**, measure it, and keep traffic running for about five seconds.
   Inspect warning text default styling. Press Right then Left while small; record
   selection changes and that returning selects lo.
4. Grow back to **80×24**, ideally before about 50 seconds into traffic so the
   earlier marker still fits 100 samples. Record the measured dimensions, whether
   the recognizable older pattern reappears, new history/totals accumulated while
   small, and two-sample/half-cell alignment without repeated or fabricated
   samples. Resizing alone does not prove retention: describe the earlier pattern
   and what reappears. Exact sample pairing/mutation is established by §5's fixtures;
   note if motion is too fast to distinguish visually.
5. Type a **physical Ctrl+C keypress in A**, separately from q and external
   signals. Immediately record:

   ```sh
   smoke_status=$?; printf 'Run B typed Ctrl+C status: %s\n' "$smoke_status"
   ```

   Expected status **130**. Independently record cursor visibility, default
   foreground/background, normal shell interaction, and Run B's prior-screen return.
   Let B's generator finish and collect its complete output before starting Run C,
   so the separate traffic runs do not overlap.

## 4. Run C — small startup at 60×15, growth, external SIGTERM

1. Set A genuinely to **60×15** (`15 60`) and measure it. The following command
   starts the same release binary inheriting A's terminal streams, and sends
   SIGTERM to **that owned child only** after 25 seconds. It prints the **child's**
   status; the Python wrapper's shell status is a separate value.

   ```sh
   python3 -c 'import signal, subprocess, time; print("TICKET04 RUN C prior-screen marker — SIGTERM after 25 seconds", flush=True); p = subprocess.Popen(["./target/btop-review-fixes/release/nload-rs", "-i", "2048", "-o", "4096", "lo"]); time.sleep(25); p.send_signal(signal.SIGTERM); print("Run C SIGTERM child status:", p.wait(), flush=True)'
   ```

2. Immediately in B start a shorter generator:

   ```sh
   python3 .scratch/nload-rust-implementation/loopback-traffic.py --phase-seconds 3
   ```

3. Check valid small startup, lo selection, default-style warning and no full-size
   graphs. Press Right then Left while small. After roughly 12 seconds grow to
   **60×16**, then **80×24**, measuring each. Describe actual traffic history and
   totals recovered from the small period, rather than assuming sampling continued.
4. Do not type q/Ctrl+C in this run. Allow the owned child to receive external
   SIGTERM. Record the printed **child status 143**, any diagnostics, and separately
   visible cursor, default foreground/background, normal shell interaction and
   Run C prior-screen/marker return. The wrapper itself normally exits 0.

## 5. Practical scripted coverage for states not reliably visible live

Do not remove interfaces, disrupt `/proc`, or infer unavailable states from idle.
The production CLI has no injection option. These existing deterministic tests
drive real Monitor selected views through the opaque screen-frame seam; they are
**scripted state/encoding evidence, not human visual observations**:

```sh
cargo test --locked screen::tests::baseline_absence_failure_idle_and_recovery_have_exact_rows
cargo test --locked screen::tests::measured_idle_shifts_older_traffic_by_real_blank_half_cells
cargo test --locked screen::tests::paired_saturated_halves_and_real_zeros_use_direction_foregrounds
cargo test --locked screen::tests::fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view
cargo test --locked screen::tests::sampling_and_navigation_while_small_reappear_as_actual_history_on_growth
```

The first covers baseline/absent/read-failed/idle/recovery/decreased counters and
exact placeholders/totals/blank graphs. The next two cover true ASCII-space zeros,
idle retaining older traffic and exact odd/even half-cell placement. The last two
cover retained history and small-period sampling/navigation without render mutation.
All passed in the corrected-source gates recorded in
[review-fixes evidence](review-fixes-evidence.md#actual-gates-on-corrected-sourceartifact).
Exact 59×16/60×15 boundary behavior also passed
`screen::tests::small_and_zero_dimensions_show_only_bounded_warning_selection_status`
and Linux PTY `small_startup_resize_sampling_navigation_and_quit_keep_bounded_rows`.
The completed human run supplies the visual below-floor behavior at actual 56×17
and during Run C's small startup/growth; exact half-cell arithmetic and
uninducible states remain clearly scripted evidence. For repeat checks, cite what
was actually seen and return any failures, skipped actions or ambiguous items.
Ticket 04's supplied report has been resolved; no human observation remains
pending for its closure.
