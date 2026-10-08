# Ticket 04 — human emulator report template

**Blank template: no observations recorded yet.** Copy and fill this in, or return
the equivalent in chat. Use `not run`, `not seen`, or `unclear` where applicable.
Keep q, typed Ctrl+C and external SIGTERM reports separate. See
[runnable commands/actions](04-emulator-checklist.md).

## Identity and setup

Current target: permanent `/mnt/programming/rust/nload-rs` integration checkout,
direct `target/btop-review-fixes/release/nload-rs` artifact built from
`5e4661164f7afda85c266b2a45ce730276c71bad` (`implement/btop-review-fixes`),
SHA-256 `d39b77045a2c09ac8aebac41d1a5ef3ca4593d806f90fa3a888578dd7c8b49d7`.
Record actual checkout HEAD and source delta: integration was still `a2214c6`
when the corrected executable was built in the separate review worktree.
Graph rounding changed; the old `50d9a750…` artifact's live smoke does not
validate this one. Fresh agent checks are linked in the checklist; **all human
fields below remain pending**, including separate handled exits.

- Date / observer:
- Worktree / branch / HEAD:
- Executable path / tested artifact source revision:
- Checkout source delta (`git diff 5e4661164f7afda85c266b2a45ce730276c71bad -- src tests Cargo.toml Cargo.lock`):
- Build result / version / SHA-256:
- Emulator name / version / client OS:
- Font / size / fallback / Braille probe and actual graph glyph appearance:
- Theme / default foreground and background:
- UTF-8 and 256-color setup:
- TERM / LANG / LC_ALL / NO_COLOR / COLORTERM:
- SSH / multiplexer layers:
- A tty path / A and B network namespace IDs / UID and EUID:
- Actual interfaces and natural startup positions:

## Run A — 60×16 (T14, V05, D04)

- Actual `stty size` / baseline empty-state observation (or too brief):
- Initial default header:
- Right / Down / Right / Up / Left / Down actual headers and promptness:
- Incoming / outgoing ceiling label text:
- 30×6 stacked graphs, incoming indigo/lavender, outgoing magenta:
- Both bottom shades legible / both tops pale / complete palettes visible:
- Braille single-cell coverage, half-cell appearance, wrap or stray rows:
- Default-style header/labels/status/gap and statistics contrast:
- Separate incoming/outgoing Ttl before / after traffic:
- Separate incoming/outgoing Curr low / high / idle; Min / Max if noted:
- Graph activity/saturation vs uncapped numeric rates:
- Actual measured zeros / true-blank newest positions / older history retained:
- Generator command/status and complete stdout (paste):
- Small size measured before q / warning style and bounds:

## Run B — 80×24 and shrink/grow (T14, V05, D04)

- Actual `stty size` / explicit lo header:
- Incoming / outgoing ceiling labels:
- 50×10 stacked graphs and **both** palettes, bottom/top shades, font coverage:
- Default-style statistics/text contrast / wrapping or stray rows:
- Separate incoming/outgoing Ttl before / after; Curr low / high / idle:
- Measured zero / blank positions / graph activity and saturation:
- Generator command/status and complete stdout (paste):
- Resize dimensions/timing: 80×24 → 60×16 → 59×16 → 80×24 (actual values):
- Earlier recognizable low/high/idle pattern before shrink:
- Right / Left while small: actual selection and warning style:
- What earlier half-cell history reappeared, new small-period history and totals:
- Half-cell recovery/alignment observation (or unclear; exact scripted fixture cited):

## Run C — small startup and external SIGTERM (T14, V05)

- Startup size (target 60×15), warning/default style, initial lo selection:
- Traffic command/status/stdout:
- Small Right / Left actual selection:
- Growth sizes (targets 60×16 then 80×24):
- Actual recovered history and totals acquired while small:
- Printed SIGTERM **child** status (wrapper status separately if recorded):

## Separate handled-exit observations (T14)

Fill **every** cell for each run. Status alone does not establish restoration.

| Run / cause | Actual status | Cursor visible | Default foreground AND background | Normal typing/echo/editing | Prior shell screen/marker returned | Diagnostics |
| --- | --- | --- | --- | --- | --- | --- |
| A / q while small (expected 0) | pending | pending | pending | pending | pending | pending |
| B / physical Ctrl+C (expected 130) | pending | pending | pending | pending | pending | pending |
| C / external SIGTERM child (expected 143) | pending | pending | pending | pending | pending | pending |

## Scripted evidence / failures / pending items

- Baseline / absent / read-failed / recovery states not reliably observed live:
  cite exact scripted tests from checklist §5; identify what was actually seen.
- Measured zero if concurrent traffic prevented a true live zero:
- Exact half-cell history if visual motion was unclear:
- Concrete failures, reproduction action/size/traffic/exit and diagnostics:
- Unperformed or unclear required observations:
- Optional screenshots (identify run, exact dimensions, revision/artifact):

The implementer will link supplied observations to `04-evidence.md` and review
their applicability. The blank template and expected outcomes are not evidence.
