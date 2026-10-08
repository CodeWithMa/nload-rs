# Verified styled graph implementation handoff

**Acceptance complete — 2026-10-08.** Version **0.1.1**, Linux-only, stable Rust
edition 2024, GPL-2.0-only. All five implementation tickets are complete;
no acceptance blocker or required follow-up remains.

## Governing contract

The [styled graph specification](../btop-graph-spec/spec.md) points to the
[canonical styled graph resolution](../btop-graph-spec/issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract).
It supersedes historical ASCII thresholds, one-sample-per-cell presentation,
ASCII-row seams and affected output/restoration promises. The
[minimal Rust port specification](../nload-rust-port/spec.md) governs unaffected
CLI, startup, acquisition, statistics, history, input, scheduling and delivery.
Its initial 0.1.0 is historical: Cargo metadata is the sole version authority.

The single-interface view displays right-aligned chronological pairs of actual
samples, with independently bottom-filled Braille halves and positional incoming
indigo/lavender and outgoing magenta palettes. True zeros are spaces; numeric
rates are uncapped. Sampling remains 500 ms and monitor-owned history remains
five minutes for every startup interface. Full view begins at 60×16, fitting
60 samples; 80×24 fits 100. Resize and navigation do not manufacture samples.

Interactive sessions require terminal stdin/stdout, UTF-8, single-cell Braille
font coverage and xterm-256color. TERM is a declaration, not a probe: unset/empty,
`dumb` and `linux` reject before mutation; other declarations assume support.
Full views always use colored Braille under every locale and NO_COLOR declaration.
Small views retain default-style warning, acquisition/navigation/quit and growth
recovery. Help/version remain exempt.

## Verified source and artifact

Worktree `/tmp/opencode/nload-btop-05`, branch `implement/btop-05`, based on clean
integration `66cd1369a36bd039b6a8ce9d7e2b815ce8146c6c`. Final gates ran on that
revision with **zero uncommitted delta**. Executable inputs match corrected source
`5e4661164f7afda85c266b2a45ce730276c71bad`; delivery edits are documentation only.
Accepted delivery revision: **`7b0a6a3b7632178eec5e9ccf20839bc1c398a58f`**
(`docs: complete styled graph acceptance and delivery handoff`). The subsequent
revision-record finalization changes only this handoff and final evidence; its
uncommitted implementation delta is zero. The
[final evidence](05-evidence.md#delivery-revision-and-integration-reconciliation)
records committed revision/delta and integration reconciliation.

Release artifact: `/tmp/opencode/nload-btop-05/target/release/nload-rs`.
Also verified byte-identical to both permanent checkout artifacts:
`/mnt/programming/rust/nload-rs/target/release/nload-rs` and
`/mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs`.
SHA-256: **`d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`**.
These identities establish applicability of the recorded observations; future
rebuilds are not promised byte-identical.

## Acceptance evidence

| Evidence | Verified result / record |
| --- | --- |
| Final build and deterministic gates | [Final gates/environment](05-evidence.md#final-gates-and-environment): fmt, locked tests (**115 unit/native/scripted + 16 PTY/decoder**), optimized locked release build. rustc 1.98.1, Cargo 1.98.0, `x86_64-unknown-linux-gnu`, Linux 7.2.9. |
| Release Linux PTYs | [Final gates](05-evidence.md#final-gates-and-environment): **16 passed** with `NLOAD_RS_PTY_BINARY=./target/release/nload-rs`; [exact transport/exit mappings](03-evidence.md#linux-pty-mappings--debug-and-identified-release-executable). |
| Exact acceptance matrix | [Final case-to-test/manual mappings](05-evidence.md#acceptance-matrix): A01–A12, S01–S13, styled V01–V08, T01–T14 and D01–D05; [screen migration detail](02-evidence.md#exact-v01v08-mappings-and-twelve-function-migration) plus [rounding correction](review-fixes-evidence.md#meaningful-test-first-execution-at-the-agreed-frame-seam). |
| Human emulator | [Actual report](04-human-observations.md), [completed applicability assessment](04-evidence.md#current-acceptance-and-applicability--2026-10-08): Alacritty 0.17.0, both 60×16/80×24 palettes/contrast/Braille, blank idle, navigation/small-history recovery and separate 0/130/143 visible restoration. |
| Nonprivileged live Linux | [Corrected-artifact loopback smoke](review-fixes-evidence.md#fresh-ordinary-loopback-live-smoke-on-corrected-release): actual discovery/navigation, independent 2/4 MBit/s labels, low/high/idle traffic, advancing totals and clean q; human ordinary-emulator confirmation linked above. |
| Modules, cost, metadata/licenses | [Final artifact review](05-evidence.md#module-cost-and-delivery-review): opaque frame boundary, bounded suffix/cell work, concrete Linux unsafe, actual target/features and MIT notice routes; no added dependencies or copied upstream algorithm. |

Configured font/client OS/theme details were unavailable; actual clean single-cell
Braille and visual behavior were explicitly confirmed. Human small q size was
56×17 and small startup had no exact-size transcript; deterministic/PTY fixtures
establish exact 59×16/60×15 boundaries and sample arithmetic. Uninducible faults
use named scripted/native evidence; PTYs are protocol/OS checks, and unwritable
output establishes restoration attempts rather than successful visible reset.
No unrun check is counted as passing.

Repeat the three Cargo gates and release PTY command in [README](../../README.md)
when executable inputs change, and repeat affected visual/live checks when their
applicability changes. Historical ASCII and pre-correction evidence remains linked
for provenance. [Implementation resolutions](implementation.md#decisions-so-far)
and [Complete styled-graph acceptance and delivery handoff](issues/05-acceptance-delivery-handoff.md)
identify the completed delivery; source planning/specification parents retain their
original scope.
