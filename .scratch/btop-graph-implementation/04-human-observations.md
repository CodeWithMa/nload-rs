# Human emulator observations — 2026-10-08

Reported by the human in this implementation session. These are actual human
reports; command expectations and automated PTY results are recorded elsewhere.

## Supplied identity and setup output

```text
/mnt/programming/rust/nload-rs
integration/btop-styled-graphs
3765b3162944def12f8fb0f9c8853ca3a229c716
git status --short: no output
git diff 5e4661164f7afda85c266b2a45ce730276c71bad -- src tests Cargo.toml Cargo.lock: no output
nload-rs 0.1.1
d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7  ./target/btop-review-fixes/release/nload-rs
rustc 1.98.1 (48a229cea 2026-09-01) (built from a source tarball)
binary: rustc
commit-hash: 48a229ceaefd4985c50990b14116b6d856af0985
commit-date: 2026-09-01
host: x86_64-unknown-linux-gnu
release: 1.98.1
LLVM version: 21.1.8
uid/euid: 1000 1000
{'TERM': 'alacritty', 'COLORTERM': None, 'TERM_PROGRAM': None, 'TERM_PROGRAM_VERSION': None, 'LANG': 'en_US.UTF-8', 'LC_ALL': None, 'NO_COLOR': None, 'TMUX': None, 'STY': None, 'SSH_TTY': '/dev/pts/2'}
network namespace: net:[4026531833]
/dev/pts/2
22 86
alacritty 0.17.0 (94e7c887)
```

Initial `stty size` was 22 rows × 86 columns, before resizing for the runs.
Client OS, theme name and configured font details were not supplied. Font
details were explicitly unknown; actual single-cell Braille coverage was
confirmed in the follow-up below.

## First report (human wording)

```text
RUN A

Set to 60x16
2-3 all looks good
4-5 looking good
6 headers show values and are not colored
7 nubmers match the python script that cretaed the traffic
8 smoke_status=$?; printf 'Run A q status: %s\n' "$smoke_status"
   stty size
Run A q status: 0
17 56

previous text and typing works

RUN B

every thing works
smoke_status=$?; printf 'Run B typed Ctrl+C status: %s\n' "$smoke_status"
Run B typed Ctrl+C status: 130
typing works, prev text visible

RUN C

when small traffic got recorded and i could see it after i made it larger again, matchign what the script was sending

TICKET04 RUN C prior-screen marker — SIGTERM after 25 seconds
Run C SIGTERM child status: 143
```

Step numbers refer to [the runnable emulator checklist](04-emulator-checklist.md).
Run A's observed small exit size was **56×17**, not the checklist's suggested
59×16. This is below the full-view width threshold. Exact 59×16 and 60×15
boundary behavior is additionally covered by deterministic/PTY evidence.
The traffic comparison is qualitative live evidence, not an exact numerical
rate oracle.

## Follow-up confirmation

The assistant asked the human to confirm:

1. Font name/size or fallback if known, and whether Braille rendered as clean
   single-cell glyphs.
2. Run B was 80×24 and Run C started below 60×16 before growing.
3. At both full sizes, incoming indigo/lavender and outgoing magenta had legible
   medium-dark bottoms/pale tops, statistics stayed default-colored, and idle
   showed blank new positions.
4. After **each** q/Ctrl+C/SIGTERM exit, the cursor was visible,
   foreground/background were default, typing/editing worked, and the prior
   shell screen returned.

The human selected: **“All observed; font unknown”**. The offered answer
explicitly confirmed all listed visual behaviors and dimensions were observed,
with configured font details unavailable.

## Evidence scope

This report supplies human visual evidence for T14, V05-emulator, and D04's
ordinary-emulator live smoke. It reports both full-size appearances, navigation,
small-period sampling/history recovery, measured-idle blank positions, default
text contrast, and separate handled-exit restoration. Baseline/absent/read-failed
states and exact half-cell sample arithmetic remain covered by the clearly
identified scripted fixtures in the checklist, rather than invented live
observations. No human-reported failure remains unresolved.
