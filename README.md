# nload-rs

A Linux single-interface network traffic monitor written independently in Rust.

Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.

## Implementation status

The application runs a single-interface view with incoming/outgoing graphs,
traffic statistics and arrow-key navigation through the startup interface list.
Deterministic module/application-loop checks and automated Linux product PTY
checks pass. Recorded emulator observations, live smoke and final artifact review
are the next verification steps.

## Build and run

Use stable Rust with edition 2024 support on Linux. `Cargo.toml` is the sole
package-version authority; `Cargo.lock` records the selected dependency versions.
The application was verified with rustc 1.98.1 and Cargo 1.98.0 on
`x86_64-unknown-linux-gnu`. Exact toolchain details and check outcomes are recorded
in the [implementation verification record](.scratch/nload-rust-implementation/verification.md).

From the repository root:

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked
./target/release/nload-rs
```

Interactive runs require terminal stdin/stdout and an xterm-compatible `TERM`
declaration. Use arrows to navigate, q/Q to quit, or Ctrl+C to exit with status
130. An optional interface name selects the initial view without filtering the
list. Independent `-i VALUE`/`-o VALUE` graph ceilings use integer binary-scaled
kBit/s; each defaults to 10240, and zero restores that direction's default.

```sh
./target/release/nload-rs -i 20480 -o 5120 eth0
./target/release/nload-rs --help
./target/release/nload-rs --version
```

Help/version work with redirected streams. Interactive and live evidence is
recorded as verification progresses. Repeatable builds use the recorded compiler
and locked dependencies; byte-identical executable output is not promised.

On Linux, `cargo test --locked` also runs the real binary through owned PTYs;
these tests require accessible `/proc` and PTY allocation, without administrative
privileges. To run the same checks against the built release executable:

```sh
NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty
```

The environment override is read only by the test harness. PTY protocol/state
checks do not establish visible emulator behavior.

## License

This application is licensed under **GPL-2.0-only**. See [LICENSE](LICENSE) for
the GNU General Public License version 2 text. It is provided without warranty.
Dependency license routes and notices are recorded in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
