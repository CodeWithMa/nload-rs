# Styled graph review fixes — Standards and Spec evidence

Recorded 2026-10-08 by the single review-fixes implementer (OpenCode).
Review fixed range: `53f1ab7..a2214c6`.
Worktree: `/tmp/opencode/nload-btop-review-fixes`.
Branch: `implement/btop-review-fixes`.

**Current applicability — 2026-10-08:** The source correction and corrected-artifact
gates/live smoke below remain applicable. The later
[human emulator observations](04-human-observations.md) and
[ticket-04 completion assessment](04-evidence.md#current-acceptance-and-applicability--2026-10-08)
resolved the then-pending human blocker. Final combined acceptance is recorded in
[Ticket 05 — final acceptance evidence](05-evidence.md) and the
[verified styled graph handoff](handoff.md). Pending-human statements in the dated
review record below are historical, not current status.

Authority: AGENTS.md, [local issue tracker guidance](../../docs/agents/issue-tracker.md),
[domain guidance](../../docs/agents/domain.md), [Network Traffic Monitor glossary](../../GLOSSARY.md),
the [styled graph specification handoff](../btop-graph-spec/spec.md) and its
[canonical resolution](../btop-graph-spec/issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract).
Read all five implementation tickets and the existing frame, graph, transport and
live/emulator evidence. No ADR files were present. Historical planning/source
specifications/maps retain their original content and authority/provenance.

## Source, ancestry and artifact identity

- Initial worktree status was clean on the requested branch. HEAD and integration
  both resolved to `a2214c6dcdb969b3ec356fdceb99ae444af1d382`.
  `git merge-base --is-ancestor integration/btop-styled-graphs HEAD` exited 0.
  The branch was already based correctly; no reset was needed.
- Tested source commit: **`5e4661164f7afda85c266b2a45ce730276c71bad`**
  (`fix: round graph half steps without additive promotion`). Gates and live smoke
  ran on exactly that source tree before committing it. Follow-up changes are
  documentation/tracker evidence only, with no executable-input delta.
- Release executable: **`/mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs`**.
  Built from the review source worktree into this durable ignored target directory;
  the permanent checkout remains on integration. This distinguishes artifact
  source from the checkout's older HEAD instead of implying integration already
  contains the correction.
- Version: **`nload-rs 0.1.1`**. SHA-256:
  **`d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`**.
  Artifact identity only; no byte-identical rebuild guarantee. No binary committed.
- After the source commit, `git merge --no-edit integration/btop-styled-graphs`
  returned `Already up to date.` at `a2214c6`; ancestry passed again. Status,
  complete intended diffs and recent ten commits were inspected before commits.
  Final reconciliation after documentation preparation also returned `Already
  up to date.` at the same integration tip; ancestry and whitespace checks passed.

Environment: Linux **7.2.9**, x86_64 GNU/Linux, glibc **2.44**, UID/EUID
**1000/1000**, network namespace `net:[4026531833]`. Stable rustc
**1.98.1 (48a229cea 2026-09-01)**, full compiler commit
`48a229ceaefd4985c50990b14116b6d856af0985`, LLVM **21.1.8**, target/host
`x86_64-unknown-linux-gnu`; Cargo **1.98.0 (797e8a9bc 2026-08-05)**;
Python **3.14.7**. RUSTFLAGS/CARGO_ENCODED_RUSTFLAGS/CARGO_BUILD_TARGET unset.
Agent stdin/stdout are not terminals. TERM=`alacritty`, LANG=`en_US.UTF-8`;
LC_ALL/NO_COLOR/TERM_PROGRAM/TERM_PROGRAM_VERSION/DISPLAY/WAYLAND_DISPLAY/TMUX/STY
unset. These declarations are not emulator identity or visual evidence.

## Standards summary — findings resolved

| Finding | Actual fix and verification |
| --- | --- |
| Minor: linked titles required by tracker guidance | Replaced number-only ticket labels in `02-evidence.md` (transport, live and delivery pointers) and `04-evidence.md` (authority) with the actual linked titles. |
| Heuristic: numeric cause/variant loops couple stimuli and status arrays | `src/application/tests.rs` now uses a small named `ColoredExit` fixture with adjacent case/status tables; the fault test uses an exhaustive local `Fault` enum. `tests/linux_pty.rs` uses a local typed/signal/EOF enum and named case/status/diagnostic table. Expected statuses remain explicit independent literals; named assertion contexts replace numeric variants. All original cases and byte-prefix coverage remain exercised. |
| Heuristic: duplicate baseline Monitor construction | `src/terminal/runtime/tests.rs` has one private `baseline_monitor(name, ceilings)` helper. Both frame fixtures use it; the colored fixture's accepted observation times/totals remain explicit at the call site. All 14 runtime tests passed. |

The existing application/PTY fixtures verify statuses, exact emitted bytes,
observation counts and complete restoration. Their refactors required no new
mirrored implementation tests. No unsafe production code or dependency was added.

## Spec summary — findings resolved

| Finding | Actual fix and verification |
| --- | --- |
| P2: additive +0.5 promotes a just-below half step | In `src/screen.rs`, retain `(r/c).clamp(0,1)` and multiplication by `4H`, then use nonnegative `f64::round()` and the existing dot bound. Positive ties round upward, without the intermediate addition that promoted the reviewed predecessor. Frame-interface adjacent-f64 tests have literal whole-column glyph oracles, no fuzzy tolerance. |
| P2: emulator checklist depends on disposable ticket-04 worktree and stale artifact | The checklist/template now use permanent `/mnt/programming/rust/nload-rs`, integration branch and durable corrected artifact path. They identify the artifact's review-fix source independently from integration HEAD. The unconditional old-source rebuild was removed; rebuilding applies once integration contains the source correction. Original paths/hashes/live results are retained as historical provenance in the evidence. Release/PTY/live checks were actually rerun on the corrected artifact. Human reports remain mandatory and pending. |

### Meaningful test-first execution at the agreed frame seam

Loaded `implement`, required `tdd`, and `code-review` for the closure audit.
The user's single-implementer instruction governs this session; no delegated
review is claimed. The user's requested seam is screen construction and the
opaque frame's read-only dimensions/encoded rows, already prescribed by the
canonical contract. Tests decode only permitted styling at that seam; they do
not call private quantization helpers or expose private monitor queues.

1. Added one first-threshold frame regression before changing production code:
   `cargo test --locked adjacent_rates_at_first_default_ten_row_half_step_do_not_gain_a_dot`.
   **Red, exit 101:** H=10, default c=1310720, threshold r=16384;
   `16384.0.next_down()` = `16383.999999999998` emitted `⢀` instead of ASCII space.
2. Changed only dot rounding to positive `round()`. The same command
   **passed**, establishing red → green for the real reviewed defect.
3. Expanded/renamed that test to
   `screen::tests::adjacent_rates_at_half_steps_do_not_gain_a_dot_from_additive_rounding`.
   Both directions are checked with exact full-column literals at H=10/default c
   and H=6/c=128. Adjacent f64 predecessor/at/successor inputs cover first,
   second, fifth and twentieth half steps, including transitions across Braille
   rows. Added
   `screen::tests::intermediate_normalization_ties_use_the_normalized_rate_not_a_rate_threshold`
   for intermediate normalization rounding. All **22 screen tests passed**.

Floating-point rationale: the reviewed normalized scaled value is
`0.49999999999999994`; adding 0.5 rounds the sum to 1.0 and creates the extra
dot. Positive `round()` leaves it at zero. Division and multiplication can
themselves round other adjacent inputs onto a halfway value: for example,
H=10/default c, the predecessor of r=114688 scales to exactly 3.5; H=6/c=48,
the predecessor of r=1 scales to exactly 0.5. The normalization test also covers
adjacent inputs around the default H=6 first step (r≈27306.666666666668).
Those normalized ties correctly round upward under the prescribed `r/c` then
`4H` operation order. We did not substitute a reconstructed input-rate threshold
or exact-rational algorithm. No epsilon, extra bias or forced minimum is used.
The tests assert literal glyph outcomes, not an implementation-derived rounding
formula. Existing all-step/zero/saturation/odd-height cases continue to pass.

## Actual gates on corrected source/artifact

All commands ran from the review worktree and exited 0 unless identified as the
intentional red above.

| Command | Result |
| --- | --- |
| `cargo fmt` then `cargo fmt --check` | Passed. |
| `cargo test --locked screen::tests` | 22 passed. |
| `cargo test --locked colored_` | 6 unit + 3 debug PTY passed. |
| `cargo test --locked handled_exits_after_every_colored_output_prefix_cancel_reset_and_restore` | 1 passed, all original prefix/exit cases. |
| `cargo test --locked terminal::runtime::tests` | 14 passed. |
| `cargo check --locked` | Passed on final executable source. |
| `cargo test --locked` | **115 unit + 16 Linux PTY/decoder passed**, zero failed or ignored. |
| `cargo build --release --locked --target-dir /mnt/programming/rust/nload-rs/target/btop-review-fixes` | Optimized release built. |
| `NLOAD_RS_PTY_BINARY=/mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs cargo test --locked --test linux_pty` | **16 passed** using the identified release executable. |
| `/mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs --version` | `nload-rs 0.1.1`. |
| `sha256sum /mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs` | Hash above. |
| `git diff --check` and staged whitespace review | Passed. |

Release PTYs include the named exit/EOF fixture, real colored partial-output
backpressure/resize recovery, locale/NO_COLOR at both sizes, small-terminal
sampling/navigation/growth, normal controls/signals, practical setup/runtime
faults and OS restoration. Existing detailed case mappings in
[styled transport evidence](03-evidence.md) still describe those passing tests.

## Fresh ordinary loopback live smoke on corrected release

One-off process/PTY observation invoked as
`python3 /tmp/opencode/btop-review-live-smoke.py`; exited 0. This is operational
evidence collection, not a new reusable application helper/test oracle. It reused
the existing [ordinary loopback generator](../nload-rust-implementation/loopback-traffic.py)
unchanged, per the method in [live/emulator evidence](04-evidence.md).

- Real allocated Linux PTY terminal stdin/stdout, redirected stderr;
  TERM=`xterm-256color`, inherited LANG; no emulator attached.
- At 60×16: direct durable release executable with `-i 2048 -o 4096`.
  Initial `docker0 (1/3)`. Right, Down, Right, Up, Left, Down gave
  `enp1s0 (2/3)`, `lo (3/3)`, `docker0 (1/3)`, `lo (3/3)`,
  `enp1s0 (2/3)`, `lo (3/3)`; headers collected after 100-ms intervals.
- At 80×24: same direct executable with `-i 2048 -o 4096 lo`, initial `lo (3/3)`.
- Both runs retained `Incoming: (ceiling 2.00 MBit/s)` and
  `Outgoing: (ceiling 4.00 MBit/s)`. Frame cell lengths/row counts were checked;
  all twelve foreground indices occurred in each run. Baseline had `--` rates,
  available totals and blank newest graph cells; measured warm-up was zero.
- After idle warm-up, each run used
  `python3 .scratch/nload-rust-implementation/loopback-traffic.py --phase-seconds 2`:
  ordinary IPv4 UDP, 8192-byte payloads, immediate receive, two repetitions of
  32/128/0 datagrams-per-second low/high/idle. No privilege or interface mutation.

Actual displayed checkpoints (~1.75 seconds into each two-second phase):
incoming and outgoing happened to match on this workload. Rates/proc deltas are
observations, not pacing-derived numerical or exact sample-count oracles.

| Size / checkpoint | Curr, both | Avg, both | Ttl, both |
| --- | --- | --- | --- |
| 60×16 before | 0.00 Bit/s | 0.00 Bit/s | 21.66 MByte |
| low 1 | 2.01 MBit/s | 899.01 kBit/s | 22.04 MByte |
| high 1 | 7.90 MBit/s | 2.91 MBit/s | 23.66 MByte |
| idle 1 | 0.00 Bit/s | 2.65 MBit/s | 24.15 MByte |
| low 2 | 2.01 MBit/s | 2.42 MBit/s | 24.53 MByte |
| high 2 | 7.90 MBit/s | 3.12 MBit/s | 26.14 MByte |
| idle 2 | 0.00 Bit/s | 2.95 MBit/s | 26.63 MByte |
| after | 0.00 Bit/s | 2.84 MBit/s | 26.63 MByte |
| 80×24 before | 0.00 Bit/s | 0.00 Bit/s | 26.63 MByte |
| low 1 | 2.01 MBit/s | 1.09 MBit/s | 27.04 MByte |
| high 1 | 7.90 MBit/s | 3.35 MBit/s | 28.72 MByte |
| idle 1 | 0.00 Bit/s | 2.84 MBit/s | 29.12 MByte |
| low 2 | 2.01 MBit/s | 2.57 MBit/s | 29.52 MByte |
| high 2 | 7.90 MBit/s | 3.33 MBit/s | 31.21 MByte |
| idle 2 | 0.00 Bit/s | 3.06 MBit/s | 31.60 MByte |
| after | 0.00 Bit/s | 2.95 MBit/s | 31.60 MByte |

Min remained zero; final Max was 7.96 MBit/s in each run. Numeric high rates
exceeded both ceilings; encoded graphs saturated without capping the numbers.
Older activity remained during idle. For example, the last six incoming bottom
cells at 60×16 were `    ⣸⣿` (low 1), `  ⣸⣿⣿⣿` (high 1), `⣸⣿⣿⣿⣿ `
(idle 1); after traffic both directions at both sizes ended `⣿⣿⣿⣿⡇ `.
These are decoded stream observations, not font/color acceptance.

Actual generator output summaries (each also printed low/high/idle twice,
two seconds per phase, and the concurrent-traffic caveat):

```text
60×16:
uid/euid=1000/1000; interface=lo; IPv4 UDP 127.0.0.1
Before: RX=22712424 TX=22712424 bytes
Elapsed: 12.002 seconds; received datagrams=634
After: RX=27923904 TX=27923904 bytes
Observed proc delta: RX=5211480 TX=5211480 bytes
Generator status: 0 stderr: b''

80×24:
uid/euid=1000/1000; interface=lo; IPv4 UDP 127.0.0.1
Before: RX=27923904 TX=27923904 bytes
Elapsed: 12.002 seconds; received datagrams=634
After: RX=33135384 TX=33135384 bytes
Observed proc delta: RX=5211480 TX=5211480 bytes
Generator status: 0 stderr: b''
```

Both smoke runs quit with q, **status 0**, empty stderr, saved termios and shared
file-status flags restored, exact CAN/reset/show/leave cleanup suffix present.
Complete captured frame counts were 35 and 28; not sample-count oracles.

## Historical review closure audit and then-pending blockers

The single implementer checked all supplied findings along the separate Standards
and Spec axes above; this is a closure audit, not an independent delegated review.
All five actionable findings are addressed. The correctness ticket
[Display paired Braille graph history with direction palettes](issues/02-paired-braille-direction-palettes.md)
remains closed with a linked follow-up. Current release/live applicability is
recorded in [live/emulator evidence](04-evidence.md), with the updated
[durable emulator checklist](04-emulator-checklist.md) and
[blank human report template](04-human-report-template.md).

**Still blocked:** [Record emulator appearance and nonprivileged live traffic](issues/04-emulator-live-traffic.md)
requires actual human T14, V05-emulator and ordinary-emulator D04 observations
at both exact sizes, including font/palettes/statistics contrast, navigation/
small-window history recovery and separate q/physical Ctrl+C/external SIGTERM
visual restoration reports. [Complete styled-graph acceptance and delivery handoff](issues/05-acceptance-delivery-handoff.md)
remains blocked on that human evidence. No prior live run or PTY stream is
claimed as current human visual acceptance.
