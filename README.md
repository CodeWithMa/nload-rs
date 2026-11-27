# nload-rs

A Linux single-interface network traffic monitor written independently in Rust.

Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.

## Implementation status

The application runs a single-interface view with incoming/outgoing
Braille graphs, traffic statistics and arrow-key navigation through a naturally
sorted interface collection that grows as interfaces appear during monitoring.
Graphs use the terminal's default foreground unless `--color` is supplied.
With colors enabled, incoming uses indigo/lavender grades; outgoing uses magenta.
The [0.1.1 graph handoff](https://github.com/CodeWithMa/nload-rs/blob/c3ae462db5283bbe3eefdaa9112c86e7829b281e/.scratch/btop-graph-implementation/handoff.md)
records the original styled graph verification.

## Build and run

Use stable Rust with edition 2024 support on Linux. `Cargo.toml` is the sole
package-version authority; `Cargo.lock` records the selected dependency versions.

From the repository root:

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked
./target/release/nload-rs
```

Interactive runs require terminal stdin/stdout (stderr may be redirected), UTF-8
output support, and a font with Unicode Braille coverage. `--color` additionally
requires xterm-256color support.
Unset/empty `TERM` and exact `dumb`/`linux` are rejected before terminal mutation;
other values declare assumed support, without capability or locale probes.
Colors are off by default; `--color` enables the existing graph palettes even
under `NO_COLOR`. Locale declarations do not affect this choice. There is no
ASCII mode or compatibility fallback. Below
60×16, a bounded default-style warning replaces the graphs while sampling,
navigation and quit remain active.

Use arrows to navigate, q/Q to quit, or Ctrl+C to exit with status 130.
An optional interface name selects the initial view without filtering the
list. If that name is absent at startup (for example, `wg0` before the VPN starts),
the first naturally sorted interface is selected silently, just as when no name
is supplied, then switches to the requested name the first time it appears, even
if you navigate elsewhere while waiting. This request is honored only once;
disappearance and return never trigger another automatic switch.
Interfaces that appear later become navigable automatically on the
regular 500-ms sampling schedule. Additions keep the same interface selected;
disappeared interfaces remain navigable as unavailable entries, and returning
names reuse their entries with fresh statistics.
Starting with no interfaces remains an error.
Independent `-i VALUE`/`-o VALUE` graph ceilings use integer binary-scaled
kBit/s; each defaults to 10240, and zero restores that direction's default.

Each graph cell combines two actual current-rate observations, older on the
left and newer on the right, with independent bottom-up dot fill. The newest
sample is right-aligned; an unmatched oldest half and measured zero are true
ASCII blanks. Five minutes of observation-driven history are retained for every
retained interface, including those discovered off-screen, independent of screen
size. At the unchanged 500-ms cadence,
one cell nominally spans one second; delayed observations remain observation
spaced rather than forming an exact time axis. Shrink/grow and redraw only hide
or reveal the fitting suffix (60 samples at 60×16, 100 at 80×24).

```sh
./target/release/nload-rs -i 20480 -o 5120 eth0
./target/release/nload-rs --color eth0
./target/release/nload-rs --help
./target/release/nload-rs --version
```

Help/version work with redirected streams and are exempt from interactive
terminal requirements. Locked dependencies do not guarantee byte-identical
executables across builds.

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
the alternate screen. Default-style resets apply with or without color.
Graph presentation is inspired by btop and implemented independently using
Unicode and ANSI escape sequences, without adding dependencies.

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
