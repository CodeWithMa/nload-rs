# nload-rs

A Linux single-interface network traffic monitor written independently in Rust.

Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.

## Implementation status

The application runs a single-interface view with incoming/outgoing colored
Braille graphs, traffic statistics and arrow-key navigation through the startup
interface list. Incoming uses indigo/lavender grades; outgoing uses magenta.
The [verified styled graph handoff](.scratch/btop-graph-implementation/handoff.md)
links completed build, deterministic, Linux PTY, human emulator, live-traffic,
and artifact acceptance for version 0.1.1. The
[implementation record](.scratch/btop-graph-implementation/implementation.md)
retains the ticket resolutions and evidence history. The earlier
[verified Rust port handoff](.scratch/nload-rust-implementation/handoff.md)
records historical ASCII-version acceptance.

## Build and run

Use stable Rust with edition 2024 support on Linux. `Cargo.toml` is the sole
package-version authority; `Cargo.lock` records the selected dependency versions.
The application was verified with rustc 1.98.1 and Cargo 1.98.0 on
`x86_64-unknown-linux-gnu`. Exact toolchain details and check outcomes are recorded
in the [final styled graph verification record](.scratch/btop-graph-implementation/05-evidence.md).

From the repository root:

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked
./target/release/nload-rs
```

Interactive runs require terminal stdin/stdout (stderr may be redirected), UTF-8
output support, a font with Unicode Braille coverage, and xterm-256color support.
Unset/empty `TERM` and exact `dumb`/`linux` are rejected before terminal mutation;
other values declare assumed support, without capability or locale probes.
Full-size views always emit colored Braille, including under `NO_COLOR` and any
locale declaration. There is no ASCII mode or compatibility fallback. Below
60×16, a bounded default-style warning replaces the graphs while sampling,
navigation and quit remain active.

Use arrows to navigate, q/Q to quit, or Ctrl+C to exit with status 130.
An optional interface name selects the initial view without filtering the
list. If that name is absent at startup (for example, `wg0` before the VPN starts),
the first naturally sorted interface is selected silently, just as when no name
is supplied. Interfaces that appear later are available after restarting.
Independent `-i VALUE`/`-o VALUE` graph ceilings use integer binary-scaled
kBit/s; each defaults to 10240, and zero restores that direction's default.

Each graph cell combines two actual current-rate observations, older on the
left and newer on the right, with independent bottom-up dot fill. The newest
sample is right-aligned; an unmatched oldest half and measured zero are true
ASCII blanks. Five minutes of observation-driven history are retained for every
startup interface, independent of screen size. At the unchanged 500-ms cadence,
one cell nominally spans one second; delayed observations remain observation
spaced rather than forming an exact time axis. Shrink/grow and redraw only hide
or reveal the fitting suffix (60 samples at 60×16, 100 at 80×24).

```sh
./target/release/nload-rs -i 20480 -o 5120 eth0
./target/release/nload-rs --help
./target/release/nload-rs --version
```

Help/version work with redirected streams and are exempt from interactive
terminal requirements. Interactive and live evidence is linked from the
implementation records. Repeatable builds use the recorded compiler
and locked dependencies; byte-identical executable output is not promised.

On Linux, `cargo test --locked` also runs the real binary through owned PTYs;
these tests require accessible `/proc` and PTY allocation, without administrative
privileges. To run the same checks against the built release executable:

```sh
NLOAD_RS_PTY_BINARY=./target/release/nload-rs cargo test --locked --test linux_pty
```

The environment override is read only by the test harness. PTY protocol/state
checks do not establish visible emulator behavior.

Screen constructs opaque frames with their own dimensions, bounded terminal
cells, Braille/palette encoding, and default-style row/statistics resets. Terminal
owns default-style clearing, positioning, nonblocking byte-exact output, and
staged restoration, including cancel/reset before showing the cursor and leaving
the alternate screen. The
[newer styled graph contract](.scratch/btop-graph-spec/spec.md#supersession-index)
supersedes the historical ASCII thresholds, one-sample-per-cell presentation,
ASCII-row interface, and affected output/restoration promises. Graph presentation
is inspired by btop and implemented independently from standard Unicode/ANSI
facts, without adding dependencies.

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
