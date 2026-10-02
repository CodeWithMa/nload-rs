# Verified nload-rs implementation handoff

Accepted 2026-10-02 against [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), sections 1–11.
The [final acceptance audit](issues/10-final-acceptance.md#resolution--2026-10-02--alice)
records the resolution; the [verification record](verification.md#final-acceptance-audit--2026-10-02)
holds revision, environment, commands, coverage and artifact evidence.

## Delivered application

- Linux package/executable **nload-rs 0.1.0**, independently authored Rust,
  edition 2024, **GPL-2.0-only**.
- Single-interface incoming/outgoing graphs and current/average/run-extrema/
  cumulative-total statistics; naturally ordered fixed startup membership.
- Exact optional original-byte interface selection, all-four-arrow navigation,
  independent `-i`/`-o` ceilings, standalone help/version.
- BOOTTIME-anchored 500-ms sampling, paired availability/reset handling,
  five-minute off-screen history and small-terminal warning/recovery.
- Owned raw/nonblocking Linux terminal session with handled quit/signal/failure
  cleanup. Interactive stdin/stdout must be terminals with an xterm-compatible
  TERM declaration; unset/empty/dumb/linux declarations are rejected.

## Run and reproduce

From the repository root, on Linux with the recorded stable toolchain:

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked
./target/release/nload-rs
```

Optional initial selection and binary-scaled ceiling overrides, using an actual
interface name from your startup list:

```sh
./target/release/nload-rs -i 20480 -o 5120 lo
./target/release/nload-rs --help
./target/release/nload-rs --version
```

Arrows navigate, q/Q exits 0, typed Ctrl+C/external SIGINT exits 130 and external
SIGTERM exits 143. Each ceiling defaults independently to 10240 binary kBit/s;
zero restores that direction's default. Rates are binary-scaled bits/s and totals
binary-scaled bytes. Totals are interface-accounted counters, not session sums.

## Acceptance evidence

- All three final gates pass; **93 unit/scripted/native tests plus 11 real-product
  Linux PTY tests pass**, with zero failed or ignored tests.
- The direct release executable is identical to the earlier release-PTY-tested
  and human-emulator-tested binary. Alacritty **0.17.0 (`94e7c887`)** observations
  pass layout/navigation/small-mode history recovery and prior-screen/cursor
  cleanup after q/Ctrl+C/SIGTERM. Nonprivileged live loopback smoke passes.
- All **52 acceptance cases** have their required evidence. Practical PTY fault
  limitations and corresponding exact scripted/native checks remain explicit in
  the verification record; PTY bytes are not substituted for visual evidence.
- Compiler: rustc **1.98.1**, commit
  `48a229ceaefd4985c50990b14116b6d856af0985`, LLVM 21.1.8; Cargo **1.98.0**,
  commit `797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`.
  Verified target **x86_64-unknown-linux-gnu**, NixOS 26.11.0, Linux 7.2.8.
- Required direct dependencies: libc **0.2.189** and signal-hook **0.4.4**,
  signal-hook defaults disabled. Locked target/features and all six third-party
  MIT notice routes are reviewed; [dependency notices](../../THIRD-PARTY-NOTICES.md)
  and [GPL v2 text](../../LICENSE) accompany the source.

## Repository and executable identity

Tested repository revision: **`1e3a2da49002cbf054199d635ee54c239701a5e0`**.
Application source, tests, manifest, lockfile and license artifacts have no
uncommitted changes. `Cargo.lock` is tracked and was committed in
`1977703de7d9321c92545ad6fc864c2099e2dc50`; the committed-lockfile condition passes.

The audit produced a documentation-only delta: README, this handoff, the
verification record, final acceptance ticket and parent map. No commit was made
in the audit session. These files were subsequently committed in `53adbfa`
(`docs: complete final acceptance audit and implementation handoff`), without
changing the application implementation. The tested revision above identifies
the implementation accepted by that audit.

Release SHA-256:
`7ad245c475a9a29c3abac31b8f7e69d8ef6c06b216d20a9cebc21a4fefd998c1`.
Build output stays ignored under `target/`; rebuild with the commands above.
Repeatability uses the recorded compiler and locked versions, with no
byte-identical-build promise.
