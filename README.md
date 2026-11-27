# nload-rs

Watch live incoming and outgoing network traffic on Linux with Braille graphs
and per-interface statistics.

![nload-rs showing incoming and outgoing traffic on the loopback interface with colors enabled](docs/screenshot.png)

*The loopback interface (`lo`) under local test traffic, with `--color` enabled.*

## Quick start

From this checkout, with stable Rust that supports edition 2024:

```sh
cargo run --release --locked --
```

Press the arrow keys to switch interfaces and `q` to quit. The graphs use your
terminal’s foreground color by default. To enable color and start on a specific
interface:

```sh
cargo run --release --locked -- --color lo
```

Replace `lo` with the interface you want to watch. If it isn’t present at
startup, nload-rs starts on the first available interface and switches to the
requested one when it first appears. New interfaces become available as they
appear; disconnected interfaces stay in the list and are marked unavailable.

Use `-i VALUE` and `-o VALUE` to set the incoming and outgoing graph ceilings.
Both default to 10,240 kBit/s (10 MBit/s in binary units). Run
`cargo run --release --locked -- --help` for all options.

Use a terminal at least 60×16 with UTF-8 and a font that renders Braille.
`TERM` must be nonempty and neither `dumb` nor `linux`. `--color` requires
256-color support and takes precedence over `NO_COLOR`.

## Install on Arch Linux

Build the `x86_64` package from this checkout using the
[PKGBUILD](packaging/arch/PKGBUILD):

```sh
sudo pacman -S --needed base-devel rust
cd packaging/arch
makepkg -si
nload-rs
```

Run `makepkg` as your normal user. If you already have stable Rust and Cargo
through Rustup, you can skip installing the `rust` package.

## Development

```sh
cargo fmt --check
cargo test --locked
```

The tests require access to `/proc` and the ability to allocate a
pseudoterminal (PTY).

## Credits and license

Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel,
with graph styling inspired by [btop](https://github.com/aristocratos/btop).

Licensed under [GPL-2.0-only](LICENSE). See
[third-party notices](THIRD-PARTY-NOTICES.md) for dependency licenses.
