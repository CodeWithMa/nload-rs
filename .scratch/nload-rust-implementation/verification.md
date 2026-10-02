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

## Screen run — 2026-10-02

Scope: [Implement the pure ASCII screen and formatting](issues/04-ascii-screen.md).
Caller contract: [Screen rows interface](screen-interfaces.md).

### Revision, environment and commands

- Base revision: `98aedd7744825774db0090242bbef1bdce381058`; the worktree was
  clean before claiming the ticket. Tested implementation is the uncommitted
  renderer and fixtures in `src/screen.rs` on that base. No commit was made.
- `rustc --version --verbose` reconfirmed **1.98.1**, commit
  `48a229ceaefd4985c50990b14116b6d856af0985`, host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8. `cargo --version --verbose`
  reconfirmed **1.98.0**, commit
  `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`, NixOS 26.11.0.
  Dependencies, manifest/lockfile and other application modules were unchanged.
- Commands ran from the repository root. Fixtures use supplied dimensions, views
  and explicit observation times, with exact row/text/glyph assertions; they have
  no real-time sleeps, actual terminal I/O or private monitor-queue access.

| Command | Actual outcome |
| --- | --- |
| `cargo fmt && cargo test --locked screen::tests` (initial run) | Formatting exit 0; focused tests exit 101, 11 passed/1 failed at the exact 1.70-cell bar threshold |
| `cargo fmt && cargo test --locked screen::tests` (after scaling correction) | Exit 0; **12 passed**, 0 failed; 28 unrelated tests filtered out |
| `cargo fmt --check` | Exit 0 |
| `cargo test --locked` | Exit 0; **40 passed**, 0 failed/ignored |
| `cargo build --release --locked` | Exit 0; 61 dead-code warnings for modules awaiting application wiring |
| `git diff --check` | Exit 0; no tracked-file whitespace diagnostics |

The initial threshold failure exposed production arithmetic: calculating
`10 * (1.7 / 10)` can round below the exact 1.70-cell boundary. Scaling before
division and comparing in whole-bar coordinates avoids that artifact and the
separate loss caused by subtracting a cell's integer level. Saturation is explicit.
The fixture's exact glyph expectations were retained; no epsilon/tolerance was
introduced. The resize fixture also uses a distinct final pair to detect an
oldest-prefix/reversed-history bug that a purely periodic pattern would miss.
The final checks above ran after these changes.

### Deterministic case-to-test evidence

All names below are actual passing tests in `screen::tests`. Render fixtures
exercise the rows interface with supplied selected views or actual Monitor views.
Formatting fixtures also check the pure formatting helpers directly, including
adjacent exact integer totals beyond f64 integer precision.

| Passing test | Applicable cases and observations |
| --- | --- |
| `exact_80_by_24_stacked_rows_and_bottom_aligned_fields` | V01/V03: all 24 padded rows compared exactly; 50x10 directional graphs, required label/header/separator rows, rightmost 28-column fields/two-column gaps and field order; half-height/full/saturated columns newest-right and no footer/IP/axis/time text |
| `minimum_and_odd_geometry_keep_gap_fields_and_outgoing_extra_row` | V01: 60x16, 61x17 and 80x25; exact direction labels/fields/gaps, bottom alignment and outgoing extra row |
| `fractional_cell_thresholds_are_exact_bottom_up_and_saturating` | V02: both graphs just below/at/above 0.30/0.70/1 at the bottom and next cell; explicit above-ceiling saturation and uncapped numeric current text |
| `rate_units_choose_prefix_before_explicit_halfway_up_rounding` | V07: zero, 128 bytes/s, exact 0.125 Bit/s halfway, 127.9995 bytes/s pre-rounding boundary, both sides/at all binary prefix boundaries, halfway examples in each prefix and values beyond the G cap |
| `total_units_round_exact_widened_integers_including_maximum_u64` | V07/V08: exact integer totals, 1024-byte prefix, halfway-up totals, both sides/at every prefix boundary, max-u64 `17179869184.00 GByte` and one-byte-separated large totals straddling a hundredth boundary |
| `independent_ceiling_labels_and_numeric_prefixes_do_not_cap_rates` | V02/V07/V08: separate 20.00/5.00 MBit/s labels, uncapped numeric 20.00 MBit/s above outgoing ceiling, independently different units in Curr/Avg/Min/Max/Ttl |
| `largest_exact_totals_and_ceilings_and_overwide_rates_keep_labels` | V06/V08: actual Monitor u64-max deltas/totals over one nanosecond, largest valid incoming ceiling with exact widened bit conversion, independent smallest outgoing ceiling, `[too wide]` for each rate field without losing labels, complete exact total text and saturated graphs |
| `baseline_absence_failure_idle_and_recovery_have_exact_rows` | V04: actual Monitor baseline/absence/failure/recovery transitions; exact statuses/header/placeholders/totals, empty unavailable graphs and genuine measured idle zeros with normal separator |
| `raw_names_are_ascii_escaped_shortened_and_keep_position` | V06: NUL/ESC/DEL/non-ASCII bytes escape to uppercase `\xHH`; exact full-size ellipsis/header, no split escape at shortening, tiny-size position preservation and unchanged identity bytes |
| `small_and_zero_dimensions_show_only_bounded_warning_selection_status` | V05: all states at 59x16/60x15/60x16 and zero/extremely tiny dimensions; exact warning/header/status where fitting, printable bounded ASCII rows and blank remaining warning rows |
| `fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view` | V03/V05/S12: 70 actual delayed/idle samples, fitting 70/30-column suffixes, short-history right alignment with a measured-zero column, shrink/small/grow/repeated redraw returning identical rows and unchanged complete borrowed view/sample count |
| `sampling_and_navigation_while_small_reappear_as_actual_history_on_growth` | V03/V05/S11: sample both startup entries while rendering tiny rows, navigate to the never-displayed entry while small, recover its three real columns and actual numeric current on growth without catch-up or redraw-created columns |

### Module review and remaining integration evidence

- Screen's sole caller entry point takes a borrowed selected view and dimensions,
  returning owned printable ASCII rows. It performs no I/O, unsafe operations,
  clock reads, monitor mutation, history expiry or terminal protocol output.
  Graph suffix selection neither pads with samples nor changes their timestamps.
- Prefix choice precedes rounding. Rates use unrounded f64 bytes/s until bit/unit
  conversion and explicit positive halfway-up hundredths. Integer totals and
  ceiling bit conversions use u128 throughout prefix selection and rounding.
  Maximum legal Monitor-derived rates also fit u128 hundredths at the G cap.
  Overflow presentation keeps field labels and replaces a complete too-long
  value; names are escaped/shortened separately from their original identity.
- The stable rows interface records padding, ownership and zero dimensions.
  Actual positioning/right-margin behavior, bounded frame output and redraw/input
  effects remain terminal/application obligations. V05 quit continuity, required
  Linux PTY and recorded emulator checks are pending; pure rows cannot establish
  visual no-wrap/cleanup behavior. V01 visual evidence belongs to T14.
- No specification conflict, new domain term or newly specifiable corrective work
  surfaced. Existing runtime, integration and acceptance tickets own the remaining
  evidence. The foundation entry point still awaits wiring, so no direct UI,
  PTY/emulator or live acceptance run is claimed by this pure-module ticket.

## Terminal session run — 2026-10-02

Scope: [Implement staged terminal ownership and shutdown cleanup](issues/05-terminal-session.md).
Caller contract: [Terminal session and exit interface](terminal-session-interfaces.md).

### Revision, environment and commands

- Base revision: `a892ef28683faebba47fc230e43eef48cef5ddea`; the worktree was
  clean before the ticket claim. Tested implementation is the uncommitted
  `src/terminal.rs` update and `src/terminal/{linux.rs,session.rs,session/tests.rs}`
  additions on that base. No commit was made.
- `rustc --version --verbose` reconfirmed **1.98.1**, commit
  `48a229ceaefd4985c50990b14116b6d856af0985`, host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8. `cargo --version --verbose`
  reconfirmed **1.98.0**, commit
  `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`, NixOS 26.11.0.
  Manifest, dependencies, lockfile and other application modules were unchanged.
- Commands ran from the repository root. Scripts use effect logs/controlled
  failures, not sleeps or timing as a numerical oracle. Native adapter checks use
  an owned Linux PTY, a full nonblocking pipe, and a subprocess of the test
  harness for process-global signal registration. There is no product test flag.

| Command | Actual outcome |
| --- | --- |
| `cargo fmt && cargo test --locked terminal::` (first implementation) | Exit 0; 14 terminal fixtures pass; compiler identified private-bound visibility warnings, subsequently corrected |
| `cargo fmt && cargo test --locked terminal::` (after exit-interface/adapter additions) | Exit 0; 17 fixtures pass |
| `cargo fmt && cargo test --locked terminal::` (final source) | Exit 0; **19 passed**, 0 failed; 40 unrelated tests filtered out |
| `cargo fmt --check` | Exit 0 |
| `cargo test --locked` | Exit 0; **59 passed**, 0 failed/ignored |
| `cargo build --release --locked` | Exit 0; 99 unused-import/dead-code warnings for modules awaiting application/runtime wiring |
| `git diff --check` | Exit 0; no tracked-file whitespace diagnostics |

The final harness has three expected unused-interface warnings. No private-bound
warnings remain. The product entry point still emits its foundation diagnostic;
this ticket does not claim direct product UI/PTY/emulator/live verification.

### Session case-to-test evidence

All names below are actual passing tests under `terminal::session::tests`.
Assertions observe Session results, returned ExitReport, effect calls and the
scripted external terminal state, not the owner's private stage flags.

| Passing test | Applicable cases and observations |
| --- | --- |
| `support_rejects_each_stream_and_term_before_snapshot_or_mutation` | T01: independent nonterminal streams; TERM unset/empty/dumb/linux; only support read occurs, no saved-state/notification/mutation effects |
| `ownership_snapshot_and_notifications_precede_every_mutation` | T02/T09: exact validation→snapshot→notification→raw→flags→dimensions→screen order; original flags retained, exact setup/restoration bytes, all states restored, cleanup/Drop idempotence |
| `existing_nonblocking_flags_are_preserved_without_claiming_changes` | T02/T10: both already-nonblocking descriptions avoid changes/restoration claims while termios/screen/notifications still restore |
| `every_initialization_failure_restores_only_applicable_attempted_effects` | T10: failure at snapshot, notification preparation, raw, each flag mutation, dimensions, screen entry and cursor hide; prior attempted changes restored, later untouched changes unclaimed, primary diagnostic/status 1 after all applicable attempts |
| `partial_screen_commands_are_recorded_even_when_the_remainder_fails` | T10: screen-entry/cursor-hide commands fail after four bytes and would-block; attempted changes undone with cancel-prefixed commands, all applicable OS states restored |
| `interrupted_and_short_setup_writes_resume_without_repeating_emitted_bytes` | T10 setup-write portion: EINTR/short writes resume exact suffixes without duplicates |
| `zero_write_is_fatal_and_still_restores_the_attempted_screen_change` | T10: zero output becomes WriteZero; attempted screen still left, saved OS states restored |
| `small_and_resized_dimensions_are_not_session_failures` | T01/V05 session portions: other raw/non-UTF-8 TERM declaration accepted; zero/tiny/normal dimensions returned; resize does not fail the session |
| `shutdown_statuses_and_runtime_failures_report_only_after_cleanup` | T09/T11 boundary portions: normalized Quit/Interrupt/Terminate choose 0/130/143; caller-supplied input/clock/output failures choose 1; complete restoration before ExitReport is returned. These caller-supplied errors do not establish actual runtime detection |
| `multiple_cleanup_failures_preserve_selected_status_and_attempt_every_operation` | T13: five simultaneous screen/termios/flag failures; notification release still attempted; Quit promotes to 1, 130/143/1 preserved, each cleanup error diagnosed, no duplicate Drop attempts |
| `cleanup_backpressure_is_bounded_and_does_not_skip_os_restoration` | T13: partial cursor-show then would-block/failed writability wait; alternate leave and every OS restoration still attempted; one cleanup error |
| `interrupted_cleanup_wait_resumes_within_the_same_bounded_write` | T13: interrupted writability wait, partial write and repeated backpressure resume exact restoration bytes |
| `drop_fallback_restores_an_owner_without_explicit_cleanup` | T10/T12: dropping an initialized owner performs every applicable restoration and releases notifications |
| `drop_during_unwinding_survives_an_additional_restoration_panic` | T10/T12: owner drops while unwinding; a second panic in cursor restoration is caught without abort/double-panic escape; remaining states restore |
| `pre_session_startup_errors_keep_argument_and_failure_statuses_without_effects` | A08 boundary portion: supplied selection/enumeration errors retain 2/1 with zero session effects. Actual discovery/error dispatch remains integration-owned |
| `caught_initialization_runtime_and_cleanup_panics_restore_the_prior_hook` | T12/T13: unwinding during setup/runtime and two restoration panics; no previous-hook calls before restoration, chosen status retained, caught-panic/cleanup diagnostics returned, explicit subsequent panic invokes the restored prior hook |

### Linux adapter evidence

These three passing tests are under `terminal::linux::tests`; they supplement
scripted logic evidence with native Linux operations, not the product PTY campaign.

| Passing test | Applicable cases and observations |
| --- | --- |
| `full_pipe_and_collated_tokens_do_not_replace_atomic_signal_identity` | T06 adapter portion: fill pipe to EAGAIN, set INT/TERM/resize flags, drain to independently checked EAGAIN before flag handling; INT wins, resize consumed once, shutdown sticky and independent of token count |
| `linux_adapter_raw_flags_and_dimensions_round_trip_on_owned_pty` | T02/V05 adapter portions: actual owned PTY cfmakeraw disables ISIG/canonical/echo/extensions/input/output translation, eight-bit mode and VMIN/VTIME; shared OFD nonblocking flags and 80x24→1x1 dimensions; termios fields/speeds/control bytes and exact original flags restored |
| `registered_signals_wake_and_unregister_without_owned_fd_leaks` | T06 adapter portion: isolated test subprocess registers actual SIGWINCH/SIGTERM/SIGINT actions; raises signals synchronously, zero-time poll sees a pipe wake, INT wins and resize consumed; unregister closes reader/all owned FDs, frees flag registrations, subsequent WINCH no longer sets the detached flag, FD count returns to baseline |

### Review and remaining integration evidence

- Unsafe operations are confined to the concrete Linux adapter and its native
  checks. Signal handlers use signal-hook atomic and Unix pipe actions with
  disabled default features; flag-before-wake order follows the locked registry's
  documented registration order. No thread/iterator, frame renderer, clock or
  counter work occurs in the signal path.
- The staged owner exists before snapshots/setup; notification actions are owned
  before registration/mutation. Partial writes and syscall errors conservatively
  claim attempts. Cleanup keeps output nonblocking for cancel/show/leave writes,
  using finite 100-ms per-command retries, then attempts termios and each changed
  flag regardless of earlier failures. Drop and per-operation panic catching
  supply the best-effort fallback. Signal-hook unregister removes owned actions,
  not the previous OS handler; the application returns normally to process exit.
- The guarded closure covers startup and runtime; it returns diagnostics only
  after explicit restoration and prior-hook restoration. Selected status 2 is
  also representable for pre-session selection errors. The integration ticket
  must place discovery/selection/init/execution inside that closure and emit
  stderr after its return, without `process::exit`.
- T01 informational exemptions/redirected stderr and real stream/status behavior
  remain application/product-PTY work. T02 typed Ctrl+C/Ctrl+Z needs decoding and
  integrated PTY behavior. T06 product idle-poll wake and loop shutdown priority,
  T09 actual control/signal exits, T10 practical product initialization faults,
  T11 real detection of runtime faults and T12 application boundary wiring remain
  pending under the existing runtime/integration/PTY tickets.
- T03–T05, T07–T08 and BOOTTIME/timerfd are not implemented by this session ticket.
  T13 has scripted owner/boundary evidence; actual application use remains to be
  verified. No full product P/E/L case is promoted by these adapter checks.
- No contractual conflict, new domain decision or concrete corrective ticket
  surfaced; the existing fog and downstream tickets remain applicable.

## Terminal runtime run — 2026-10-02

Scope: [Implement incremental input and nonblocking runtime effects](issues/06-terminal-events.md).
Caller contract: [Terminal runtime and clock interface](terminal-runtime-interfaces.md).

### Revision, environment and commands

- Base revision: `4f9ac1ad4ad518e1da8a9eb0b2058a82f83093bb`; worktree was clean
  before the ticket claim. Tested source is the uncommitted terminal root/Linux
  re-export updates and new `src/terminal/input.rs`, `src/terminal/runtime.rs`,
  `src/terminal/runtime/{scripted.rs,tests.rs}` and
  `src/terminal/linux/runtime{.rs,/tests.rs}` on that base. No commit was made.
- `rustc --version --verbose` reconfirmed **1.98.1**, commit
  `48a229ceaefd4985c50990b14116b6d856af0985`, host
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8. `cargo --version --verbose`
  reconfirmed **1.98.0**, commit
  `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`, NixOS 26.11.0.
  Manifest, dependency graph and lockfile are unchanged.
- Commands ran at the repository root. Deterministic fixtures use explicit clock
  readings and shared effect-script handles; they observe ordered events, effect
  requests and emitted bytes, never decoder/frame internals. Native checks use
  owned nonblocking pipes and actual BOOTTIME timerfds. No sleeps or actual elapsed
  rates serve as numerical oracles; the native due-timer check uses a finite
  one-second poll wait for an already-past absolute deadline.

| Command | Actual outcome |
| --- | --- |
| `cargo fmt && cargo test --locked terminal::` (initial runtime implementation) | Exit 0; **33 passed** (14 new), 40 unrelated tests filtered out |
| `cargo fmt && cargo test --locked terminal::` (after extra input/ready-output coverage) | Exit 0; **35 passed** (16 new), 40 unrelated tests filtered out |
| `cargo fmt --check` | Exit 0 |
| `cargo test --locked` | Exit 0; **75 passed**, 0 failed/ignored |
| `cargo build --release --locked` | Exit 0; release binary builds; unused-import/dead-code warnings for interfaces awaiting application wiring |
| `git diff --check` | Exit 0; no tracked-file whitespace diagnostics |

The final test harness reports five unused-interface warnings; no failure occurred.
The product entry point is still the foundation scaffold, so these intermediate
commands do not establish direct product UI/PTY/emulator/live acceptance.

### Deterministic runtime case-to-test evidence

All names below are actual passing tests under `terminal::runtime::tests`.

| Passing test | Applicable cases and observations |
| --- | --- |
| `every_csi_and_ss3_fragment_boundary_yields_one_ordered_arrow` | T03/A11 input portions: each of eight unmodified arrow forms split at every boundary, individual ESC/introducer/final reads, repeated mixed CSI/SS3 events, exact forward/backward order and no repeated event on timeout processing |
| `typed_interrupt_does_not_wait_behind_an_incomplete_escape_sequence` | T02/T04/T09 input portions: raw Ctrl+C immediately yields Interrupt during ESC, CSI, SS3 and control-string fragments, without waiting for parser completion/timeout |
| `unknown_modified_and_string_sequences_consume_command_looking_bytes` | T04: every byte fragmented for modified CSI/SS3, PageUp/PageDown, Tab/Enter/Ctrl+Z, ESC/CSI q/Q finals, intermediate sequences and OSC/DCS/SOS/PM/APC payloads; no spurious navigation/quit; plain q/Q/3 yield exact Quit/Quit/Interrupt events |
| `idle_timeout_is_a_wait_deadline_and_resets_at_exact_boundary` | T05: below/exactly/above 100-ms boundary, exact residual wait timeout and zero-time overdue poll request, fragment completion clears timeout, no input read during timeout-only processing; parser timeout creates no counter-acquisition effect |
| `sustained_input_and_overlong_sequences_are_bounded_and_discarded` | T05/T07 runtime portion: 10,000-byte unknown CSI consumed in one 256-byte read per batch, command-looking final ignored, following genuine arrow/quit retained; overlong incomplete sequence resets after idle timeout; constant-size/no-byte-buffer parser verified by source review |
| `untouched_frames_replace_but_partial_frames_finish_without_duplicate_bytes` | T08: initial EAGAIN leaves frame replaceable; short writes including a split ESC control prefix make replacement fail; EINTR and additional short writes resume exact suffix; final full emitted stream is current frame then latest frame, with no obsolete frame, duplication or interleaving |
| `backpressure_keeps_clock_input_signals_resize_and_timer_effects_available` | T06/T08 runtime portions: repeated EAGAIN output across explicit sample-length times, simultaneous input/signal/timer readiness, exact navigation, dimension reads, clock readings and pending Interrupt/resize remain available; every wait monitors pending output alongside the other effects |
| `output_batches_are_finite_and_frames_reject_out_of_bounds_protocol_injection` | T08/V05/V06 terminal portions: no frame payload outside supplied width/height, nonprintable/Unicode/control injection rejected; zero-size encoding; a >20-KiB row emits only 16 KiB in the first batch, then finishes with correct row positioning/trailing-space trimming |
| `input_eof_and_effect_faults_are_fatal_while_transient_io_yields` | T11: input EOF/BrokenPipe, zero/invalid-count/BrokenPipe output, poll, signal inspection and dimension failures return fatal errors; input EINTR/EAGAIN yields an empty batch without blocking |
| `clock_failure_regression_and_idle_deadline_overflow_have_no_fallback` | S13/T11: explicit clock failure prevents a wait effect, backwards readings fail, u64-end input deadline overflow fails, no substitute clock/time or counter-unavailable outcome |

These fixtures exercise `Runtime<ScriptedEffects>` through the same caller-facing
methods used by production. The scripted clock and terminal effects are available
to application integration via `terminal::scripted`; application scheduling and
counter calls are intentionally observed by that downstream ticket's own fixtures.

### Native Linux runtime evidence

All names below are actual passing tests under `terminal::linux::runtime::tests`.
They supplement deterministic evidence with real FD syscalls, not the product PTY
campaign or visible terminal protocols.

| Passing test | Applicable cases and observations |
| --- | --- |
| `checked_timespec_conversion_rejects_invalid_fields_and_overflow` | S13: negative seconds/nanoseconds, nanosecond field at one billion, multiplication overflow and exact u64 addition-overflow boundary rejected; exact round trips for 0/1/subsecond/second, values above f64 integer precision and maximum-u64 nanoseconds |
| `owned_boottime_timer_is_absolute_nonblocking_and_consumes_due_zero_deadline` | S13/S07 runtime portions: actual BOOTTIME readings are nondecreasing; FD has NONBLOCK/CLOEXEC; absolute zero deadline is immediately due rather than disarmed; readiness count consumed; future timer has positive remaining time and zero repeat interval (one shot) |
| `poll_detects_each_owned_fd_hangup_error_and_invalid_descriptor` | T11/S13: native pipe hangup and POLLNVAL exercised in each input/signal/timer/output poll position; broken output-reader POLLERR detected without a dangerous SIGPIPE write |
| `timer_effect_errors_and_incomplete_reads_are_fatal` | S13/T11: replace owned timerfd with an owned pipe to induce settime failure; transient EAGAIN yields, short expiration-count read and EOF fail; replacement drops the real FD normally without dangling ownership |
| `linux_runtime_reports_ready_output_transient_input_and_actual_eof` | T02/T11 runtime portions: output ready only when requested, EAGAIN input, actual bytes 3/26/Q read unchanged, input EOF and subsequent poll hangup detected on native nonblocking pipes |
| `real_nonblocking_backpressure_does_not_hide_input_signal_or_timer_readiness` | T03/T06/T08 native runtime portions: fill actual output pipe to EAGAIN, retain queued frame, poll simultaneous fragmented input and signal wake with output not ready, decode following arrow, drain/inspect sticky INT flag, read clock and wake on due timer while output remains full |

The established full/collated signal-pipe and real registered-signal tests also
continue passing in the full run. Runtime delegates notification inspection to
that same drain-before-flags implementation; it does not interpret signal bytes.

### Module review and remaining integration evidence

- Unsafe syscalls are confined to `terminal/linux.rs` and its concrete runtime
  adapter/native checks. Decoder, frame state and shared scripted effects are
  safe Rust. No threads, async runtime, new dependencies, counter acquisition or
  monitor mutation were introduced. Clock and waiting use BOOTTIME throughout;
  existing session `Instant` use remains limited to finite cleanup retry budgets.
- The clock owns the timerfd; the runtime exclusively borrows session/clock
  owners. Input/output batches each perform one syscall, preserving caller control
  for shutdown-first fairness. Poll monitors all required owned FDs and gates
  writable interest on the single pending frame. Expiration counts are discarded;
  next-slot scheduling and completion timestamp placement remain application-owned.
- Decoder memory is constant: no accumulated sequence bytes, a length capped at
  65 to detect >64 and a discard state through terminator/timeout. Exact idle
  expiry is checked before the next fragment; genuine raw Ctrl+C remains an
  immediate explicit shutdown control. q/Q inside an unknown sequence is consumed.
- One encoded frame owns its emitted offset. A newer frame replaces only offset
  zero, including after EAGAIN/EINTR. Partial frames resume without an output-only
  wait. Encoding clears/positions bounded ASCII rows without CR/LF or mode changes;
  right-margin/resize visual observations still require the product PTY/emulator
  work. Cleanup cancels a possible abandoned escape prefix through the existing
  Session restoration contract.
- A10/S06/S07/S13 acquisition timestamps/first-future deadline behavior, T07
  shutdown-first sample/input ordering and T08 continuing **actual application
  acquisitions** under backpressure are pending in
  [Integrate startup, sampling and the interactive application loop](issues/07-application-integration.md).
  Module availability under backpressure is passing evidence, not fairness proof.
- T03/T04 real fragmented product input, T06 idle product signal wake, T09 actual
  exits, and T11 propagation/detection plus exhaustive restoration in the live
  application require integration and practical PTY evidence. The scripted/native
  failures above establish effect detection, while existing session tests establish
  cleanup/error arbitration; their connection is not yet claimed passing.
- No new product/domain decision, specification conflict or concrete corrective
  ticket surfaced. Existing downstream tickets and map fog remain applicable.

## Application integration run — 2026-10-02

Scope: [Integrate startup, sampling and the interactive application loop](issues/07-application-integration.md).

### Revision, environment and commands

- Base revision: `fa03cdd22ce8bbcdc5f281db90dc6fb82e596e58`; the worktree was
  clean before claiming the ticket. Tested implementation is the uncommitted
  application/main wiring, application fixtures and terminal test-adapter/hook-lock
  changes on that base, plus the updated README. No commit was made.
- Compiler reconfirmed: rustc **1.98.1**, commit
  `48a229ceaefd4985c50990b14116b6d856af0985`, host/target
  `x86_64-unknown-linux-gnu`, LLVM 21.1.8. Cargo **1.98.0**, commit
  `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`, NixOS 26.11.0.
  Kernel: `Linux nixos-dev 7.2.8 #1-NixOS SMP PREEMPT_DYNAMIC Fri Sep 25
  14:37:14 UTC 2026 x86_64 GNU/Linux`.
- Manifest, dependencies and committed lockfile are unchanged. The dependency/
  license review above continues to describe the actual graph; final review is
  still owned by the final-acceptance ticket.
- All commands ran from the repository root. Tests use explicit BOOTTIME integers,
  acquisition durations, readiness and effect failures. No sleeps or private
  monitor queues are used as oracles.

| Command/check | Actual outcome |
| --- | --- |
| `cargo fmt && cargo test --locked application::tests` (final focused run) | Exit 0; **18 passed**, 75 unrelated tests filtered out |
| `cargo fmt --check` | Exit 0 |
| `cargo test --locked` | Exit 0; **93 passed**, 0 failed/ignored |
| `cargo build --release --locked` | Exit 0; real application binary produced; one dead-code warning for Session dimension/notification convenience methods, whose production paths are accessed through Runtime |
| `env -u HOME LC_ALL=C ./target/debug/deps/nload_rs-ed75f073191bbfc4 application::tests::startup_reuses_discovery_and_orders_clock_before_terminal_mutation --exact` | Exit 0; 1 passed, 92 filtered out |
| `env HOME=/tmp/opencode LC_ALL=en_US.utf8 ./target/debug/deps/nload_rs-ed75f073191bbfc4 application::tests::startup_reuses_discovery_and_orders_clock_before_terminal_mutation --exact` | Exit 0; identical startup/default/list/view fixture, 1 passed, 92 filtered out |
| Direct release binary, subprocess-captured streams with TERM unset: `--version`, `--help`, `--unknown`, no arguments | Checks exit 0: version status 0/exact `nload-rs 0.1.0\n`/empty stderr; help status 0/721 stdout bytes/required usage/empty stderr; unknown option status 2/empty stdout/stderr diagnostic; interactive nonterminal status 1/empty stdout/`interactive stdin and stdout must be terminals` diagnostic |
| `git diff --check` | Exit 0; no tracked whitespace diagnostics |

Early fixture development had two compiler errors from consuming an owned saved
view before subsequent harness assertions; those assertions now borrow the view.
The first compiled focused run passed 14/16: a failed cursor-hide *attempt* correctly
requires cursor-show cleanup, and a 300-byte input fixture left unread suffixes
ahead of its scripted quit. Corrected expectations and 255-byte sustained batches
passed 16/16. Adding shared ordering/diagnostic checks and two integration cases
then passed 18/18; the full final gates above ran on those final sources.

These Git blob identities identify the exact tested source without claiming an
implementation commit (computed with `git hash-object` in the listed order):

| File | Tested blob |
| --- | --- |
| `src/main.rs` | `2e198d9d7e5785ae87a230e0fc3e412eb556fd30` |
| `src/application.rs` | `489bab1a8586acd0c192444b6fd7e95ddb114be9` |
| `src/application/tests.rs` | `18ae130b1b0967190d3fead90b38f04b5d29b337` |
| `src/terminal.rs` | `6d89ef529bc248d3ba33fa215cc3578d985dae39` |
| `src/terminal/session.rs` | `0689f772e338f6f7890c8a285b5d45c12a4d2617` |
| `src/terminal/session/scripted.rs` | `0668feab96681dc6c2522a02c37750bc9617ef1b` |
| `src/terminal/session/tests.rs` | `80de527347b6059f10876a3588aef5291995aedc` |
| `README.md` | `c45bd199389aae888d844cebe4e0950940f83506` |

### Deterministic integration case-to-test evidence

All names below are actual passing tests under `application::tests`. They cross
`invoke`, the same guarded startup function as production, and/or the same
`drive` loop with scripted counter/clock/terminal adapters. Saved monitor assertions
use selected views only. The scripted stderr writer asserts notification release
has already occurred before writing any diagnostic after notification setup.

| Passing test | Applicable cases and observations |
| --- | --- |
| `informational_and_syntax_paths_have_correct_streams_status_and_zero_effects` | A01–A03/A06/T01: exact informational output, argument diagnostics/statuses, aliases/combinations/range/grammar errors; interactive effects never called, no counter/clock/session work |
| `startup_errors_stop_before_terminal_validation_and_remain_distinct` | A04/A08/S13: missing exact selection status 2; empty/read/parse/clock-init/completion-clock failures status 1 with distinct diagnostics and no terminal effects |
| `startup_reuses_discovery_and_orders_clock_before_terminal_mutation` | A07/A09/A10: shared effect log proves clock initialization→initial clock read→snapshot→completion timestamp→support/snapshot/notifications→mutation; natural default selection/full membership, original discovery totals/no graph column, exactly one acquisition, deadline completion+500 ms; HOME/locale variants above |
| `raw_selection_and_ceiling_options_reach_the_fixed_monitor` | A04–A06: non-UTF-8 exact selection, leading-dash name after `--`, options before/after name, repeated/leading-zero/direction-local zero behavior, minimum/maximum valid ceilings, navigation retaining complete membership and global independent ceilings |
| `slow_initialization_and_slow_acquisition_skip_slots_use_completion_and_actual_time` | A10/S06/S07/S13: two-second UI setup plus one-second acquisition; one overdue round, completion timestamp at three seconds, actual 1,000 bytes/s/one column and next anchored deadline at 3.5 seconds |
| `navigation_resize_and_parser_timeout_redraw_without_acquisition` | A11/S12/T03–T05: CSI/SS3 forward/reverse arrows and excluded aliases/modified keys, prompt full-size selection frame after small/resize/timeout operations; only discovery acquisition, no graph samples, parser idle deadline monitored in waits |
| `sustained_bounded_input_rechecks_due_acquisition_before_next_batch` | T07/A11/S07: 255-byte arrow batches, time becoming due during a read, acquisition before the following read, continued due sampling, every read bounded to 256 bytes |
| `shutdown_preempts_due_sampling_input_resize_and_rendering` | T06/T07/T09: pending initial signals and simultaneous signal/input/output/timer wakes; INT/TERM statuses 130/143; no due acquisition, read, resize or frame output after pending shutdown; full cleanup |
| `runtime_failure_absence_addition_and_return_are_quiet_nonfatal_fixed_membership` | A12/S02/S10: valid absent versus global parse/read failure, fixed A/B entries despite additions/rename targets, exact unavailable frame statuses, quiet continuation/retry and higher-counter return baseline/no bridged samples |
| `backpressure_and_partial_frames_preserve_sampling_and_latest_redraw` | T07/T08: EAGAIN permits untouched baseline replacement, four-byte emitted prefix pins the old selected frame, another EAGAIN does not stop actual acquisition/navigation, old frame finishes before latest measured state; exactly two frames/no duplicate prefixes and real off-screen sample |
| `blocked_output_still_handles_shutdown_without_an_extra_sample_or_write` | T06–T08: output remains blocked when shutdown/deadline coincide; no further sample/write, chosen 130 and cleanup |
| `shutdown_arriving_during_acquisition_stops_before_input_or_frame_work` | T06/T07: signal injected during an already-started counter round is inspected at completion; no subsequent input, resize or output work; status 130 and cleanup |
| `small_session_keeps_offscreen_sampling_and_recovers_real_history_on_growth` | A11/S11/S12/V05: start 10x3, navigate and sample while small, grow to 80x24 with the selected off-screen entry's two actual timestamped columns/current numeric value; quit/cleanup remain available |
| `actual_loop_runtime_faults_restore_before_reporting` | S13/T11: actual loop detects EOF, poll/output/clock/resize/notification errors and returns fatal 1 with complete cleanup before diagnostics |
| `initialization_and_caught_startup_runtime_panics_use_the_guarded_application_path` | T10/T12: each setup-effect failure is propagated; counter startup/runtime and cursor-hide panics go through the production guarded path, return caught-panic diagnostic/status 1 and perform applicable restoration |
| `real_loop_quit_interrupt_term_and_cleanup_fault_arbitration` | T09/T13: actual decoded q/Q/Ctrl+C and normalized external INT/TERM; five cleanup failures all attempted, q/Q become 1 while 130/143 remain, five diagnostics after attempts |
| `unsupported_interactive_session_is_rejected_after_discovery_before_mutation` | T01/A10: discovery completed before support validation; rejected session has only support read, no mutation and stderr/status 1 |
| `deadline_overflow_is_fatal_before_mutation_or_after_one_delayed_acquisition` | S07/S13/T11: checked initial and first-future deadline arithmetic near u64 maximum fail rather than wrap; runtime failure restores the session |

The existing passing argument/proc/monitor/screen/runtime/session fixtures supply
the remaining exact numerical, natural-order, parser, simultaneous-INT/TERM-flag,
partial-setup-write, exhaustive restoration and prior-panic-hook checks described
above. Application checks now establish their orchestration, not a replacement
for their detailed fixtures or for real product terminal evidence.

### Implementation review and remaining acceptance

- [Main](../../src/main.rs) uses `args_os`, ordinary `ExitCode` return and the
  application dispatch; it no longer runs a scaffold diagnostic. Informational
  dispatch finishes before creating any interactive adapters.
- [Application](../../src/application.rs) owns scheduling/redraw only. Its guarded
  startup creates the BOOTTIME clock before bulk discovery, timestamps completion,
  sorts/selects once, reuses that snapshot for all baselines, then initializes
  the terminal. Production execution borrows the owned session/clock runtime.
- Every loop batch inspects drain-before-flags notifications before checking
  deadlines. Due rounds traverse proc once, timestamp completion, and skip to the
  first future anchored slot. Input has bounded reads; resize/navigation/parser
  processing never requests acquisition. Shutdown is checked after acquisition
  and between bounded work; dirty redraws coalesce behind the single resumable
  Runtime frame. No clock fallback, thread or new dependency was introduced.
- Application and scripted adapters contain no unsafe operations. SessionOps is
  re-exported internally to express the generic startup seam, not as a public
  library. The existing process-global panic-hook tests and new application tests
  share a mutex; production still installs the guard once on its single thread.
  Diagnostics are written only after guarded cleanup/hook restoration returns.
- README now documents the working invocation, controls and independent ceiling
  overrides and accurately identifies the outstanding verification.
- Product Linux PTY checks, recorded emulator observations and nonprivileged live
  smoke are **pending**, as is final revision/artifact acceptance. The redirected
  subprocess checks are direct Linux execution but not interactive/live evidence.
  No new contractual conflict, domain decision or corrective ticket surfaced.

## Acceptance case ledger

These are section 10 case identifiers, not issue references. Each future evidence
entry must give actual test names or recorded manual observations, not an intended
implementation. Deterministic evidence now passes for the integrated application;
remaining PTY/emulator/live and final build/artifact-review obligations stay explicit.
The parent effort's acceptance remains open.

| Cases | Latest status | Evidence / remaining work |
| --- | --- | --- |
| A01–A03 | D passes; A01 final B review pending | Module text/grammar and application streams/status/zero-effect fixtures pass; redirected release help/version/invalid invocation checks also pass |
| A04–A06 | D passes | Raw selection/full membership, before/after-name options, independent/global ceilings, limits/repetition/zero and grammar/no-effect dispatch pass |
| A07 | D passes; final B review pending | Application startup fixture passes under set/unset HOME and differing locale; source contains no configuration access |
| A08–A10 | D passes | Distinct discovery/selection/clock failures, natural ordering, exact effect order, baseline reuse/completion timestamp and long-initialization anchored deadline fixtures pass |
| A11 | D passes; real input/visual evidence pending in T03/T14 | Symmetric arrows/wrapping/singleton, aliases ignored, zero acquisition/sample/reset effects and prompt redraw fixtures pass |
| A12 | D passes | Module and loop removal/rename/addition/failure/return cases retain membership/selection and reset to baseline without bridging gaps |
| S01–S05 | D passes | Complete proc/EOF/fresh traversal and exact numerical fixtures above, with quiet nonfatal failure/absence behavior connected in the application |
| S06–S07 | D passes; S06 final B review pending | Actual intervals/nonpositive observations, BOOTTIME/timerfd native checks, anchored completion deadlines/missed slots, one delayed sample and first-future checked arithmetic pass |
| S08–S12 | D passes | Exact deltas/extrema/resets/off-screen history/boundary expiry/resize rendering pass; loop also preserves actual history through small-mode sampling/navigation/growth |
| S13 | D and intermediate B review pass; final B review pending | Owned BOOTTIME/timerfd and checked conversions/effect faults, bulk traversal and application completion timestamps pass |
| V01–V04, V06–V08 | D passes; V01 visual evidence pending in T14 | Exact pure screen geometry/bar/state/name/overflow/unit/rounding/large-value fixtures pass and rows are connected to runtime frames |
| V05 | D passes; P/E pending | Small-mode actual loop sampling/navigation/quit and growth/history recovery pass; product PTY and emulator evidence required |
| T01–T04 | D passes; P pending | Existing support/raw/decoder fixtures and integrated informational/no-effect/support/typed-control handling pass; native adapter checks are not product PTY acceptance |
| T05 | D passes | Exact idle/buffering module fixtures and integrated parser wait/timeout-without-acquisition pass |
| T06 | D and intermediate B review pass; P/final B pending | Drain-before-flags/full-pipe/simultaneous-INT arbitration and native wake checks pass; integrated pending/during-acquisition/backpressured shutdown passes |
| T07–T08 | D passes | Bounded batches, due-before-next-input fairness, shutdown priority, coalesced dirty redraw, untouched replacement/partial resumption and continuing actual acquisitions under EAGAIN pass |
| T09–T11 | D passes; practical product P pending | Actual loop controls/failure propagation, staged/partial initialization and exhaustive restoration before diagnostics pass |
| T12–T13 | D passes; T12 final B review pending | Application startup/runtime panics use guarded cleanup; prior hook restored/nonpanicking fallback verified; actual loop exits with multiple cleanup failures choose/preserve required statuses |
| T14 | Pending | Recorded emulator identity, visual protocols and handled-exit observations required; TERM declaration is insufficient |
| D01 | Pending (intermediate commands pass) | Final application gates and authorized repository handoff still required; exact compiler/commands recorded above |
| D02 | Intermediate review passes; final B review pending | Graph/licenses, five deep modules and application connections, production/scripted effect seams, pure views/rows and adapter-only unsafe reviewed |
| D03 | Intermediate files reviewed; final B review pending | Working invocation/controls/build docs, Cargo-derived identity/version, license, credit, compiler and committed Cargo.lock established; final artifact/revision review required |
| D04 | Pending | No nonprivileged live proc/interface/traffic UI smoke has been run |
| D05 | Pending | Final implemented reference/departure and supersession review required |

## Outstanding verification

- Final deterministic coverage/artifact review against every specification section;
  current module and application mappings/checks pass as recorded above.
- Automated Linux PTY checks with actual OS-state restoration evidence, including
  practical real faults and explicit limitations backed by scripted tests.
- Human-recorded xterm-compatible emulator observations and nonprivileged live
  smoke; neither is replaced by redirected informational/nonterminal execution.
- Final source/artifact review, required command gates and authorized final
  implementation revision. Cargo.lock is already committed in the foundation;
  the intermediate results alone do not satisfy
  the parent map's destination.
