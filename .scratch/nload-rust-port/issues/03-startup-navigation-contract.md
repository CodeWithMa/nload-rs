# Decide the minimal startup and navigation contract

Status: closed
Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: startup/navigation)
Blocked by: 01

## Question

For the scoped single-interface view, what exact observable startup and keyboard contract should the port provide? Decide initial interface selection and ordering (including loopback and inactive interfaces), supported arrow keys and wrapping, quit behavior, whether upstream configuration files can affect defaults, and behavior when interfaces are added, renamed, removed, or absent. Specify `--help`, `--version`, an interface-name argument for initial selection, unsupported arguments, and non-terminal use without adding upstream feature modes. Preserve the human's narrow scope and identify any deliberate departures from the release.

Use [Establish the release's default single-interface behavior](01-release-default-behavior.md) as evidence: the release discovers interfaces once, sorts by name, includes loopback/inactive interfaces, and wraps only forward. Configuration can change the default view. These are facts to discuss, not requirements already accepted by the human.

## Comments

### Discussion — 2026-10-01 — first round

The human accepted including all interfaces and selecting the first, but required natural numeric name ordering (`eth2` before `eth10`) rather than upstream lexicographic ordering. The human corrected the scope to include `--help`, `--version`, and an interface-name argument for initial selection; argument syntax and edge cases remain under discussion.

The human accepted startup-only discovery; all four arrows, symmetric wrapping and one step per key event with prompt selection redraw; no PageUp/PageDown, Tab or Enter aliases; fixed defaults without reading/writing upstream configuration or depending on `HOME`; `q`/`Q`, SIGINT and SIGTERM exits with terminal restoration, status 0/130/143 respectively; ignoring other keys; and requiring terminal stdin/stdout for the interactive view (stderr may be redirected; failure exits 1 with a stderr diagnostic before terminal initialization). This is an interim discussion record, not the ticket's resolution. Informational argument behavior and lifecycle edge cases remain open.

### Discussion — 2026-10-01 — second round

The human accepted all second-round recommendations: natural numeric ordering with case-sensitive, locale-independent non-digit comparison and complete raw-name tie-breaking (`eth02` before `eth2`); at most one exact, case-sensitive positional interface name selecting only the initial view, with `--` as an option terminator; unknown names and multiple names are argument errors (stderr, status 2, no UI); standalone `--help` and `--version` write stdout and exit 0 without terminal requirements or interface discovery, while combinations/unknown flags are argument errors and short aliases are omitted. Version output identifies the Rust port's own version; the value remains with the engineering-constraints decision.

The human also accepted distinct startup diagnostics/status 1 for no discovered interfaces versus enumeration failure; a fixed, name-based startup list retaining unavailable entries and selection across removal/rename, ignoring new names until restart, and resuming monitoring if an original name becomes readable again (not a guarantee of underlying interface identity); and diagnostic/status 1 plus restoration of any changed terminal state if a usable UI cannot initialize. Missing requested names in a successfully discovered list exit 2. Baseline/reset handling remains with the statistics decision; screen layout, sizing/resizing and exact unavailable presentation remain with the screen decision. All discussion branches are settled; the complete-contract summary awaits final human confirmation before resolution.

### Resolution — 2026-10-01 — alice

The human explicitly confirmed the complete contract after accepting both discussion rounds. This resolution is the canonical startup/navigation decision; the preceding comments preserve discussion history.

#### Startup and arguments

- With no arguments, open the single-interface view with the first naturally sorted interface selected. Include every discovered interface, including loopback and inactive interfaces.
- Compare digit runs by numeric value (`eth1`, `eth2`, `eth3`, `eth10`), and non-digit characters case-sensitively without locale-dependent ordering. If names compare equal naturally, break the tie by the complete raw name (`eth02` before `eth2`).
- Allow one optional positional interface name, matched exactly and case-sensitively. It changes only initial selection; the entire startup list remains navigable. Support `--` to terminate option parsing, including selection of names beginning with `-`.
- Standalone `--help` and `--version` write informational output to stdout and exit 0 without interface discovery, terminal requirements, or terminal initialization. Version output identifies the Rust port's own version, not an assertion that it is upstream v0.7.4. The port's version value remains an engineering-constraints decision.
- Unknown flags, multiple interface names, unsupported argument combinations (including help/version combined with each other or an interface name), and a requested name absent from a successfully discovered list are argument errors: diagnostic to stderr, status 2, no terminal UI. No short help/version aliases are included.
- Neither read nor write upstream configuration files. Defaults do not depend on `HOME` being set.

#### Navigation and interface lifecycle

- Discover interface names once at startup and retain their natural ordering for the process lifetime.
- Right/Down selects the next interface; Left/Up selects the previous interface. Wrap symmetrically at both ends, applying one step per key event. Redraw selection promptly without waiting for the next traffic sample. With one interface, arrows have no effect.
- PageUp/PageDown, Tab and Enter are not navigation aliases. Ignore other keys except the quit controls below.
- Removed or renamed interfaces retain their original entries as unavailable. Preserve the current selection and allow navigation away; do not automatically select another entry.
- Ignore newly added names, including rename targets, until restart. If an original startup name becomes readable again, resume monitoring that entry. This contract is name-based and does not promise continuity of underlying interface identity across disappearance or name reuse.

#### Exit and terminal behavior

- `q`/`Q` exits 0; Ctrl+C/SIGINT exits 130; SIGTERM exits 143. Restore terminal state on these exits.
- Interactive mode requires terminal stdin and stdout; stderr may be redirected. Non-terminal interactive invocation reports a diagnostic to stderr and exits 1 before terminal initialization. Informational invocations are exempt.
- With no interfaces discovered, or with interface enumeration failure, report a diagnostic to stderr and exit 1 before terminal initialization. Distinguish empty discovery from failed discovery; an explicitly requested name absent from a successfully discovered list is instead status 2.
- If a usable terminal UI cannot initialize, restore any terminal state already changed, report a diagnostic to stderr, and exit 1.

#### Deliberate release departures

Natural numeric ordering replaces lexicographic ordering. Symmetric per-event wrapping and prompt selection redraw replace forward-only wrapping, repaint-time index normalization and sampling-delayed navigation. Configuration files and the `HOME` requirement are removed. Navigation aliases and additional upstream argument/mode support are omitted. Empty discovery and unusable interactive invocation receive defined error behavior rather than inheriting upstream's undefined or terminal-dependent paths. Signal-triggered exits have explicit statuses. An interface-name argument selects an initial view without filtering the navigation list.

The release's startup-only discovery, inclusion of loopback/inactive interfaces, four arrow directions, and quit controls remain. The accepted port behavior is the contract above, not a requirement to reproduce other release quirks.

#### Remaining decisions and map review

- [Decide traffic statistics and sampling behavior](04-statistics-contract.md) owns baseline/reset semantics for unavailable periods, returning names and possible replacement, plus sampling and statistic lifetimes. Prompt navigation redraw must not implicitly create a traffic sample.
- [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md) owns exact unavailable/read-error presentation, layout, terminal sizing/resizing and remaining cleanup expectations.
- [Decide Rust and delivery constraints](06-engineering-constraints.md) owns the Rust port's version value and naming/delivery constraints.
- [Decide the port's architecture and test seams](07-module-boundaries.md) owns the backend and mechanisms implementing this name-based contract, input handling, prompt redraw, sampling and cleanup.
- [Decide acceptance criteria and the specification handoff](08-acceptance-handoff.md) owns verifiable acceptance scenarios for the agreed CLI, sorting, navigation, lifecycle and exit behavior.

These are already precise questions in existing tickets; no new ticket or fog graduation is required. The map destination and scope were corrected during discussion to include help/version and initial interface selection. This session resolves only this ticket and produces no application implementation.

### Subsequent scope extension — 2026-10-01

The human expanded the argument scope during [Decide traffic statistics and sampling behavior](04-statistics-contract.md#resolution--2026-10-01--alice). Its confirmed resolution owns the graph-ceiling option extension and supersedes this ticket's restricted argument set to that extent. Other requirements in this resolution still apply.
