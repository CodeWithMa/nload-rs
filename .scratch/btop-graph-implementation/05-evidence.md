# Ticket 05 — final styled graph acceptance evidence

Recorded 2026-10-08 by implementer-05 (OpenCode), branch `implement/btop-05`,
worktree `/tmp/opencode/nload-btop-05`. **Final acceptance passed.**
Ticket: [Complete styled-graph acceptance and delivery handoff](issues/05-acceptance-delivery-handoff.md).
Concise delivery: [Verified styled graph implementation handoff](handoff.md).

## Authority and supersession

The [styled graph specification](../btop-graph-spec/spec.md) and
[canonical resolution](../btop-graph-spec/issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract)
govern the new presentation/frame/output contract. The
[minimal Rust port specification](../nload-rust-port/spec.md) supplies unaffected
application requirements and D/P/E/L/B evidence rules. Research/prototype algorithms
are context, not normative alternatives. AGENTS.md, tracker/domain guidance and
GLOSSARY.md were read; no ADR or additional coding-standard files were present.
All implementation tickets 01–04 and their evidence, review fixes and actual human
observations were audited. Source planning/specification parents/maps retain their
historical content and scope.

| Historical promise | Governing final contract / verification |
| --- | --- |
| Port §4.5/§4.6 and S03/S07/S09/S11/S12 “column” | One actual sample per accepted observation remains; newest 2W samples occupy chronological half-cells, not one sample per terminal cell. Monitor alone expires/resets five-minute history. Affected S mappings below verify counts independently of display capacity. |
| §5.1/§5.2 ASCII rows and 0.30/0.70/1 bars; §11 ASCII departure | Opaque dimension-owning frames, cell-bounded text/Braille construction, normalized nearest-dot halfway-up quantization and standard Unicode masks. Adjacent-f64 review regression is fixed without additive promotion or fuzzy tolerance. |
| §5.3–§5.5 and V01–V08 presentation | Existing geometry, states, labels, default text, units/rounding and overflow remain; paired glyphs/positional palettes replace ASCII appearance. All twelve original screen functions migrated by coverage, not snapshot replacement alone; current 22 screen tests pass. |
| §6.1 terminal support / T01 | Existing stream/TERM rejection and help/version exemption remain. Accepted TERM declares assumed UTF-8/Braille/xterm-256color support; always-colored full view, no locale/NO_COLOR guard, probe or fallback. |
| §6.3 output, §7.2 restoration, §8 ownership/ASCII-row seam; T08–T13 | Screen owns glyph/palette/cell/row encoding and resets. Terminal owns default clear/position/byte transport using frame-owned dimensions; partial UTF-8/SGR writes resume exactly. Staged explicit/Drop cleanup cancels/reset before show/leave, attempts every applicable step and preserves status arbitration. |
| A01/D03 initial 0.1.0 literals | The port explicitly permits future Cargo-authoritative versions. Existing `4d4e8ac` established 0.1.1; this ticket introduces no version change. |

The old port specification and historical ASCII verification are not rewritten to
claim styled output. Updated README/current effort links select applicable evidence;
dated original runs, source revisions, hashes and then-pending observations remain
available as provenance.

## Revision and artifact applicability

- Before work: `git status --short --branch` showed clean `implement/btop-05`.
  HEAD and integration both were **`66cd1369a36bd039b6a8ce9d7e2b815ce8146c6c`**;
  integration ancestry check exited 0. No reset was needed. The permanent checkout
  `/mnt/programming/rust/nload-rs` was clean on `integration/btop-styled-graphs`.
- **Final gate revision:** that full baseline, with **zero uncommitted delta** at
  gate execution. Subsequent README/evidence/tracker edits are documentation only;
  there is no uncommitted implementation/helper/test/manifest/lockfile delta.
- `git diff 5e4661164f7afda85c266b2a45ce730276c71bad HEAD -- src tests Cargo.toml Cargo.lock`
  returned no output. Thus final executable/test inputs match the corrected source
  reviewed in [Review fixes — Standards and Spec evidence](review-fixes-evidence.md).
- Final built/executed artifact:
  **`/tmp/opencode/nload-btop-05/target/release/nload-rs`**, `nload-rs 0.1.1`.
  SHA-256 **`d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`**.
- `sha256sum` returned the same full hash for that artifact and both permanent
  artifacts, `/mnt/programming/rust/nload-rs/target/release/nload-rs` and
  `/mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs`.
  `cmp --silent` of the final artifact and the durable review artifact exited 0.
  `readelf -h` identified ELF64 little-endian x86-64 PIE; actual Linux execution and
  release PTYs passed. Build products remain ignored; no executable is committed.
- The [actual human report](04-human-observations.md) identifies that durable hash
  at clean integration checkout `3765b31`, with no executable-input diff from
  corrected `5e46611`. The corrected live/gate evidence and later
  [ticket-04 applicability assessment](04-evidence.md#current-acceptance-and-applicability--2026-10-08)
  apply to this exact final artifact. Documentation work invalidates neither.
  Live/human checks were retained, not recreated or re-labelled as fresh runs.
- This is observed identity of these actual artifacts, not a byte-identical future
  build guarantee. Revision and documentation finalization are recorded below.

## Final gates and environment

All commands ran at `/tmp/opencode/nload-btop-05`, exiting **0**. No failed/ignored
test or unrun gate is counted as passing. Cargo cache-lock waits were harmless.

| Exact command | Actual outcome |
| --- | --- |
| `cargo fmt --check` | Passed. |
| `cargo test --locked` | **115 unit/native/scripted** (0.34 s) + **16 Linux PTY/decoder** (3.06 s), zero failures/ignored/filtered tests. |
| `cargo build --release --locked` | Optimized release built, no warnings (1.63 s). |
| `NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty` | **16 passed**, zero failures/ignored/filtered tests (3.07 s). The harness used the identified release binary, not Cargo's debug product. Two of these are decoder fixtures; fourteen exercise the compiled product. |
| `./target/release/nload-rs --version` | Exactly `nload-rs 0.1.1` plus newline. |
| `cargo tree --locked --target x86_64-unknown-linux-gnu -e features` | Actual Linux feature graph reviewed below. |
| `cargo tree --locked --target all -e features` | Cross-target Windows-only edges reviewed below. |
| `cargo metadata --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu` and unfiltered metadata | Actual resolved features/licenses/targets inspected, not inferred from research. |
| `sha256sum` / `cmp --silent` / `readelf -h` | Artifact identity/applicability above passed. |

Exact toolchain/environment:

- Stable-release `rustc 1.98.1 (48a229cea 2026-09-01) (built from a source tarball)`;
  compiler commit **`48a229ceaefd4985c50990b14116b6d856af0985`**, LLVM **21.1.8**.
  Host/default build and explicitly audited target: **`x86_64-unknown-linux-gnu`**.
  Sysroot `/nix/store/53lvgpxsb3b9ghkgb4v0jra6zzlnyw8n-rustc-1.98.1`.
- `cargo 1.98.0 (797e8a9bc 2026-08-05)`, full commit
  `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`; rustfmt **1.9.0**.
  Cargo verbose environment: NixOS **26.11.0 (zokor)** [64-bit], libgit2 **1.9.4**
  (sys 0.21.0 vendored), libcurl **8.22.0** (sys 0.4.90+curl-8.21.0 system SSL
  OpenSSL **3.5.8**).
- `uname -a`: `Linux nixos-dev 7.2.9 #1-NixOS SMP PREEMPT_DYNAMIC Sat Oct 3
  10:43:16 UTC 2026 x86_64 GNU/Linux`; glibc **2.44**, Python **3.14.7**.
  UID/EUID **1000/1000**, GID **100**, namespace **`net:[4026531833]`**.
- RUSTFLAGS/CARGO_ENCODED_RUSTFLAGS/CARGO_BUILD_TARGET/CARGO_TARGET_DIR/RUSTC/
  RUSTC_WRAPPER/RUSTC_BOOTSTRAP unset. No nightly/bootstrap feature used.
- Agent stdin/stdout are nonterminals. TERM=`alacritty`, LANG=`en_US.UTF-8`,
  SSH_TTY=`/dev/pts/3`; LC_ALL/NO_COLOR/COLORTERM/TERM_PROGRAM/
  TERM_PROGRAM_VERSION/DISPLAY/WAYLAND_DISPLAY/TMUX/STY unset. These declarations
  do not constitute an agent emulator observation. PTYs allocate actual Linux
  terminal stdin/stdout, redirect stderr and declare xterm-256color except explicit
  rejection/environment cases. Human emulator identity is recorded separately.

## Acceptance matrix

**Every entry below passed in the final locked test run or the explicitly retained
applicable manual/artifact record.** D = deterministic/scripted/native, P = Linux
PTY, E = actual human emulator, L = nonprivileged live smoke, B = artifact review.
Names are exact current test names. Namespace prefixes apply to every name in their
cell; PTY names are in `tests/linux_pty.rs`. No private monitor queue is an oracle.
Counters/state/counts/timestamps/glyphs/grades/mandated text are exact; derived
rates use the inherited `1e-9 * max(1, abs(expected))` tolerance, never for glyphs.

### Inherited arguments, startup and lifecycle — A01–A12

| Case | Exact passing test(s) / artifact mapping |
| --- | --- |
| A01 (D/B) | `arguments::tests::informational_outcomes_and_cargo_text`; `application::tests::informational_and_syntax_paths_have_correct_streams_status_and_zero_effects`; PTY `rejected_term_and_independent_nonterminal_streams_do_not_mutate`; manifest/metadata and direct release version above: Cargo-derived 0.1.1 identity/newline, status 0, redirected streams, no interactive effects. |
| A02 (D) | Same argument/application tests: standalone help stdout/status 0, required invocation/flags/binary units/defaults/zero and zero effects; redirected PTY exemptions supplement. |
| A03 (D) | `arguments::tests::rejects_unknown_flags_multiple_names_and_information_combinations`; `application::tests::informational_and_syntax_paths_have_correct_streams_status_and_zero_effects`: unknown/short aliases/multiple names/combinations, status 2/stderr before all interactive work. |
| A04 (D) | `arguments::tests::preserves_os_name_bytes_and_option_terminator`; `proc_sampler::tests::startup_sorts_all_names_and_selects_exact_original_bytes`, `os_invocation_to_startup_preserves_identity_and_full_membership`; `application::tests::raw_selection_and_ceiling_options_reach_the_fixed_monitor`, `startup_reuses_discovery_and_orders_clock_before_terminal_mutation`: exact bytes/leading dash/full list; missing/case mismatch silently select first with empty stderr. |
| A05 (D) | `arguments::tests::independent_defaults_overrides_repetitions_and_zero`; `application::tests::raw_selection_and_ceiling_options_reach_the_fixed_monitor`; `monitor::tests::navigation_wraps_symmetrically_preserving_views_and_global_ceilings`: independent defaults/overrides, placement, last wins/leading zeros/direction-local zero, every interface. |
| A06 (D) | `arguments::tests::ceiling_range_and_whole_ascii_digit_grammar`; both application informational/syntax and raw-selection tests above: exact minimum/maximum conversion, rejected malformed/overflow/attached forms status 2 and no effects. |
| A07 (D/B) | `arguments::tests::independent_defaults_overrides_repetitions_and_zero`; `application::tests::startup_reuses_discovery_and_orders_clock_before_terminal_mutation`. These final fixture passes plus source audit of arguments/startup show no HOME/config I/O. Historical set/unset-HOME and locale runs are retained in [original verification](../nload-rust-implementation/verification.md#application-integration-run--2026-10-02); they are not claimed as fresh variant runs. Those input/config/default paths are unchanged. |
| A08 (D) | `application::tests::startup_errors_stop_before_terminal_validation_and_remain_distinct`, `startup_reuses_discovery_and_orders_clock_before_terminal_mutation`; `proc_sampler::tests::failed_acquisition_is_global_not_absence_and_retries_cleanly`: distinct empty/enumeration failures status 1 before mutation; successful missing-name fallback is silent UI startup. Earlier missing-name status-2 fixture descriptions are superseded. |
| A09 (D) | `proc_sampler::tests::startup_sorts_all_names_and_selects_exact_original_bytes`, `natural_order_handles_unbounded_runs_and_full_byte_ties`; application startup-reuse test: natural digit order, full raw-byte ties, huge runs, all names/default first. Comparator has no locale dependency; old locale runs remain historical supplements. |
| A10 (D) | `application::tests::startup_reuses_discovery_and_orders_clock_before_terminal_mutation`, `slow_initialization_and_slow_acquisition_skip_slots_use_completion_and_actual_time`; `terminal::session::tests::ownership_snapshot_and_notifications_precede_every_mutation`: ordered initialization/discovery/completion/support/ownership/mutation, single discovery baseline, anchored deadline and one overdue acquisition. |
| A11 (D/P/E) | `monitor::tests::navigation_wraps_symmetrically_preserving_views_and_global_ceilings`; `application::tests::navigation_resize_and_parser_timeout_redraw_without_acquisition`; `terminal::runtime::tests::every_csi_and_ss3_fragment_boundary_yields_one_ordered_arrow`, `unknown_modified_and_string_sequences_consume_command_looking_bytes`; PTY `fragmented_repeated_arrows_and_ignored_sequences_reach_expected_selection`; human checklist/report confirms prompt navigation. |
| A12 (D) | `proc_sampler::tests::retained_name_rounds_ignore_additions_and_recognize_raw_name_return`; `monitor::tests::snapshot_lifecycle_retains_raw_membership_selection_and_retries_all_names`; `application::tests::runtime_failure_absence_addition_and_return_are_quiet_nonfatal_fixed_membership`: fixed names, removal/rename/new names ignored, unavailable selection preserved/retried, return baseline without bridged gap. |

### Acquisition, statistics and affected history — S01–S13

| Case | Exact passing test(s) / inherited versus new coverage |
| --- | --- |
| S01 (D) | `proc_sampler::tests::snapshot_preserves_padding_raw_names_and_exact_paired_fields`, `acquisition_reads_through_eof_and_opens_fresh_each_round`: raw padded names, exact RX1/TX9 zero/max-u64, >128-KiB seven-byte chunks, EOF/interruption and fresh reopen. |
| S02 (D) | `proc_sampler::tests::rejects_malformed_headers_rows_fields_overflow_and_duplicates`, `failed_acquisition_is_global_not_absence_and_retries_cleanly`; `monitor::tests::absence_and_read_failure_hide_totals_and_never_bridge_return_gaps`; `application::tests::runtime_failure_absence_addition_and_return_are_quiet_nonfatal_fixed_membership`: no salvage/zero fabrication, global read-failed distinct from absence, paired reset/quiet nonfatal retry. |
| **S03** (D) | `monitor::tests::warm_up_exposes_exact_totals_and_independent_directional_rates`: baseline totals/no sample, 200→400 independent rates/extrema/timestamps and one/two actual samples. `screen::tests::paired_saturated_halves_and_real_zeros_use_direction_foregrounds` verifies the new pairing, not new sample creation. |
| S04 (D) | `monitor::tests::partial_current_window_weights_only_interval_overlap`: current 350/average 320, independent TX with actual overlaps. |
| S05 (D) | `monitor::tests::exact_and_partial_average_boundaries_preserve_crossing_intervals`: exact 302-second example and partial-average 303, crossing-interval retention. |
| S06 (D/B) | `monitor::tests::nonpositive_times_ignore_entire_outcomes_without_advancing_any_view`, `delayed_and_suspend_length_intervals_use_actual_elapsed_time_once`; application slow-initialization/acquisition test; native `terminal::linux::runtime::tests::owned_boottime_timer_is_absolute_nonblocking_and_consumes_due_zero_deadline`. Production CLOCK_BOOTTIME/syscall audit excludes wall time for statistics; elapsed suspend-equivalent fixtures and shorter warm-up remain unpadded. |
| **S07** (D) | `monitor::tests::delayed_and_suspend_length_intervals_use_actual_elapsed_time_once`; `application::tests::slow_initialization_and_slow_acquisition_skip_slots_use_completion_and_actual_time`, `deadline_overflow_is_fatal_before_mutation_or_after_one_delayed_acquisition`; `screen::tests::fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view`: 3000/3s = 1000, one actual sample/acquisition and first future slot; delayed samples now pair. Timer expiration counts are wake information only (native timer test/source review). |
| S08 (D) | `monitor::tests::exact_small_deltas_and_nanosecond_differences_survive_large_values`, `full_u64_interval_delta_is_checked_before_derived_conversion`, `unrounded_run_extrema_survive_average_expiry_and_measured_idle`, `navigation_wraps_symmetrically_preserving_views_and_global_ceilings`: exact deltas/endpoints, fractional/unrounded independent extrema, whole-run retention/navigation and measured zero minimum. |
| **S09** (D) | `monitor::tests::decrease_in_either_counter_resets_both_directions_to_new_baseline`; `screen::tests::baseline_absence_failure_idle_and_recovery_have_exact_rows`: RX/TX decrease resets both, total 10000/new baseline without samples; next RX 1000 and one actual sample, paired rendering never preserves stale history. |
| S10 (D) | `monitor::tests::absence_and_read_failure_hide_totals_and_never_bridge_return_gaps`, `initial_unavailable_outcomes_and_reason_changes_remain_explicit`; application lifecycle/nonfatal test; screen state test: both reset, no stale totals or gap rate, return baseline then fresh measured run. |
| **S11** (D) | `monitor::tests::never_selected_interfaces_accumulate_real_history_at_every_round`; `screen::tests::sampling_and_navigation_while_small_reappear_as_actual_history_on_growth`; `application::tests::small_session_keeps_offscreen_sampling_and_recovers_real_history_on_growth`: unseen interfaces accumulate real samples independently; navigation adds none, growth displays `(blank,a)(b,c)`. |
| **S12** (D) | `monitor::tests::history_expiry_retains_exact_boundary_and_reads_only_hide_fitting_suffixes`; `screen::tests::fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view`, `fitting_suffix_displays_sixty_or_one_hundred_of_actual_retained_observations`; application redraw-without-acquisition test: exact t−300 retained, older expired only on observation; 2W fitting suffix/capacities 60/100; resize/redraw preserve full view and actual counts. |
| S13 (D/B) | `terminal::linux::runtime::tests::checked_timespec_conversion_rejects_invalid_fields_and_overflow`, `owned_boottime_timer_is_absolute_nonblocking_and_consumes_due_zero_deadline`, `timer_effect_errors_and_incomplete_reads_are_fatal`; `terminal::runtime::tests::clock_failure_regression_and_idle_deadline_overflow_have_no_fallback`; application startup/slow/overflow/runtime-fault tests: bulk proc once, completion timestamp, owned BOOTTIME/timerfd, checked integer conversion and fatal clocks/no fallback. Concrete adapter/unsafe review below. |

### Styled screen and twelve-function migration — V01–V08

All names here are in `screen::tests`. Each observes screen construction/read-only
frame dimensions/rows with a test-side permitted-protocol whitelist and terminal
cell counting, not byte-offset graph assertions or private quantization helpers.
[Ticket-02 migration detail](02-evidence.md#exact-v01v08-mappings-and-twelve-function-migration)
accounts for each of the twelve original functions and additional frame/glyph cases;
[review regression detail](review-fixes-evidence.md#meaningful-test-first-execution-at-the-agreed-frame-seam)
adds two exact current V02 tests. All 22 pass in the final 115-test suite.

| Case | Exact passing tests / applicable manual evidence |
| --- | --- |
| V01 (D/E) | `exact_80_by_24_stacked_rows_and_bottom_aligned_fields`, `minimum_and_odd_geometry_keep_gap_fields_and_outgoing_extra_row`, `positional_palettes_use_each_direction_height_and_leave_text_default`, `opaque_frame_retains_dimensions_and_encodes_trimmed_default_rows`, `fitting_suffix_displays_sixty_or_one_hundred_of_actual_retained_observations`: 80×24, 60×16, odd heights/widths, exact cells/layout/28-column fields/two-cell gaps, all twelve indices and 6/7/10/11-row grades. Human Run A/B confirms stacked appearance/default contrast at both exact full sizes. |
| V02 (D) | `nearest_dot_half_steps_are_exact_bottom_up_without_minimum_or_tolerance`, `adjacent_rates_at_half_steps_do_not_gain_a_dot_from_additive_rounding`, `intermediate_normalization_ties_use_the_normalized_rate_not_a_rate_threshold`, `standard_braille_masks_cover_every_bottom_filled_half_combination`, `independent_half_and_direction_quantization_preserves_uncapped_text`, `odd_graph_heights_quantize_half_ceiling_and_first_half_dot_independently`, `paired_saturated_halves_and_real_zeros_use_direction_foregrounds`, `independent_ceiling_labels_and_numeric_prefixes_do_not_cap_rates`: exact half steps/adjacent f64, standard 25 masks, zero/tiny positives blank, odd heights, independent half/direction saturation and actual uncapped text. |
| V03 (D) | `paired_saturated_halves_and_real_zeros_use_direction_foregrounds`, `fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view`, `fitting_suffix_displays_sixty_or_one_hundred_of_actual_retained_observations`, `sampling_and_navigation_while_small_reappear_as_actual_history_on_growth`, `measured_idle_shifts_older_traffic_by_real_blank_half_cells`: newest 2W suffix, one/odd/even right alignment/no duplicate, real zero/delayed positions, half-cell shifts and unchanged history across redraw/shrink/grow. |
| V04 (D/E/L) | `baseline_absence_failure_idle_and_recovery_have_exact_rows`, `measured_idle_shifts_older_traffic_by_real_blank_half_cells`: exact baseline/absent/read-failed/recovery status/totals/placeholders and empty graphs, measured zero/normal separator with older traffic retained. Human/live idle blank positions supplement; unavailable states remain scripted rather than falsely claimed live observations. |
| V05 (D/P/E) | `small_and_zero_dimensions_show_only_bounded_warning_selection_status`, `sampling_and_navigation_while_small_reappear_as_actual_history_on_growth`, `fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view`; `application::tests::small_session_keeps_offscreen_sampling_and_recovers_real_history_on_growth`; PTY `small_startup_resize_sampling_navigation_and_quit_keep_bounded_rows`, `colored_backpressure_resumes_a_complete_large_frame_then_recovers_latest_resize`: exact 59×16/60×15/60×16, zero/extreme bounds, no palette warning, continued sampling/navigation/quit and real half-history growth. Human Run C starts small/grows; Run A exits at actual 56×17 below width floor. |
| V06 (D) | `raw_names_are_ascii_escaped_shortened_and_keep_position`, `largest_exact_totals_and_ceilings_and_overwide_rates_keep_labels`, `positional_palettes_use_each_direction_height_and_leave_text_default`: raw NUL/ESC/control/non-ASCII becomes safe printable `\xHH`, complete escapes/ellipsis/position, original identity preserved, `[too wide]` preserves labels, graph-adjacent text/spaces default and no raw ESC injection. |
| V07 (D) | `rate_units_choose_prefix_before_explicit_halfway_up_rounding`, `total_units_round_exact_widened_integers_including_maximum_u64`, `independent_ceiling_labels_and_numeric_prefixes_do_not_cap_rates`: all inherited exact examples, prefix-before-rounding boundaries, independent fields/capped G, explicit halfway-up decimals separate from graph quantization. |
| V08 (D) | `total_units_round_exact_widened_integers_including_maximum_u64`, `largest_exact_totals_and_ceilings_and_overwide_rates_keep_labels`, `independent_ceiling_labels_and_numeric_prefixes_do_not_cap_rates`, `independent_half_and_direction_quantization_preserves_uncapped_text`: exact maximum-u64 totals and largest ceilings with widened accounting/labels, independent minima/ceilings and saturation. |

### Terminal, scheduling and exhaustive styled restoration — T01–T14

The [full deterministic transport/restoration mappings](03-evidence.md#deterministic-case-to-test-mappings--actual-passing-outcomes)
and [all sixteen PTY mappings](03-evidence.md#linux-pty-mappings--debug-and-identified-release-executable)
remain applicable to final unchanged tests. The following is the consolidated case
index, including unaffected T02–T07 and all expanded T08–T13 obligations.

| Case | Exact passing tests / applicable manual evidence |
| --- | --- |
| T01 (D/P/B) | `terminal::session::tests::support_rejects_each_stream_and_term_before_snapshot_or_mutation`; application informational/syntax and unsupported-session tests; PTY `rejected_term_and_independent_nonterminal_streams_do_not_mutate`, `accepted_full_views_emit_colored_braille_under_no_color_and_locale_declarations`: independent streams/TERM rejected before mutation, redirected stderr and informational exemption, accepted full view colored at both sizes under absent/C/C.UTF-8/unrecognized locale and absent/empty/nonempty NO_COLOR. README/module docs declare support assumptions, not successful probing. |
| T02 (D/P) | `terminal::session::tests::ownership_snapshot_and_notifications_precede_every_mutation`, `existing_nonblocking_flags_are_preserved_without_claiming_changes`; `terminal::linux::tests::linux_adapter_raw_flags_and_dimensions_round_trip_on_owned_pty`; `terminal::runtime::tests::typed_interrupt_does_not_wait_behind_an_incomplete_escape_sequence`; PTY `normal_controls_restore_real_modes_shared_flags_and_protocol`, `shared_terminal_description_and_existing_nonblocking_flags_are_preserved`: full raw/Ctrl+C byte/Ctrl+Z ignored, saved shared/OFD flags, notification ownership before mutation. |
| T03 (D/P) | `terminal::runtime::tests::every_csi_and_ss3_fragment_boundary_yields_one_ordered_arrow`; application navigation/redraw test; PTY `fragmented_repeated_arrows_and_ignored_sequences_reach_expected_selection`: all forms/fragments/repeats, one event/order and prompt nonblocking navigation. |
| T04 (D/P) | `terminal::runtime::tests::unknown_modified_and_string_sequences_consume_command_looking_bytes`, `typed_interrupt_does_not_wait_behind_an_incomplete_escape_sequence`; same fragmented-input PTY plus normal-controls PTY: unknown/modified/aliases/string suffixes/Ctrl+Z ignored, plain q/Q and Ctrl+C retained. |
| T05 (D) | `terminal::runtime::tests::idle_timeout_is_a_wait_deadline_and_resets_at_exact_boundary`, `sustained_input_and_overlong_sequences_are_bounded_and_discarded`; application redraw-without-acquisition test: 100-ms idle deadline, no blocking fragment reads/acquisitions, overlong input consumed. Decoder has constant-size state/count rather than a stored byte buffer, satisfying the 64-byte bound. |
| T06 (D/P/B) | `terminal::linux::tests::full_pipe_and_collated_tokens_do_not_replace_atomic_signal_identity`, `registered_signals_wake_and_unregister_without_owned_fd_leaks`; `terminal::linux::runtime::tests::real_nonblocking_backpressure_does_not_hide_input_signal_or_timer_readiness`; application shutdown tests; PTY `queued_external_interrupt_and_terminate_choose_interrupt` and normal/signal exits: drain-before-flags, handler atomics/wake only, idle/backpressure responsiveness and INT wins simultaneous pending flags. |
| T07 (D/P) | `application::tests::sustained_bounded_input_rechecks_due_acquisition_before_next_batch`, `shutdown_preempts_due_sampling_input_resize_and_rendering`, `shutdown_arriving_during_acquisition_stops_before_input_or_frame_work`, `blocked_output_still_handles_shutdown_without_an_extra_sample_or_write`, `colored_backpressure_coalesces_navigation_and_resize_without_stale_frames_or_samples`: bounded ordered batches, due acquisition before/among reads, dirty/latest redraw coalescing, no starvation/redraw samples, shutdown preempts work. Colored-backpressure and blocked-quit/signal PTYs supplement real effects. |
| **T08** (D/P) | `terminal::runtime::tests::every_colored_frame_byte_boundary_resumes_exactly_before_the_latest_frame`, `untouched_colored_frames_replace_after_transients_and_large_frames_yield_each_batch`, `untouched_frames_replace_but_partial_frames_finish_without_duplicate_bytes`, `every_prefactor_protocol_byte_boundary_resumes_exactly_without_interleaving`, `output_batches_are_finite_for_screen_constructed_frames`, `backpressure_keeps_clock_input_signals_resize_and_timer_effects_available`; application colored coalescing and inherited backpressure tests; PTY `colored_backpressure_resumes_a_complete_large_frame_then_recovers_latest_resize`: every colored UTF-8/SGR byte split with Interrupted/WouldBlock/short writes, untouched-only replacement, exact resumed suffix then latest state, <=16-KiB writes and input/signals/acquisitions continue. |
| **T09** (D/P/E) | `application::tests::handled_exits_after_every_colored_output_prefix_cancel_reset_and_restore`; `terminal::session::tests::explicit_and_drop_cleanup_cancel_partial_sgr_and_reset_before_screen_restoration`; PTY `colored_normal_signal_and_runtime_eof_exits_restore_os_state_and_protocol`, normal-controls/shared-flags/queued-signals fixtures: q/Q/Ctrl+C/INT/TERM statuses 0/0/130/130/143, every colored prefix cleanup, saved OS state and exact CAN/reset/show/leave. Human separate q/Ctrl+C/TERM confirms visible restoration and actual 0/130/143. |
| **T10** (D/P) | `terminal::session::tests::every_initialization_failure_restores_only_applicable_attempted_effects`, `every_partial_initialization_sequence_failure_or_panic_resets_applicable_stages`, `partial_screen_commands_are_recorded_even_when_the_remainder_fails`, `interrupted_and_short_setup_writes_resume_without_repeating_emitted_bytes`, `zero_write_is_fatal_and_still_restores_the_attempted_screen_change`; application guarded initialization/panic test; PTY `read_only_terminal_output_fails_setup_and_restores_changed_os_state`, `flow_stopped_output_during_setup_restores_os_state_despite_cleanup_failure`: every stage and partial-prefix reset/applicable attempts, status 1 after attempts; practical native setup faults. |
| **T11** (D/P) | `application::tests::colored_runtime_faults_and_panics_restore_before_diagnostics_and_prior_hook_probe`, `actual_loop_runtime_faults_restore_before_reporting`; `terminal::runtime::tests::input_eof_and_effect_faults_are_fatal_while_transient_io_yields`, `clock_failure_regression_and_idle_deadline_overflow_have_no_fallback`; native poll/timespec/timer error tests; PTY `colored_normal_signal_and_runtime_eof_exits_restore_os_state_and_protocol`, `canonical_eof_on_real_input_is_fatal_and_restores_the_saved_session`, `runtime_input_and_output_hangups_fail_and_attempt_remaining_restoration`: EOF/IO/clock/poll/resize/signal faults fatal 1 with cancel/reset and all attempts before diagnostic; counter failures stay nonfatal. |
| **T12** (D/B) | `application::tests::initialization_and_caught_startup_runtime_panics_use_the_guarded_application_path`, `colored_runtime_faults_and_panics_restore_before_diagnostics_and_prior_hook_probe`; `terminal::session::tests::caught_initialization_runtime_and_cleanup_panics_restore_the_prior_hook`, `drop_fallback_restores_an_owner_without_explicit_cleanup`, `drop_during_unwinding_survives_an_additional_restoration_panic` plus every-stage/prefix panic tests: caught 1, quiet hook during execution/restoration, previous hook restored before stderr, cancel/reset and nonpanicking Drop. Main returns ExitCode; no early process exit. |
| **T13** (D/P) | `application::tests::colored_exit_cleanup_errors_and_panics_preserve_status_and_attempt_all_stages`; `terminal::session::tests::each_cleanup_error_or_panic_keeps_remaining_stages_and_selected_status`, `multiple_cleanup_failures_preserve_selected_status_and_attempt_every_operation`, `every_cleanup_sequence_split_resumes_or_cancels_before_remaining_restorations`, `cleanup_backpressure_is_bounded_and_does_not_skip_os_restoration`, `interrupted_cleanup_wait_resumes_within_the_same_bounded_write`: each/all seven effects incl. reset error/panic, every CAN/reset/show/leave split, bounded cleanup, quit becomes 1 while 130/143/1 retained, diagnostics after attempts. PTY flow-stopped/hangup and `backpressured_runtime_still_handles_quit_and_signal_with_bounded_cleanup` prove practical attempts/OS restoration and failure status, not visible reset on unwritable output. |
| **T14** (E) | [Human report](04-human-observations.md#follow-up-confirmation) and [completed ticket-04 assessment](04-evidence.md#human-observations-and-evidence-limits): Alacritty **0.17.0 (94e7c887)**, both exact full sizes, clean single-cell Braille, both positional palettes/contrast/default statistics, true-blank idle, prompt navigation, small startup/shrink/grow/history and default warning, separate q/Ctrl+C/TERM visible cursor/default foreground/background/normal editing/prior screen after actual 0/130/143. Configured font/client OS/theme unavailable, not fabricated. |

### Delivery — D01–D05

| Case | Actual passing artifact/manual record |
| --- | --- |
| D01 (B) | Final exact gates/compiler/target/environment/artifact above; committed lockfile (latest lockfile/version commit `4d4e8ac`); stable edition-2024 build. |
| D02 (B) | Concrete module/frame/unsafe/target/features/license review below, final D/P passes and inherited narrow scripted effect seams. No new dependency or backend abstraction. |
| D03 (B) | Manifest/metadata single binary nload-rs/0.1.1, Cargo version authority; GPL-2.0-only metadata/source/README, full GPL v2 text, exact nload credit, notices, committed lockfile and current build/direct-run documentation. |
| D04 (L) | [Corrected-artifact fresh loopback smoke](review-fixes-evidence.md#fresh-ordinary-loopback-live-smoke-on-corrected-release) plus [human ordinary-emulator completion](04-evidence.md#completed-criterion-and-case-mappings): UID/EUID 1000, proc discovery docker0/enp1s0/lo, default and explicit selection/all arrows, independent 2/4 MBit/s labels, ordinary low/high/idle UDP twice at 60×16/80×24, advancing totals/nonzero graphs/numeric saturation/blank idle, q 0 and OS/protocol restoration. Actual corrected runs each received 634 datagrams over 12.002 s, RX/TX deltas 5,211,480 each; exact live rates/counts are not a numerical oracle. Human observation is qualitative and no fresh run is claimed by ticket 05. |
| D05 (B) | Authority/supersession table above and module/inherited source comparison below. nload v0.7.4 reference remains context; retained stacked fields/units/default ceilings/cadence and accepted startup/natural-order/BOOTTIME/history/failure departures are explicit in the inherited spec. Braille/palettes/frame/reset replace affected ASCII promises under the newer contract; btop biased rounding/table/prototype duplication are not normative. |

### Practical evidence boundaries

PTY streams prove Linux protocol/state, not actual font/palette quality or shell
screen appearance. T14/V05-E/D04 ordinary-emulator evidence is supplied by the human
report on this exact hash. Human Run A small exit was **56×17** rather than suggested
59×16; Run C starts small/grows without an exact small-size transcript. These are
honestly reported visual checks; named fixtures establish exact floor boundaries,
history counts/order and baseline/absent/read-failed/recovered states. Configured
font name/size/fallback, client OS/theme were unavailable, while actual glyph
coverage/contrast/restoration were explicitly confirmed as **“All observed; font
unknown”**. No missing visual behavior is inferred from environment variables.

Exact every-byte splits, chosen clock faults, caught Rust panics and arbitrary
independent setup/restoration errors cannot reliably be forced by PTYs; the mapped
scripted/application/native matrices cover them under the agreed practicality
rule. Disconnected input may make termios unrestorable (diagnosed); blocked output
may prevent reset/show/leave reaching the terminal (diagnosed). Every applicable
attempt and healthy saved-state restoration is checked. Cross-stream ordering is
proved by the scripted diagnostic writer/hook probe, not reconstructed from two
captured PTY/stderr streams. No harness limit hides an unresolved required case.

## Module, cost and delivery review

### Deep boundaries and rendering cost (B)

Inspected [screen](../../src/screen.rs), [monitor](../../src/monitor.rs),
[application](../../src/application.rs), [runtime](../../src/terminal/runtime.rs),
[session](../../src/terminal/session.rs), [Linux adapter](../../src/terminal/linux.rs),
[clock/runtime adapter](../../src/terminal/linux/runtime.rs),
[arguments](../../src/arguments.rs), [proc sampler](../../src/proc_sampler.rs) and
[main](../../src/main.rs), alongside the named tests.

- `screen::Frame` has private width/height/rows, read-only dimensions/encoded rows,
  and construction only through screen render. No external encoded-string builder,
  mismatched dimensions, generic styled-cell API or arbitrary ANSI input.
- Screen builds exactly H logical rows of W cells, clips safe text at cells,
  escapes original bytes, chooses independent masks/palettes, trims blank cells
  before encoding, and emits only printable ASCII/nonblank Braille plus agreed
  foreground SGR/reset. Row start/end/statistics resets remain intact; blanks and
  text default; no positioning/clearing/session commands. Zero dimensions valid.
- Terminal accepts only frames, reads their dimensions, copies rows verbatim and
  owns default-style clear/absolute addressing/final home and one resumable pending
  byte vector. It never scans display widths or slices/trims encoded rows. Only
  untouched output replaces; started output finishes before latest state. Writes
  are <=16 KiB and reads <=256 bytes, single nonblocking operations per batch.
- Session ownership/snapshots/notifications precede mutation; attempts staged
  before effects. Explicit cleanup and Drop attempt cancel/reset, show, leave,
  saved termios, changed flags and notification release exhaustively. Each output
  restoration has 100-ms budget, errors/panics do not skip stages, status arbitration
  and quiet-hook-before-report ordering remain inherited.
- Selected views borrow just the selected interface's samples; private interval
  queues/reset bookkeeping remain hidden. Test adapters cover existing concrete
  counter/clock/terminal effects; no hypothetical backend/thread/framework/public
  library was introduced. Production unsafe is confined to concrete Linux adapter
  files and their native tests; the separate PTY harness has its own concrete OS
  calls. Pure modules/application/scripted fixtures have no unsafe.
- Graph selection uses `VecDeque::range(len-fitted..)` for at most **2W** samples
  per direction, no retained-history clone or other-interface traversal. It visits
  H rows for each fitted sample with at most four dot operations per half/row;
  logical cell allocation/encoding is O(W×H), graph work O(H×min(N,2W)), bounded by
  screen area. Palette lookup is fixed and text formatting bounded to fields/name.
  Runtime serialization scales with the encoded frame. Source inspection plus
  exact suffix/large-frame batch fixtures suffice: no timing benchmark was required
  or invented as a gate/performance claim.
- Comparing inherited baseline `53f1ab7` to final source: monitor production,
  arguments/proc, manifest/lockfile are unchanged; application behavior changes only
  from rows+separate dimensions to the frame call (and docs). 500-ms BOOTTIME
  acquisition completion deadlines, actual 1s/300s rate overlaps, exact u64 totals,
  paired reset/extrema/off-screen five-minute retention, CLI/config semantics and
  single-interface membership remain inherited and pass the final mappings.

### Actual locked target, features and licenses (B)

Both filtered/unfiltered metadata and Linux/all-target feature trees were inspected
in this session. Actual Linux graph has four third-party packages:

| Package | Version | Enabled features (metadata) | Linux dependency edges | License route / actual source notice |
| --- | --- | --- | --- | --- |
| nload-rs | 0.1.1 | none | libc, signal-hook | GPL-2.0-only; edition 2024 |
| libc | 0.2.189 | default, std | none | MIT from MIT OR Apache-2.0; `LICENSE-MIT` |
| signal-hook | 0.4.4 | **none** | libc, signal-hook-registry | MIT from MIT OR Apache-2.0; `LICENSE-MIT` |
| signal-hook-registry | 1.4.8 | none | errno, libc | MIT from MIT OR Apache-2.0; `LICENSE-MIT` |
| errno | 0.3.14 | default, std | libc | MIT from MIT OR Apache-2.0; `LICENSE-MIT` |

`Cargo.toml` pins libc `=0.2.189`, signal-hook `=0.4.4` with
`default-features = false`. The feature-tree registry “default” edge enables no
registry features (metadata confirms none). signal-hook channel/iterator/extended
siginfo/cc features and associated thread/build-tool dependencies are absent;
production uses atomic flags and Unix signal pipe registrations.

Cross-target lockfile also includes **windows-sys 0.61.2** via errno's Windows-only
edge and **windows-link 0.2.1** via windows-sys. All-target tree enables windows-sys
default/Win32_Foundation/Win32_System_Diagnostics_Debug with Win32/System/Diagnostics
parent features; windows-link has none. Neither appears in the Linux filtered
resolved graph or compiled package list. Both offer MIT OR Apache-2.0 with actual
`license-mit` notices. Linux-only delivery remains the contract; Windows lockfile
entries are dependency portability metadata, not a supported product target.

Actual downloaded notice texts for **all six** were read under
`/home/alice/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/<name>-<version>/`:
first four `LICENSE-MIT`, Windows packages `license-mit`. Compared to preserved
[third-party notices](../../THIRD-PARTY-NOTICES.md): Rust Project Developers (libc),
2017 tokio-jsonrpc developers (both signal crates), 2014 Chris Wong (errno),
Microsoft Corporation (both Windows crates), with permission/warranty text intact.
Selected MIT routes permit GPL-2.0-only combination; no Apache-only route needed.
The original notice review date is retained, and this is its actual final re-audit.

Manifest/metadata confirms nload-rs/0.1.1, one binary plus linux_pty test target,
no library, edition 2024, GPL-2.0-only. `arguments::version_text` uses
`CARGO_PKG_NAME`/`CARGO_PKG_VERSION`; no parallel version literal authority.
LICENSE contains the full GNU GPL version 2 text; the generic appendix does not
override the GPL-2.0-only metadata/README/SPDX grant. README retains exact
**“Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.”**
and independently authored btop-presentation credit, current support/build/evidence
links and opaque frame ownership. No dependency was added for Braille, cell
accounting, SGR or palettes. Implementation computes masks from standard dot
numbers independently rather than copying btop's table/biased rounding; documented
review/TDD provenance and source agree.

## Test-first applicability and final review

Loaded required `tdd`, `implement` and `code-review`. The audit found no needed
behavioral/helper/test change, so no new test seam or red/green cycle was fabricated
for documentation. Meaningful existing test-first graph/frame/cleanup regressions
from tickets 01/02 and the rounding correction remain linked and all passed here.
Inspection, actual gates, metadata/license/artifact comparison and evidence/link
audit are the appropriate verification for this final delivery slice.

Reviewed against fixed requested baseline **66cd136**, along separate Standards
and Spec axes. No delegation tool is available; this is same-agent review, not an
independent parallel-agent review.

- **Standards:** tracker resolution/named-link/gist, domain vocabulary, surrounding
  Markdown/Rust documentation and licensing conventions checked. No unresolved
  documented-standard violation or actionable heuristic smell. Exact detailed
  tests stay in evidence while the handoff is concise and linked.
- **Spec:** all eight ticket criteria mapped to the audit above; final gates,
  revision/delta, exact inherited/new cases, supersession, boundaries/cost,
  dependencies/licenses/version/credit, product docs and human/live applicability
  accounted for. No unrun evidence counted, concrete blocker or scope creep.
  Source planning/specification parent/map remains outside the delivery edits.

Documentation audit: a one-off external inspection script
`python3 /tmp/opencode/audit-nload-btop-05.py` passed **104 local file/anchor links**,
**56 fully qualified + 74 short test-name references** against the compiled
**131-test inventory**, all **52 acceptance cases**, all **eight checked ticket
criteria**, intended-documentation scope and zero executable-input delta. Listing
the compiled tests did not rerun them; this script is inspection tooling, not an
application/helper change or new behavioral test seam. `git diff --check` passed.
Reviewed status, full intended changes and recent ten commits before staging only
README and the six intended implementation-effort Markdown files. No source
planning/specification parent/map is among the edits.

Final commit/integration reconciliation is recorded below. **No acceptance blocker
remains.**

### Delivery revision and integration reconciliation

- Accepted delivery commit:
  **`7b0a6a3b7632178eec5e9ccf20839bc1c398a58f`**,
  `docs: complete styled graph acceptance and delivery handoff`.
  Seven intended Markdown files committed, 535 insertions/21 deletions. Status was
  clean immediately after that commit. All eight checklist items are checked,
  ticket 05 is closed, implementation status is complete and Decisions so far has
  its named resolution link/gist. No production/test/helper/dependency delta.
- Before that commit, `git merge --no-edit integration/btop-styled-graphs`
  returned **`Already up to date.`**; latest integration tip was
  **`66cd1369a36bd039b6a8ce9d7e2b815ce8146c6c`**, ancestry check exited 0.
  Staged full intended diff, status/recent ten commits and whitespace were inspected.
- Final revision-record preparation is based on clean accepted commit `7b0a6a3`.
  At recording time its **only uncommitted delta** is this reconciliation section
  in `05-evidence.md` and the accepted-revision pointer in `handoff.md`.
  **Uncommitted implementation delta: zero**; executable/test/manifest/lockfile
  inputs still equal corrected `5e46611`. This documentary finalization is committed
  separately as `docs: record final styled graph delivery revision`; its resulting
  commit ID is the delivery branch HEAD reported to the user. No gate or human/live
  evidence is invalidated by this delta.
- Latest-tip reconciliation before finalization again returned **`Already up to
  date.`** at the same full integration tip; ancestry passed. The documentation
  audit passed again with the same counts, and status/diff/recent log/whitespace
  checks showed only the two intended revision-record files. Standards and Spec
  review of `git diff 66cd136...HEAD` plus that documentary delta had zero unresolved
  findings on either axis.
