# 05: Complete styled-graph acceptance and delivery handoff

**What to build:** Deliver a verified styled-graph release with current support documentation, complete acceptance evidence, and an implementation handoff that clearly identifies the governing newer contract and applicable inherited behavior.

Authority: the [styled graph implementation handoff specification](../../btop-graph-spec/spec.md), its [canonical contract](../../btop-graph-spec/issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract), and unaffected requirements and evidence rules in the [minimal Rust port specification](../../nload-rust-port/spec.md).

**Blocked by:**
- [03: Verify styled output under backpressure and interrupted exits](03-styled-output-backpressure-exits.md) (closed; prerequisite satisfied).
- [04: Record emulator appearance and nonprivileged live traffic](04-emulator-live-traffic.md).

**Status:** blocked / awaiting ticket 04 human evidence

- [ ] Audit every governing requirement against deterministic tests, Linux PTY results, recorded emulator observations, live smoke, or artifact review as applicable. Maintain exact case-to-test/manual mappings for V01–V08, affected S03/S07/S09/S11/S12, T01/T08–T14, and inherited behavior. No unrun evidence is counted as passing.
- [ ] Make the supersession of earlier ASCII graph thresholds, single-sample-per-cell wording, ASCII-row seams, and affected output/restoration promises explicit in implementation documentation and verification records. Preserve the earlier port specification as historical authority rather than rewriting its account of the earlier application.
- [ ] On the final implementation, run and record `cargo fmt --check`, `cargo test --locked`, and `cargo build --release --locked`. Record revision plus any uncommitted implementation delta, exact stable compiler/target/environment, and release artifact identity. Confirm applicable release-binary Linux PTY evidence; repeat affected checks only when final changes invalidate earlier evidence.
- [ ] Review the deep screen/terminal ownership: opaque screen-constructed frames, frame-owned dimensions, bounded cell construction, screen-owned glyph/palette/encoding/reset guarantees, terminal-owned clearing/positioning/nonblocking transport, and staged exhaustive restoration. Test surfaces do not expose private monitor queues or hypothetical backend abstractions; unsafe remains confined to concrete adapters.
- [ ] Confirm rendering/encoding work scales with screen area and the fitting visible sample suffix, not copies of every interface's retained history. Sampling cadence, rate calculations, monitor retention, CLI, and configuration semantics remain consistent with the inherited contract.
- [ ] Review stable Rust edition 2024/Linux-only delivery, GPL-2.0-only metadata/license, actual locked dependency/features/license graph, version authority, and inspiration credit. No dependency was added for Braille, display-cell accounting, SGR, or palette selection; algorithm implementation is independently authored.
- [ ] Product/module documentation accurately states UTF-8, Braille-font, and xterm-256color assumptions, declaration-based TERM support, always-colored full-view behavior, and the new frame ownership. Replace stale current-status/evidence links with applicable styled-graph records while retaining historical evidence provenance.
- [ ] Publish a concise verified implementation handoff linking build, deterministic, PTY, emulator, live-smoke, and artifact evidence. Any concrete remaining gap is an explicit follow-up blocker; acceptance stays incomplete until required evidence passes. Do not close or modify the source planning/specification parent issue or map.
