# Rust nload implementation verification

Acceptance authority: [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), sections 1–11. Parent effort: [Implement and verify the minimal Rust port of nload](map.md).

## Foundation run — 2026-10-02

Scope: [Establish the Rust project and delivery foundation](issues/01-project-foundation.md).
This record establishes a buildable foundation, not finished application
acceptance. Later tickets must append their actual checks and case-to-test/manual
evidence mappings; unrun acceptance cases remain pending.

### Revision and environment

- Base repository revision: `178b1d2adaafc3c82e7e25d61f7e6bd7581efee5`.
- Implementation tested: uncommitted foundation files introduced by the linked
  ticket, on that base. No implementation commit exists yet; an authorized final
  handoff must record its revision and commit `Cargo.lock`.
- OS reported by Cargo: NixOS `26.11.0 (zokor)`, 64-bit.
- Kernel from `uname -a`: `Linux nixos-dev 7.2.8 #1-NixOS SMP PREEMPT_DYNAMIC Fri Sep 25 14:37:14 UTC 2026 x86_64 GNU/Linux`.
- Target: default host `x86_64-unknown-linux-gnu`, with no
  `CARGO_BUILD_TARGET` override. The Linux-filtered metadata and feature tree were
  inspected explicitly for this target.
- `TERM=alacritty` is only an environment declaration. No emulator session or
  visible behavior was observed in this foundation run.
- `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `RUSTC`, `RUSTC_WRAPPER`, and
  `RUSTC_BOOTSTRAP` were unset.
- `rustc --print sysroot`:
  `/nix/store/xvp6nfxayb07si2jaggqwvx3iykw89g2-rustc-1.98.1`.
- This is the system-provided stable-release compiler, built from a source
  tarball. `rustup` is not installed; `rustup show active-toolchain` could not run.
  It is not needed for the successful Cargo checks below. No nightly features
  or bootstrap override are used.

`rustc --version --verbose`:

```text
rustc 1.98.1 (48a229cea 2026-09-01) (built from a source tarball)
binary: rustc
commit-hash: 48a229ceaefd4985c50990b14116b6d856af0985
commit-date: 2026-09-01
host: x86_64-unknown-linux-gnu
release: 1.98.1
LLVM version: 21.1.8
```

`cargo --version --verbose`:

```text
cargo 1.98.0 (797e8a9bc 2026-08-05)
release: 1.98.0
commit-hash: 797e8a9bca276c1c9f9f738d2a20f484fa4eea9d
commit-date: 2026-08-05
host: x86_64-unknown-linux-gnu
libgit2: 1.9.4 (sys:0.21.0 vendored)
libcurl: 8.22.0 (sys:0.4.90+curl-8.21.0 system ssl:OpenSSL/3.6.4)
os: NixOS 26.11.0 (zokor) [64-bit]
```

`rustfmt --version`: `rustfmt 1.9.0`.

### Commands and outcomes

All commands ran from the repository root.

| Command | Actual outcome | Evidence scope |
| --- | --- | --- |
| `cargo info libc@0.2.189` | Exit 0; required version available | Baseline availability; metadata `MIT OR Apache-2.0` |
| `cargo info signal-hook@0.4.4` | Exit 0; required version available | Baseline availability; default features are channel/iterator, explicitly disabled by our manifest |
| `cargo generate-lockfile` | Exit 0; lock format 4, six third-party packages resolved | Required baselines unchanged |
| `cargo fetch --locked` | Exit 0 | Locked source availability, including Windows-only transitive notice review |
| `cargo fmt --check` | Exit 0 | Foundation source formatting |
| `cargo test --locked` | Exit 0; **0 tests**, 0 failures | Compilation/test-harness foundation only; no behavioral coverage claimed |
| `cargo build --release --locked` | Exit 0; optimized binary `target/release/nload-rs` | Foundation build, not final application acceptance |
| `cargo metadata --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu` | Exit 0 | Identity, edition, target, dependency features and licenses |
| `cargo metadata --locked --format-version 1` | Exit 0 | License metadata of all locked packages |
| `cargo tree --locked --target x86_64-unknown-linux-gnu -e features` | Exit 0 | Actual Linux dependency/features tree below |
| `cargo tree --locked --target all -e features` | Exit 0 | Windows-only lockfile entries explained below |
| `./target/release/nload-rs` | Expected scaffold exit **1**; empty stdout; diagnostic on stderr | Direct Linux execution of the foundation only; no UI/live acceptance |

The direct executable was invoked by a subprocess capturing both output streams
and its return status. Its complete stderr was:

```text
nload-rs 0.1.0: application foundation only; traffic monitoring is not implemented yet
```

### Dependency, feature and license review

The manifest pins the two agreed direct baselines exactly; signal-hook has
`default-features = false`. The resolved Linux packages/features are:

| Package | Version | Enabled features | Dependencies on this target | License route |
| --- | --- | --- | --- | --- |
| nload-rs | 0.1.0 | None | libc, signal-hook | GPL-2.0-only application |
| libc | 0.2.189 | default, std | None | MIT |
| signal-hook | 0.4.4 | **None** | libc, signal-hook-registry | MIT |
| signal-hook-registry | 1.4.8 | None | libc, errno | MIT |
| errno | 0.3.14 | default, std | libc | MIT |

`Cargo.lock` also contains `windows-sys 0.61.2` and `windows-link 0.2.1`:
errno's `cfg(windows)` edge selects windows-sys, which selects windows-link.
The all-target tree shows windows-sys's default, Win32_Foundation,
Win32_System_Diagnostics_Debug and their parent features. Neither Windows crate
appears in the Linux tree or compiled package list. Their metadata also offers
`MIT OR Apache-2.0`; their actual `license-mit` notices were inspected.

For the Linux crates, the downloaded `LICENSE-MIT` texts were inspected, not just
their SPDX metadata. The selected MIT routes and original copyright/permission
notices for all six locked third-party packages are preserved in
[Third-party dependency notices](../../THIRD-PARTY-NOTICES.md). MIT permits the
GPL-2.0-only combination with those notices retained; no Apache-only route is
needed.

No channel, iterator, extended-siginfo, extended-siginfo-raw or cc feature is
enabled on signal-hook; no cc/build-tool dependency is resolved. The downloaded
signal-hook `src/low_level/mod.rs` gates its Unix pipe module on `not(windows)`,
not on a feature. Its `flag` module is also available without default features.
Thus the selected graph retains the required atomic/Unix-pipe APIs. Their actual
runtime use belongs to subsequent terminal tickets.

No terminal framework, async runtime or argument-parser crate is present. The
graph has four Linux third-party packages and six cross-target locked packages.

### Foundation artifact review

- [Cargo manifest](../../Cargo.toml): package `nload-rs`, version `0.1.0`,
  edition `2024`, license `GPL-2.0-only`; metadata confirms a single binary
  target named `nload-rs`, with no public library target.
- [Main entry point](../../src/main.rs) obtains the identity and version via
  `CARGO_PKG_NAME`/`CARGO_PKG_VERSION`. No independent application-version
  constant exists. Standalone `--version` parsing/output is still pending.
- The five private module roots are
  [arguments](../../src/arguments.rs), [proc sampler](../../src/proc_sampler.rs),
  [monitor](../../src/monitor.rs), [screen](../../src/screen.rs), and
  [terminal/runtime](../../src/terminal.rs). They document settled ownership
  without imposing unfinished shared interfaces. They contain no behavior yet.
  The application entry point remains small; pure views/rows and production/
  scripted effect adapters must still be implemented by their owning tickets.
- [GPL v2 license text](../../LICENSE) comes from
  <https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt>. The application's
  GPL-2.0-only grant is explicit in Cargo metadata, README and source SPDX
  headers; the generic license appendix does not grant later versions.
- [README](../../README.md) includes the exact agreed inspiration credit,
  all three required locked-build/check commands, direct execution, tested
  compiler/target, license links and an explicit foundation status.
- [Cargo.lock](../../Cargo.lock) is generated and retained for source control;
  [.gitignore](../../.gitignore) excludes only `/target/`, not the lockfile.
  Git staging/commit is deferred to the explicitly authorized repository
  handoff. The committed-lockfile acceptance requirement remains pending.
- No toolchain or dependency availability conflict was found.
- `git diff --check` reported no tracked-file whitespace errors. Each new
  foundation file was also checked with `git diff --no-index --check /dev/null
  <file>`: no whitespace diagnostics. An initial check wrapper incorrectly
  expected exit 0 for these new-file diffs; the corrected wrapper accepts exit 1
  for differences and requires empty diagnostic output. No source fix was needed.
  `git check-ignore --no-index target/release/nload-rs` confirmed build artifacts
  are ignored; Git status confirmed `Cargo.lock` is retained and not ignored.

## Acceptance case ledger

These are section 10 case identifiers, not issue references. Each future evidence
entry must give actual test names or recorded manual observations, not an intended
implementation. **No complete acceptance case is marked passing yet.**

| Cases | Status after foundation | Evidence / remaining work |
| --- | --- | --- |
| A01 | Pending (metadata portion reviewed) | Identity and Cargo-version authority reviewed above; actual standalone version output/no-effect test still required |
| A02–A12 | Pending | Arguments, startup, sorting, selection, navigation and lifecycle have no implemented tests yet |
| S01–S13 | Pending | Proc acquisition, BOOTTIME/scheduling, statistics, resets and history have no implemented tests yet |
| V01–V08 | Pending | Screen geometry, bars, states, resize, names and formatting have no implemented tests yet |
| T01–T13 | Pending | Scripted terminal/event/integration and Linux PTY evidence not run |
| T14 | Pending | Recorded emulator identity, visual protocols and handled-exit observations required; TERM declaration is insufficient |
| D01 | Pending (foundation commands pass) | Final application gates and committed lockfile still required; exact foundation compiler/commands recorded above |
| D02 | Pending (dependency portion reviewed) | Actual graph/licenses reviewed above; implemented deep modules, pure contracts, scripted/production adapters and unsafe confinement review still required |
| D03 | Pending (foundation files reviewed) | Identity, license, credit, instructions and compiler record reviewed above; committed lockfile and final artifact review still required |
| D04 | Pending | No nonprivileged live proc/interface/traffic UI smoke has been run |
| D05 | Pending | Final implemented reference/departure and supersession review required |

## Outstanding verification

- Complete deterministic case-to-test mappings and required application behavior.
- Automated Linux PTY checks with actual OS-state restoration evidence, including
  practical real faults and explicit limitations backed by scripted tests.
- Human-recorded xterm-compatible emulator observations and nonprivileged live
  smoke; neither is replaced by this scaffold's direct execution.
- Final source/artifact review, required command gates, implementation revision
  and authorized Cargo.lock commit. The foundation results alone do not satisfy
  the parent map's destination.
