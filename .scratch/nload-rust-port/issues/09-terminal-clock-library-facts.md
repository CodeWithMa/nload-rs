# Establish terminal, clock and dependency feasibility

Status: closed
Labels: wayfinder:research
Type: research
Mode: AFK
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (research agent: terminal/clock feasibility)
Blocked by:

## Question

Which small, maintained Rust/Linux library choices can implement the confirmed terminal and timing contract on stable Rust/edition 2024 under GPL-2.0-only? Check current crossterm, libc and signal-hook as candidates, using primary documentation and source. Establish concrete versions, license/transitive-dependency compatibility, timed input and resize behavior, raw-mode Ctrl+C/Ctrl+Z semantics, SIGINT/SIGTERM notification and wake-up options, CLOCK_BOOTTIME access, alternate-screen/input restoration, and whether prior cursor visibility can actually be queried/restored. Distinguish supported guarantees from terminal-dependent limitations. Compare a crossterm-based approach with a direct Linux terminal implementation sufficiently to inform the architecture decision, without choosing for the human.

Consult the existing Linux-counter research for bulk `/proc/net/dev` acquisition and name-byte parsing facts; flag any factual obstacle to efficient whole-list acquisition. Link one research asset with sources and a branch/worktree context pointer. Do not write application code or change agreed user-visible behavior.

## Comments

### Research context — 2026-10-01 — alice

Created while working [Decide the port's architecture and test seams](07-module-boundaries.md). That decision needs externally verified crate and terminal facts before final library/mechanism selection. Research runs independently while the human discusses decisions that do not depend on these findings.

### Isolated research context — 2026-10-01 — alice

- Branch/worktree: unborn `research/nload-terminal-clock` at `/tmp/nix-shell.TfI7CV/opencode/nload-terminal-clock/`; created with `git worktree add --orphan -b research/nload-terminal-clock`. The primary worktree remains on unborn `dev`.
- Asset: [Terminal, clock and dependency feasibility](../research/terminal-clock-dependencies.md); isolated mirror: `/tmp/nix-shell.TfI7CV/opencode/nload-terminal-clock/research/terminal-clock-dependencies.md`.
- Fresh-repository limitation: no HEAD commit, branch commit ID, committed checkout of planning context, or capture commit exists. Both copies are uncommitted; the worktree registration and branch name are the context pointer. Existing planning was read in the primary worktree.
- Loaded `research` first, followed the tracker/domain instructions, and read [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md), [Decide traffic statistics and sampling behavior](04-statistics-contract.md), [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md), and [Decide Rust and delivery constraints](06-engineering-constraints.md), plus the existing Linux-counter asset. The skill requests background delegation, but this research subagent exposes no further agent/delegation tool; primary-source investigation was performed in this research session instead.

### Resolution — 2026-10-01 — alice

Resolved the factual investigation with [Terminal, clock and dependency feasibility](../research/terminal-clock-dependencies.md). Primary-source findings establish current stable **crossterm 0.29.0**, **libc 0.2.189**, and **signal-hook 0.4.4**, their features, and a concrete source-audited GPL-2.0-only-compatible Linux dependency graph using MIT alternatives. The asset includes default-feature differences and Unicode-3.0 obligations; it is not a generated application lockfile or build certification.

- Crossterm's minimal synchronous candidate is `default-features = false, features = ["events"]`; its examined graph retains **signal-hook 0.3.18**, **signal-hook-mio 0.2.5**, **mio 1.2.3**, and **rustix 1.1.5**, rather than upgrading its signal-hook requirement to current 0.4.4. Direct libc plus signal-hook 0.4.4 with defaults disabled needs only registry 1.4.8 and errno 0.3.14 as additional transitives.
- Full raw mode delivers typed Ctrl+C/Z as input rather than terminal-generated signals. Explicit Ctrl+C handling and external SIGINT/SIGTERM notification are required; ignored Ctrl+Z input is feasible. The asset compares atomic flags, iterator notification, self-pipe wakeups, signalfd and ppoll, including crossterm's private wait loop and its source-level timeout/EOF/read-error limitations.
- `libc::clock_gettime(CLOCK_BOOTTIME)` directly supplies the agreed elapsed-time semantics. BOOTTIME timerfd can join a caller-owned Linux FD loop; it includes suspend without waking the machine or manufacturing missed observations. Rust Instant is not the required BOOTTIME clock.
- **Parent decision blocker:** crossterm has no prior-cursor-visibility query. Xterm DECRQM and private-mode save/restore are conditionally feasible, but Linux v6.12 virtual-console source is a concrete counterexample to universal cursor/alternate-screen restoration. “Show on exit” does not restore an initially hidden cursor. [Decide the port's architecture and test seams](07-module-boundaries.md) must take the supported-terminal/fallback question to the human; this research does not amend [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md) or select an implementation.
- Whole-list `/proc/net/dev` remains feasible: read bytes through EOF, preserve exact interface-name bytes, and parse RX/TX fields 1/9. Tagged source rejects colon/whitespace in valid names but does not require UTF-8; `%6s` is minimum padding, not truncation. Bulk acquisition is neither one-read nor atomic, and failed/incomplete reads are not evidence of absence or zero traffic.

Verification before closure: checked registry facts, published manifests/source/license texts, owning terminal/clock/signal manuals, and tagged Linux proc/name/virtual-console source. An OS-only fresh-PTY probe read raw bytes `031a` and restored saved termios exactly; a local BOOTTIME read succeeded. No application code, Cargo build, benchmark, real-emulator compatibility test, host suspend test, or commit was produced. The canonical asset and isolated mirror have identical Git blob hashes, **`5fabdb7640f5ca43f6e86e89a397b73f9587c344`**; `git diff --no-index` reported no differences. This is a content identity, not a commit SHA. Worktree inspection confirmed primary `dev` and isolated `research/nload-terminal-clock` remain unborn.

Parent indexing gist: **Established terminal/clock mechanisms and a compatible concrete dependency graph; exposed nonportable prior-cursor/screen restoration and crossterm wait/error caveats.** The parent session owns adding this closed ticket to the map's Decisions so far. Remaining human library/support choices and eventual locked-build/PTY acceptance verification stay with the existing architecture and acceptance tickets.

### Post-resolution context — 2026-10-02 — alice

The parent replaced numeric agreement links in the asset with named titles, preserving identical canonical/mirror copies. `git diff --no-index` again reported no differences; both updated copies have blob hash `b96fc9eb26ed55a52593198b0d500de9b7f82414`. The earlier verification hash identifies the pre-edit version, not a commit. The human has now settled the terminal/library choices in [Decide the port's architecture and test seams](07-module-boundaries.md#resolution--2026-10-02--alice).
