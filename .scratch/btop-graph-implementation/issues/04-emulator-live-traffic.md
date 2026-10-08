# 04: Record emulator appearance and nonprivileged live traffic

**What to build:** Provide recorded human observations that the implemented Braille graph palettes, statistics contrast, history recovery, and handled-exit restoration work in an actual emulator, together with a nonprivileged real Linux traffic smoke. Prepare a runnable checklist and collect observations; completion requires the human's visual reports.

Authority: the [styled graph contract](../../btop-graph-spec/issues/04-screen-seam-contract.md#acceptance-and-test-migration), especially T14 and emulator aspects of V05, and D04/live-evidence rules in the [minimal Rust port specification](../../nload-rust-port/spec.md).

**Blocked by:** [02: Display paired Braille graph history with direction palettes](02-paired-braille-direction-palettes.md). Ticket 03 does not gate collecting these observations.

**Status:** ready-for-agent

**Assignee:** implementer-04 (OpenCode); human observations required

- [ ] Prepare a precise checklist for the direct release executable using actually discovered interface names, separate incoming/outgoing ceiling overrides, and nonprivileged ordinary generated traffic. Identify the source revision and tested release artifact so observations can be checked for applicability later.
- [ ] Record emulator identity/version where obtainable, font/Braille coverage, UTF-8 and 256-color setup, TERM declaration, dimensions, traffic setup, actions, and actual observations. Do not infer visual success from environment declarations, prototype screenshots, or PTY streams.
- [ ] At both 60×16 and 80×24, observe stacked directions and both positional palettes, medium-dark bottom/pale top contrast, legible default-style statistics/labels, and correct independent ceiling labels. Observe true-blank measured zeros and empty states; use clearly identified practical scripted evidence for states live traffic cannot reliably induce.
- [ ] Observe prompt arrow-key selection, small-terminal startup and shrink/grow, continued sampling/navigation while small, and retained half-cell history recovery on growth. Confirm small-terminal text stays default styled.
- [ ] In separate q, typed Ctrl+C, and external SIGTERM runs, record statuses 0/130/143 plus visible cursor, default styling, normal shell interaction, and prior-screen return after each handled exit.
- [ ] Run or guide nonprivileged live Linux discovery/selection/navigation and ordinary traffic on a monitored interface. Record advancing cumulative totals and rate/graph activity with independent ceilings; do not use exact live rates as a numerical oracle or require privileged interface lifecycle changes.
- [ ] Link the actual observations and map them to T14, V05 emulator coverage, and D04. Fix or record concrete failures and keep unperformed/unclear observations pending; this ticket is complete only when required human observations have been supplied and resolved.
