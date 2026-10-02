# Decide Rust and delivery constraints

Status: closed
Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: OpenCode
Blocked by:

## Question

What constraints must the subsequent implementation satisfy for Rust toolchain/MSRV, third-party dependencies, license and upstream attribution, binary/package naming, and installation/distribution? Clarify whether the delivered command must be literally `nload`, and how the port should coexist with upstream nload. The earlier suggestions of stable Rust, established terminal libraries, GPL-2.0-or-later, and Cargo installation were recommendations, not agreed requirements. Decide only what this small Linux port needs, and create research questions if missing external facts block a choice.

Decide the Rust port's version value/authority for the `--version` behavior confirmed in [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice), consistently with package naming and upstream attribution.

## Comments

### Resolution — 2026-10-01 — alice

The human settled the choices through the live discussion and explicitly confirmed the final version/output decision and complete shared understanding. This resolution is the canonical engineering and delivery contract.

#### Naming and coexistence

- Use `nload-rs` for both the Cargo package and executable. This is the working name; the human may choose a new name later.
- The command need not be literally `nload`. A distinct executable name allows coexistence with upstream nload without replacing its command. The human intends to use a personal `n` alias; providing an alias is not a delivery requirement.
- Treat the program as an independently written Rust application inspired by nload, with the agreed feature set and deliberate behavioral differences. The pinned upstream release remains the planning/reference baseline, not the new application's identity or version.

#### Rust and dependencies

- Use stable Rust and edition 2024. There is no older-compiler compatibility requirement or separate older MSRV promise.
- Record the exact compiler version used to verify the build.
- Established, maintained third-party libraries are acceptable for terminal handling, argument parsing and Linux counter access. Keep the dependency set small without a numerical cap, and require license compatibility with `GPL-2.0-only`.
- [Decide the port's architecture and test seams](07-module-boundaries.md) selects the actual libraries and backend within these constraints; no specific crate was selected here.

#### License and attribution

- License the newly authored application as **GPL-2.0-only**, not GPL-2.0-or-later. Declare `GPL-2.0-only` in Cargo metadata and include the GPL version 2 license text.
- Write the application from scratch; upstream is an inspiration and behavioral reference, rather than source to translate or adapt.
- Use this README credit, without requiring additional upstream release/commit attribution in the product README:

  > Inspired by [nload](https://github.com/rolandriegel/nload) by Roland Riegel.

- Existing planning/research records retain their upstream release and source references. Upstream's own GPL-2.0-or-later grant is an established fact, not the selected license for this independently authored application.

#### Build and delivery

- Require a reproducible Cargo build and the ability to run the compiled binary directly on Linux.
- Commit `Cargo.lock`, record the tested compiler version, and document `cargo build --release --locked` plus running `./target/release/nload-rs`.
- Here, reproducible means repeatable builds with recorded compiler and locked dependency versions; byte-identical executable output is not required.
- Cargo installation, Git/crates.io publication, downloadable binaries and distribution packages are not required for this delivery target.

#### Version authority and output

- The initial application version is **`0.1.0`**.
- `Cargo.toml`'s package version is the single authority; embed it at build time using Cargo's package-version metadata rather than maintaining a second version literal.
- Initial `--version` output is exactly `nload-rs 0.1.0` followed by a newline. Future version changes follow the package version; upstream `0.7.4` is not the application's version.
- Retain the confirmed startup contract: standalone `--version` writes stdout and exits 0 without interface discovery, terminal checks or terminal initialization; invalid combinations retain the agreed argument-error behavior.

#### Map review and remaining decisions

- Update the map destination's command to `nload-rs` and describe upstream as the inspiration/reference baseline.
- Closing this ticket unblocks [Decide the port's architecture and test seams](07-module-boundaries.md). [Decide acceptance criteria and the specification handoff](08-acceptance-handoff.md) remains blocked on that architecture decision and must include the agreed name, version output and locked-build delivery requirements.
- No new research blocker or decision ticket surfaced. Existing architecture and acceptance tickets own the remaining work; this session resolves only this ticket.
