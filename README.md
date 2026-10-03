# nload-rs

A Linux single-interface network traffic monitor written independently in Rust.

Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.

## Implementation status

The application runs a single-interface view with incoming/outgoing graphs,
traffic statistics and arrow-key navigation through the startup interface list.
Deterministic module/application-loop checks, automated Linux product PTY checks,
recorded Alacritty emulator observations and nonprivileged live traffic smoke
pass. Final acceptance gates and the specification/artifact audit also pass;
see the [verified implementation handoff](.scratch/nload-rust-implementation/handoff.md).

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
list. If that name is absent at startup (for example, `wg0` before the VPN starts),
the first naturally sorted interface is selected silently, just as when no name
is supplied. Interfaces that appear later are available after restarting.
Independent `-i VALUE`/`-o VALUE` graph ceilings use integer binary-scaled
kBit/s; each defaults to 10240, and zero restores that direction's default.

```sh
./target/release/nload-rs -i 20480 -o 5120 eth0
./target/release/nload-rs --help
./target/release/nload-rs --version
```

Help/version work with redirected streams. Interactive and live evidence is
linked from the verification record. Repeatable builds use the recorded compiler
and locked dependencies; byte-identical executable output is not promised.

On Linux, `cargo test --locked` also runs the real binary through owned PTYs;
these tests require accessible `/proc` and PTY allocation, without administrative
privileges. To run the same checks against the built release executable:

```sh
NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty
```

The environment override is read only by the test harness. PTY protocol/state
checks do not establish visible emulator behavior.

## Arch Linux installation

The [local-checkout PKGBUILD](packaging/arch/PKGBUILD) builds an `x86_64` package
from this repository, including local source edits. After cloning, install the
build tools and run these commands from the repository root:

```sh
sudo pacman -S --needed base-devel rust
cd packaging/arch
makepkg -si
nload-rs
```

Run `makepkg` as your normal user; it requests elevated privileges when installing
dependencies or the package. An existing Rustup installation with the stable
toolchain can also provide Cargo in place of the `rust` package.

The recipe reads the version from `Cargo.toml`, stages sources separately from
the checkout's `target/`, fetches locked dependencies, then builds and tests
offline. Dependency fetching requires network access unless they are already
cached. Tests require accessible `/proc` and PTY allocation. The package installs
`/usr/bin/nload-rs`, the README, and license notices; `pacman` tracks these files.

To rebuild and reinstall after updating or editing the checkout, run
`makepkg -fsi` from `packaging/arch/`; `-f` replaces an existing package archive
even when the version has not changed. To uninstall:

```sh
sudo pacman -R nload-rs
```

## License

This application is licensed under **GPL-2.0-only**. See [LICENSE](LICENSE) for
the GNU General Public License version 2 text. It is provided without warranty.
Dependency license routes and notices are recorded in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
