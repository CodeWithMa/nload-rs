# nload-rs

A Linux single-interface network traffic monitor written independently in Rust.

Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.

## Implementation status

The Cargo application foundation is buildable. Traffic monitoring, argument
handling and the interactive terminal session are still under implementation.
The current executable reports that status on stderr and exits with status 1.

## Build and run

Use stable Rust with edition 2024 support on Linux. `Cargo.toml` is the sole
package-version authority; `Cargo.lock` records the selected dependency versions.
The foundation was verified with rustc 1.98.1 and Cargo 1.98.0 on
`x86_64-unknown-linux-gnu`. Exact toolchain details and check outcomes are recorded
in the [implementation verification record](.scratch/nload-rust-implementation/verification.md).

From the repository root:

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked
./target/release/nload-rs
```

The direct executable currently runs the foundation status diagnostic described
above. Interactive and live verification will be recorded as implementation
progresses. Repeatable builds use the recorded compiler and locked dependencies;
byte-identical executable output is not promised.

## License

This application is licensed under **GPL-2.0-only**. See [LICENSE](LICENSE) for
the GNU General Public License version 2 text. It is provided without warranty.
Dependency license routes and notices are recorded in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
