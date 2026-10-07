# Prototype the Braille and color graph look

Status: open
Labels: wayfinder:prototype
Type: prototype
Mode: HITL
Parent: [Find the way to a btop-style graph spec for nload-rs](../map.md)
Assignee:
Blocked by: 01, 02

## Question

What should the replacement graphs look like to the human? Use a cheap throwaway mockup (HTML or terminal capture, not application code) showing representative histories in three forms: current ASCII `. | #`, same-geometry Braille + value gradient, and higher-resolution 2x4 Braille + value gradient. Include saturation, measured idle zeros, baseline/unavailable empty states, and incoming/outgoing sections at 80x24 and below 60x16. Get the human's reaction to dot density, gradient steps, and readability; link the prototype rather than building the application.

Use the confirmed btop facts from [Establish what btop actually renders](01-btop-rendering-facts.md) and feasibility limits from [Establish Braille and color terminal feasibility](02-terminal-color-feasibility.md). Decide only the accepted visual direction; exact thresholds, palette codes, and seam mechanics belong to the contract ticket.
