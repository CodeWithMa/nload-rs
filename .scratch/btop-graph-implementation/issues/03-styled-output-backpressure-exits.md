# 03: Verify styled output under backpressure and interrupted exits

**What to build:** Establish that colored single-interface frames stay byte-exact and responsive when writes are partial or blocked, and that handled exits attempt complete terminal restoration after interrupted output. Exercise the complete application/terminal path, fixing concrete failures found by the verification.

Authority: the [styled graph contract](../../btop-graph-spec/issues/04-screen-seam-contract.md#acceptance-and-test-migration), particularly T08 and T09–T13, together with inherited scheduling/restoration/status requirements in the [minimal Rust port specification](../../nload-rust-port/spec.md).

**Blocked by:** [02: Display paired Braille graph history with direction palettes](02-paired-braille-direction-palettes.md).

**Status:** ready-for-agent

**Assignee:** implementer-03 (OpenCode)

- [ ] Deterministic partial-output fixtures use actual screen-constructed colored Braille frames. Split writes inside multibyte glyphs and every relevant foreground/reset SGR fragment, including interrupted/would-block outcomes, and compare exact resumed bytes without duplicate prefixes, lost suffixes, malformed glyphs, or interleaved frames.
- [ ] Verify only an untouched pending frame can be replaced. Once any byte is emitted, finish that frame before rendering the latest requested view. Navigation/resize requests coalesce to the latest state rather than creating a stale frame queue.
- [ ] Application-loop fixtures demonstrate ongoing input, signals, and observation-driven sampling under colored-output backpressure and sustained input/resize batches. Preserve bounded output batches, shutdown priority, due-acquisition ordering, and zero redraw-induced samples.
- [ ] Verify cancel/reset before cursor-show/alternate-screen-leave on q/Q, typed Ctrl+C, SIGINT, SIGTERM, applicable partial-initialization failures, runtime input/output/clock faults, caught initialization/runtime panics, and Drop fallback. Restore saved termios and changed file flags; diagnostic reporting follows restoration attempts and prior panic-hook restoration.
- [ ] Inject reset and other cleanup failures, including multiple failures, partial writes, interruptions, and restoration panics. Every applicable restoration is attempted, cleanup remains bounded and nonpanicking, cleanup-only failure changes successful quit to status 1, and previously selected 130/143/1 statuses remain unchanged.
- [ ] Real compiled-binary Linux PTY checks cover UTF-8/permitted-SGR frame bounds, normal/signal exits, resize/small-terminal operation, backpressure, and practical initialization/runtime faults. Confirm saved OS state and observable cleanup ordering; do not count PTY bytes as visible palette/font evidence.
- [ ] Record exact automated case-to-test mappings and actual outcomes for affected T01/T07–T13 and V05 coverage. Identify failures a real PTY cannot reliably induce and link their deterministic/scripted evidence. Best-effort cleanup with unwritable output is not reported as proof the reset reached an emulator.
- [ ] Run and record `cargo fmt --check`, `cargo test --locked`, and `cargo build --release --locked`; exercise the release executable through the Linux PTY harness and identify the tested revision/artifact. Concrete failures are resolved or remain explicit open blockers rather than being recorded as passes.
