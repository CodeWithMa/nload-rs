# Arguments and proc snapshot interfaces

Implemented by [Implement arguments, proc snapshots and startup ordering](issues/02-arguments-proc-startup.md).
Acceptance authority: [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), particularly sections 2, 3.1, 4.1 and 8.
These are crate-private contracts for the monitor/application consumers.

## Arguments

[Arguments](../../src/arguments.rs) exposes:

```rust
parse(tokens: impl IntoIterator<Item = OsString>) -> Result<Invocation, ArgumentError>
help_text() -> String
version_text() -> String
```

The application passes `std::env::args_os().skip(1)`; **exclude the executable
token**. Tests can supply OS tokens directly, including non-UTF-8 bytes.

- `Invocation::Help` / `Version`: standalone informational outcome. The text
  functions return newline-terminated text; the application writes stdout and
  returns 0 before clock/discovery/terminal work. Both identity and version text
  use Cargo build metadata.
- `Invocation::Run(RunConfig)`: `interface: Option<Vec<u8>>` holds exact optional
  selection bytes, not a display name. `ceilings: Ceilings` holds positive
  `incoming` / `outgoing` bytes/s shared by every startup interface.
- `ArgumentError`: implements `Display` / `Error`; the application writes stderr
  and returns 2 before effects. No invalid occurrence is forgiven by a later
  replacement. Attached options and mixed informational invocations are errors.

`Ceilings::default()` supplies 1,310,720 bytes/s independently in each direction.
The parser owns ASCII decimal validation, checked conversion by 128, zero/default
behavior, repetition, the positional limit and option termination. A token after
`--` is a name even if it is `--help`, `-i` or `--`. Selection existence is checked
only against successful nonempty discovery: an empty/non-kernel positional name
therefore remains an exact requested name rather than a syntactic error.

These functions perform no I/O or runtime environment/configuration access. Text
generation does not write a stream or choose an exit status.

## Complete snapshot and counter effect seam

[Proc sampler](../../src/proc_sampler.rs) exposes:

```rust
trait CounterSource {
    fn acquire(&mut self) -> Result<Snapshot, AcquisitionError>;
}

Snapshot::parse(bytes: &[u8]) -> Result<Snapshot, SnapshotError>
snapshot.get(name: &[u8]) -> Option<Counters>
snapshot.startup(requested: Option<&[u8]>) -> Result<StartupSelection, StartupError>
counter_outcome(round: &Result<Snapshot, AcquisitionError>, name: &[u8]) -> CounterOutcome
```

- `ProcSampler::default()` is the production `CounterSource` adapter: opens
  `/proc/net/dev` freshly for every call, reads through EOF using a reusable
  growable byte buffer, then validates the whole snapshot. There is no fixed
  buffer, per-name traversal, clock access or acquisition timestamp in this
  module. It does not emit diagnostics.
- `Snapshot` privately owns a complete map of original byte names to exact
  `Counters { incoming: u64, outgoing: u64 }`. `get` copies paired counters;
  `None` means absence **only in this valid snapshot**.
- Parsing verifies both kernel headers (allowing padding differences), a final
  newline, a nonempty name after removing proc padding, all 16 ASCII unsigned
  decimal u64 fields and unique names. RX is field 1 and TX is field 9.
  Non-UTF-8 name bytes survive unchanged. Blank/malformed/extra/truncated rows,
  internal name whitespace/NUL, malformed/overflowing fields and duplicates
  reject the entire snapshot. Headers alone are a valid empty discovery.
- `AcquisitionError::Read(io::Error)` and `Parse(SnapshotError)` implement
  `Display` / `Error`. No rows are salvaged after either failure.
- `counter_outcome` supplies `Readable(Counters)` / `Absent` from `Ok(snapshot)`
  and `ReadFailed` for **every** retained name on `Err`. Its caller iterates the
  retained startup names, ignoring newly added/renamed names and retrying every
  original name. Runtime failure is nonfatal; the monitor owns resets and the
  application avoids repeated stderr diagnostics.

`ScriptedCounters` is the test-only `CounterSource` adapter. Construct it with
`ScriptedCounters::new(rounds)` where rounds are ordered `Result<Snapshot,
AcquisitionError>` values; it counts calls and panics on unexpected extra
acquisition. Build successful fixture snapshots through `Snapshot::parse`; build
read/parse failures through the error enum. This supports explicit-time
application tests without production test-only flags.

## Startup selection and ownership

`snapshot.startup(config.interface.as_deref())` returns all names in natural
numeric order and a zero-based `selected` index. Natural digit runs are compared
by significant-digit length and bytes, never bounded integer conversion;
non-digits use case-sensitive byte comparisons. Only a fully equal natural
comparison falls back to complete original bytes. No locale or interface
activity/address filter participates.

- `StartupError::EmptyDiscovery`: distinct diagnostic/status 1, taking precedence
  over a requested name when discovery is empty.
- If exact requested bytes are absent from successful nonempty discovery, select
  the first naturally sorted entry silently, just as with no requested name.
- Initial acquisition errors remain enumeration failure/status 1, never an empty
  list. Discovery failures finish before terminal mutation.

Startup sorting borrows the snapshot, preserving the same paired counters for
monitor baselines. The application initializes BOOTTIME, acquires once, timestamps
completion using the clock seam, then computes startup selection before terminal
validation/setup. Retain the returned membership/selection for the monitor;
do not rerun discovery at runtime or acquire a second initialization snapshot.
Use the shared completion timestamp for the initial baselines and anchor the
first deadline to completion + 500 ms. Clock conversion/scheduling and terminal
effects remain with their owning modules/application.

The binary is still the foundation entry point. Actual stream/status dispatch,
effect-order assertions, monitor construction and runtime wiring are owned by
[Integrate startup, sampling and the interactive application loop](issues/07-application-integration.md).
Passing parser tests alone does not establish those application guarantees.
