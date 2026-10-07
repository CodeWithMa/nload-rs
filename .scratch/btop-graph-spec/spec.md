# btop-style graph spec for nload-rs — implementation handoff

Status: ready for implementation
Map: [Find the way to a btop-style graph spec for nload-rs](map.md)
Confirmed: 2026-10-07, through the human's acceptance of both contract rounds

## Authority

Read the resolution of [Decide the styled graph screen contract](issues/04-screen-seam-contract.md#resolution--2026-10-07--alice-opencode-session-styled-graph-contract) as the canonical specification for this change. Its detail lives in that ticket, not duplicated here.

The [Minimal Rust port spec](../nload-rust-port/spec.md) supplies inherited application behavior and verification requirements. Where its ASCII presentation or module-ownership promises conflict with the styled graph contract, the newer resolution supersedes them. Research and the throwaway prototype explain the route; they are not alternative normative algorithms.

## Implementation reading index

All links below point into the same canonical resolution:

- [Scope and authority](issues/04-screen-seam-contract.md#authority-and-scope)
- [Geometry, availability, idle, text, and small-terminal states](issues/04-screen-seam-contract.md#geometry-and-presentation-states)
- [Horizontal history placement and cadence clarification](issues/04-screen-seam-contract.md#horizontal-sample-placement)
- [Dot quantization and Unicode mapping](issues/04-screen-seam-contract.md#vertical-quantization-and-braille-mapping)
- [Exact palettes and positional grades](issues/04-screen-seam-contract.md#exact-per-direction-palettes)
- [Opaque styled-frame seam, encoding, output, and cleanup](issues/04-screen-seam-contract.md#deep-screenterminal-seam-and-encoding-guarantees)
- [Terminal declaration policy](issues/04-screen-seam-contract.md#terminal-declaration-policy)
- [Acceptance and test migration](issues/04-screen-seam-contract.md#acceptance-and-test-migration)
- [Dependencies, licensing, and rendering cost](issues/04-screen-seam-contract.md#dependencies-licensing-and-cost)

## Supersession index

| Earlier port authority | Governing newer resolution section |
| --- | --- |
| §4.5 and sample/column wording in §4.6, S03/S07/S09/S11/S12 | Horizontal sample placement; retained observation/history semantics remain inherited. |
| §5.1 ASCII row contract, §5.2 ASCII bar thresholds | Geometry and presentation states; vertical quantization and Braille mapping; deep screen/terminal seam. |
| §5.3–§5.5 presentation | Geometry and presentation states; exact per-direction palettes; acceptance and test migration. Existing state/text/formatting rules remain inherited. |
| §6.1 terminal requirements, §6.3 frame output, §7.2 restoration | Terminal declaration policy; deep screen/terminal seam and encoding guarantees. |
| §8 screen/terminal ownership and ASCII-row test seam | Deep screen/terminal seam and encoding guarantees. |
| V01–V08; affected terminal/evidence/artifact cases in §10 | Acceptance and test migration. |
| §11 ASCII screen departure and historical screen resolution | Authority and scope; vertical quantization and Braille mapping. |

The implementation's documentation and verification records should make this supersession explicit. The earlier port spec remains a historical source rather than being rewritten to imply the existing application already emits styled graphs.

## Route context

- [Establish what btop actually renders](issues/01-btop-rendering-facts.md)
- [Establish Braille and color terminal feasibility](issues/02-terminal-color-feasibility.md)
- [Prototype the Braille and color graph look](issues/03-graph-prototype.md)
- [Decide the styled graph screen contract](issues/04-screen-seam-contract.md)
- [Network Traffic Monitor glossary](../../GLOSSARY.md)

## Readiness

All child tickets are closed and the map has no remaining fog. This is the planning handoff; build/test/PTY/emulator/live verification results belong to the subsequent implementation effort.
