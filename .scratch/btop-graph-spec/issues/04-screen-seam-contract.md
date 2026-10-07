# Decide the styled graph screen contract

Status: open
Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Parent: [Find the way to a btop-style graph spec for nload-rs](../map.md)
Assignee:
Blocked by: 01, 02, 03

## Question

What is the implementation-ready contract for styled graphs that replaces ASCII bars outright with no fallback? Decide with the human, using the btop facts, terminal feasibility, and accepted prototype direction: screen-owns-escapes seam (new styled-cell/rows interface replacing pure ASCII rows), exact 2x4 Braille fill mapping replacing 0.30/0.70/1.0 thresholds, value-gradient color application and saturation rule, width accounting with multi-byte + zero-width escapes, 60x16 small-terminal and baseline/absent/read-failed/idle presentation, statistics-text interaction, and migration of the 12 screen fixtures plus V01–V08 acceptance cases. Record sharpened glossary terms (graph glyph, Braille cell, color grade) without diluting GLOSSARY.md.

Use [Establish what btop actually renders](01-btop-rendering-facts.md), [Establish Braille and color terminal feasibility](02-terminal-color-feasibility.md), and [Prototype the Braille and color graph look](03-graph-prototype.md). Consult `grilling`, `domain-modeling`, and `codebase-design` for the seam. The resolution closes the map only if no fog remains; otherwise graduate newly specifiable tickets and clear them from Not yet specified.
