# Record emulator behavior and nonprivileged live traffic smoke

Status: closed
Labels: wayfinder:task
Type: task
Mode: HITL
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: emulator and live traffic smoke)
Blocked by: 08

## Question

Does direct execution of the release binary in one identified xterm-compatible emulator visibly satisfy T14 and the emulator aspects of V05, and does actual nonprivileged Linux traffic satisfy D04?

Prepare a precise runnable checklist using `./target/release/nload-rs`, actual discovered interface names and independent ceiling overrides. The human must supply observed emulator behavior: stacked layout, prompt navigation, small-terminal startup/shrink/grow and retained-graph recovery; return to the prior screen and a visible cursor after separate q, typed Ctrl+C and external SIGTERM runs. Record emulator identity/version where obtainable, dimensions, environment, actions and actual outcomes rather than inferring success from TERM or PTY escape streams.

Run or guide nonprivileged live discovery/selection/navigation and ordinary generated traffic on a monitored interface. Record ceiling labels, advancing totals and rate/graph activity without treating exact live rates as a numerical oracle. No privileged interface creation/removal/rename is required. Link the resulting records and fix or ticket concrete failures; unperformed observations remain pending. This HITL ticket closes only after the live exchange provides the required observations.

## Comments

### Preparation — 2026-10-02 — alice

Claimed after confirming the automated product PTY prerequisite is closed.
Prepared the [Emulator and live traffic smoke checklist and record](../emulator-live-smoke.md)
with actual discovered interfaces, independent ceiling labels, all-four-arrow
navigation, real-window small startup/shrink/grow/history checks, and separate
q/typed Ctrl+C/external SIGTERM runs. It includes an observation table awaiting
the human's reports and a targeted fifteen-second SIGTERM launcher.

Added the [Nonprivileged loopback traffic source](../loopback-traffic.py), using
ordinary local UDP sockets and low/high/idle phases. A three-second agent check
at UID/EUID 1000/1000 succeeded and advanced actual `lo` RX/TX proc counters by
1,315,200 bytes each. The release build and version check pass; the release hash
matches the earlier PTY-tested executable.

[Emulator/live preparation evidence](../verification.md#emulator-and-live-smoke-preparation--2026-10-02)
records the revision/environment and evidence limits. Agent standard streams
are nonterminal and the emulator is not visible here. **V05/T14 emulator and
D04 application live observations remain pending.** The ticket stays open and
claimed for the live exchange; final acceptance remains blocked on it.

### Human checkpoint — 2026-10-02

The human selected **“Not run yet”** when asked for the checklist observations.
No emulator or application live-smoke result has been supplied. Keep the ticket
open, the observation table pending, and final acceptance blocked until the
required live exchange records actual outcomes.

### Human observations — 2026-10-02

The human ran all three checklist runs in Alacritty `0.17.0 (94e7c887)` over an
SSH terminal `/dev/pts/3`, initially 86x22, shrinking to 92x8 and starting Run B
at 92x7. They report successful layout/navigation/ceiling labels, actual loopback
traffic, warning-mode navigation and older-history recovery on growth. Separate
q/typed Ctrl+C/external SIGTERM statuses are 0/130/143; visible cursor and normal
typing work after each. The sixty-second nonprivileged traffic run advanced both
proc totals by 26,057,400 bytes, with UI total 26.10 MByte.

Full outcomes and exact traffic output are recorded in the
[Emulator and live traffic smoke checklist and record](../emulator-live-smoke.md#observations-to-return).
Two details await clarification: explicit prior-screen return on all three exits,
and whether the reported maximum `797 MBit/s` omitted a decimal. Preserve that
observation as reported; do not infer either a defect or a corrected value.
The ticket remains open until those clarifications arrive.

### Resolution — 2026-10-02 — alice

The human completed the emulator/live checklist and supplied both clarifications:
the displayed maximum was **7.97 MBit/s** (a decimal omitted from the original
notes), and the prior shell screen returned after **all three** handled exits.
Visible cursor and normal typing also work on each exit; q, typed Ctrl+C and
external SIGTERM report the required **0/130/143** statuses.

[Recorded emulator and live smoke](../verification.md#recorded-emulator-and-live-smoke--2026-10-02)
maps passing **V05 E, T14 E and D04 L** evidence to the human's actual Alacritty
`0.17.0 (94e7c887)` SSH session, 86x22 full size, 92x8 small quit and 92x7 small
startup. Layout/selection/navigation and independent 20.00/5.00 MBit/s ceiling
labels pass; sampling/navigation continue while small, growth reveals older
traffic, and nonprivileged loopback traffic produces advancing totals/rates/graphs.

The [Emulator and live traffic smoke checklist and record](../emulator-live-smoke.md#observations-to-return)
holds the detailed actual environment, supplied traffic transcript, observations
and clarifications. It preserves the limits of the report: exact grown dimensions,
client OS, rate direction and separate before/after UI numeric totals were not
supplied. These do not prevent the required behavioral evidence from passing;
live rates are not treated as an exact numerical oracle.

Source revision remains `c018e0279a66df8121195a449bcd0f63c03496ee`; the release
hash is unchanged from the earlier passing product PTY run. No production defect,
corrective ticket or new specifiable fog surfaced. Source/dependencies are
unchanged; README now accurately reflects passing emulator/live verification.
No commit was made.

This closes the HITL ticket and unblocks
[Audit final acceptance evidence and the implementation handoff](10-final-acceptance.md).
Final gates/artifact/revision review and the parent map's completion remain pending.
