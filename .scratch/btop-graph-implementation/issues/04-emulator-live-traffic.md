# 04: Record emulator appearance and nonprivileged live traffic

**What to build:** Provide recorded human observations that the implemented Braille graph palettes, statistics contrast, history recovery, and handled-exit restoration work in an actual emulator, together with a nonprivileged real Linux traffic smoke. Prepare a runnable checklist and collect observations; completion requires the human's visual reports.

Authority: the [styled graph contract](../../btop-graph-spec/issues/04-screen-seam-contract.md#acceptance-and-test-migration), especially T14 and emulator aspects of V05, and D04/live-evidence rules in the [minimal Rust port specification](../../nload-rust-port/spec.md).

**Blocked by:** Human emulator observations (not yet supplied). [02: Display paired Braille graph history with direction palettes](02-paired-braille-direction-palettes.md) is closed; its implementation prerequisite is satisfied. Ticket 03 does not gate collecting these observations.

**Status:** blocked / awaiting-human

**Assignee:** implementer-04 (OpenCode); human observations required

- [x] Prepare a precise checklist for the direct release executable using actually discovered interface names, separate incoming/outgoing ceiling overrides, and nonprivileged ordinary generated traffic. Identify the source revision and tested release artifact so observations can be checked for applicability later.
- [ ] Record emulator identity/version where obtainable, font/Braille coverage, UTF-8 and 256-color setup, TERM declaration, dimensions, traffic setup, actions, and actual observations. Do not infer visual success from environment declarations, prototype screenshots, or PTY streams.
- [ ] At both 60×16 and 80×24, observe stacked directions and both positional palettes, medium-dark bottom/pale top contrast, legible default-style statistics/labels, and correct independent ceiling labels. Observe true-blank measured zeros and empty states; use clearly identified practical scripted evidence for states live traffic cannot reliably induce.
- [ ] Observe prompt arrow-key selection, small-terminal startup and shrink/grow, continued sampling/navigation while small, and retained half-cell history recovery on growth. Confirm small-terminal text stays default styled.
- [ ] In separate q, typed Ctrl+C, and external SIGTERM runs, record statuses 0/130/143 plus visible cursor, default styling, normal shell interaction, and prior-screen return after each handled exit.
- [ ] Run or guide nonprivileged live Linux discovery/selection/navigation and ordinary traffic on a monitored interface. Record advancing cumulative totals and rate/graph activity with independent ceilings; do not use exact live rates as a numerical oracle or require privileged interface lifecycle changes.
- [ ] Link the actual observations and map them to T14, V05 emulator coverage, and D04. Fix or record concrete failures and keep unperformed/unclear observations pending; this ticket is complete only when required human observations have been supplied and resolved.

## Assets

- [04 evidence — build, ancestry, actual live rates/totals, applicability and pending cases](../04-evidence.md)
- [04 emulator checklist — exact dimensions, separate ceilings and exit commands](../04-emulator-checklist.md)
- [04 human report template — no observations supplied yet](../04-human-report-template.md)
- [04 live-smoke transcript — actual agent traffic-source output](../04-live-smoke.txt)
- [Review fixes — Standards and Spec evidence, corrected artifact and fresh live smoke](../review-fixes-evidence.md)

## Comments

### Independent work and human blocker — 2026-10-08 — implementer-04 (OpenCode)

Verified clean branch/ancestry at baseline `48584d0`; integration was at the same
revision and no reset was needed. There is no ref named `main`; the supplied main
checkout path is on the integration branch. Loaded required `tdd`; this ticket's
changes are evidence/docs only and reuse the existing traffic generator unchanged,
so no new helper implementation/red-green cycle was needed.

Passed `cargo fmt --check`, `cargo test --locked` (104 unit + 14 PTY/decoder),
`cargo build --release --locked`, and all 14 release-binary PTYs. Identified the
release source/revision/artifact in 04-evidence.md. Actually discovered `docker0`,
`enp1s0`, `lo`; ran nonprivileged ordinary loopback traffic with the direct release
binary at both 60×16 and 80×24 through Linux PTYs. Observed independent 2.00/4.00
MBit/s labels, advancing cumulative totals, numeric zero → low/high → zero idle
rates, Braille activity and q restoration/status 0. These observations establish
real-counter/protocol behavior, not visual palette/font/cleanup success.

Prepared precise human runs and separate 0/130/143 status/restoration reporting.
All criteria requiring human observations remain unchecked, including the
ordinary-emulator portion of D04. Earlier ASCII human evidence cannot establish
styled acceptance. No new human report exists, no closure is claimed, and ticket
04 remains blocked / awaiting-human. Required next action: run the linked checklist
on the identified executable and return the linked report template filled with
actual observations (including failures and skipped/unclear steps).

### Integration and intended-file review — 2026-10-08 — implementer-04 (OpenCode)

After completing independent evidence, `git merge --no-edit
integration/btop-styled-graphs` returned `Already up to date.`; the latest tip was
still `48584d006ea3b3184f56e7bbae24f2254425e946` and is an ancestor of this
branch. Reviewed status, diff and the last ten commits before staging the five
intended ticket-04 files. All new assets are under this implementation effort.
Full staged diff review and `git diff --cached --check` passed.
The checked-in scope is documentation/evidence/tracker metadata; no executable
delta invalidates the recorded gates/artifact. Human evidence remains pending.

### Review applicability follow-up — 2026-10-08 — review-fixes implementer (OpenCode)

The graph correction at `5e4661164f7afda85c266b2a45ce730276c71bad` changes the
release artifact. Earlier ticket-04 source/hash/live results retain historical
provenance; fresh corrected-artifact fmt/locked tests/release build and all 16
release PTYs passed. Ordinary nonprivileged UDP smoke was rerun at 60×16/80×24
with independent 2/4 MBit/s labels, discovered selection/all-arrow navigation,
advancing totals, colored graphs/all twelve indices, zero idle and clean q/OS
restoration. See the linked review evidence and appended evidence follow-up.

The runnable checklist and blank human report now use the permanent integration
checkout `/mnt/programming/rust/nload-rs` and durable corrected executable
`target/btop-review-fixes/release/nload-rs`, identifying its source separately
from integration HEAD. The disposable ticket-04 worktree is no longer required.
**Human T14, V05-emulator and ordinary-emulator D04 remain mandatory and pending.**
Status stays **blocked / awaiting-human**; final delivery is blocked by these
observations. No agent PTY or live result is counted as a visual report.
