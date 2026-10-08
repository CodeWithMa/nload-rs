# Styled graph implementation

Integration branch: `integration/btop-styled-graphs`

Authority: [implementation handoff specification](../btop-graph-spec/spec.md)
and its linked canonical contract. Local tracker operations follow
[issue tracker](../../docs/agents/issue-tracker.md).

## Task graph

`01 → 02 → {03, 04} → 05`

Ticket 04 requires recorded human emulator observations. Ticket 05 remains
blocked until both 03 and 04 close. Automated results do not substitute for
human visual observations.

## Decisions so far

- [Prefactor screen output into an opaque, cell-bounded frame](issues/01-opaque-cell-bounded-frame.md#resolution--2026-10-08--implementer-01-opencode): screen-owned logical cells and reset-hygienic rows now cross the opaque frame seam; runtime positioning, staged cleanup and strict PTY decoding pass with the inherited ASCII presentation.
- [Display paired Braille graph history with direction palettes](issues/02-paired-braille-direction-palettes.md#resolution--2026-10-08--implementer-02-opencode): exact right-aligned two-sample Braille fill and per-direction positional palettes now render through bounded frames; inherited statistics/history/support behavior, updated V01–V08 mappings, and debug/release Linux PTY gates pass.

## Evidence

Implementation agents record actual commands, outcomes, and case mappings
with their ticket resolutions. Unperformed checks remain pending.

- [Ticket 01 — opaque cell-bounded ASCII prefactor evidence](01-evidence.md):
  test-first execution, all seven criteria, 97 unit + 13 PTY/decoder tests and
  passing formatting/locked-test/release-build gates.
- [Ticket 02 — paired Braille and direction palettes evidence](02-evidence.md):
  test-first paired/quantized appearance, all eleven criteria, 104 unit + 14
  PTY/decoder tests, passing required gates and release-artifact PTY coverage.
