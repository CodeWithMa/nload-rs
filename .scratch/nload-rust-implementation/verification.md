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

## Arguments and proc run — 2026-10-02

Scope: [Implement arguments, proc snapshots and startup ordering](issues/02-arguments-proc-startup.md).
Caller contracts: [Arguments and proc snapshot interfaces](arguments-proc-interfaces.md).

### Revision, environment and commands

- Base revision: `1977703de7d9321c92545ad6fc864c2099e2dc50`; worktree was clean
  before claiming the ticket. Tested implementation is the uncommitted changes to
  `src/arguments.rs` and `src/proc_sampler.rs` on that base. No commit was made.
- `rustc --version --verbose` reconfirmed rustc **1.98.1**, commit
  `48a229ceaefd4985c50990b14116b6d856af0985`, host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8. `cargo --version --verbose`
  reconfirmed Cargo **1.98.0**, commit
  `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`, NixOS 26.11.0.
  No dependency or manifest/lockfile changes were made.
- `git log -1 --oneline -- Cargo.lock` confirms that the human's intervening
  `1977703` foundation commit already includes the lockfile. The foundation-run
  statements about an uncommitted lockfile above are historical; final application
  acceptance still needs its own revision-linked handoff.
- All commands ran at the repository root. The final source checks were:

| Command | Actual outcome |
| --- | --- |
| `cargo fmt` | Exit 0; formatted the implemented source |
| `cargo fmt --check` | Exit 0 |
| `cargo test --locked` | Exit 0; **13 passed**, 0 failed/ignored |
| `cargo build --release --locked` | Exit 0; 27 `dead_code` warnings for interfaces awaiting application wiring |
| `env -u HOME LC_ALL=C ./target/debug/deps/nload_rs-ed75f073191bbfc4` | Exit 0; all 13 deterministic tests pass with HOME unset and C locale |
| `env HOME=/tmp/opencode LC_ALL=en_US.utf8 ./target/debug/deps/nload_rs-ed75f073191bbfc4` | Exit 0; the same 13 tests pass with different HOME/locale |
| `git diff --check` | Exit 0; no whitespace diagnostics |

The environment variants execute the Cargo-built **test harness**, not the
product binary. They establish identical pure defaults/order/selection fixtures;
they do not establish final startup/view or stream behavior. The foundation entry
point is still awaiting integration, so no direct UI/live run is claimed here.

### Deterministic case-to-test evidence

All names below are actual passing test names from `cargo test --locked`.
Argument tests cross `parse`/text interfaces; snapshot/selection/round tests
cross the proc caller interfaces. Chunking/open/read-failure tests additionally
use the private I/O seam called by the production adapter.

| Passing test | Applicable cases and observations |
| --- | --- |
| `arguments::tests::informational_outcomes_and_cargo_text` | A01/A02: standalone outcomes; exact Cargo-derived name/version/newline; required help contents |
| `arguments::tests::rejects_unknown_flags_multiple_names_and_information_combinations` | A03: unknown options, excluded short aliases, multiple names and informational combinations return nonempty diagnostics |
| `arguments::tests::preserves_os_name_bytes_and_option_terminator` | A04: raw OS bytes/case preserved; leading-dash and informational-looking names after `--`; positional limit |
| `arguments::tests::independent_defaults_overrides_repetitions_and_zero` | A05/A07: independent 1,310,720 defaults; exact override example before/after name; last occurrence and direction-local zero reset |
| `arguments::tests::ceiling_range_and_whole_ascii_digit_grammar` | A06: both flags at 1 and max; checked decimal/conversion overflows, empty/missing/sign/fraction/whitespace/suffix/non-ASCII/attached rejection; unbounded leading zeros; earlier invalid occurrence not forgiven |
| `proc_sampler::tests::snapshot_preserves_padding_raw_names_and_exact_paired_fields` | S01/A04: proc padding, raw bytes, case-sensitive identity, zero/max u64 and RX/TX field indices despite different other fields |
| `proc_sampler::tests::acquisition_reads_through_eof_and_opens_fresh_each_round` | S01: 3,000 rows/>128 KiB in seven-byte chunks, interrupted read retry, EOF observed, two opens/rounds, second different snapshot has no stale bytes/membership |
| `proc_sampler::tests::rejects_malformed_headers_rows_fields_overflow_and_duplicates` | S02/A08: malformed headers, missing/fewer/extra fields, invalid name/number/overflow, duplicate names and incomplete final rows reject globally even after a good row |
| `proc_sampler::tests::failed_acquisition_is_global_not_absence_and_retries_cleanly` | S02/A08: open failure, read error after otherwise valid bytes and parse error all make every retained name read-failed; valid missing name is absent; clean retry |
| `proc_sampler::tests::startup_sorts_all_names_and_selects_exact_original_bytes` | A04/A08/A09: complete natural startup order (`eth1`, `eth02`, `eth2`, `eth3`, `eth10`), case/raw-byte order, loopback/inactive inclusion, default first, exact selection without filtering, distinct empty/missing errors and initial counters retained |
| `proc_sampler::tests::natural_order_handles_unbounded_runs_and_full_byte_ties` | A09: thousand-digit values, 1,500 leading zeros, zero runs, prefixes/multiple runs and full-byte ties, all observed through startup ordering |
| `proc_sampler::tests::os_invocation_to_startup_preserves_identity_and_full_membership` | A04/A05: OS-token parsing through snapshot startup selection for raw and leading-dash names; complete list and independent converted configuration retained |
| `proc_sampler::tests::retained_name_rounds_ignore_additions_and_recognize_raw_name_return` | A12/S02 acquisition portions: scripted adapter calls, raw-name removal/rename/additions/return; retained original list/selection and paired outcomes |

### Review and remaining integration gaps

- Arguments perform no runtime environment, config, terminal, clock or proc I/O.
  Snapshot acquisition performs one bulk traversal per round; bytes/names/counters
  are exact, complete validation is required, and no timestamps are acquired here.
  Neither module contains unsafe code or prints diagnostics.
- Parsing validates the two standard kernel headers and every one of the 16
  unsigned fields, including the unexposed accounting fields. Header whitespace
  is flexible; final newline, field count and uniqueness prevent identifiable
  partial/malformed data from escaping. A syntactically complete snapshot cannot
  detect unreported kernel-side races; no atomicity/freshness claim is made.
- A01–A03/A06 still require application stdout/stderr/status and rejecting
  clock/discovery/terminal adapters; parser/text passes do **not** establish
  no-effect application control flow. A04/A08 need application selection/error
  dispatch before mutation. A05 needs identical ceilings after navigation.
- A07 has pure fixture passes under set/unset HOME and source review showing no
  config access; complete application defaults/list/view and final artifact
  review remain. A09 has passing ordering/locale fixtures; fixed process-lifetime
  membership is also a monitor/integration obligation.
- A10 requires startup order, clock initialization/completion timestamps, reuse
  of discovery baselines and deadline anchoring/slow-initialization checks. S02
  requires monitor resets, nonfatal UI continuation/retry and absence of repeated
  runtime stderr output. A12 requires monitor baseline-on-return/no-gap-delta
  and navigation behavior. S13 clock/timerfd ownership remains pending.
- Those gaps already belong to [Implement the pure monitor and statistics runs](issues/03-monitor-statistics.md)
  and [Integrate startup, sampling and the interactive application loop](issues/07-application-integration.md).
  No specification conflict, newly specifiable fog or corrective ticket surfaced.

## Monitor run — 2026-10-02

Scope: [Implement the pure monitor and statistics runs](issues/03-monitor-statistics.md).
Caller contracts: [Monitor and selected-view interface](monitor-interfaces.md).

### Revision, environment and commands

- Base revision: `5495d069a1c99f66c81d5f4927cc04d64afa8e09`; the worktree was
  clean before the ticket claim. Tested source is the uncommitted implementation
  and fixtures in `src/monitor.rs` on this base. No commit was made.
- `rustc --version --verbose` reconfirmed **1.98.1**, commit
  `48a229ceaefd4985c50990b14116b6d856af0985`, host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8. `cargo --version --verbose`
  reconfirmed **1.98.0**, commit
  `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`, NixOS 26.11.0.
  No manifest, dependency, lockfile or other application-module changes were made.
- All commands ran at the repository root; no real-time sleeps, clock I/O or
  private interval/reset assertions are used by monitor tests.

| Command | Actual outcome |
| --- | --- |
| `cargo fmt && cargo test --locked monitor::tests` (initial run) | Formatting exit 0; focused tests exit 101, 13 passed/2 failed. Two fixture expectations were corrected as explained below. |
| `cargo test --locked monitor::tests` (after corrections) | Exit 0; **15 passed**, 0 failed; 13 unrelated tests filtered out |
| `cargo fmt --check` | Exit 0 |
| `cargo test --locked` | Exit 0; **28 passed**, 0 failed/ignored |
| `cargo build --release --locked` | Exit 0; 45 `dead_code` warnings for implemented interfaces awaiting application wiring |
| `git diff --check` | Exit 0; no tracked-file whitespace diagnostics |

The initial failures were test expectations, not production changes: after a
complete forward cycle from the middle entry, the first backward step selects
the first entry; and zero delta in a tiny interval does not erase earlier traffic
still inside the current one-second window. The retention fixture now creates
genuinely measured-zero history with zero counters throughout that warm-up.
The passing final commands above ran after those corrections. The entry point
still awaits integration, so direct UI/PTY/emulator/live acceptance was not run
or claimed for this pure module.

### Deterministic case-to-test evidence

All tests below are in `monitor::tests`; names are actual passing tests. They
construct/observe/navigate/read through the same Monitor interface available to
the application. Assertions use selected-view fields and exposed graph samples,
never private interval queues. Derived comparisons use the specification's
`1e-9 * max(1, abs(expected))` tolerance; exact totals/timestamps/state/counts
and unrounded extrema identities are also checked where applicable.

| Passing test | Applicable cases and observations |
| --- | --- |
| `warm_up_exposes_exact_totals_and_independent_directional_rates` | S03: baseline totals/no column, numerical 200→400 RX warm-up, independent TX, run extrema, latest cumulative rather than session total and exact column timestamps |
| `partial_current_window_weights_only_interval_overlap` | S04: partial current example 350, average 320; independent paired TX example |
| `exact_and_partial_average_boundaries_preserve_crossing_intervals` | S05: exact 302-second example and partial-average 303 example; both directions and current rates using actual long intervals |
| `delayed_and_suspend_length_intervals_use_actual_elapsed_time_once` | S06/S07 monitor portions: 3,000 bytes over three seconds yields 1,000 bytes/s and one column; a 600-second interval uses actual elapsed duration and one surviving column, not catch-up samples |
| `nonpositive_times_ignore_entire_outcomes_without_advancing_any_view` | S06: equal/earlier times ignored at baseline, measured and unavailable states; lower counters/failure/absence ignored without changing views; next valid interval still uses last accepted endpoints/counters |
| `exact_small_deltas_and_nanosecond_differences_survive_large_values` | S08/S06: counters/times above f64 integer precision near u64 maximum preserve one-byte/one-nanosecond differences; exact maximum totals and timestamps remain available |
| `full_u64_interval_delta_is_checked_before_derived_conversion` | S08: zero→maximum-u64 delta remains valid without signed narrowing or integer intermediate overflow; exact independent totals |
| `unrounded_run_extrema_survive_average_expiry_and_measured_idle` | S08: currents differing below display precision retain distinct extrema; minimum survives five-minute expiry, then measured idle sets genuine zero minimum/column; TX extrema independent |
| `decrease_in_either_counter_resets_both_directions_to_new_baseline` | S09: RX 4,000,000,000→10,000 with increasing TX, and independent TX decrease variant, reset both rates/extrema/histories; next RX +500/0.5s yields 1,000 and one column |
| `absence_and_read_failure_hide_totals_and_never_bridge_return_gaps` | S10/S02: each unavailable reason clears both directions with no stale totals; repeated failure followed by higher counters is baseline only, then a fresh independent measured interval |
| `initial_unavailable_outcomes_and_reason_changes_remain_explicit` | S10: unavailable baseline outcomes, reason changes and successful recovery retain distinct states for every entry |
| `navigation_wraps_symmetrically_preserving_views_and_global_ceilings` | A11/A05/S08: initial middle selection, forward/reverse wraps and singleton no-op; every selected view survives navigation unchanged; raw names, one-based position/count, independent ceilings across entries and uncapped numeric rates |
| `snapshot_lifecycle_retains_raw_membership_selection_and_retries_all_names` | A12/S02/S10: actual validated snapshot lookup for A/B, raw-name removal/rename/C addition, every original name retried, selected unavailable A retained/navigable, global parse failure clears both entries, higher-count return is baseline; fixed count/identities maintained |
| `never_selected_interfaces_accumulate_real_history_at_every_round` | S11/A11: never-selected B has independent statistics and actual timestamped 200/300/1,000 samples over regular/delayed rounds; selecting/leaving/reselecting adds no columns or resets |
| `history_expiry_retains_exact_boundary_and_reads_only_hide_fitting_suffixes` | S12 monitor portion: before/exactly/after `t−300` at nanosecond resolution, boundary retained then expired by next observation; >50-column history, real zero sample, caller suffix fits for widths 50/1/0/400 and unchanged views across reads/shrink/grow |

### Module review and remaining integration evidence

- Monitor contains no I/O, clock reads, dimensions, formatting, diagnostics or
  unsafe code. Fixed original-byte entries and startup selection are consumed
  once. The narrow outcome lookup visits retained names only; it does not acquire
  counters, expose membership mutation or require a second discovery snapshot.
- Run intervals retain exact paired u64 deltas and integer endpoints. Deltas are
  checked before conversion; endpoint differences are calculated before f64
  conversion. Only positive rounds reach calculation. Average pruning retains
  crossing intervals; graph pruning retains exact-boundary samples. Statistics
  and graph samples remain unrounded/uncapped; extrema span the whole run.
- Baseline and both unavailable states enforce paired totals/rates/history
  invariants. Both-direction reset and no-gap bridging are verified for decreases,
  failures and absence. Navigation and read-only borrowed views cannot acquire or
  expire samples. Source review and passing fixture comparisons cover these
  module responsibilities of specification sections 3, 4 and 8.
- A11 still needs decoded all-four-arrow/ignored-alias events and prompt requested
  redraw with zero adapter acquisitions in runtime/integration, plus T03/T14
  real input/visual evidence. A12 still needs application-loop lifecycle use;
  fixed-list transitions themselves have passing combined snapshot/monitor fixtures.
- S02 still needs nonfatal application continuation/quiet stderr evidence. S06
  clock semantics/wall-clock/failure/conversion and S07 deadline anchoring,
  timerfd/missed-slot behavior remain runtime/integration checks. The pure monitor
  accepts explicit u64 nanoseconds; suspend-length interval fixtures do not prove
  production BOOTTIME ownership. S13 is still pending for clock/timerfd effects.
- S12 fitting/read invariance is verified at the monitor interface; actual
  shrink/grow screen-row rendering and oldest-left/newest-right placement remain
  with Screen (S12/V03/V05). No rendering test is claimed by this ticket.
- No specification conflict or newly specifiable corrective work surfaced. Those
  remaining checks already belong to existing screen, runtime/integration and
  final-acceptance tickets; the map's fog does not change.

## Acceptance case ledger

These are section 10 case identifiers, not issue references. Each future evidence
entry must give actual test names or recorded manual observations, not an intended
implementation. **No complete acceptance case is marked passing yet.**

| Cases | Latest status | Evidence / remaining work |
| --- | --- | --- |
| A01–A03 | Partial (module checks pass) | Informational outcomes/text and strict grammar tests mapped above; application streams/status/no-effect tests pending |
| A04–A06 | Partial (module checks pass) | Raw names, selection, independent ceilings and range/grammar tests above; monitor confirms global independent ceilings after navigation and uncapped rates; application dispatch/no-effects pending |
| A07 | Partial (module checks/review pass) | Pure defaults/selection tests pass with set/unset HOME; no configuration I/O in these modules; complete application view/artifact review pending |
| A08–A09 | Partial (module checks pass) | Complete snapshot errors versus empty/missing selection; natural raw-byte order/long digit/locale and fixed monitor membership pass; application dispatch/use pending |
| A10 | Pending | Startup effect ordering and clock/deadline anchoring pending; Monitor supports discovery-data baselines without redundant acquisition |
| A11 | Partial (monitor checks pass) | Symmetric forward/backward wrapping, singleton, view/history/statistics invariance pass; decoded arrows/ignored aliases/prompt redraw and no-effect loop assertions pending |
| A12 | Module fixtures pass; application use pending | Combined snapshot/monitor raw-name removal/rename/additions/return, fixed membership/selection, navigable unavailability and new-run baseline checks pass |
| S01 | Module checks pass; final application use pending | Whole-snapshot raw/max-u64/field tests, large chunked EOF/fresh acquisition tests and production reopen review above |
| S02 | Partial (acquisition/monitor checks pass) | Global read/parse failure versus valid absence, paired reset, unavailable totals and baseline return pass; nonfatal quiet application continuation pending |
| S03–S05 | Module checks pass; final application use pending | Warm-up/totals/independent directions and exact/partial current/average numerical examples pass |
| S06–S07 | Partial (monitor checks pass) | Explicit nanosecond actual-time intervals, ignored nonpositive rounds and no catch-up graph columns pass; production BOOTTIME, wall-clock independence, clock failures and deadline/timerfd effects pending |
| S08–S11 | Module checks pass; final application use pending | Exact deltas/totals, unrounded run extrema/idle/window expiry, both counter-decrease resets, failures/return and never-viewed history pass |
| S12 | Partial (monitor checks pass) | Exact-boundary expiry and retained fitting suffix/read invariance pass; actual screen resize/redraw/placement pending |
| S13 | Partial (proc shape reviewed) | Bulk traversal already reviewed; production clock/timerfd and conversion/failure fixtures pending |
| V01–V08 | Pending | Screen geometry, bars, states, resize, names and formatting have no implemented tests yet |
| T01–T13 | Pending | Scripted terminal/event/integration and Linux PTY evidence not run |
| T14 | Pending | Recorded emulator identity, visual protocols and handled-exit observations required; TERM declaration is insufficient |
| D01 | Pending (intermediate commands pass) | Final application gates and authorized repository handoff still required; exact compiler/commands recorded above |
| D02 | Partial (dependencies and arguments/proc/monitor reviewed) | Graph/licenses, implemented pure contracts and selected borrowed views, production/scripted counter adapters reviewed; screen/runtime/integration effects and final review pending |
| D03 | Pending (foundation files reviewed; lockfile committed) | Identity, license, credit, instructions and compiler record reviewed above; Cargo.lock is in base revision 1977703; final artifact/revision review still required |
| D04 | Pending | No nonprivileged live proc/interface/traffic UI smoke has been run |
| D05 | Pending | Final implemented reference/departure and supersession review required |

## Outstanding verification

- Complete deterministic case-to-test mappings and required application behavior.
- Automated Linux PTY checks with actual OS-state restoration evidence, including
  practical real faults and explicit limitations backed by scripted tests.
- Human-recorded xterm-compatible emulator observations and nonprivileged live
  smoke; neither is replaced by this scaffold's direct execution.
- Final source/artifact review, required command gates and authorized final
  implementation revision. Cargo.lock is already committed in the foundation;
  the intermediate results alone do not satisfy
  the parent map's destination.
