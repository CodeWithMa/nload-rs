# Ticket 02 — paired Braille and direction palettes evidence

Recorded 2026-10-08 by implementer-02 (OpenCode), in
`/tmp/opencode/nload-btop-02` on `implement/btop-02`.

Authority: [Display paired Braille graph history with direction palettes](issues/02-paired-braille-direction-palettes.md)
and its linked canonical styled graph resolution. The historical port spec and
planning maps retain their original wording; the newer contract supersedes ASCII
thresholds, one-sample-per-cell presentation, and ASCII-row/byte-offset fixtures.
Unchanged application behavior is inherited. Actual Cargo version remains
**0.1.1**, as established by recent commits, rather than the port's initial 0.1.0.

## Baseline and integration

- Initial status was clean; branch was `implement/btop-02`.
- Initial HEAD and `integration/btop-styled-graphs` both resolved to
  `b784de4c71628d0c7c38ca6b7eb1eae77be0aa0b`.
  `git merge-base --is-ancestor integration/btop-styled-graphs HEAD` exited 0.
  No reset was needed. Ticket 01's frame seam/evidence at `ba62d95` was read.
- After final automated gates, `git merge integration/btop-styled-graphs`
  returned `Already up to date.`; the integration tip remained `b784de4`.
- Tested source: `b784de4` plus this ticket's changes to `src/screen.rs`, monitor
  test-only coverage, terminal module documentation, `tests/linux_pty.rs`, and
  README. The subsequent evidence/tracker edits contain no executable changes.

## Test-first execution

Loaded `tdd` and `implement`. The accepted ticket/contract already prescribes the
test seams: selected read-only monitor view, screen construction/read-only frame
rows, and the real binary through Linux PTYs. No private monitor queues or private
formatters are used by the migrated screen tests.

| Slice / command | Observed red | Observed green |
| --- | --- | --- |
| `cargo test --locked paired_saturated_halves_and_real_zeros_use_direction_foregrounds` | The inherited single-sample ASCII output emitted `#`, without the required right-half `⢸` or foreground SGR. | Same command passed after pairing saturated halves, real zero placement, and default-safe direction foreground encoding. |
| `cargo test --locked nearest_dot_half_steps_are_exact_bottom_up_without_minimum_or_tolerance` | The saturated-only tracer emitted blank at rate 0.5, where the six-row/c=24 halfway boundary requires one bottom dot `⢀`. | Same command passed after independent normalization, nearest-dot rounding and standard Unicode dot placement; `cargo check --locked` also passed. |

The first cycle implemented saturated samples only; fractional quantization was
added only after the second red. Follow-on fixtures extended coverage through the
same seams: all 25 bottom-filled mask combinations, unequal halves/directions,
odd heights, complete palettes, idle scrolling, fitting capacities, and locale
declarations. They verified the implemented behavior without additional product
branches or fallback modes. The final `cargo test --locked` passed
all **20** screen tests, covering all twelve inherited functions plus the frame
test and seven added tests.

Two test-oracle corrections were made, without product changes:

- The palette contrast fixture initially expected `⣿` at Curr even in a six-row
  graph where its half-height neighbor does not reach Curr. It now checks the
  required gap/statistics resets, with exact masks covered separately.
- The new PTY fixture initially required explicit gap characters on rows whose
  trailing blanks were correctly trimmed. It now treats omitted suffix cells as
  default blanks supplied by frame clearing, while enforcing explicit gaps where
  statistics follow. The targeted PTY check then passed all four environments.

Review added odd-height dot oracles at 61×17 and 81×25, including partial middle
rows and first half-dot boundaries at direction-specific c=8H. Those passed; the
full gates were rerun on this final source.

## Final gates and artifact

All commands below exited 0 in this worktree:

| Command | Result |
| --- | --- |
| `cargo fmt --check` | Passed after `cargo fmt`. |
| `cargo test --locked` | **104 unit + 14 Linux PTY/decoder tests**, no failures or ignored tests. |
| `cargo build --release --locked` | Passed; optimized executable built. |
| `NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty` | **14 passed**, no failures or ignored tests, using the release executable. |
| `cargo check --locked` | Passed during the quantization cycle. |
| `git diff --check` | Passed. |
| `./target/release/nload-rs --version` | `nload-rs 0.1.1`. |
| `cargo tree --locked --target x86_64-unknown-linux-gnu -e features` | Existing libc/signal-hook graph only; signal-hook default features remain disabled. |

Environment: Linux **7.2.9**, x86_64 GNU/Linux, UID **1000** (nonprivileged).
Compiler: `rustc 1.98.1 (48a229cea 2026-09-01) (built from a source tarball)`;
commit `48a229ceaefd4985c50990b14116b6d856af0985`, LLVM 21.1.8,
host/target `x86_64-unknown-linux-gnu`.
Cargo: `cargo 1.98.0 (797e8a9bc 2026-08-05)`.

Final tested artifact: `target/release/nload-rs`, SHA-256
`50d9a750738bf4381eabd6fb72a96967098bdd6ba90067f3d2f255c5da176746`.
This identifies the locally tested artifact; no byte-identical rebuild claim.

## Exact V01–V08 mappings and twelve-function migration

Names in the following table are under `screen::tests`. Each test observes the
opaque frame, whose test-side whitelist decodes only reset/permitted foreground
SGR, printable ASCII and nonblank Braille. It counts cells, validates default
text/blanks, row resets and logical dimensions, and pads trimmed suffixes with
default blanks. Glyph/color/text decisions use exact assertions, not rate
tolerances. The first twelve entries account for all twelve inherited screen
functions; the old ASCII threshold function is replaced by the named nearest-dot
test, rather than retaining a superseded threshold oracle.

| Actual test name | Cases and exact behavior |
| --- | --- |
| `exact_80_by_24_stacked_rows_and_bottom_aligned_fields` | V01/V03: all 24 rows, 50×10 geometry, labels/header/separator, 28-cell fields, two-cell gaps, bottom alignment, half/full-height paired samples; no extra axis/footer/time labels. |
| `minimum_and_odd_geometry_keep_gap_fields_and_outgoing_extra_row` | V01: 60×16, 61×17, 80×25; outgoing extra row, statistics alignment and exact newest right-half saturation. |
| `nearest_dot_half_steps_are_exact_bottom_up_without_minimum_or_tolerance` | V02: below/at/above every half-dot boundary across all 24 steps at H=6 (±1e−12, no fuzzy tolerance); independently both directions; literal right-half masks, true zero/tiny positive blank, half ceiling, exact/above-ceiling fill. |
| `rate_units_choose_prefix_before_explicit_halfway_up_rounding` | V07: all inherited zero/128/0.125 Bit/s/127.9995 and near-halfway cases, below/at/above each binary prefix, halfway-up at each prefix, capped G; now observes Curr through a frame. |
| `total_units_round_exact_widened_integers_including_maximum_u64` | V07/V08: inherited exact integer totals, all prefix boundaries, halfway-up, max-u64, and large values differing by one byte across a hundredth boundary; now observes Ttl through a frame. |
| `independent_ceiling_labels_and_numeric_prefixes_do_not_cap_rates` | V02/V07/V08: independent 20/5 MBit/s labels, independent numeric prefixes, exact text halfway-up, actual uncapped current/max values and saturated right halves. |
| `largest_exact_totals_and_ceilings_and_overwide_rates_keep_labels` | V06/V08: real Monitor max-u64 deltas in one nanosecond, largest valid ceiling plus minimum outgoing ceiling, exact ceiling/total text, all four `[too wide]` rate fields, saturation with preserved field labels. |
| `baseline_absence_failure_idle_and_recovery_have_exact_rows` | V04: real Monitor baseline, idle, absent, read-failed, recovered baseline, and readable-decrease baseline; exact status/header/totals/placeholders, unavailable graphs blank. |
| `raw_names_are_ascii_escaped_shortened_and_keep_position` | V06: original bytes incl. NUL/ESC/DEL/0x80/0xFF, uppercase printable `\xHH` escaping, complete escape chunks, safe ellipsis, position reservation and unchanged identity; saturated graph beside default-style text. |
| `small_and_zero_dimensions_show_only_bounded_warning_selection_status` | V05: all states below the floor at 59×16/60×15; 0×0, 0×5, 80×0, 1×1/2×2/3×3, 1×1000/1000×1; no palette codes even with saturated retained history, exact bounded warning/selection/status; full view restored at 60×16. |
| `fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view` | V03/S07/S12: 70 actual delayed observations with real zeros and distinctive newest pair; 60×16, 80×24, 100×24 and small mode; exact chronological suffixes, repeated growth recovery and unchanged complete selected view/sample count. |
| `sampling_and_navigation_while_small_reappear_as_actual_history_on_growth` | V03/V05/S11: sample both interfaces while 1×1; select previously hidden interface; growth reveals exactly three observations as `(blank,a)(b,c)`, no navigation samples/resets. |
| `opaque_frame_retains_dimensions_and_encodes_trimmed_default_rows` | V01/V05/frame protocol: private frame dimensions/row count, exact trimmed default rows/statistics resets and zero-size encodings. |
| `paired_saturated_halves_and_real_zeros_use_direction_foregrounds` | V02/V03: literal encoded incoming/outgoing rows for one/two/three/four samples, `⢸`, `⡇`, `⣿`, true zero positioning, no odd duplication, default blanks and resets. |
| `standard_braille_masks_cover_every_bottom_filled_half_combination` | V02: all 25 zero-to-four-dot left/right combinations against independently worked Unicode literals, true ASCII blank mask and blank rows above partial fills. |
| `independent_half_and_direction_quantization_preserves_uncapped_text` | V02/V08: unequal two-dot/saturated halves at c=24, one-dot/fourteen-dot halves at c=48, literal whole columns, independent labels and uncapped 224.00 Bit/s text. |
| `odd_graph_heights_quantize_half_ceiling_and_first_half_dot_independently` | V01/V02: 61×17 and 81×25, H=6/7 and 10/11, literal half-ceiling columns with partial middle row, below/at/above first half-dot at separate c=8H. |
| `positional_palettes_use_each_direction_height_and_leave_text_default` | V01/V06: every row's exact foreground at 60×16/60×17/80×24/80×25, all twelve indices, 6/7/10/11-row grades and exact half-grade ties; colors uniform across different fills, all ASCII cells default, reset at gap and before statistics, plain labels/header/status. |
| `measured_idle_shifts_older_traffic_by_real_blank_half_cells` | V03/V04: real Monitor traffic then idle, exact `⢸` → `⡇` → `⢸ ` half-cell movement, genuine numeric zero current with normal separator and one sample per observation. |
| `fitting_suffix_displays_sixty_or_one_hundred_of_actual_retained_observations` | V01/V03/S12: 101 actual observations; exact 60- and 100-sample suffixes, 81×25 reveals previously hidden oldest marker, both directions, real final zero, complete selected view and all 101 retained samples unchanged. |

## Affected sampling/history mappings

These passing tests preserve actual observation counts independently of doubled
display capacity. Names are fully qualified to distinguish the public seams:

| Case | Actual tests / evidence |
| --- | --- |
| S03 | `monitor::tests::warm_up_exposes_exact_totals_and_independent_directional_rates`: baseline no samples, then exactly one/two observations, independent rates/totals/timestamps. Pairing is covered by `screen::tests::paired_saturated_halves_and_real_zeros_use_direction_foregrounds`. |
| S07 | `monitor::tests::delayed_and_suspend_length_intervals_use_actual_elapsed_time_once`; `application::tests::slow_initialization_and_slow_acquisition_skip_slots_use_completion_and_actual_time`: one measured sample for delayed acquisition, actual elapsed denominator, skipped cadence slots. `screen::tests::fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view` renders actual delayed samples at two per cell. |
| S09 | `monitor::tests::decrease_in_either_counter_resets_both_directions_to_new_baseline`: either-direction decrease clears both, next sample count one. `screen::tests::baseline_absence_failure_idle_and_recovery_have_exact_rows` checks decrease/recovery blank baseline with fresh totals. |
| S11 | `monitor::tests::never_selected_interfaces_accumulate_real_history_at_every_round`; `screen::tests::sampling_and_navigation_while_small_reappear_as_actual_history_on_growth`; `application::tests::small_session_keeps_offscreen_sampling_and_recovers_real_history_on_growth`: fixed off-screen counts/timestamps/rates, no navigation acquisition, continued small-mode acquisition/quit and growth recovery. |
| S12 | `monitor::tests::history_expiry_retains_exact_boundary_and_reads_only_hide_fitting_suffixes`: exact t−300 boundary, 302 retained samples, observation-only expiry, doubled consumer capacities 100/2/0/302 for widths 50/1/0/400. `screen::tests::fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view` and `screen::tests::fitting_suffix_displays_sixty_or_one_hundred_of_actual_retained_observations` verify real glyph suffixes/capacities and no rendering mutation. |

## Linux PTY and support mappings

All fourteen `tests/linux_pty.rs` functions passed with both Cargo's debug binary
and the release artifact above. PTYs check Linux protocol/OS state, not visual
font quality or emulator appearance.

| Cases | Actual PTY test names / assertions |
| --- | --- |
| T01/V01/V02/V06 | `accepted_full_views_emit_colored_braille_under_no_color_and_locale_declarations`: real `/proc` loopback view, independent `-i 1 -o 1`, nonprivileged UDP traffic, saturated Braille in both directions, every exact palette index, cell bounds/default gaps/no LF/CR, redirected stderr, q cleanup and OS restoration. Matrix: 60×16 with LANG/LC_ALL/NO_COLOR absent; 60×16 with C and NO_COLOR=1; 80×24 with C.UTF-8 and empty NO_COLOR; 80×24 with unrecognized locale and NO_COLOR=1. Baseline has no palette codes. No exact live rate/sample count is an oracle. |
| T01 | `rejected_term_and_independent_nonterminal_streams_do_not_mutate`: unset/empty/dumb/linux TERM and independently nonterminal stdin/stdout rejected without mutation; help/version exempt with rejected TERM and redirected input; version uses actual Cargo metadata. Deterministic companion: `terminal::session::tests::support_rejects_each_stream_and_term_before_snapshot_or_mutation`. |
| Protocol/V01/V06 | `decoder_counts_styled_braille_as_cells_and_preserves_default_text`; `decoder_rejects_unknown_protocol_style_leaks_and_cell_overflow`: UTF-8 cell accounting and all allowed palette indices; rejects arbitrary Unicode/U+2800, controls, backgrounds, unknown SGR/commands, styled text/spaces, unstyled Braille and missing row resets. |
| V05/T03/T09 | `small_startup_resize_sampling_navigation_and_quit_keep_bounded_rows`: 59×16/60×15/1×1 startup, continued acquisition/navigation, 60×16/80×24 growth and 1×1 shrink, quit and saved-state/protocol restoration. |
| T02/T09 | `normal_controls_restore_real_modes_shared_flags_and_protocol`: raw mode, Ctrl+Z ignored, q/Q/Ctrl+C/SIGINT/SIGTERM status and CAN/reset/show/leave ordering; `shared_terminal_description_and_existing_nonblocking_flags_are_preserved`: shared/existing OFD flags restored. |
| T03/T04 | `fragmented_repeated_arrows_and_ignored_sequences_reach_expected_selection`: CSI/SS3 fragments/repeats, ignored modified/unknown sequences and aliases, prompt selection and quit cleanup. |
| T06/T09 | `queued_external_interrupt_and_terminate_choose_interrupt`: pending INT wins, status 130 and restoration. |
| T10/T13 | `read_only_terminal_output_fails_setup_and_restores_changed_os_state`; `flow_stopped_output_during_setup_restores_os_state_despite_cleanup_failure`: practical setup/output failures, every applicable saved OS state restored with bounded failure/diagnostics. |
| T11/T13 | `runtime_input_and_output_hangups_fail_and_attempt_remaining_restoration`; `canonical_eof_on_real_input_is_fatal_and_restores_the_saved_session`: real EOF/hangup faults, status 1 and applicable remaining restoration. |
| T07/T08/T09/T13 | `backpressured_runtime_still_handles_quit_and_signal_with_bounded_cleanup`: real blocked/partial output, quit/INT/TERM responsiveness, bounded cleanup-only failure/status preservation and saved OS state restoration. |

The inherited deterministic application/runtime/session tests also all passed.
Expanded every-byte **colored** resumption, scripted fault/panic coverage and
their detailed T08–T13 audit belong to [Verify styled output under backpressure and interrupted exits](issues/03-styled-output-backpressure-exits.md);
this record does not claim that later verification ticket is complete.

## Criterion / implementation review

- Criteria 1–4: fitting chronological half-cell suffix, independently clamped
  unrounded quantization, Unicode dot numbering and exact positional palettes
  pass the exact tests above. Zero masks encode ASCII spaces, never U+2800.
- Criteria 5–7: existing geometry/status/text/value rules pass through the frame
  seam, with default-style gaps, text and all warning paths.
- Criterion 8: all twelve inherited functions migrated by coverage; numeric
  formatting now also crosses the frame seam. V01–V08 and affected S cases are
  mapped above, replacing byte-offset/ASCII-only assumptions with cell assertions.
- Criterion 9: monitor production code and 500-ms cadence are unchanged;
  monitor alone expires/resets five-minute history. Screen uses
  `VecDeque::range(len−fitted..)` to visit only at most 2W samples, bounded rows
  and constant per-cell dot work. It neither clones complete histories nor reads
  other interfaces. README/module docs explain nominal one second per cell and
  observation-based spacing during delays.
- Criterion 10: README/module support declaration and T01/PTYs pass; UTF-8,
  Braille coverage and xterm-256color are assumptions. Existing rejection,
  redirected-stderr and informational exemptions remain intact. Always-colored
  full-view behavior is verified under all four environment combinations.
- Criterion 11: no manifest/lockfile change or new dependency; Unicode mapping
  was independently authored from standard dot numbering, with no upstream
  source copied. All required gates and release PTYs passed.

Loaded `code-review` and reviewed the working diff against pinned `b784de4`
along Standards and Spec axes. No delegation tool was available, so review was
same-agent rather than independent sub-agent review. Repository standards are
AGENTS.md plus tracker/domain guidance and neighboring Rust/licensing conventions.
Standards: no unresolved violations or actionable smell findings. Spec: all
eleven ticket criteria accounted for, no unresolved omissions or scope creep.
Historical parent specs/maps and actual version were preserved. Status, diff and
recent log were inspected before staging intended files.

No blocker remains for ticket 02. Human palette/font/statistics-contrast and
handled-exit visual observations, plus the dedicated live-smoke record, remain
assigned to [Record emulator appearance and nonprivileged live traffic](issues/04-emulator-live-traffic.md). Final combined
acceptance/handoff remains [Complete styled-graph acceptance and delivery handoff](issues/05-acceptance-delivery-handoff.md).

## Review correctness follow-up — 2026-10-08

The preceding gates/artifact describe the original implementation. Review of
`53f1ab7..a2214c6` found that adding 0.5 before flooring could promote the f64
immediately below a half step. Source correction
`5e4661164f7afda85c266b2a45ce730276c71bad` uses positive `round()` after the
prescribed normalization and scaling, without additive promotion or tolerance.
The closed correctness ticket remains closed with this defect fixed.

Additional exact V02 frame-interface mappings:

- `screen::tests::adjacent_rates_at_half_steps_do_not_gain_a_dot_from_additive_rounding`:
  adjacent f64 below/at/above first, second, fifth and twentieth half steps,
  both directions at H=10/default c=1310720 and H=6/c=128; literal whole-column
  Unicode oracles. The reviewed first-threshold predecessor failed before the
  fix and passed afterward.
- `screen::tests::intermediate_normalization_ties_use_the_normalized_rate_not_a_rate_threshold`:
  adjacent inputs that normalize/scale onto the same halfway decision at H=6
  (including the default ceiling's first step) and H=10. Preserves `r/c` then
  multiplication, rather than replacing it with a rate-threshold algorithm.

All 22 screen tests and final gates passed: 115 unit + 16 debug PTY/decoder,
release build and 16 release-artifact PTY tests. Fresh ordinary loopback smoke at
both required sizes also passed. See [Review fixes — Standards and Spec evidence](review-fixes-evidence.md)
for exact commands, red/green outcomes, normalized floating-point rationale,
source/artifact identity and current human blockers. Earlier artifact hashes and
test counts above retain their historical provenance.
