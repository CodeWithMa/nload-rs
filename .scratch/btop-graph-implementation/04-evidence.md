# Ticket 04 — live smoke and emulator evidence

Recorded 2026-10-08 by implementer-04 (OpenCode), worktree
`/tmp/opencode/nload-btop-04`, branch `implement/btop-04`.

**Status: blocked / awaiting human.** Independent preparation, build gates and
nonprivileged real-binary traffic observations are complete. No human observations
of the styled implementation have been supplied. T14 and the emulator aspects of
V05 remain pending; D04 has agent live-counter/protocol evidence and a runnable
emulator procedure, but its ordinary-emulator observation remains pending.

Authority: [Record emulator appearance and nonprivileged live traffic](issues/04-emulator-live-traffic.md), the
[canonical styled graph contract](../btop-graph-spec/issues/04-screen-seam-contract.md#acceptance-and-test-migration),
and inherited [port acceptance/evidence rules](../nload-rust-port/spec.md#10-acceptance-matrix-and-verification-procedure).
The newer styled contract supersedes affected ASCII presentation promises.

**Current applicability (2026-10-08 review follow-up):** Sections below record
the original ticket-04 source/artifact and observations. The graph logic has since
changed at `5e4661164f7afda85c266b2a45ce730276c71bad`; those old release/live
runs are historical evidence, not validation of the corrected artifact. Current
gates and fresh live smoke are linked in the appended follow-up. The checklist
and blank human template now identify the corrected durable artifact. All human
observations remain pending.

## Ancestry and scope

- Initial `git status --short --branch`: clean, `implement/btop-04`.
- HEAD, `implement/btop-04`, integration and requested baseline all resolved to
  `48584d006ea3b3184f56e7bbae24f2254425e946`.
- Both `git merge-base --is-ancestor 48584d0 HEAD` and
  `git merge-base --is-ancestor integration/btop-styled-graphs HEAD` exited 0.
  No reset was needed.
- There is no Git branch/ref named `main`. The supplied main checkout path
  `/mnt/programming/rust/nload-rs` is on `integration/btop-styled-graphs` at the
  baseline. This was established with `git worktree list`, `git branch --list`
  and `git -C /mnt/programming/rust/nload-rs symbolic-ref --short HEAD`.
- Tested source revision is the full baseline above. Ticket 02's implementation
  is already present. Ticket 04 introduces documentation/evidence/tracker files
  only, with no executable delta from that source. Final integration reconciliation
  is recorded in the ticket's comments.

### Test-first applicability

Loaded the required `tdd` skill and read AGENTS.md, tracker/domain guidance,
GLOSSARY.md, the contract and previous evidence. No reusable helper or application
code was added or modified, so there was no new implementation seam requiring
confirmation or red/green implementation cycle. The reusable checklist uses the existing
[loopback traffic generator](../nload-rust-implementation/loopback-traffic.py)
unchanged. A one-off shell-invoked process/PTY observation collected the actual
measurements below; it is operational evidence collection, not a new checked-in
helper or numerical test oracle. Existing meaningful tests were run as gates.

## Build, environment and artifact

All following commands ran at this worktree root, exiting 0:

| Command | Actual result |
| --- | --- |
| `cargo fmt --check` | Passed. |
| `cargo test --locked` | 104 unit tests + 14 Linux PTY/decoder tests passed; no failures/ignored tests. |
| `cargo build --release --locked` | Passed; optimized `target/release/nload-rs`. |
| `NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty` | All 14 passed against the release executable. |
| `./target/release/nload-rs --version` | `nload-rs 0.1.1` (actual Cargo metadata, rather than the port spec's initial 0.1.0). |
| `git diff 48584d0 -- src tests Cargo.toml Cargo.lock` | Empty at the time of these runs. |

Artifact: `/tmp/opencode/nload-btop-04/target/release/nload-rs`, SHA-256
`50d9a750738bf4381eabd6fb72a96967098bdd6ba90067f3d2f255c5da176746`.
This matches ticket 02's recorded artifact; it is an identity check, not a
byte-identical rebuild guarantee. The optional `file` utility was unavailable
(`command not found`); the release executable was actually executed successfully.
Final whitespace/status/commit review is recorded with integration reconciliation
in the ticket comments.

- Linux `7.2.9`, `x86_64-unknown-linux-gnu`, glibc `2.44`; UID/EUID `1000/1000`.
- rustc `1.98.1 (48a229cea 2026-09-01) (built from a source tarball)`;
  full compiler commit `48a229ceaefd4985c50990b14116b6d856af0985`, LLVM `21.1.8`.
- Cargo `1.98.0 (797e8a9bc 2026-08-05)`; Python `3.14.7`.
- `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `CARGO_BUILD_TARGET` unset.
- Agent stdin/stdout are **not terminals**. Agent environment declares
  `TERM=alacritty`, `LANG=en_US.UTF-8`, `SSH_TTY=/dev/pts/3`;
  LC_ALL, NO_COLOR, TERM_PROGRAM, TERM_PROGRAM_VERSION, TMUX, STY, DISPLAY and
  WAYLAND_DISPLAY unset. These declarations do not establish the emulator's
  identity/version, font, colors or visible behavior. Those fields remain pending.
- The allocated smoke PTYs declared `TERM=xterm-256color` and inherited LANG;
  no actual emulator was attached to the captured stream.

## Discovery and ordinary generated traffic

Accessible `/proc/net/dev` discovery actually returned `lo`, `enp1s0`, `docker0`.
Initial independent read (before gates/traffic) was:

| Interface | RX bytes | TX bytes |
| --- | ---: | ---: |
| lo | 1,649,600 | 1,649,600 |
| enp1s0 | 48,482,626 | 77,763,980 |
| docker0 | 0 | 0 |

These are observations, not future expected counters. The real binary at 60×16
started with `Interface docker0 (1/3)`. Sending Right, Down, Right, Up, Left, Down
one at a time produced, respectively, `enp1s0 (2/3)`, `lo (3/3)`,
`docker0 (1/3)`, `lo (3/3)`, `enp1s0 (2/3)`, `lo (3/3)` in collected frames.
Each header was available within the 80-ms collection interval; this is a
protocol observation, not a human judgment of perceived responsiveness.
The separate 80×24 run explicitly selected `lo (3/3)` at startup.

Actual executable invocations:

```sh
./target/release/nload-rs -i 2048 -o 4096
./target/release/nload-rs -i 2048 -o 4096 lo
```

Each had a real allocated Linux PTY for terminal stdin/stdout, redirected stderr,
and its specified 60×16 or 80×24 dimensions. After an idle warm-up, a subprocess
in the same network namespace ran:

```sh
python3 .scratch/nload-rust-implementation/loopback-traffic.py --phase-seconds 2
```

The existing generator used ordinary IPv4 UDP sockets on `127.0.0.1`, an
OS-assigned port, 8192-byte payloads, receiving each datagram. It alternated
low/high/idle twice (32/128/0 datagrams per second, two seconds per phase).
Both executions returned 0, with empty stderr, 635 received datagrams and
12.001 seconds reported. Full actual generator output is retained in
[04 live-smoke transcript](04-live-smoke.txt).

| Run | Actual generator RX/TX before | Actual generator RX/TX after | Observed delta each direction |
| --- | ---: | ---: | ---: |
| 60×16 | 2,177,472 | 7,397,172 | 5,219,700 bytes |
| 80×24 | 7,397,172 | 12,616,872 | 5,219,700 bytes |

Independent ceilings remained `Incoming: (ceiling 2.00 MBit/s)` and
`Outgoing: (ceiling 4.00 MBit/s)` in both runs. Below are actual **displayed**
values from the last complete frame collected about 1.75 seconds into each
phase. RX/TX displayed statistics happened to be equal on this loopback workload;
that does not imply coupled direction calculations. Exact live rates are not an
oracle, and proc deltas include any concurrent loopback traffic.

| Run / checkpoint | Curr (both) | Avg (both) | Ttl (both) |
| --- | --- | --- | --- |
| 60×16 before generator | 0.00 Bit/s | 0.00 Bit/s | 2.08 MByte |
| low 1 | 2.01 MBit/s | 954.05 kBit/s | 2.48 MByte |
| high 1 | 7.90 MBit/s | 3.08 MBit/s | 4.19 MByte |
| idle 1 | 0.00 Bit/s | 2.66 MBit/s | 4.57 MByte |
| low 2 | 2.01 MBit/s | 2.44 MBit/s | 4.98 MByte |
| high 2 | 7.90 MBit/s | 3.21 MBit/s | 6.69 MByte |
| idle 2 | 0.00 Bit/s | 2.95 MBit/s | 7.05 MByte |
| after generator | 0.00 Bit/s | 2.84 MBit/s | 7.05 MByte |
| 80×24 before generator | 0.00 Bit/s | 0.00 Bit/s | 7.05 MByte |
| low 1 | 2.01 MBit/s | 1.09 MBit/s | 7.46 MByte |
| high 1 | 7.97 MBit/s | 3.37 MBit/s | 9.16 MByte |
| idle 1 | 0.00 Bit/s | 2.84 MBit/s | 9.54 MByte |
| low 2 | 2.01 MBit/s | 2.57 MBit/s | 9.95 MByte |
| high 2 | 7.96 MBit/s | 3.34 MBit/s | 11.65 MByte |
| idle 2 | 0.00 Bit/s | 3.06 MBit/s | 12.03 MByte |
| after generator | 0.00 Bit/s | 2.95 MBit/s | 12.03 MByte |

Min stayed `0.00 Bit/s`; final Max was `7.96 MBit/s` at 60×16 and
`7.97 MBit/s` at 80×24. Totals were cumulative counters, including pre-run
traffic, rather than a zero-based session sum. Numeric high rates exceeded both
ceilings while encoded graph activity saturated; these were actual observations,
not a comparison to pacing-derived expected rates.

Baseline frames had `Waiting for rate sample`, rates/extrema `--`, available
totals, blank graph areas and no foreground codes. The idle warm-up had measured
numeric zeros, `=` separator and blank graphs. Traffic introduced colored Braille;
high phases included all twelve foreground indices. Older nonzero history remained
during idle while new blank positions entered at the right. For example, the last
six incoming bottom cells at 60×16 were `    ⣿⣿` (low 1), `  ⣿⣿⣿⣿`
(high 1), `⣿⣿⣿⣿⣿ ` (idle 1), and `⣿⣿⣿⣿⡇ ` after the generator.
That last left-only glyph followed by blank positions was also present at
80×24. These are decoded stream observations, not visual font/color acceptance.

Both smoke runs quit with q, status 0, empty stderr, original termios and original
shared file-status flags restored, and the CAN/reset/show-cursor/leave-alt-screen
cleanup suffix present. Collected complete frame counts were 35 and 28 (including
navigation in the first run); these counts are not a sample-count oracle.

## Acceptance mapping and pending human work

| Case | Available current evidence | Remaining evidence |
| --- | --- | --- |
| D01/build | Three gates above; release artifact identified. | Recheck applicability if executable source changes during integration. |
| D04/live Linux | Actual discovery/default and explicit selection, all-arrow navigation, independent labels, ordinary nonprivileged generated traffic, advancing totals/rate/Braille stream activity above. | Human ordinary-emulator report of the same workload. |
| T14 | Release PTYs establish protocol/OS behavior only. | Emulator identity/version, font/Braille coverage, UTF-8/256-color setup, both palettes at both exact sizes, perceived prompt navigation, visual history recovery and visible restoration. |
| V05 emulator | Release `small_startup_resize_sampling_navigation_and_quit_keep_bounded_rows` passed for 59×16, 60×15 and 1×1 with growth to 60×16/80×24. | Actual small-window startup/shrink/navigation/sampling/growth and default-style warning appearance. |
| Zero/empty states | Smoke measured zero and baseline above; exact absent/read-failed/recovery/idle fixtures mapped in [02 evidence](02-evidence.md#exact-v01v08-mappings-and-twelve-function-migration). | Human zero/empty observation where practical; explicitly identify scripted coverage where a live state cannot be seen or induced reliably. |
| q / Ctrl+C / SIGTERM | Release `normal_controls_restore_real_modes_shared_flags_and_protocol` passed separate statuses 0/130/143 (also Q/SIGINT), saved OS state and cleanup bytes. Smoke q also recorded above. | Separate **human** reports of each status plus cursor, default styling, normal typing/echo and prior-screen return. |

Use [04 emulator checklist](04-emulator-checklist.md), then return the completed
[04 human report template](04-human-report-template.md). All human observation
fields are pending. No ticket closure is warranted yet.

### Previous evidence applicability

[Historical emulator/live evidence](../nload-rust-implementation/emulator-live-smoke.md)
records actual human Alacritty 0.17.0 observations for the earlier ASCII artifact
on 2026-10-02. Its artifact/revision and sizes differ (86×22, 92×7/8); it cannot
establish styled palette/font/text contrast or the required exact 60×16/80×24
checks. Its successful handled exits are historical context, not new observations.
Ticket 02's current deterministic glyph/palette/state coverage and release PTYs
are useful scripted evidence; neither is a substitute for T14 visual evidence.

## Review follow-up — corrected artifact and fresh validation — 2026-10-08

Source: `5e4661164f7afda85c266b2a45ce730276c71bad` on
`implement/btop-review-fixes`, based on clean integration `a2214c6`. The correction
removes additive graph-dot rounding promotion. Build output is now at the durable
path `/mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs`,
SHA-256 `d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`.
It was built from the review-fix source worktree; the permanent checkout's
integration branch still points to `a2214c6` at recording time. The artifact can
be exercised there directly; source applicability must be recorded separately.

Fresh successful checks on corrected source/artifact:

- `cargo fmt --check`; `cargo test --locked`: **115 unit + 16 PTY/decoder**.
- `cargo build --release --locked --target-dir /mnt/programming/rust/nload-rs/target/btop-review-fixes`.
- `NLOAD_RS_PTY_BINARY=/mnt/programming/rust/nload-rs/target/btop-review-fixes/release/nload-rs cargo test --locked --test linux_pty`:
  **16 passed**, including styled exits, backpressure, small/resize recovery and
  colored locale/NO_COLOR checks.
- Ordinary nonprivileged loopback UDP with the existing generator's two-second
  low/high/idle phases, twice, at **60×16 and 80×24**. Discovery, all-arrow
  selection, separate 2.00/4.00 MBit/s labels, advancing totals, nonzero graphs,
  all twelve foreground indices, measured-zero idle with older traffic retained,
  and q/status-0/OS restoration passed. Each run received 634 datagrams over
  12.002 seconds with actual RX/TX proc deltas of 5,211,480 bytes per direction.

Actual checkpoint values, generator output, gate commands, artifact identity and
Standards/Spec resolutions are in [Review fixes — Standards and Spec evidence](review-fixes-evidence.md).
The original [live-smoke transcript](04-live-smoke.txt) and `/tmp/opencode/nload-btop-04`
paths above retain provenance only. They are not required for the new human runs.

Use the updated [runnable styled-graph emulator checklist](04-emulator-checklist.md)
from `/mnt/programming/rust/nload-rs` and return the
[human emulator report template](04-human-report-template.md) with the corrected
artifact identity. **T14, V05-emulator and ordinary-emulator D04 remain pending;
this ticket is blocked / awaiting-human and final acceptance remains blocked.**
