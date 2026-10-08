# 01: Prefactor screen output into an opaque, cell-bounded frame

**What to build:** Run the existing single-interface view through a screen-owned frame whose dimensions and encoded rows cannot disagree. Establish safe cell-based construction, default-style row output, and restoration before introducing colored Braille graph glyphs. This is an intermediate prefactor preserving the existing graph appearance, not an ASCII compatibility mode in the finished application.

Authority: the [styled graph contract](../../btop-graph-spec/issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract), especially its deep screen/terminal seam and encoding guarantees, with unaffected behavior inherited from the [minimal Rust port specification](../../nload-rust-port/spec.md).

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Assignee:** implementer-01 (OpenCode)

- [ ] Screen construction owns an opaque frame with supplied logical dimensions and private rows. Its narrow read-only interface exposes dimensions and encoded rows; callers cannot construct frames from arbitrary strings, supply inconsistent dimensions, or access a generic public styled-cell builder.
- [ ] Rendering builds exactly the supplied logical row count and terminal-cell width, including zero dimensions. Text placement and clipping operate on bounded cells, preserving existing geometry, status text, statistics formatting, escaped original-byte interface names, ellipsis, position reservation, and numeric overflow behavior.
- [ ] Trailing blank cells are trimmed before encoding without changing logical dimensions. Encoded rows contain no LF/CR, arbitrary controls, or user-supplied escapes; screen emits no positioning, clearing, or session commands. Every emitted row starts and ends with an SGR reset, with a reset before the statistics area; final resets are never trimmed from serialized output.
- [ ] Runtime accepts only screen-constructed frames and uses their dimensions. It clears in default styling, owns absolute positioning and final cursor placement, and does not scan display widths, parse arbitrary ANSI, or slice/trim serialized rows. Existing right-margin and no-LF/CR guarantees remain valid.
- [ ] Applicable explicit cleanup and nonpanicking Drop fallback cancel incomplete control sequences and attempt SGR reset before cursor-show and alternate-screen-leave. Reset is a staged restoration attempt; failure does not skip other applicable restorations or change existing exit-status arbitration.
- [ ] Application, scripted/native runtime tests, and the real Linux PTY harness consume the new protocol. The PTY decoder understands printable ASCII, nonblank UTF-8 Braille, the agreed palette foreground SGR/reset subset, and existing terminal commands with terminal-cell bounds; unknown protocol is rejected by the harness rather than silently stripped.
- [ ] Existing presentation, replacement/resumption, cleanup, and Linux PTY coverage passes through the new interface. Changed module documentation accurately describes the new ownership. Record passing `cargo fmt --check` and `cargo test --locked`, keeping the prefactor independently verifiable.
