# Styled graph implementation

Integration branch: `integration/btop-styled-graphs`

Authority: [implementation handoff specification](../btop-graph-spec/spec.md)
and its linked canonical contract. Local tracker operations follow
[issue tracker](../../docs/agents/issue-tracker.md).

## Task graph

`01 → 02 → {03, 04} → 05`

**Status: complete — all five tickets closed, no acceptance blockers.** Required
human emulator observations are supplied and resolved; final combined acceptance
and delivery are recorded in the [verified styled graph handoff](handoff.md) and
[final acceptance evidence](05-evidence.md). Automated results do not substitute
for human visual observations.

## Decisions so far

- [Prefactor screen output into an opaque, cell-bounded frame](issues/01-opaque-cell-bounded-frame.md#resolution--2026-10-08--implementer-01-opencode): screen-owned logical cells and reset-hygienic rows now cross the opaque frame seam; runtime positioning, staged cleanup and strict PTY decoding pass with the inherited ASCII presentation.
- [Display paired Braille graph history with direction palettes](issues/02-paired-braille-direction-palettes.md#resolution--2026-10-08--implementer-02-opencode): exact right-aligned two-sample Braille fill and per-direction positional palettes now render through bounded frames; inherited statistics/history/support behavior, updated V01–V08 mappings, and debug/release Linux PTY gates pass.
- [Verify styled output under backpressure and interrupted exits](issues/03-styled-output-backpressure-exits.md#resolution--2026-10-08--implementer-03-opencode): exhaustive colored-byte resumption, latest-state coalescing, sampling/shutdown fairness and staged fault/panic/cleanup status checks pass; 113 unit + 16 PTY/decoder tests and identified-release PTY gates establish protocol/OS evidence with explicit emulator limits.
- [Record emulator appearance and nonprivileged live traffic](issues/04-emulator-live-traffic.md#resolution--2026-10-08--implementer-04-opencode): applicable corrected-artifact live/gate evidence and supplied Alacritty human observations at 60×16/80×24 complete T14/V05/D04, including small-history recovery and separate 0/130/143 visible restoration; unavailable configured font/client details and actual 56×17 small q size are recorded honestly.
- [Complete styled-graph acceptance and delivery handoff](issues/05-acceptance-delivery-handoff.md#resolution--2026-10-08--implementer-05-opencode): final inherited/styled acceptance mappings, supersession and module/cost/dependency/license audits pass; 115 unit + 16 debug PTY/decoder and 16 release PTY/decoder checks, matching corrected artifact and applicable human/live evidence complete the verified 0.1.1 handoff with no blockers.

## Evidence

Implementation agents record actual commands, outcomes, and case mappings
with their ticket resolutions. Unperformed checks remain pending.

- [Ticket 01 — opaque cell-bounded ASCII prefactor evidence](01-evidence.md):
  test-first execution, all seven criteria, 97 unit + 13 PTY/decoder tests and
  passing formatting/locked-test/release-build gates.
- [Ticket 02 — paired Braille and direction palettes evidence](02-evidence.md):
  test-first paired/quantized appearance, all eleven criteria, 104 unit + 14
  PTY/decoder tests, passing required gates and release-artifact PTY coverage.
- [Ticket 03 — styled transport, backpressure and exits evidence](03-evidence.md):
  exact T01/T07–T13/V05 mappings, exhaustive colored partial-output and handled
  exits, cleanup/error/panic matrices, required gates and release PTY artifact.
- [Ticket 04 — live smoke and emulator evidence](04-evidence.md#current-acceptance-and-applicability--2026-10-08):
  all seven completion mappings, corrected artifact/source applicability and
  retained 115-unit/16-PTY gates, with linked actual human report and scripted
  evidence for exact boundaries/uninducible states.
- [Review fixes — corrected source, test-first regression and fresh live evidence](review-fixes-evidence.md):
  rounding correction at `5e46611`, adjacent-f64 exact glyph coverage and corrected
  artifact identity/live observations; then-pending human blocker resolved by 04.
- [Ticket 05 — final styled graph acceptance evidence](05-evidence.md):
  final fmt/locked-test/release-build/release-PTY gates, exact complete acceptance
  matrix, compiler/environment/source/artifact applicability and delivery audit.
- [Verified styled graph implementation handoff](handoff.md):
  concise completed contract/source/artifact/evidence delivery, all tickets closed.
