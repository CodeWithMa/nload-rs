# Ticket 01 — opaque cell-bounded ASCII prefactor evidence

Recorded 2026-10-08 by implementer-01 (OpenCode), in
`/tmp/opencode/nload-btop-01` on `implement/btop-01`.

Authority: [Prefactor screen output into an opaque, cell-bounded frame](issues/01-opaque-cell-bounded-frame.md).
This is the independently verifiable ASCII intermediate step. The newer styled
contract governs frame ownership, encoding and cleanup; existing ASCII graph
presentation remains the ticket's oracle until ticket 02 replaces the glyphs.

## Baseline and integration

- Initial `git status --short`: empty; `git branch --show-current`:
  `implement/btop-01`.
- Initial `git rev-parse HEAD integration/btop-styled-graphs`: both
  `71805e272bb29a16a5a1affbdc1a9588ad6a59c3`. No reset was needed.
- After implementation and passing gates, `git merge integration/btop-styled-graphs`
  in this worktree returned `Already up to date.`; integration remained `71805e2`.

## Test-first execution

Loaded `implement` and required `tdd` skills. The ticket supplies the agreed
seams: screen construction/read-only frame, runtime output, staged session
restoration and the real Linux PTY protocol decoder. Each new behavioral slice
was exercised red before its production change:

| Slice / initial command | Observed red | Observed green |
| --- | --- | --- |
| `cargo test --locked opaque_frame_retains_dimensions_and_encodes_trimmed_default_rows` | E0599: old `Vec<String>` has no frame dimensions/encoded-row interface. | `cargo test --locked screen::tests`: 13 passed, including all twelve inherited presentation fixtures. |
| `cargo test --locked screen_frames_clear_in_default_style_and_keep_row_resets_and_cell_bounds` | E0061: old runtime takes arbitrary rows and separate dimensions rather than a screen frame. | `cargo test --locked terminal::runtime::tests`: 11 passed; `cargo test --locked terminal::linux::runtime::tests`: 6 passed. |
| `cargo test --locked explicit_and_drop_cleanup_cancel_partial_sgr_and_reset_before_screen_restoration` | Assertion failed: old cleanup omitted CAN + SGR reset before show/leave. | `cargo test --locked terminal::session::tests`: 17 passed. |
| `cargo test --locked --test linux_pty decoder_counts_styled_braille_as_cells_and_preserves_default_text` | Old ASCII/address decoder failed parsing the styled fixture. | Same command: 1 passed; application tests: 18 passed. |
| `cargo test --locked --test linux_pty decoder_rejects_unknown_protocol_style_leaks_and_cell_overflow` | Decoder incorrectly accepted `ESC[+1;1H`. | `cargo test --locked --test linux_pty decoder_`: 2 passed after numeric-address validation. |

One migrated runtime oracle initially used width 30 for the 31-cell warning;
the observed clipping exposed that fixture mistake. Corrected its dimensions to
32 before the runtime suite passed; no presentation rule was changed.

Additional verification:

- `cargo test --locked every_prefactor_protocol_byte_boundary_resumes_exactly_without_interleaving`:
  1 passed. Every byte split in the literal prefactor output, including every
  reset fragment, resumes exactly after would-block; navigation remains available
  and a started frame cannot be replaced.
- Reset was included in the inherited all-operation failure/status matrix;
  interrupted/short cleanup now exercises reset itself, and Drop during unwinding
  survives a reset panic while continuing other restorations.
- `cargo check --locked` passed throughout the frame and runtime migrations and
  after the remaining changes.

## Final required gates

All commands ran successfully in the worktree:

| Command | Result |
| --- | --- |
| `cargo fmt --check` | Passed. |
| `cargo test --locked` | 97 unit tests + 13 Linux PTY/decoder tests passed; zero failures or ignored tests. |
| `cargo build --release --locked` | Passed; optimized `target/release/nload-rs` built. |

Compiler: `rustc 1.98.1 (48a229cea 2026-09-01) (built from a source tarball)`.
Cargo: `cargo 1.98.0 (797e8a9bc 2026-08-05)`.

## Criteria / acceptance mapping

| Ticket criterion | Evidence / public seam |
| --- | --- |
| Opaque construction and dimensions | `screen::Frame` has private dimensions/rows, no external constructor or public cell builder, and only read-only dimension/encoded-row accessors; `opaque_frame_retains_dimensions_and_encodes_trimmed_default_rows`. |
| Bounded placement and inherited presentation | Private character-cell rows; all twelve inherited screen fixtures traverse the frame and validate reset-only encoding before comparing their independent presentation literals. V01–V08 ASCII-prefactor coverage includes minimum/odd/full geometry, exact thresholds/history, all availability states, tiny/zero sizes, byte-escaped names/ellipsis/position, numeric overflow, units and maximum values. |
| Encoding and trimming | Screen trims logical blank cells before serialization, emits only default SGR resets, and resets every row's start/end and before statistics; literal frame test plus frame-decoding helper across presentation fixtures. No positioning/clearing/session output in screen. |
| Runtime ownership | Only `&screen::Frame` accepted; encoded rows copied verbatim, reset before clear, absolute positioning and final home retained. `screen_frames_clear_in_default_style_and_keep_row_resets_and_cell_bounds`, untouched/partial replacement, every-byte resumption and finite-batch tests (T08). |
| Cleanup | Explicit/Drop literal reset-order test plus all inherited session setup/fault/panic/status tests with reset staged and consumed independently (T09–T13). Reset failure does not skip other restorations; status arbitration unchanged. |
| Application/native/PTY migration | Application frame path and cleanup assertions; 18 application tests; 6 native runtime tests; real compiled-binary PTY tests retain input/navigation/resize/raw flags/normal and signal exits/fault/backpressure coverage (T01–T13). PTY start/restore markers include reset. |
| Strict cell-aware decoder | Two decoder tests use literal nonblank multibyte Braille and all twelve permitted palette indices; reject overflow, arbitrary Unicode/U+2800, LF/CR/controls, foreign commands/SGR/background and style leaks. Unknown prefixes before frames are rejected. Width is counted in cells, never UTF-8 bytes. |
| Documentation and gates | Screen/application/terminal/runtime ownership docs updated; final gates above passed. |

## Review and evidence limits

Loaded `code-review` and reviewed the complete working diff against `71805e2`
along Standards and Spec axes. No delegation tool was available, so this was a
same-agent review, not an independent parallel-agent review.

- Standards: repository tracker/domain guidance, neighboring Rust conventions,
  licensing headers and smell baseline checked; no unresolved findings.
- Spec: all seven ticket criteria checked against their canonical/inherited
  authority; no unresolved findings. No dependency, CLI, cadence, monitor or
  production Braille/palette changes were introduced.

The PTY tests exercise actual Linux process/terminal behavior, not emulator
appearance. New human emulator observations, a fresh live-smoke record, and
finished colored-graph acceptance remain for the later implementation tickets;
none are claimed here. Historical planning/specification/map/ticket sources were
not edited. No blockers remain for ticket 01.
