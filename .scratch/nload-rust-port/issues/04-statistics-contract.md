# Decide traffic statistics and sampling behavior

Status: closed
Labels: wayfinder:grilling
Type: grilling
Mode: HITL
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (OpenCode session: statistics/sampling)
Blocked by: 01, 02, 03

## Question

What precise semantics should every rate, average, minimum, maximum, transferred-data total, and graph sample in the default view have? Use the release and Linux-counter findings to decide first-sample behavior, elapsed-time measurement, refresh cadence, averaging window, units and rounding, graph scaling/history, and statistic lifetime across navigation. Specify reset/wrap, delayed sampling, temporary read failure, and interface replacement behavior. Decide which release quirks to retain or fix, with concrete examples that can become acceptance cases.

Use the confirmed [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice) as a constraint. Specify baseline/reset semantics for unavailable periods and returning or reused startup names under its name-based lifecycle policy, and distinguish navigation redraws from traffic samples.

The human expanded scope during this ticket to require command-line incoming and outgoing graph-ceiling overrides. Settle their syntax, units, validation and interaction with the previously confirmed startup arguments here; this extension supersedes the earlier restricted argument set only where explicitly confirmed.

## Comments

### Discussion — 2026-10-01 — first round

The human accepted actual one-second current-rate and five-minute average-rate windows, with shorter available history during warm-up; latest kernel cumulative counters as totals without a since-boot/session claim; restarting both directions' rates/extrema/graph history on either counter decreasing, interface disappearance or failed acquisition, with the next successful observation establishing a baseline and no guessed wraps or cross-unavailable deltas; retaining upstream's automatic binary-scaled bit-rate and byte-total labels with two decimals while avoiding premature truncation/low precision; and graph history for every startup interface independent of prior display, newest on the right, with navigation never resetting or manufacturing history.

The human accepted deadline-based 500 ms sampling, monotonic actual elapsed time, skipping missed slots without fabricated catch-up samples, and redraws not creating observations. Off-screen sampling is acceptable provided its resource cost is modest; the architecture decision should consider collection work for the entire startup list. No performance measurement or hard resource budget has been established.

The human accepted fixed saturating graph ceilings of 10240 binary-scaled kBit/s as defaults, and explicitly required user-supplied incoming/outgoing graph ceilings via arguments. Argument syntax/validation remains to be confirmed. Boundary interpolation, first-sample/extrema rules, suspend timing, formatting thresholds, graph retention/resizing, and the complete-contract confirmation remain open. This is an interim discussion record, not a resolution.

### Supporting evidence — upstream ceiling arguments

Targeted source inspection verified the existing clean `v0.7.4` checkout at `a5715a77eb2c11702a7ffc7664240b9250cafa4d`. Upstream uses separate-token `-i max_scaling` and `-o max_scaling`, binary-scaled kBit/s, independently defaulting to 10240. Zero resets the relevant ceiling to its default; repeated occurrences are last-wins. Its parser accepts a digit-leading integer prefix using `atol`, so decimals/suffix junk are silently truncated, while negative/plus-leading values fail; there is no reliable overflow handling. Attached arguments are not recognized as options. These are evidence, not accepted port rules.

Sources: [pinned argument parser](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L156-L197), [pinned documentation](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/docs/nload.1.in#L60-L69), and [pinned ceiling conversion](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/device.cpp#L105-L120). Local checkout: `/tmp/nix-shell.TfI7CV/opencode/nload-release-defaults/upstream-nload/`. No application code was changed.

### Discussion — 2026-10-01 — second round

The human accepted separate-token `-i VALUE`/`-o VALUE` options before or after the optional interface name until `--`; whole-token unsigned decimal integer validation; binary-scaled kBit/s converted by multiplying by 128; zero restoring the relevant 10240 default; repeated options last-wins; and argument-error status 2/no UI for missing, malformed or overflowing values, with the converted bytes/s ceiling required to fit unsigned 64-bit. Help/version remain standalone, so combining them with ceiling options is an argument error. Signs, fractions, suffixes and attached forms are rejected.

The human accepted uniform-rate estimation within each observed interval for exact window boundaries, weighting interval portions overlapping the one-second/five-minute window and dividing by covered duration; shorter warm-up history is not padded with zeros, and delayed observations use their full actual intervals. Baselines expose cumulative totals but no rates/extrema or graph columns. The first positive-duration interval initializes current/average/minimum/maximum rates. Min/max compare unrounded current rates over the statistics run, without rolling expiry. Navigation does not reset anything; measured idle intervals can establish a zero minimum. A readable decrease immediately becomes the new baseline after resetting both directions; after missing/failed acquisition the next successful observation becomes the baseline. Either counter being invalid invalidates the pair.

The human accepted suspend-aware monotonic elapsed time if straightforward, noting they never suspend and do not need extra suspend-specific work. Linux CLOCK_BOOTTIME provides those semantics as a clock choice, without separate suspend detection/tracking (see [Linux clock evidence](../research/linux-counters.md#6-limits-on-rates-and-totals-from-consecutive-samples)). Observations are timestamped at acquisition completion, with a shared completion timestamp permitted for bulk acquisition; nonpositive-duration observations are ignored without advancing statistics/history.

The human accepted five minutes of graph history per startup interface, one column per successful rate calculation, newest fitting observations shown oldest-left/newest-right, with resize hiding/revealing retained history rather than changing it. Empty space is blank rather than measured zero, and delayed sampling does not manufacture columns; horizontal spacing represents observations, not exact time. Formatting chooses each value's binary prefix before rounding, caps prefixes at GBit/GByte, and rounds to two decimals, nearest with halfway upward. Calculations/extrema/scaling remain unrounded. All discussion branches are settled; the complete-contract summary awaits final human confirmation before resolution.

### Resolution — 2026-10-01 — alice

The human explicitly confirmed the complete contract after accepting both discussion rounds. This resolution is the canonical statistics/sampling decision, including the graph-ceiling argument extension. The preceding comments preserve discussion history.

#### Sampling and time

- Establish a baseline for each readable startup interface and sample every startup interface on a deadline-based 500 ms cadence, including off-screen entries. Keep sampling independent of which interface is selected. Collection work must remain modest; the architecture decision owns backend selection with this requirement in view. No hard resource budget or measured performance claim was established.
- Measure actual elapsed time using suspend-aware monotonic time with Linux `CLOCK_BOOTTIME` semantics. The human accepted this as a straightforward clock choice, not a separate suspend-detection/tracking feature. Wall-clock changes do not affect the calculations.
- Timestamp observations at acquisition completion. A bulk acquisition may give its observations a shared completion timestamp; the API does not promise simultaneous kernel accounting or fresh driver updates.
- When deadlines are missed, take a fresh observation and skip missed slots. Do not fabricate catch-up observations. A delayed observation uses its full actual elapsed interval, including suspend time.
- Ignore an observation with nonpositive elapsed time without advancing statistics or history. Prompt navigation and resize redraws do not create observations or graph samples.

#### Rates, windows and extrema

- Calculate incoming and outgoing statistics independently within the same interface statistics run. `Curr` uses an actual one-second window ending at the latest accepted observation; `Avg` uses an actual 300-second window. During warm-up, use only the shorter available run history, without padding unobserved time with zero traffic.
- Estimate uniform traffic within each consecutive observed interval. Weight the interval's byte delta by the fraction overlapping the requested window, sum those contributions, and divide by the window's covered duration. This is an estimation convention, not recovered packet timing. Long intervals can contribute partially to either window.
- Precisely, for horizon `H`, run baseline time `b`, latest accepted observation time `t`, and consecutive interval `j` with duration `dt_j` and delta `d_j`, use `a = max(b, t - H)` and `rate_H = sum(d_j * overlap_duration(interval_j, [a, t]) / dt_j) / (t - a)`. There is no rate when `t <= b`. Retaining the boundary-crossing interval is necessary even when its earlier endpoint predates the window.
- Preserve unrounded rates for calculations, extrema comparisons and graph scaling; do not truncate to integer bytes/s before those operations.
- The first positive-duration interval initializes `Curr`, `Avg`, `Min` and `Max`. Min/max compare successive unrounded `Curr` values over the statistics run, separately per direction. They do not expire with the average window and are not affected by navigation. A measured zero-traffic interval may establish a zero minimum.

#### Totals, baseline and discontinuity

- `Ttl` is the latest successful kernel cumulative byte count for that direction, without wrap correction. It is neither a session-transferred-byte sum nor a guaranteed since-boot total.
- A baseline makes cumulative totals available but supplies no measured rate, extrema or graph column. Rates/extrema remain not yet available until a positive-duration interval is observed; do not substitute a synthetic startup zero.
- A decrease in either counter, failed acquisition of either counter, or an absent interface resets both directions' rates, extrema and graph history. Either invalid counter invalidates the pair; failed/missing data is not evidence of zero traffic.
- When a successful readable observation reveals a decrease, use that observation immediately as the new baseline after the reset. After failed/missing acquisition, use the next successful observation as the baseline. Do not calculate deltas across an unavailable period or guess a counter wrap.
- Continue attempting acquisition of retained unavailable startup names. A name that returns begins a new statistics run. The name-based lifecycle follows [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice); an undetected replacement entirely between successful observations cannot reliably be distinguished from continuity.
- Exact presentation of not-yet-available statistics versus an unavailable interface belongs to the screen decision. Neither state is measured zero traffic.

#### Graph scale and history

- Each direction has a fixed, independently configurable graph ceiling, defaulting to 10240 binary-scaled kBit/s, equivalently 1,310,720 bytes/s. The ceilings apply throughout the interface list, not only to initial selection.
- Graph samples use the unrounded `Curr` rate. Rates at or above the ceiling saturate the full-height graph; numeric statistics remain uncapped. Exact glyphs and visibility of scale labels belong to the screen decision.
- Retain five minutes of graph samples for every startup interface, independently of whether it has been displayed. Add one timestamped sample per successful rate calculation; a baseline, redraw or ignored observation adds none.
- Show the newest retained samples that fit the graph width, oldest on the left and newest on the right. Shrinking only hides older retained samples; growing reveals retained samples. Missing/unfilled space is blank rather than measured zero traffic. A reset clears history for both directions.
- Delayed acquisition contributes one column rather than invented intermediate columns. Horizontal spacing represents observations, not an exact time axis. Continued off-screen acquisition gives previously unviewed interfaces actual history.

#### Graph-ceiling arguments

- Extend the previously confirmed startup contract with separate-token `-i VALUE` (incoming) and `-o VALUE` (outgoing). Options may appear before or after the at-most-one optional interface name, until `--` terminates option parsing. Existing initial-selection and name-matching behavior remains in force.
- Accept a nonempty whole token of decimal digits only; leading zeros are decimal. Reject signs, fractions, whitespace, suffixes and attached forms such as `-i20480` or `-i=20480`.
- Interpret each value as binary-scaled kBit/s and multiply by 128 to obtain bytes/s. Require the converted ceiling to fit unsigned 64-bit: positive input values are at most 144115188075855871. Avoid overflow during parsing and conversion.
- `0` restores that direction's default of 10240. Each direction defaults independently; repeated occurrences are last-wins. There is no positive-value minimum clamp above 1.
- Missing, malformed or overflowing values are argument errors: diagnostic to stderr, status 2, no terminal UI. Other unsupported arguments and multiple interface names follow the existing argument-error contract.
- Help/version remain standalone informational invocations; combining either with ceiling options is an argument error. Help must describe the ceiling flags, units, defaults and zero behavior. The extension does not introduce short help/version aliases or other upstream options.
- This decision extends the earlier restricted argument set in [Decide the minimal startup and navigation contract](03-startup-navigation-contract.md#resolution--2026-10-01--alice). All other startup, navigation, configuration-independence and exit requirements still apply.

#### Units and rounding

- Retain automatic `Bit/s`, `kBit/s`, `MBit/s`, `GBit/s` rate units, and `Byte`, `kByte`, `MByte`, `GByte` cumulative-total units, with factors of 1024 rather than 1000.
- Select a prefix independently for each displayed value before rounding, choosing the smallest prefix with a value below 1024, capped at GBit/GByte. Values at or beyond the highest prefix remain in that unit.
- Display two fractional decimal places, rounded to nearest with halfway values upward. A pre-rounding value just below a prefix boundary may display `1024.00` in the smaller unit.
- Avoid upstream's low-precision arithmetic, premature integer rate truncation and overflowing intermediate bit-unit conversion. Numerical representation and formatting mechanisms belong to the architecture decision.

#### Concrete acceptance examples

- **Warm-up and current rate:** incoming counters of 1000 bytes at `t=0`, 1100 at `t=0.5`, and 1400 at `t=1` give no rate at baseline, then 200 bytes/s, then 400 bytes/s. The initial minimum is 200 rather than synthetic zero, the maximum becomes 400, and the cumulative total is 1400 rather than the session delta of 400.
- **Partial interval boundary:** counters of 0 at `t=0`, 100 at `t=0.5`, and 400 at `t=1.25` yield current rate `(100 * 0.25/0.5 + 300) / 1 = 350 bytes/s`, and warm-up average `400 / 1.25 = 320 bytes/s`.
- **Five-minute boundary:** counters of 0 at `t=0`, 600 at `t=2`, and 90800 at `t=302` give average `(90800 - 600) / 300` bytes/s. The window starts at `t=2`, excluding the first interval rather than retaining an arbitrary frame count.
- **Delay:** an additional 3000 bytes over a three-second interval represents 1000 bytes/s for that interval, not 6000 bytes/s from dividing by the nominal 0.5-second cadence. Add one graph sample, with no catch-up samples. Suspend duration similarly remains in the denominator.
- **Counter decrease:** RX falling from 4000000000 to 10000 resets both directions and establishes fresh totals/baseline, with no guessed wrap, enormous rate or new graph column. After 500 further bytes over 0.5 seconds, the new RX current/minimum/maximum rates are 1000 bytes/s.
- **Read failure and return:** after a failed RX or TX read, neither direction supplies a measured-zero interval. On return, even higher counters provide a baseline only; do not bridge the gap with a delta. A returning/reused startup name follows the same rule.
- **Navigation and resize:** an interface never viewed still has actual sampled statistics/history when selected. Navigation does not add a column. Shrinking and then growing reveals retained history, without zero-padding or resetting statistics.
- **Ceilings:** `nload -i 20480 -o 5120 eth0` selects `eth0`, with incoming ceiling 2621440 bytes/s and outgoing ceiling 655360 bytes/s; the same ceiling settings apply when navigating. `-i 0` restores only incoming default. `-i 10 -i 20` uses 20. `-i 12.5`, `-o 100M`, a missing value, an overflowing value, and `--help -i 10` exit 2 without UI.
- **Formatting:** 128 bytes/s displays `1.00 kBit/s`; 1024 cumulative bytes displays `1.00 kByte`. Measured zero rates display `0.00 Bit/s`, distinct from baseline/unavailable states whose placeholders the screen decision will select.

#### Deliberate release departures

Actual time windows with interval overlap replace sample-count windows. Suspend-aware monotonic timing replaces wall-clock timing; deadline scheduling replaces sleep-plus-processing drift. Fractional rates survive calculations and extrema comparison until text formatting. Counter decreases/failures reset safely instead of guessing wraps or accepting failed reads as zeros. Baselines do not invent measured zeros. Every interface retains five minutes of actual graph history from startup, without first-display/resize-dependent loss or zero-padding. Ceiling argument validation is strict and uses the port's status-2 argument-error convention rather than permissive prefix parsing and undefined overflow. The release-like field meanings, nominal cadence, binary-scaled units, defaults, saturating bars and graph-ceiling option names remain.

#### Remaining decisions and map review

- [Validate the single-interface screen and terminal behavior](05-single-interface-screen.md) owns layout/glyphs, scale-label visibility, placeholders, unavailable/read-error presentation, sizing and terminal behavior within this contract.
- [Decide the port's architecture and test seams](07-module-boundaries.md) owns efficient whole-list acquisition, clocks, interval/history representation, numerical precision, argument parsing and deterministic test seams.
- [Decide acceptance criteria and the specification handoff](08-acceptance-handoff.md) owns the final acceptance suite and handoff, using the examples above and the earlier startup contract together.
- No newly specifiable question requires a new ticket: the remaining questions already belong to these live tickets. Remove the generic release-calculation-versus-Linux-counter compatibility fog, now resolved here; retain fog that may emerge from screen and architecture decisions. This session resolves only this ticket and produces no application implementation.
