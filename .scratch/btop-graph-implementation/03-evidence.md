# Ticket 03 — styled transport, backpressure and exits evidence

Recorded 2026-10-08 by implementer-03 (OpenCode), in
`/tmp/opencode/nload-btop-03`, branch `implement/btop-03`.

Authority: [Verify styled output under backpressure and interrupted exits](issues/03-styled-output-backpressure-exits.md),
the [canonical styled screen/terminal and acceptance contract](../btop-graph-spec/issues/04-screen-seam-contract.md#deep-screenterminal-seam-and-encoding-guarantees),
and inherited scheduling, restoration and status rules in the
[minimal Rust port specification](../nload-rust-port/spec.md#6-terminal-support-input-and-event-scheduling).
Read [ticket 01 evidence](01-evidence.md) and [ticket 02 evidence](02-evidence.md).
The newer styled contract supersedes the historical ASCII presentation/test
promises. Historical specifications/maps were not rewritten.

## Baseline, source revision and integration

- Initial status: clean; branch: `implement/btop-03`.
- Initial HEAD, integration tip and merge-base all resolved to
  `48584d006ea3b3184f56e7bbae24f2254425e946`. No reset was needed.
- Verification source commit:
  **`81797393a2dbbf56a614ede88e3cf41ef9c77d05`**
  (`test: verify styled transport backpressure and interrupted exits`).
  Gates ran on precisely that source tree before the commit; subsequent changes
  are evidence/tracker documentation only. Production code and dependencies are
  unchanged from the integration baseline: the audit found no concrete product
  failure requiring a fix.
- With clean status after the source commit,
  `git merge integration/btop-styled-graphs` returned `Already up to date.`;
  integration was still `48584d0`, and
  `git merge-base --is-ancestor integration/btop-styled-graphs HEAD` exited 0.
  The latest integration tip is checked/merged again before reporting.

## Test-first verification and sensitivity checks

Loaded `tdd` and `implement`. The accepted ticket supplies the agreed seams:
screen construction/read-only frame rows, terminal runtime effects/output,
complete application invocation with counter/clock/terminal effects, staged
session cleanup and the compiled executable through real Linux PTYs.
No private monitor queues or arbitrary encoded-frame constructors are used.

Verification proceeded in vertical slices: colored byte transport, application
coalescing, interrupted exits, runtime faults/hook ordering, cleanup/setup faults,
then real PTY recovery/exits. Each slice's targeted suite passed before the next.
These are regression-verification additions to already implemented behavior;
their initial runs were green, not claimed as newly discovered red product bugs.
Two temporary, explicitly intentional mutations checked that the new assertions
actually reject the promised failure classes:

| Deliberate regression / exact command | Observed red | Restored green |
| --- | --- | --- |
| Replace accumulated runtime byte offset `+= n` with `= n`; `cargo test --locked every_colored_frame_byte_boundary_resumes_exactly_before_the_latest_frame` | Exit 101: frame remained pending after the expected complete suffix. | Restore original offset; `cargo test --locked colored_`: all six matching unit tests and three PTY tests passed. |
| Remove SGR reset from cleanup, leaving CAN alone; `cargo test --locked handled_exits_after_every_colored_output_prefix_cancel_reset_and_restore` | Exit 101: exact cleanup bytes lacked `ESC[0m` before show/leave. | Restore original CAN/reset; same final colored suite passed. |

Both mutations were reverted before gates/commit and have no final production
diff. No real test failure was relabelled a pass. Review bounded the large-frame
fixture's iteration count so an offset regression fails rather than hanging;
the PTY recovery audit also explicitly rejects palette codes in the small frame.

### Exact transport oracle

`terminal::runtime::tests::every_colored_frame_byte_boundary_resumes_exactly_before_the_latest_frame`
constructs an actual 60×16 screen frame from two accepted saturated observations
(24-byte/s ceilings, totals 24@1s and 48@2s). It checks a literal `⣿`/183/reset
row suffix and the presence of all twelve foreground indices. Its expected wire
payload is the public frame's encoded rows, with contract-specified literal
clear/home/addressing. This is a byte-transfer oracle, not a recomputation of
private runtime buffering. Independent exact glyph/palette oracles remain in
the screen tests mapped by [ticket 02](02-evidence.md#exact-v01v08-mappings-and-twelve-function-migration).

For **each byte offset `1..wire_length`**, independently with Interrupted and
WouldBlock, it emits the prefix, rejects replacement, yields through input,
emits a second one-byte short write and the remaining suffix, then emits an
80×24 latest frame. Exact complete-byte equality rules out repeated prefixes,
lost suffixes, interleaved frames and malformed resumed UTF-8. Offsets include
all interior bytes of every foreground SGR, every reset, every three-byte
Braille glyph and every positioning fragment.

## Final gates and release artifact

All final commands below exited 0 in this worktree:

| Command | Actual result |
| --- | --- |
| `cargo fmt --check` | Passed (after `cargo fmt`). |
| `cargo test --locked` | **113 unit + 16 Linux PTY/decoder tests passed**; zero failed or ignored. Unit suites include 22 application, 14 runtime, 20 staged-session and all 20 screen tests. |
| `cargo build --release --locked` | Passed, optimized release executable built. |
| `NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty` | **16 passed**, zero failed or ignored, explicitly using the release executable. |
| `cargo check --locked` | Passed during application/session verification. |
| `git diff --check` | Passed. |
| `./target/release/nload-rs --version` | `nload-rs 0.1.1`. |

Environment: Linux **7.2.9**, x86_64 GNU/Linux, UID **1000** (nonprivileged).
Compiler: `rustc 1.98.1 (48a229cea 2026-09-01) (built from a source tarball)`;
compiler commit `48a229ceaefd4985c50990b14116b6d856af0985`, LLVM **21.1.8**,
host/target `x86_64-unknown-linux-gnu`.
Cargo: `cargo 1.98.0 (797e8a9bc 2026-08-05)`.

Tested release artifact:
`/tmp/opencode/nload-btop-03/target/release/nload-rs`, SHA-256
**`50d9a750738bf4381eabd6fb72a96967098bdd6ba90067f3d2f255c5da176746`**.
It matches ticket 02's recorded binary hash, consistent with test-only source
changes; this identifies the tested artifact, not a general byte-reproducibility
claim. No release binary is committed.

## Deterministic case-to-test mappings — actual passing outcomes

All names below are fully qualified Rust test names. Effect call ordering is
asserted where the contract requires ordering/bounded system effects, never as
a substitute for observation/sample or emitted-byte assertions.

| Case(s) | Exact test(s) and established outcome |
| --- | --- |
| T01 | `terminal::session::tests::support_rejects_each_stream_and_term_before_snapshot_or_mutation`: independent stdin/stdout checks and absent/empty/dumb/linux TERM reject before snapshot/mutation. `application::tests::informational_and_syntax_paths_have_correct_streams_status_and_zero_effects` and `unsupported_interactive_session_is_rejected_after_discovery_before_mutation`: exemptions and startup ordering. Colored environment declarations have real PTY coverage below. |
| T07/T08 | `terminal::runtime::tests::every_colored_frame_byte_boundary_resumes_exactly_before_the_latest_frame`: exhaustive actual-colored-byte offsets with both transient outcomes, ordered exact old/new wire output and available navigation. |
| T07/T08 | `terminal::runtime::tests::untouched_colored_frames_replace_after_transients_and_large_frames_yield_each_batch`: WouldBlock/Interrupted before emission allow replacement, both obsolete colored frames emit nothing; a 1000×1000 colored frame uses exactly one write per batch, each <=16 KiB, with input between batches and exact final wire equality. |
| T07/T08/V05 | `application::tests::colored_backpressure_coalesces_navigation_and_resize_without_stale_frames_or_samples`: started 60×16 colored frame split inside UTF-8; 85 forward and 85 backward events, 60×15/61×17/80×24 resize requests and a due observation under repeated WouldBlock. Output equals only the old frame plus latest b/80×24 frame, with no intermediate small/odd/selection frame queue. Final selected-view samples are exactly `(500ms,256)` and `(1s,256)`; three acquisitions including discovery. Due acquisition precedes first input, and crossing the deadline within a read causes acquisition before the next read; reads <=256 bytes/writes <=16 KiB. |
| T07/T08/T09 | `application::tests::handled_exits_after_every_colored_output_prefix_cancel_reset_and_restore`: every byte prefix of an actual colored frame, separately q/Q/typed Ctrl+C/external INT/TERM; statuses exactly 0/0/130/130/143, no further frame bytes or synthetic samples, reset/show/leave exact literal bytes, exhaustive restoration log. Signals at the next sample deadline preempt sampling/input/resize/output. |
| T07 | `application::tests::sustained_bounded_input_rechecks_due_acquisition_before_next_batch`, `shutdown_preempts_due_sampling_input_resize_and_rendering`, `shutdown_arriving_during_acquisition_stops_before_input_or_frame_work`, `blocked_output_still_handles_shutdown_without_an_extra_sample_or_write`: inherited sustained-batch fairness and shutdown-before-work rules remain passing. |
| T08 | `terminal::runtime::tests::untouched_frames_replace_but_partial_frames_finish_without_duplicate_bytes`, `backpressure_keeps_clock_input_signals_resize_and_timer_effects_available`, `every_prefactor_protocol_byte_boundary_resumes_exactly_without_interleaving`: inherited text/reset fixtures remain passing alongside colored coverage. |
| T09/Drop | `terminal::session::tests::explicit_and_drop_cleanup_cancel_partial_sgr_and_reset_before_screen_restoration`: every byte prefix of each of twelve actual screen-colored rows, explicit and Drop fallback; literal CAN/reset then CAN/show then CAN/leave, exact retained prefix and saved termios/flags restored. |
| T10/T12 | `terminal::session::tests::every_initialization_failure_restores_only_applicable_attempted_effects`: both errors and panics at snapshot/notification preparation/raw/input flags/output flags/dimensions/enter/hide; applicable exact restoration sets and saved state. `every_partial_initialization_sequence_failure_or_panic_resets_applicable_stages`: every interior byte boundary in enter/hide, followed by WouldBlock or panic; exact emitted partial command plus applicable CAN/reset/show/leave. |
| T10/T12 | `application::tests::initialization_and_caught_startup_runtime_panics_use_the_guarded_application_path`: complete invocation, all staged setup errors and startup/init/runtime panics; prior-hook probe now runs from diagnostic writing after cleanup. `terminal::session::tests::caught_initialization_runtime_and_cleanup_panics_restore_the_prior_hook`: prior hook is silent during caught panics and restored afterward. |
| T11/T12 | `application::tests::colored_runtime_faults_and_panics_restore_before_diagnostics_and_prior_hook_probe`: after emitting two bytes of a Braille glyph, independently EOF, input BrokenPipe, output BrokenPipe, clock error, poll error, dimensions error, signal-drain error, write panic, clock regression and runtime acquisition panic. Status 1, no extra colored bytes, all cleanup, exact cancel/reset/show/leave. The stderr writer asserts final signal release, asserts zero prior-hook calls during execution/restoration, then catches a sentinel panic to prove the previous hook has already been restored before diagnostics. |
| T11 | `application::tests::actual_loop_runtime_faults_restore_before_reporting`; `terminal::runtime::tests::input_eof_and_effect_faults_are_fatal_while_transient_io_yields`, `clock_failure_regression_and_idle_deadline_overflow_have_no_fallback`: inherited fatal IO/clock boundaries and transient yields. Native companions `terminal::linux::runtime::tests::checked_timespec_conversion_rejects_invalid_fields_and_overflow`, `poll_detects_each_owned_fd_hangup_error_and_invalid_descriptor`, `timer_effect_errors_and_incomplete_reads_are_fatal` also pass. |
| T13 | `application::tests::colored_exit_cleanup_errors_and_panics_preserve_status_and_attempt_all_stages`: after partial colored UTF-8, all seven restoration effects fail together, separately as errors and panics, across q/Q/typed Ctrl+C/INT/TERM/runtime EOF. Exactly seven cleanup diagnostics, exhaustive attempts, q/Q becomes 1 and selected 130/143/1 remains; hook/report ordering is checked at stderr. |
| T13 | `terminal::session::tests::each_cleanup_error_or_panic_keeps_remaining_stages_and_selected_status`: every single restoration error/panic, including notification release, across quit/INT/TERM/failure; one cleanup diagnostic, all stages attempted, no repeated Drop cleanup. `multiple_cleanup_failures_preserve_selected_status_and_attempt_every_operation` retains independent multiple-error coverage. |
| T13 | `terminal::session::tests::every_cleanup_sequence_split_resumes_or_cancels_before_remaining_restorations`: every interior offset in CAN/reset, CAN/show and CAN/leave, with short write + Interrupted + WouldBlock; interrupted wait success resumes exact bytes, failed wait preserves exact emitted prefix and still attempts remaining restoration. Every requested wait <=100ms, cleanup idempotent/nonrepeating on Drop. `cleanup_backpressure_is_bounded_and_does_not_skip_os_restoration` and `interrupted_cleanup_wait_resumes_within_the_same_bounded_write` also pass. |
| T12/T13/Drop | `terminal::session::tests::drop_fallback_restores_an_owner_without_explicit_cleanup`, `drop_during_unwinding_survives_an_additional_restoration_panic`: nonpanicking fallback, including reset panic during existing unwinding, still attempts remaining operations and restores saved OS-state fixtures. |
| V05 | `screen::tests::small_and_zero_dimensions_show_only_bounded_warning_selection_status`, `sampling_and_navigation_while_small_reappear_as_actual_history_on_growth`, `fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view`; `application::tests::small_session_keeps_offscreen_sampling_and_recovers_real_history_on_growth`: exact warning/floor/extreme bounds, no palette codes while small, observation-driven history while small, navigation/quit and growth recovery. Full exact screen mappings remain in ticket 02 evidence. |

## Linux PTY mappings — debug and identified release executable

Every one of the following sixteen tests passed in both final runs. The strict
decoder counts UTF-8 Braille as cells and permits only contract foreground/reset
SGR plus the existing addressing/session protocol. Saved termios is compared by
defined fields; saved input/output OFD flags have retained observers after exit.

| Case(s) | Exact `tests/linux_pty.rs` test name and actual outcome |
| --- | --- |
| T01 | `rejected_term_and_independent_nonterminal_streams_do_not_mutate`: rejected TERM/stream combinations preserve state; help/version exemptions and redirected streams. |
| T01/V01 | `accepted_full_views_emit_colored_braille_under_no_color_and_locale_declarations`: actual loopback traffic at 60×16/80×24, all twelve exact palette indices, correct cell bounds/default text/gaps/no LF/CR, colored under absent/C/C.UTF-8/unrecognized locale declarations and absent/empty/nonempty NO_COLOR; baseline uncolored and saved-state/cleanup success. |
| T08/protocol | `decoder_counts_styled_braille_as_cells_and_preserves_default_text`; `decoder_rejects_unknown_protocol_style_leaks_and_cell_overflow`: permitted UTF-8/SGR and strict rejection of unknown Unicode/SGR/commands, bounds overflow and style leaks. |
| T07/T08/V05 | **`colored_backpressure_resumes_a_complete_large_frame_then_recovers_latest_resize`**: real colored 60×16 loopback view; resize to 1000×1000 and observe no POLLOUT; deliver ordered navigation plus 59×15 resize while backpressured, then drain. A complete colored 1000-row frame is strictly decoded before the warning (no lost/interleaved suffix); small frame has no palette codes. Grow to 80×24, retained Braille reappears; q restores saved state and exact cancel/reset/show/leave. Distinct fixture heights allow decoding any complete old/new frame at its actual dimensions. |
| T09/T11 | **`colored_normal_signal_and_runtime_eof_exits_restore_os_state_and_protocol`**: colored view before separate q/Q/typed Ctrl+C/INT/TERM/canonical-EOF runs, raw/nonblocking assertions, statuses 0/0/130/130/143/1, complete saved termios/OFD flags and literal restoration sequence; EOF diagnostic. |
| T02/T09 | `normal_controls_restore_real_modes_shared_flags_and_protocol`: raw mode/Ctrl+Z ignored plus all normal/signal exit controls. `shared_terminal_description_and_existing_nonblocking_flags_are_preserved`: shared OFD and already-nonblocking saved flags. |
| T06/T09 | `queued_external_interrupt_and_terminate_choose_interrupt`: both signals kernel-pending while stopped, INT wins after continuation, reset/cleanup and status 130. |
| T03/T04 | `fragmented_repeated_arrows_and_ignored_sequences_reach_expected_selection`: real fragmented CSI/SS3 input, repeated navigation/ignored command-looking sequences and quit. |
| V05/T09 | `small_startup_resize_sampling_navigation_and_quit_keep_bounded_rows`: 59×16, 60×15 and 1×1 startup; real sampling/navigation, 60×16/80×24 growth and 1×1 shrink, bounded frames/quit/restoration. |
| T10/T13 | `read_only_terminal_output_fails_setup_and_restores_changed_os_state`: actual EBADF on terminal output during setup, reset/leave cleanup diagnostics and restored changed OS state. `flow_stopped_output_during_setup_restores_os_state_despite_cleanup_failure`: practical WouldBlock initialization fault, bounded cleanup including reset failure and all saved-state restoration. |
| T11/T13 | `runtime_input_and_output_hangups_fail_and_attempt_remaining_restoration`: now starts from a colored view. Disconnected input/output separately fail with status 1; shared flags restored, surviving PTY termios restored, healthy output has literal cleanup. Output hangup diagnoses reset/show/leave failure; disconnected input cannot restore termios and diagnoses that failure. `canonical_eof_on_real_input_is_fatal_and_restores_the_saved_session`: inherited real EOF boundary also passes. |
| T07/T08/T09/T13 | `backpressured_runtime_still_handles_quit_and_signal_with_bounded_cleanup`: now starts colored, then a real partial large frame; freeze output flow after no-POLLOUT observation. q/INT/TERM remain responsive within harness timeout, cleanup reset/show/leave failures diagnosed, statuses 1/130/143, saved termios/OFD flags restored despite blocked output. Harness flow stop is undone afterward. |

### Evidence limits and practical fault allocation

PTYs establish protocol and Linux state, **not** font/palette appearance or visible
prior-screen recovery. Human emulator/live acceptance is assigned to ticket 04;
no such observation is claimed by this record. Ordinary UDP is used only to make
colored graphs visible on the wire; exact live rates/sample timing are not an
oracle and this is not ticket 04's complete live-smoke record.

Real PTYs cannot reliably force each particular UTF-8/SGR byte split, clock
failure/regression/conversion overflow, caught Rust panics, every partial setup
prefix or independently chosen restoration error/panic combination. Their exact
deterministic/scripted evidence is linked by the named tests above. Native owned
FD tests supply timerfd/clock/poll edge evidence. Diagnostic-versus-cleanup/prior
hook ordering is established by the scripted complete application writer; reading
separate PTY/stdout and stderr after exit is not claimed to reconstruct a total
cross-stream ordering.

Unwritable output tests establish bounded **attempts** and OS restoration. They
do not prove that CAN/reset/show/leave reached an emulator. A disconnected PTY's
unrestorable termios is explicitly diagnosed, not reported as restored. Clean,
writable output cases do establish the observable CAN/reset-before-show/leave
byte sequence and saved-state equality.

## Criteria and review resolution

All eight ticket criteria passed: exhaustive actual-colored partial transport;
untouched-only replacement/latest-state coalescing; observation-driven fairness
and shutdown priority; complete staged exit/fault/panic/Drop restoration; single
and multiple cleanup error/panic status arbitration and bounded partial output;
real compiled-binary PTY verification; exact affected T01/T07–T13/V05 mappings
and honest evidence limits; required gates and release-artifact identification.

Loaded `code-review` and reviewed the complete changes against fixed `48584d0`
on separate Standards and Spec axes. No delegation tool is available in this
harness, so this is same-agent review, not independent parallel-agent review.
Standards sources: AGENTS.md, tracker/domain guidance, glossary and neighboring
Rust/licensing/test conventions; no additional standards/ADR files were found.

- **Standards:** no unresolved hard violations or actionable smell findings.
  Script extensions are test-only and used by the concrete fault cases; shared
  UDP traffic generation is factored once. Large fixture output iteration is
  bounded; the hook guard and existing lock keep process-global hook tests owned.
- **Spec:** no unresolved requirements or scope creep. The frame wire oracle
  tests exact transport, with independent screen glyph/grade oracles retained;
  actual observation counts and exact cleanup bytes/statuses are independently
  asserted. Practical PTY limitations are explicitly allocated to scripted tests.

Status, full intended diff and recent log were inspected before committing only
the five intended test/harness files. Evidence/tracker resolution is a separate
documentation commit. **No blocker remains for ticket 03.**
