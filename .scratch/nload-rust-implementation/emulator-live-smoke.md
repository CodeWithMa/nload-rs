# Emulator and live traffic smoke checklist and record

Ticket: [Record emulator behavior and nonprivileged live traffic smoke](issues/09-emulator-live-smoke.md).
Acceptance: V05/T14 (visible emulator behavior), D04 (nonprivileged live Linux).
Status: **Passed — human observations and clarifications recorded.** The runs
below are the checklist; actual outcomes are in the observation table and report.

## Recorded preparation — 2026-10-02

- Source revision: `c018e0279a66df8121195a449bcd0f63c03496ee`; clean worktree
  before the ticket claim. This session adds tracker/checklist/traffic assets only.
- `cargo build --release --locked`: exit 0; the existing unused Session
  convenience-method warning remains. `./target/release/nload-rs --version`:
  exit 0, `nload-rs 0.1.0` followed by a newline.
- Release executable SHA-256:
  `7ad245c475a9a29c3abac31b8f7e69d8ef6c06b216d20a9cebc21a4fefd998c1`,
  matching the prior product PTY run. `git diff --check`: exit 0.
- rustc `1.98.1` (`48a229ceaefd4985c50990b14116b6d856af0985`), LLVM 21.1.8;
  Cargo `1.98.0` (`797e8a9bc`); target `x86_64-unknown-linux-gnu`.
- Linux `7.2.8`, x86_64, glibc 2.42; Python `3.14.7`; UID/EUID `1000/1000`.
- Accessible `/proc/net/dev` names: `lo`, `enp1s0`, `docker0`;
  expected natural startup order: `docker0 (1/3)`, `enp1s0 (2/3)`, `lo (3/3)`.
  Initial sampled counters: lo RX/TX `0/0`; enp1s0 RX/TX
  `130761829/215875141`; docker0 RX/TX `0/0` bytes. These are historical
  counters, not expected future totals. If membership changes before running,
  report the actual list/positions instead of forcing these expectations.
- Agent stdin/stdout are nonterminal. `TERM=alacritty`, `SSH_TTY=/dev/pts/0`,
  `LANG=en_US.UTF-8`; LC_ALL, TMUX, STY, DISPLAY, WAYLAND_DISPLAY,
  TERM_PROGRAM and TERM_PROGRAM_VERSION unset. Alacritty executable unavailable
  here. This does not identify or visually verify the client emulator.

## Run setup

Use two terminal tabs/windows connected to the **same Linux network namespace**,
both at repository root `/mnt/programming/rust/nload-rs`. Run directly in an
xterm-compatible emulator. Record any SSH or terminal-multiplexer layers.
Keep terminal A for the UI and terminal B for traffic/dimension queries.

In terminal A, before launching:

```sh
python3 -c 'import os; print({k: os.environ.get(k) for k in ("TERM", "COLORTERM", "TERM_PROGRAM", "TERM_PROGRAM_VERSION", "LANG", "LC_ALL", "TMUX", "STY", "SSH_TTY")})'
tty
stty size
```

`stty size` reports **rows then columns**; the checklist uses **width x height**.
Tell us the client emulator name/version (for Alacritty, run `alacritty --version`
on the machine actually running it, if available). Keep its real TERM declaration.
Resize the actual emulator window for size checks; changing only `stty rows/cols`
would not establish visible rendering at that size. To measure terminal A during
a run, use `stty -F /dev/pts/N size` in terminal B, substituting the path from A's
`tty`. Record actual dimensions even if slightly different from the targets.

## Run A — default discovery, live traffic, navigation, shrink/grow, q

1. Make terminal A approximately **80x24** (at least 60x16). Leave recognizable
   shell text on the prior screen, then launch:

   ```sh
   ./target/release/nload-rs -i 20480 -o 5120
   ```

2. Record the initial header: expected `Interface docker0 (1/3)` for today's
   discovered list. After warm-up, check the stacked incoming/outgoing graphs,
   adjacent right-hand `Curr`, `Avg`, `Min`, `Max`, `Ttl` fields, and no footer,
   wrapping or stray rows. At 80x24, each graph is 50 columns by 10 rows.
   Check exact direction labels **Incoming: (ceiling 20.00 MBit/s)** and
   **Outgoing: (ceiling 5.00 MBit/s)**.
3. Press **Right, Down, Right, Up, Left, Down**, one at a time. Expected headers:
   `enp1s0 (2/3)`, `lo (3/3)`, `docker0 (1/3)`, `lo (3/3)`, `enp1s0 (2/3)`,
   `lo (3/3)`. Report whether each change looks prompt, and confirm ceilings stay
   the same. This ends on `lo`.
4. In terminal B, run the ordinary, local-only traffic source (no sudo):

   ```sh
   python3 .scratch/nload-rust-implementation/loopback-traffic.py
   ```

   It sends/receives IPv4 UDP between ordinary sockets on `127.0.0.1`, using
   an OS-assigned port. Six ten-second phases alternate **low/high/idle** twice.
   Low/high payload pacing is approximately 256 KiB/s and 1 MiB/s; actual
   interface accounting/rates differ and are not an exact numerical oracle.
   Keep the generator output for the record.
5. While on `lo`, note before/after incoming/outgoing `Ttl` values and numeric
   `Curr`/graph activity. At these ceilings, high traffic should make incoming
   bars visibly taller and outgoing saturate; idle later lowers the current
   rate. Report actual displayed text and activity, not a guessed pass.
6. After roughly 20 seconds, briefly navigate away and back. Confirm the earlier
   low/high history remains. Shrink the real window below either floor
   (target **59x16**, or **60x15**). Check warning/selection/status only as fits;
   no wrapped full-size graphs. While small, press Right then Left to leave and
   return to `lo`; keep traffic running for several seconds.
7. Grow back to the original size. Check stacked layout recovery, the earlier
   low/high history still present, newer history from the small period, and
   advancing totals. A temporary narrower *full-size* width (e.g. 60x24) followed
   by regrowth can also make the hiding/revealing of older columns easy to see.
   Describe what actually reappears; resizing itself is not evidence of retention.
8. Shrink below the floor again and press **q** to establish quit while small.
   Immediately run:

   ```sh
   smoke_status=$?; printf 'q exit status: %s\n' "$smoke_status"
   stty size
   ```

   Expected status **0**. Observe whether the recognizable prior shell screen
   returns, the cursor is visible, and normal typing/echo works.

## Run B — small startup, explicit selection, growth, typed Ctrl+C

1. With A still genuinely small (record its actual dimensions), launch:

   ```sh
   ./target/release/nload-rs -i 20480 -o 5120 lo
   ```

2. Check valid warning-mode startup and initial selection `lo (3/3)` where it
   fits. Restart the traffic command in B if its earlier run has ended. Let it
   run small for about 12 seconds; press Right then Left and check navigation.
3. Grow A to at least 60x16, preferably 80x24. Check recovered full layout,
   unchanged independent ceilings, measured rates/totals, and traffic history
   accumulated during small startup rather than an empty new graph.
4. Type **Ctrl+C** in A (a physical keypress), then immediately run:

   ```sh
   smoke_status=$?; printf 'typed Ctrl+C exit status: %s\n' "$smoke_status"
   ```

   Expected status **130**. Report prior-screen return, cursor visibility and
   normal typing/echo independently of Run A.

## Run C — separate external SIGTERM and visible cleanup

At full size in A, this wrapper inherits the real terminal streams and directly
starts the same release executable. It sends SIGTERM to **that child only** after
15 seconds; the delay is just time to observe the active UI.

```sh
python3 -c 'import signal, subprocess, time; print("SIGTERM smoke: prior-screen marker; termination after 15 seconds", flush=True); p = subprocess.Popen(["./target/release/nload-rs", "-i", "20480", "-o", "5120", "lo"]); time.sleep(15); p.send_signal(signal.SIGTERM); print("SIGTERM child exit status:", p.wait(), flush=True)'
```

Do not type q or Ctrl+C during this run. Expected **child** status **143** (the
Python wrapper's status is separate). Check marker/prior-screen return, cursor
visibility and normal typing/echo. Record any diagnostics or unusual behavior.

## Observations to return

Short factual answers are enough; screenshots are optional supporting assets.
Any unperformed or ambiguous item stays pending.

The initial human checkpoint on 2026-10-02 was **“Not run yet.”** The human
subsequently supplied the following actual results from the checklist, then
confirmed the maximum was **7.97 MBit/s** and prior-screen return worked on **all
three** exits. The human's aggregate “All OK”/“all good” answers confirm the
corresponding checklist conditions, rather than establishing additional exact
measurements. Client OS and exact grown dimensions were not supplied; growth
and recovered full layout/history were explicitly reported as successful.

| Item | Actual observation |
| --- | --- |
| Emulator name/version; client OS; SSH/multiplexer layers | `alacritty 0.17.0 (94e7c887)`; SSH session declared by SSH_TTY; TMUX/STY unset. Client OS not supplied. |
| A environment output and tty path | TERM=`alacritty`, LANG=`en_US.UTF-8`, SSH_TTY=`/dev/pts/3`, tty=`/dev/pts/3`; COLORTERM, TERM_PROGRAM, TERM_PROGRAM_VERSION, LC_ALL, TMUX, STY unset. |
| Full/small/start-small/grown dimensions | Initial `stty size`=`22 86`: **86x22**. After q, `8 92`: **92x8**. Run B startup `7 92`: **92x7**. Growth reported successful; exact grown dimensions not supplied. |
| Default header; explicit `lo` header | Run A steps 1–3 “All OK”; Run B steps 1–3 “all good”, confirming default discovery and explicit `lo` selection per checklist. |
| All-four-arrow sequence and prompt response; stable ceilings | Run A steps 1–3 “All OK”, including the specified Right/Down/Right/Up/Left/Down selection sequence and unchanged independent 20.00/5.00 MBit/s ceiling labels. |
| Stacked layout/fields/labels; wrapping or stray rows | Run A steps 1–3 “All OK” at the reported full size. No wrapping/stray-row failure reported. |
| Traffic helper output; before/after UI totals; rates/bars | Actual helper output below; proc totals each 1,315,200 → 27,372,600 bytes. UI Avg approximately 900 kBit/s, Min 0, Max **7.97 MBit/s** (confirmed correction of the omitted decimal), Ttl 26.10 MByte; direction not specified. Run A step 5 “looks good”, confirming live totals/rate/bar activity per checklist; exact separate before/after UI values not supplied. |
| Shrink warning; navigation while small; regrowth/retained history | “history still there, warning shows and navigation on warning works”; “making window larger shows older traffic”. |
| Small startup; sampling/history while small; growth | Run B starts 92x7; steps 1–3 “all good”, confirming warning-mode startup, explicit selection/navigation, continued sampling/history and full-layout recovery per checklist. |
| q status; prior screen; visible cursor; typing/echo | Status **0**, ending at 92x8; “normal typing cursor all works”. Human confirmed prior shell screen returned. |
| Typed Ctrl+C status; prior screen; visible cursor; typing/echo | Status **130**; “cursor and typing works”. Human confirmed prior shell screen returned. |
| External SIGTERM child status; prior screen; visible cursor; typing/echo | Wrapper reports child status **143**; “cursor typing is fine”. Human confirmed prior shell screen returned. |
| Failures, diagnostics, unperformed steps | No failure or skipped step reported. Both clarification questions answered; no corrective application work surfaced. |

### Human's live traffic output — 2026-10-02

Command: `python3 .scratch/nload-rust-implementation/loopback-traffic.py`.
Actual supplied output:

```text
uid/euid=1000/1000; interface=lo; IPv4 UDP 127.0.0.1
Before: RX=1315200 TX=1315200 bytes
Phase: low, 10 seconds
Phase: high, 10 seconds
Phase: idle, 10 seconds
Phase: low, 10 seconds
Phase: high, 10 seconds
Phase: idle, 10 seconds
Elapsed: 60.002 seconds; received datagrams=3170
After: RX=27372600 TX=27372600 bytes
Observed proc delta: RX=26057400 TX=26057400 bytes
Proc deltas include any other loopback traffic; payload/pace are not exact UI-rate expectations.
```

Human supplied UI notes: `avg: ~900 kBit/s`, `min: 0`, initially `max: 797 MBit/s`,
`ttl: 26.10 MByte`. On clarification the human selected **“7.97 MBit/s”**,
confirming the notes omitted the decimal. Live rates are observations, not an
exact numerical oracle.
The cumulative proc count 27,372,600 bytes is consistent with 26.10 binary MByte.
The supplied SIGTERM transcript includes the prior-screen marker and child
status 143. Asked whether the prior shell screen returned on every separate
q/typed Ctrl+C/external SIGTERM exit, the human answered **“Yes, all three.”**
This supplies actual visible restoration evidence independently of cursor/typing.

### Acceptance outcome

- **V05 E passes:** warning-mode startup at 92x7, shrink below the height floor,
  navigation/continued sampling while small, quit at 92x8, and successful
  regrowth with full layout and retained history.
- **T14 E passes:** identified Alacritty 0.17.0 session, visible stacked layout,
  prompt navigation, graph recovery, and prior-screen/visible-cursor restoration
  after separate q/typed Ctrl+C/external SIGTERM runs. Their statuses are 0/130/143.
- **D04 L passes:** direct release execution against actual proc interfaces,
  discovery/explicit selection/navigation, independent ceiling labels, and
  nonprivileged ordinary loopback traffic with advancing totals/rate/graph activity.

Evidence is the human's live exchange and supplied command output, not inference
from TERM or a PTY stream. The release binary remains SHA-256
`7ad245c475a9a29c3abac31b8f7e69d8ef6c06b216d20a9cebc21a4fefd998c1`
on source revision `c018e0279a66df8121195a449bcd0f63c03496ee`; no production
code changed during these checks. Final artifact/revision acceptance belongs to
[Audit final acceptance evidence and the implementation handoff](issues/10-final-acceptance.md).

## Agent traffic-source check

Ran from the repository root without privileges:

```sh
python3 .scratch/nload-rust-implementation/loopback-traffic.py --phase-seconds 0.5
```

Exit **0**. Actual output:

```text
uid/euid=1000/1000; interface=lo; IPv4 UDP 127.0.0.1
Before: RX=0 TX=0 bytes
Phase: low, 0.5 seconds
Phase: high, 0.5 seconds
Phase: idle, 0.5 seconds
Phase: low, 0.5 seconds
Phase: high, 0.5 seconds
Phase: idle, 0.5 seconds
Elapsed: 3.002 seconds; received datagrams=160
After: RX=1315200 TX=1315200 bytes
Observed proc delta: RX=1315200 TX=1315200 bytes
Proc deltas include any other loopback traffic; payload/pace are not exact UI-rate expectations.
```

This establishes a working nonprivileged source of actual live proc activity on
`lo`. The agent helper check alone established no emulator/application UI pass;
the subsequent human report and clarifications above supply the passing emulator
and application live evidence.
