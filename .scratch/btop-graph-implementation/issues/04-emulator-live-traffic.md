# 04: Record emulator appearance and nonprivileged live traffic

**What to build:** Provide recorded human observations that the implemented Braille graph palettes, statistics contrast, history recovery, and handled-exit restoration work in an actual emulator, together with a nonprivileged real Linux traffic smoke. Prepare a runnable checklist and collect observations; completion requires the human's visual reports.

Authority: the [styled graph contract](../../btop-graph-spec/issues/04-screen-seam-contract.md#acceptance-and-test-migration), especially T14 and emulator aspects of V05, and D04/live-evidence rules in the [minimal Rust port specification](../../nload-rust-port/spec.md).

**Blocked by:** None. [02: Display paired Braille graph history with direction palettes](02-paired-braille-direction-palettes.md) is closed; mandatory human observations are supplied and resolved.

**Status:** closed

**Assignee:** implementer-04 (OpenCode)

- [x] Prepare a precise checklist for the direct release executable using actually discovered interface names, separate incoming/outgoing ceiling overrides, and nonprivileged ordinary generated traffic. Identify the source revision and tested release artifact so observations can be checked for applicability later.
- [x] Record emulator identity/version where obtainable, font/Braille coverage, UTF-8 and 256-color setup, TERM declaration, dimensions, traffic setup, actions, and actual observations. Do not infer visual success from environment declarations, prototype screenshots, or PTY streams.
- [x] At both 60×16 and 80×24, observe stacked directions and both positional palettes, medium-dark bottom/pale top contrast, legible default-style statistics/labels, and correct independent ceiling labels. Observe true-blank measured zeros and empty states; use clearly identified practical scripted evidence for states live traffic cannot reliably induce.
- [x] Observe prompt arrow-key selection, small-terminal startup and shrink/grow, continued sampling/navigation while small, and retained half-cell history recovery on growth. Confirm small-terminal text stays default styled.
- [x] In separate q, typed Ctrl+C, and external SIGTERM runs, record statuses 0/130/143 plus visible cursor, default styling, normal shell interaction, and prior-screen return after each handled exit.
- [x] Run or guide nonprivileged live Linux discovery/selection/navigation and ordinary traffic on a monitored interface. Record advancing cumulative totals and rate/graph activity with independent ceilings; do not use exact live rates as a numerical oracle or require privileged interface lifecycle changes.
- [x] Link the actual observations and map them to T14, V05 emulator coverage, and D04. Fix or record concrete failures and keep unperformed/unclear observations pending; this ticket is complete only when required human observations have been supplied and resolved.

## Assets

- [04 evidence — current completion mappings and historical build/live preparation](../04-evidence.md)
- [04 emulator checklist — exact dimensions, separate ceilings and exit commands](../04-emulator-checklist.md)
- [04 human report template — reusable prompts for repeat runs](../04-human-report-template.md)
- [Human emulator observations — actual report and “All observed; font unknown” follow-up](../04-human-observations.md)
- [04 live-smoke transcript — historical pre-correction agent traffic-source output](../04-live-smoke.txt)
- [Review fixes — Standards and Spec evidence, corrected artifact and fresh live smoke](../review-fixes-evidence.md)

## Comments

Comments preceding the resolution record historical preparation and then-pending
human evidence. Their pending statuses are superseded by the resolution below.

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

### Resolution — 2026-10-08 — implementer-04 (OpenCode)

All seven criteria are satisfied. Resumed on clean `implement/btop-04` at
`282dc3b`, merging integration **before work** to `2ee6d8d` (fast-forward). Read
the durable [Human emulator observations](../04-human-observations.md) and explicit
**“All observed; font unknown”** follow-up. The report applies to corrected source
`5e4661164f7afda85c266b2a45ce730276c71bad`, identified at human checkout `3765b31`
with an empty executable-source diff and artifact SHA-256
`d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`.
Closure checks confirmed no executable-source delta on this branch, matching
hashes and byte identity of the permanent normal and review release executables.

Alacritty 0.17.0 human observations at **60×16 and 80×24** confirm both positional
palettes, legible bottom/pale top shades, clean single-cell Braille, default-style
text/statistics, independent ceilings, true-blank idle positions, prompt navigation,
small-window sampling/history recovery and separate handled exits. Actual q,
physical Ctrl+C and external SIGTERM child statuses are **0/130/143**; the
follow-up confirms visible cursor, default foreground/background, normal shell
interaction and prior-screen return **after every exit**.

Run A's actual small q size was **56×17**, a valid below-width-floor substitute
for the suggested 59×16. Run C's small startup/growth is confirmed without an
exact small-size transcript. Exact 59×16/60×15 boundaries, half-cell arithmetic
and baseline/absent/read-failed/recovery states use explicitly mapped passing
deterministic/PTY fixtures, not invented human observations. Configured font,
client OS/theme name are unavailable; actual glyph coverage and appearance were
confirmed. Human traffic comparisons are qualitative; actual corrected-artifact
rate/total checkpoints and proc deltas are linked in the review live-smoke record.

See [current acceptance and applicability](../04-evidence.md#current-acceptance-and-applicability--2026-10-08)
for all criterion/T14/V05/D04 mappings and evidence limits. Corrected fmt/locked
test/release-build gates (**115 unit + 16 PTY/decoder**, **16 release PTYs**) remain
valid on the same artifact. Closure changes are documentation only; the already
loaded `tdd` guidance requires no new implementation cycle or repeated gates.
No unresolved ticket-04 failures or human-evidence blockers remain. **Ticket 04
is closed.**

Closure documentation review: inspected status/full diff/recent ten commits;
`git diff --check` passed, and evidence links/resolution anchors plus named
boundary fixtures were checked. Latest pre-commit integration merge returned
`Already up to date.` at `2ee6d8d`; its ancestry check passed. The intended
closure files are this ticket, 04-evidence/checklist/template and the
implementation record's named resolution/evidence pointers.
