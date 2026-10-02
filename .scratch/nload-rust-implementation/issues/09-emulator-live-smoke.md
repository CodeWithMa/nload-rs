# Record emulator behavior and nonprivileged live traffic smoke

Status: open
Labels: wayfinder:task
Type: task
Mode: HITL
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 08

## Question

Does direct execution of the release binary in one identified xterm-compatible emulator visibly satisfy T14 and the emulator aspects of V05, and does actual nonprivileged Linux traffic satisfy D04?

Prepare a precise runnable checklist using `./target/release/nload-rs`, actual discovered interface names and independent ceiling overrides. The human must supply observed emulator behavior: stacked layout, prompt navigation, small-terminal startup/shrink/grow and retained-graph recovery; return to the prior screen and a visible cursor after separate q, typed Ctrl+C and external SIGTERM runs. Record emulator identity/version where obtainable, dimensions, environment, actions and actual outcomes rather than inferring success from TERM or PTY escape streams.

Run or guide nonprivileged live discovery/selection/navigation and ordinary generated traffic on a monitored interface. Record ceiling labels, advancing totals and rate/graph activity without treating exact live rates as a numerical oracle. No privileged interface creation/removal/rename is required. Link the resulting records and fix or ticket concrete failures; unperformed observations remain pending. This HITL ticket closes only after the live exchange provides the required observations.
