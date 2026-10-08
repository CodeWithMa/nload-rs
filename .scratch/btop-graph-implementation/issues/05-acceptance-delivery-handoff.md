# 05: Complete styled-graph acceptance and delivery handoff

**What to build:** Deliver a verified styled-graph release with current support documentation, complete acceptance evidence, and an implementation handoff that clearly identifies the governing newer contract and applicable inherited behavior.

Authority: the [styled graph implementation handoff specification](../../btop-graph-spec/spec.md), its [canonical contract](../../btop-graph-spec/issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract), and unaffected requirements and evidence rules in the [minimal Rust port specification](../../nload-rust-port/spec.md).

**Blocked by:** None. [03: Verify styled output under backpressure and interrupted exits](03-styled-output-backpressure-exits.md) and [04: Record emulator appearance and nonprivileged live traffic](04-emulator-live-traffic.md) are closed; both prerequisites are satisfied.

**Status:** closed

**Assignee:** implementer-05 (OpenCode)

- [x] Audit every governing requirement against deterministic tests, Linux PTY results, recorded emulator observations, live smoke, or artifact review as applicable. Maintain exact case-to-test/manual mappings for V01–V08, affected S03/S07/S09/S11/S12, T01/T08–T14, and inherited behavior. No unrun evidence is counted as passing.
- [x] Make the supersession of earlier ASCII graph thresholds, single-sample-per-cell wording, ASCII-row seams, and affected output/restoration promises explicit in implementation documentation and verification records. Preserve the earlier port specification as historical authority rather than rewriting its account of the earlier application.
- [x] On the final implementation, run and record `cargo fmt --check`, `cargo test --locked`, and `cargo build --release --locked`. Record revision plus any uncommitted implementation delta, exact stable compiler/target/environment, and release artifact identity. Confirm applicable release-binary Linux PTY evidence; repeat affected checks only when final changes invalidate earlier evidence.
- [x] Review the deep screen/terminal ownership: opaque screen-constructed frames, frame-owned dimensions, bounded cell construction, screen-owned glyph/palette/encoding/reset guarantees, terminal-owned clearing/positioning/nonblocking transport, and staged exhaustive restoration. Test surfaces do not expose private monitor queues or hypothetical backend abstractions; unsafe remains confined to concrete adapters.
- [x] Confirm rendering/encoding work scales with screen area and the fitting visible sample suffix, not copies of every interface's retained history. Sampling cadence, rate calculations, monitor retention, CLI, and configuration semantics remain consistent with the inherited contract.
- [x] Review stable Rust edition 2024/Linux-only delivery, GPL-2.0-only metadata/license, actual locked dependency/features/license graph, version authority, and inspiration credit. No dependency was added for Braille, display-cell accounting, SGR, or palette selection; algorithm implementation is independently authored.
- [x] Product/module documentation accurately states UTF-8, Braille-font, and xterm-256color assumptions, declaration-based TERM support, always-colored full-view behavior, and the new frame ownership. Replace stale current-status/evidence links with applicable styled-graph records while retaining historical evidence provenance.
- [x] Publish a concise verified implementation handoff linking build, deterministic, PTY, emulator, live-smoke, and artifact evidence. Any concrete remaining gap is an explicit follow-up blocker; acceptance stays incomplete until required evidence passes. Do not close or modify the source planning/specification parent issue or map.

## Assets

- [Ticket 05 — final styled graph acceptance evidence](../05-evidence.md)
- [Verified styled graph implementation handoff](../handoff.md)
- [Human emulator observations](../04-human-observations.md)

## Comments

### Resolution — 2026-10-08 — implementer-05 (OpenCode)

All eight criteria passed. Clean `implement/btop-05` and integration matched
requested baseline `66cd1369a36bd039b6a8ce9d7e2b815ce8146c6c` before work; no reset
was needed. Final gates ran on that revision with zero uncommitted delta:
`cargo fmt --check`, `cargo test --locked` (**115 unit/native/scripted + 16 Linux
PTY/decoder**), `cargo build --release --locked`, and
`NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty`
(**16 passed**). Stable rustc 1.98.1/Cargo 1.98.0, Linux 7.2.9,
`x86_64-unknown-linux-gnu`; no failures/ignored tests or unrun evidence claimed.

Final release SHA-256 is
`d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`, matching the
permanent normal/review artifacts and the actual human report. Source/test/manifest/
lockfile inputs match corrected `5e4661164f7afda85c266b2a45ce730276c71bad`.
The corrected live smoke and supplied Alacritty observations remain applicable;
no documentation-only edit invalidates them. Actual unknown font/client details
and 56×17 small q size retain their honest limits and mapped scripted evidence.

[Final acceptance evidence](../05-evidence.md) maps every A/S/V/T/D case, explicit
ASCII supersessions, exact styled-frame/transport/restoration obligations and
module/cost/dependency/license/version audits. Rendering visits the fitting suffix
and bounded cells; unaffected cadence/statistics/retention/CLI/config remain
inherited. Edition 2024/Linux/GPL-2.0-only and actual 0.1.1 Cargo authority pass;
all six locked packages' MIT notices and target features were re-audited. No
dependency or copied upstream algorithm was introduced.

README and current evidence links now select completed styled acceptance while
preserving historical provenance. The concise [verified handoff](../handoff.md)
links all build/deterministic/PTY/human/live/artifact evidence. Loaded required
`tdd`; no behavioral/helper change was needed, so inspection and meaningful
existing tests were used for this documentation/artifact delivery. Standards and
Spec closure reviews found no unresolved findings. **No blocker remains; ticket
05 and this implementation effort are complete.**

Final intended-file/link/whitespace review, committed revision/delta and latest-tip
integration reconciliation are recorded in the final evidence. Source planning/
specification parent issues/maps retain their original scope.
