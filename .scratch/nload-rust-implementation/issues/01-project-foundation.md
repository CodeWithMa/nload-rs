# Establish the Rust project and delivery foundation

Status: closed
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: project foundation)
Blocked by:

## Question

Can the repository provide a buildable stable-Rust, edition-2024 application foundation with the exact package/executable identity, Cargo-authoritative version, selected dependency graph and GPL-2.0-only delivery files required by specification sections 1 and 9?

Execution is authorized by the parent map's Notes. Create the minimal Cargo application foundation for the established internal module boundaries, select the direct dependency baselines `libc 0.2.189` and `signal-hook 0.4.4` with its default features disabled, generate and track `Cargo.lock`, and inspect the actual resolved target/features and compatible license routes. Include GPL v2 license text, Cargo license metadata, the agreed README credit and locked-build/direct-run instructions. Do not present a scaffold as the finished monitor.

Begin an implementation verification record at `.scratch/nload-rust-implementation/verification.md` with actual compiler/target/environment, foundation command outcomes, dependency/license review and pending acceptance cases. Run `cargo fmt --check`, `cargo test --locked` and `cargo build --release --locked` for the foundation. Record any dependency availability or toolchain conflict as a blocker requiring resolution, not a silently changed baseline. Resolution links the files/evidence and identifies the stable shared foundation available to later tickets. Git commits require separate explicit authorization.

## Comments

### Resolution — 2026-10-02 — alice

The repository now provides the required buildable application foundation. No
dependency availability or stable-toolchain conflict was found. The detailed
command outcomes, toolchain/environment, license review and acceptance ledger
are in the [implementation verification record](../verification.md#foundation-run--2026-10-02).

- [Cargo manifest](../../../Cargo.toml): package/executable `nload-rs`, version
  `0.1.0`, edition 2024, GPL-2.0-only. Direct versions are pinned to `libc
  0.2.189` and `signal-hook 0.4.4`, with signal-hook defaults disabled.
- [Cargo lockfile](../../../Cargo.lock): generated format-4 lockfile. Linux
  resolves libc, signal-hook, signal-hook-registry 1.4.8 and errno 0.3.14. The
  lockfile additionally holds Windows-only windows-sys 0.61.2/windows-link
  0.2.1, absent from the Linux build. All six packages offer inspected MIT
  routes compatible with GPL-2.0-only; their actual notices are preserved in
  [Third-party dependency notices](../../../THIRD-PARTY-NOTICES.md).
- [Main entry point](../../../src/main.rs) embeds Cargo name/version metadata,
  returns normally via `ExitCode`, and declares five private module roots:
  [Arguments](../../../src/arguments.rs), [Proc sampler](../../../src/proc_sampler.rs),
  [Monitor](../../../src/monitor.rs), [Screen](../../../src/screen.rs), and
  [Terminal/runtime](../../../src/terminal.rs). Their documented ownership
  follows the specification. Their owning tickets define the actual shared
  configuration/snapshot/view/effect interfaces.
- [GPL version 2 text](../../../LICENSE), [README](../../../README.md) with
  the agreed credit and required check/build/direct-run instructions, and
  [build-artifact ignore rule](../../../.gitignore) are present. Cargo.lock
  is not ignored and is retained for the eventual authorized commit.
- `cargo fmt --check`, `cargo test --locked`, and `cargo build --release
  --locked` all passed on rustc 1.98.1 / Cargo 1.98.0, target
  `x86_64-unknown-linux-gnu`. The foundation has **zero behavioral tests**.
  Direct `./target/release/nload-rs` execution produced the documented
  foundation-only stderr diagnostic and expected status 1.

This is a stable build/dependency/delivery foundation for later tickets, not a
finished monitor. No complete application acceptance case is claimed passing;
the verification ledger identifies the reviewed artifact portions and pending
behavioral, PTY, emulator, live and final-handoff checks. No Git commit was made.
No new decision or corrective ticket surfaced; existing child tickets own the
remaining implementation and verification.
