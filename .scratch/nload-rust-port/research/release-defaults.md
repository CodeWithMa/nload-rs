# nload v0.7.4: Linux no-argument single-interface behavior

Research for [Establish the release's default single-interface behavior](../issues/01-release-default-behavior.md), completed 2026-10-01. Scope: upstream Linux behavior when invoked as `nload`, the single-interface view, and navigation between interfaces. This is evidence for later decisions, not a port specification.

## 1. Release identity and evidence levels

**Pinned release:** [`v0.7.4`](https://github.com/rolandriegel/nload/tree/v0.7.4) resolves to commit **`a5715a77eb2c11702a7ffc7664240b9250cafa4d`**, authored/committed by Roland Riegel on 2012-02-06, message `prepare new release`. `configure.in` declares version `0.7.4`. The author's homepage identifies the same release and links this GitHub repository and the release archive.[^release]

Verification performed in the isolated research worktree:

```text
git rev-parse HEAD refs/tags/v0.7.4 refs/tags/v0.7.4^{}
a5715a77eb2c11702a7ffc7664240b9250cafa4d
a5715a77eb2c11702a7ffc7664240b9250cafa4d
a5715a77eb2c11702a7ffc7664240b9250cafa4d

sha256sum nload-0.7.4.tar.gz
c1c051e7155e26243d569be5d99c744d8620e65fa8a7e05efcf84d01d9d469e5
```

The archive was downloaded from the author's HTTPS site; its hash matches the published signed-checksum text.[^archive] Comparing extracted `src/` and `docs/` with the tag using `git diff --no-index` found only the archive's additional generated `Makefile.in` files: the C++ sources, headers, and man-page template are identical. The clone was clean after inspection. The checksum's **PGP signature was not cryptographically verified**; hash agreement and Git tag resolution were verified, not signing-key identity. All source citations below use the immutable commit, not master.

Evidence labels:

- **Established / source:** directly traced in this commit; this does not imply every exceptional path was executed.
- **Derived:** arithmetic or consequences of those source paths, with assumptions stated.
- **Observed:** reproduced with the unmodified release archive on the research host.
- **Uncertain:** behavior not established by this investigation, especially terminal-dependent failure paths or kernel guarantees.

**Observed check:** the archive built successfully on x86_64 Linux with GCC 15.3 and ncurses 6.6 using `./configure --sysconfdir=/tmp/nix-shell.TfI7CV/opencode/nload-release-defaults` and `make -j2` inside a Nix shell providing ncurses. No installation was performed. A no-argument process was run in an 80×24 pseudo-terminal, `TERM=vt100`, with `HOME` set to the research worktree; neither startup config file existed. Its terminal stream was decoded with pyte 0.8.2. The screen positions, sorted initial selection, boundary navigation and `q` exit matched the source. This is a representative modern-host check, not a claim about every terminal or historical kernel. An initial `TERM=xterm` capture omitted repeated separator characters in the decoder; the VT100 capture reproduced the full separator, avoiding that decoder limitation.

## 2. Effective defaults: configuration-free baseline

With readable Linux interface information, at least one detected interface, usable terminal geometry, `HOME` present, and no overriding configuration, these settings apply.[^defaults]

| Setting / observable | Exact release default |
| --- | --- |
| `Devices` | `all`: all detected interfaces are navigation candidates; only one is displayed |
| `MultipleDevices` | false: single-interface view with two graphs |
| `RefreshInterval` | 500 milliseconds |
| `AverageWindow` | 300 seconds, implemented as a frame-count window |
| `TrafficFormat` | `humanReadableBit` (`-u h` equivalent): automatic Bit/s, kBit/s, MBit/s, GBit/s |
| `DataFormat` | `humanReadableByte` (`-U H` equivalent): automatic Byte, kByte, MByte, GByte |
| `BarMaxIn`, `BarMaxOut` | Each 10240 kBit/s, converted to **1,310,720 bytes/s** for graph calculations |
| Initial selected index | Zero: first entry of the effective interface list |
| Fields, separately per direction | `Curr`, `Avg`, `Min`, `Max`, `Ttl` |

**Refresh scheduling — established / source:** after baseline reads and terminal initialization, the loop first sleeps the configured interval before sampling and rendering the first traffic screen; the initially shown window is cleared. At defaults the wait is five nominal 100 ms sleeps, with keyboard polling after each. It then reads all handlers and clears/prints/refreshes the screen before starting the next wait. Thus the cadence is **500 ms of requested sleep plus input/read/render work and scheduling overhead**, not a deadline-aligned exact 2 Hz clock; actual timestamps, not the setting alone, supply rate denominators. No separate averaging timer exists.[^startup][^loop][^list][^init][^window]

**Documentation discrepancy:** the release man page incorrectly says the unit defaults are `k` and `M`; source constants and built-in help say `h` and `H`. The 2008 changelog explicitly records the change to human-readable traffic units. The man-page phrase “the default device” is also imprecise: its own options section says `all`, and the source creates handlers for all detected interfaces and initially displays the first. The tagged README still identifies itself as 0.7.3; it is not evidence against the verified 0.7.4 version declaration.[^man-defaults][^help][^changelog][^readme]

**Implication:** “no arguments” alone does not specify these defaults. Existing upstream config files can alter every setting above, including selecting a different first interface or disabling the scoped view's graphs. Configuration influence must be distinguished from the configuration-free baseline.[^config-start]

## 3. Linux startup, list ordering, and navigation

### Discovery and selection — established / source

Linux discovery prefers sysfs if `stat("/sys/class/net")` reports a directory; otherwise it uses `/proc/net/dev` if that path stats as a regular file. Sysfs discovery reads every non-dot directory entry. Proc discovery skips two header lines and extracts the trimmed text before `:` on each remaining line. Neither path filters loopback, inactive interfaces, addressless interfaces, or interfaces without traffic. Availability is tested by path type, not by successfully reading every counter; an unreadable preferred backend does not automatically trigger per-read fallback.[^linux-select][^sys][^proc][^factory]

Names and readers are stored in `std::map<string, ...>`, and handlers are constructed from that map. With `Devices=all`, the displayed/navigation list is built by iterating the handler map, so ordering is **lexicographic string ordering by interface name**, not sysfs `readdir` order, proc row order, interface index, routing preference, or “first active interface.” Initial index is zero. Example: `eth10` sorts before `eth2`; loopback is included but is not necessarily first.[^factory][^startup][^list][^navigation]

Discovery is called **once at startup**. All detected handlers get an initial sample before curses is initialized. Every later refresh updates **all handlers**, including ones not displayed and ones excluded by a configured `Devices` list. Newly added or renamed interfaces are not rediscovered in this main loop; an existing handler can instead become invalid when its original name disappears.[^startup][^list]

If configuration gives specific names, existing names are appended in the configured order; missing names are silently skipped. If no requested name resolves, it falls back to all detected handlers. The header's `(i/n)` numbering is assigned from this effective list each refresh. No selected-interface name is persisted; the view stores an index.[^list][^navigation]

**Exceptional path / source-derived bug:** there is no empty-list guard before `devices[m_curDev]->print(...)`. When discovery yields no handlers, index zero is still used: this is out-of-bounds access / undefined behavior, not a defined empty-interface screen. This path was not executed in the observation.[^navigation]

### Arrow behavior — established / source; boundaries also observed

| Main-view key | Effect before next repaint |
| --- | --- |
| Right or Down | Increment selected index by one |
| Left or Up | Decrement selected index by one |

At repaint, **any negative or out-of-range index is set to zero**. Consequently, Right/Down from the last entry returns to the first; Left/Up from the first stays at the first. This is not symmetric modulo wrapping. With one entry, all arrows lead back to that entry. PageDown, Enter/newline/carriage-return and Tab are next-interface aliases; PageUp is a previous-interface alias. The aliases are source facts, not additions required of the port.[^navigation][^man-keys]

Keyboard input is drained about every 100 ms while the refresh wait is in progress. Ordinary navigation does not force an early repaint: the visible selection changes at the scheduled refresh. Multiple keys can modify the index before its single normalization at repaint; applying modulo wrapping after every key would differ. For example, from index `N-1`, two Right events before repaint produce `N+1`, then zero, rather than index one. Arrows are redirected to the options window while that window is visible.[^loop]

`q` and `Q` set the quit flag; SIGINT/Ctrl+C and SIGTERM also set it. Normal exit destroys the window and calls `endwin()`. The man page documents `q`/Ctrl+C. These adjacent release facts help characterize the observed default view; no additional mode contract is selected here.[^loop][^init][^man-keys]

## 4. Displayed fields, units, and statistic lifetimes

### Header and fields — established / source

The header is `Device NAME [IPv4] (i/n):`, omitting the bracketed address if none was obtained. This is upstream's label “Device”; this project's domain term remains **Interface**. IPv4 is obtained anew for each sample using `SIOCGIFADDR` on a temporary IPv4 socket. The view does not show IPv6, interface status, packets, errors, drops, timestamps, or a graph-scale label. It prints a terminal-width `=` separator, then `Incoming:` and `Outgoing:`, each with a graph and the same five byte-traffic fields.[^device-screen][^reader]

| Field | Exact source meaning / lifetime |
| --- | --- |
| `Curr` | Byte-counter delta divided by elapsed sample time over the short “second” frame window; not simply the latest 500 ms delta |
| `Avg` | Byte-counter delta over the retained average frame window; warm-up uses all available retained frames |
| `Min` | Minimum of successive calculated `Curr` values, initialized by the second valid frame, retained across pruning/navigation |
| `Max` | Corresponding maximum, with the same lifetime |
| `Ttl` | Latest cumulative interface byte counter after upstream overflow correction; not the sum of bytes transferred since launching nload |

These meanings are identical for incoming and outgoing traffic. Counters begin at zero in default-constructed frames; the first valid frame makes statistics “valid” and sets totals, but leaves all calculated rates/extrema zero. The second valid frame initializes `Curr`, `Avg`, `Min`, and `Max`. Thus startup's synthetic zero does not automatically become a permanent zero minimum.[^stats-insert][^stats-header][^frame][^device-fields]

Statistics are per-interface handler and remain live while navigating away; sampling of other interfaces continues. A genuinely invalid read resets rates, extrema, retained frames, and graph values. On reappearance of that original name, rate calculation warms up again. However, the old corrected frame used by the overflow heuristic is **not cleared** by the invalid-read branch.[^device-update][^list][^stats-insert]

The comments call `Ttl` “since last system reboot.” What the application actually establishes is only **the latest kernel-provided interface counter, possibly adjusted by the application's wrap heuristic**. Interface/driver counter lifetime is a separate Linux-source question; reboot lifetime is not verified or enforced by nload. The adjacent Linux-counter research ticket owns those kernel guarantees.[^device-fields][^stats-header][^sys][^proc]

### Exact calculations — established / source and derived arithmetic

For refresh setting `R` in milliseconds and average setting `A` in seconds:[^stats-window]

```text
averageCount = unsigned(1000.0 / R * A)       # truncation toward zero
secondCount  = max(unsigned(1000.0 / R), 1)
```

After appending a valid frame, if fewer than two frames exist, calculation returns. Otherwise the oldest frames are erased until at most `averageCount` remain. For the short-window calculation, the older endpoint is `secondCount` positions behind the newest frame if that many intervals exist; otherwise it is the oldest retained frame. For `Avg`, it is always the oldest retained frame after pruning (the branch for more than `averageCount` cannot occur after that pruning). These are **sample-count windows**, not timestamp-based expiry.[^stats-insert]

At defaults, `averageCount=600` and `secondCount=2`. Derived under exactly 0.5 s spacing:

- After two valid frames: both rate windows span 0.5 s.
- After three frames: `Curr` spans 1.0 s; `Avg` spans 1.0 s and continues warming up.
- At full capacity: `Curr` spans two intervals, nominally 1.0 s; `Avg` spans **599 intervals, nominally 299.5 s**, not exactly 300 s. Actual elapsed spans include scheduling and read/render delays.

For either direction the calculation is:[^stats-math][^reader]

```text
dt = float((to.seconds + to.microseconds / 1000000.0)
         - (from.seconds + from.microseconds / 1000000.0))
rate_bytes_per_second = unsigned_long_long((to.bytes - from.bytes) / dt)
```

The subtraction uses unsigned 64-bit counters; the division uses a **float** elapsed duration and converts the result back to an integer, truncating the fractional rate. Sample timestamps come from **`gettimeofday` wall clock**, captured before reading counters, not a monotonic clock. If `dt <= 0`, the function returns without replacing the previous calculated result. This can leave stale rates/extrema/graph samples after a backward or equal wall-clock timestamp. Min/max compare calculated integer rates, not rendered rounded strings.[^stats-math][^stats-insert][^reader]

### Formatting — established / source

Rate and total display convert the integer byte quantity to `float`, divide by a unit factor, and use fixed notation with **two fractional decimal places**, followed by a space and the unit. Rates add `/s`; totals do not. Each value independently chooses its human-readable prefix, so `Curr`, `Avg`, `Min`, `Max` and `Ttl` need not use the same prefix.[^device-format]

All prefixes use **1024**, even though the labels are `kBit`, `MBit`, `GByte`, etc. For a byte quantity `v`:

- Automatic bits: choose the smallest prefix for which `8*v / 1024^p < 1024`, with `p` capped at 3; display `8*v / 1024^p` as Bit/kBit/MBit/GBit.
- Automatic bytes: same rule using `v`, with Byte/kByte/MByte/GByte.
- Prefix thresholds are evaluated before two-decimal display rounding. Zero rates print `0.00 Bit/s`, zero totals `0.00 Byte`; 128 bytes/s prints `1.00 kBit/s`; 1024 bytes total prints `1.00 kByte`.

The code implements the numeric factors and unit-name selection separately. Large values lose precision through `float`; bit-unit name selection also multiplies an unsigned 64-bit value by eight, which can overflow at extreme values. These are source limitations, not alternate unit policies.[^units]

## 5. Graphs: scale, glyphs, history, and screen geometry

### Fixed scale and glyph thresholds — established / source

Each graph is fed the calculated integer **`Curr` bytes/s**, not `Avg` or the raw latest-interval delta. Incoming and outgoing scales are independent settings; neither automatically scales to the observed maximum. The default conversion is `10240 * 1024 / 8 = 1,310,720 bytes/s`, equivalent to 10 binary-scaled MBit/s. Rate text can exceed that scale while the graph saturates.[^device-update][^device-screen][^graph]

For maximum `M`, positive bar height `H`, sample `v`, and graph row `l` counted from zero at the top, the exact integer glyph algorithm is:[^graph]

```text
trafficPerLine = M / H
lowerLimit    = M * (H - l - 1) / H
if v < lowerLimit:                ' '
else if v - lowerLimit >= trafficPerLine:          '#'
else if v - lowerLimit >= trafficPerLine * 7 / 10: '|'
else if v - lowerLimit >= trafficPerLine * 3 / 10: '.'
else:                            ' '
```

All divisions truncate. This produces a vertically filled bar with partial top cells, not a connected line plot. Values at/above the configured maximum fill all rows with `#`; no overflow marker is printed. Sub-threshold traffic can have nonzero text and a blank graph. Derived example at the default scale and height 10: `trafficPerLine=131072`, so the bottom row first shows `.` at **39321 bytes/s** and `|` at **91750 bytes/s**. There are no tick labels, numeric axes, or timestamps.[^graph][^device-screen]

### Horizontal history — established / source, including navigation quirk

The graph list stores newest values at the front; printing iterates it in reverse. Thus **oldest is left, newest is right**, one column per valid update. It pushes a new value and drops the oldest if size exceeds one, preserving the previously configured capacity. Only `Device::print` sets the capacity to `floor(2*terminal_width/3)`. A newly constructed graph has an empty list; the first update creates one entry, and further updates retain just one until the interface is first printed.[^graph][^device-update][^device-screen]

Consequences:

- An interface never yet displayed has full running text statistics but **only the newest one-column graph history**. First display expands its list with zero-filled older entries; the missing earlier samples are not recovered from the statistics buffer.
- After first display, its graph continues updating at that last configured width while it is off-screen. Switching away does not reset it.
- Growing the graph adds zero entries at the old/left end; shrinking removes the oldest entries. The width changes only when that interface is printed. A resize is not a time-preserving resampling operation.
- Invalid reads zero the current graph list; they do not discard its capacity. Later valid samples refill it.

At width 80, capacity is **53 columns**. Under exact 500 ms cadence, 53 actual samples span 52 intervals / **26 s between oldest and newest** (53 sample slots represent 26.5 s of nominal update slots). Zero-filled slots on startup/first display/expansion are placeholders, not measured idle traffic. The source records no timestamps with graph samples, so delayed updates still occupy one column.[^graph][^device-screen]

### Geometry and representative screen — source plus observed

The main window occupies the full terminal. At ordinary widths where the header fits on one row, the header is row 0, separator row 1, incoming label row 2, and incoming graph begins at row 3. Both graphs have `floor(2*width/3)` columns starting at x=0; the right-hand statistics begin at x=`floor(2*width/3)+2`.[^init][^device-screen][^window]

The actual code calculates incoming height as `(window.height - window.current_y - 1) / 2`, then prints incoming statistics at the graph's resulting cursor y minus 5. It prints the outgoing label next, sets outgoing height to `window.height - window.current_y`, and positions outgoing statistics at the resulting cursor y minus 4. This is cursor-driven curses layout, not a separate abstract layout grid; bottom-row newline behavior matters.[^device-screen][^window]

**Observed 80×24 layout:** 53 graph columns, x=55 for statistics, incoming graph rows 3–12, incoming fields rows 8–12, outgoing label row 13, outgoing graph rows 14–23, outgoing fields rows 19–23. There was no footer or navigation help line. In the check, `docker0`, `enp1s0`, `lo` were the candidates; `docker0` appeared first even with zero traffic. Left/Up at first stayed on it; Right past `lo` returned to it; Down advanced one; Left returned one; `q` exited with status zero. Host-specific names/addresses are observation examples, not release defaults.

**Small-terminal / resize uncertainty:** SIGWINCH reinitializes curses/windows without recreating handlers/statistics. However, upstream has no explicit minimum-size or overflow-safe layout guard; long headers/text can wrap, positions can become invalid, and negative height arithmetic can be converted to unsigned bar heights. Window helpers do not check curses write/move return values. Source formulas are established; exact damaged-screen behavior at every tiny size or terminal type was not tested and should not be treated as an intentional compatibility contract.[^init][^device-screen][^window][^graph]

## 6. Configuration influence on a no-argument invocation

**Established / source:** defaults and filters are registered first, then `HOME` is required. If absent, nload prints `Could not retrieve home directory.` and exits 1 before discovery or curses initialization. An empty-but-present `HOME` is not rejected. Startup reads **`SYSCONFDIR/nload.conf`**, followed by **`$HOME/.nload`**; the user file overrides corresponding system settings. Command-line parsing follows, so without arguments there is no command-line replacement of configured values.[^defaults][^config-start]

`SYSCONFDIR` is a compile-time value from configure's `sysconfdir`; `/etc/nload.conf` is the man-page path, not a hardcoded universal startup path. **F6 differs:** it hardcodes `/etc/nload.conf`, then reads `$HOME/.nload`. Reload does not restore defaults first; values absent from the reloaded files remain as they were. F5 writes current settings to `$HOME/.nload`. Those actions can affect later no-argument launches; the full options-window feature is outside this research scope.[^configure][^loop][^man-files][^store]

Configuration is line-oriented `Key="value"`, with empty lines and whole-line `#` comments skipped. Lines are trimmed, but there is no general whitespace normalization around `=`. The writer emits `Version="1"` and then settings with **human-facing filtered values**. Version is stored as an ordinary unknown setting; no schema-version enforcement appears. Recognized settings are passed through their filters. A malformed line stops parsing that file; earlier applied values remain. Unknown keys are stored. Missing/unopenable files and read return values are ignored by `main`, so there is no startup diagnostic for those failures.[^setting][^store][^strings]

The relevant configuration-free serialized values are:[^defaults][^setting][^filters]

```ini
AverageWindow="300"
BarMaxIn="10240"
BarMaxOut="10240"
DataFormat="Human Readable (Byte)"
Devices="all"
MultipleDevices="[ ]"
RefreshInterval="500"
TrafficFormat="Human Readable (Bit)"
```

- Unit settings accept the mapped names `Human Readable (Bit)`, `Human Readable (Byte)`, `Bit`, `Byte`, `kBit`, `kByte`, `MBit`, `MByte`, `GBit`, `GByte`. They do **not** accept CLI letters or raw enum integers through the config map filter; unrecognized mapped values leave the previous value unchanged.
- `MultipleDevices` accepts `[ ]` and `[x]` through its map filter. `[x]` changes the no-argument launch out of the scoped single-interface graph view; no analysis of that view is supplied here.
- Config writes clamp `AverageWindow` to at least 5, `BarMaxIn`/`BarMaxOut` to at least 10, and `RefreshInterval` to at least 50. There are no corresponding maximum filters. These clamps apply through filtered config/UI writes; ordinary direct CLI assignment bypasses that filter path.
- `Devices` is space-separated; an empty value becomes `all`, and its exclusive-token filter makes a recognized mixed-in `all` token exclusive. Valid configured names can change the candidate order and initial selection as described above.
- Reloaded settings are read on subsequent update/print calculations. Refresh/average changes do not rebuild handlers or clear extrema automatically; shortened frame/graph retention loses prior entries. Extreme configuration combinations can make calculated frame capacities invalid (e.g. zero); such malformed/extreme cases are not established usable defaults.

These behaviors follow the default/minimum/map/exclusive filters and the settings' direct-versus-filtered assignment paths; they are not inferred from the man page alone.[^defaults][^filters][^setting][^setting-header][^stats-window][^list]

## 7. Error and counter-correction behavior relevant to the view

When a frame is invalid, the handler resets statistics and graph samples. The selected interface then prints `Device NAME (i/n): does not exist` and a separator instead of graphs and values. This message also covers read failure, not only actual removal. Proc open/parse/missing-name failure leaves a frame invalid. Sysfs is different: a missing interface directory leaves the frame invalid, but a failed/malformed individual counter file returns zero and the frame is still marked valid if the interface directory exists. It may therefore produce apparent zero counters and false traffic rather than the warning.[^device-update][^device-screen][^sys][^proc]

The release applies `fixOverflow` separately to byte, packet, error and drop counters. A new value above `0xffffffff` is returned unchanged; otherwise, if it is smaller than the previous corrected value's low 32 bits, a `2^32` carry is added to the previous high bits, and those bits are combined with the new value. The 2012 changelog records the fix for newer Linux 64-bit counters causing a false 4 GB jump on every refresh.[^overflow][^changelog]

**Derived limitations:** this heuristic cannot distinguish a real 32-bit wrap from a counter reset/replacement below that threshold. It can report a large false delta; a decreasing wider counter can also underflow the unsigned subtraction used for rates. Retaining the old corrected frame across invalid reads affects subsequent correction. The source does not detect stable interface identity, explicitly handle replacement, or promise reset-safe rates. These are evidence for the existing counter/statistics decisions, not recommendations to preserve those quirks.[^overflow][^device-update][^stats-math]

## 8. Implications and unresolved questions

Facts that later port decisions need to account for:

1. A clean baseline must state its configuration conditions; no-argument upstream behavior is environment/configuration-dependent.
2. Selecting the first active/non-loopback interface, symmetric wrapping, or dynamically discovering interfaces would differ from this release's startup/navigation path.
3. `Curr`, the long average, and extrema have different time horizons; `Ttl` has a kernel-counter origin rather than an application-session origin. Monotonic-clock timing, exact time-expiring averages, or floating-point rates would alter identified calculations.
4. Graph history is independent of the statistics buffer and depends on prior display/resize state. Preserving all interfaces' full history from startup would differ from upstream's never-viewed graph behavior.
5. Unit defaults in the man page and reboot-lifetime comments cannot serve as an exact oracle. Source and the focused observed check supply the baseline; clearly labeled release bugs remain decision inputs.

**Remaining uncertainty:** kernel counter lifetime/identity/read consistency belongs to [Establish Linux interface and traffic-counter semantics](../issues/02-linux-counter-semantics.md); interface/configuration/navigation choices to [Decide the minimal startup and navigation contract](../issues/03-startup-navigation-contract.md); reset, timing, extrema and graph-contract choices to [Decide traffic statistics and sampling behavior](../issues/04-statistics-contract.md); tiny-terminal and resize outcomes to [Validate the single-interface screen and terminal behavior](../issues/05-single-interface-screen.md); dependency and terminal integration choices to [Decide Rust and delivery constraints](../issues/06-engineering-constraints.md) and [Decide the port's architecture and test seams](../issues/07-module-boundaries.md); acceptance examples to [Decide acceptance criteria and the specification handoff](../issues/08-acceptance-handoff.md). Historical upstream intent behind asymmetric boundaries and graph lazy capacity is not documented in the inspected primary material, but their implemented behavior is established. No claim is made that the current-host observation exhausts historical ncurses behavior.

**New-ticket assessment:** no additional precisely scoped research ticket is warranted by these findings. The outstanding scoped questions are already assigned to the existing Linux-counter, startup, statistics, screen, engineering, architecture, and acceptance tickets. This research does not resolve or modify any of them.

## Primary-source references

[^release]: [Author's nload homepage](https://www.roland-riegel.de/nload/) (version, archive and repository links); [pinned configure.in, lines 4–18](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/configure.in#L4-L18); [release commit](https://github.com/rolandriegel/nload/commit/a5715a77eb2c11702a7ffc7664240b9250cafa4d). Git object resolution was checked locally, as recorded above.
[^archive]: [Author's release archive](https://www.roland-riegel.de/nload/nload-0.7.4.tar.gz); [published signed SHA-256 text](https://www.roland-riegel.de/nload/nload-0.7.4.tar.gz.sha256), both retrieved 2026-10-01.
[^defaults]: [main.cpp, lines 60–111](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L60-L111): constants, initial settings and filters.
[^man-defaults]: [docs/nload.1.in, lines 50–104](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/docs/nload.1.in#L50-L104): documented defaults and “default device.”
[^help]: [main.cpp, lines 553–574](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L553-L574): built-in help defaults.
[^changelog]: [ChangeLog, lines 2–60](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/ChangeLog#L2-L60): 64-bit counter fix and historical unit/config changes.
[^readme]: [README, lines 1–12](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/README#L1-L12): stale self-identification.
[^config-start]: [main.cpp, lines 114–128](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L114-L128): HOME check and startup read order.
[^linux-select]: [devreader-linux.cpp, lines 19–32](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/devreader-linux.cpp#L19-L32): backend preference.
[^sys]: [devreader-linux-sys.cpp, lines 34–108](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/devreader-linux-sys.cpp#L34-L108): sysfs availability, enumeration, fields and errors.
[^proc]: [devreader-linux-proc.cpp, lines 32–155](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/devreader-linux-proc.cpp#L32-L155): proc enumeration and parsing.
[^factory]: [devreaderfactory.cpp, lines 36–127](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/devreaderfactory.cpp#L36-L127): reader map and Linux reader creation.
[^startup]: [main.cpp, lines 355–369](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L355-L369): single discovery call, handler construction and baseline reads.
[^list]: [main.cpp, lines 433–473](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L433-L473): effective list, numbering and all-handler updates.
[^navigation]: [traffic_window.cpp, lines 25–84](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/traffic_window.cpp#L25-L84): index initialization, key handling and normalization.
[^man-keys]: [docs/nload.1.in, lines 99–135](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/docs/nload.1.in#L99-L135): main-window key documentation.
[^loop]: [main.cpp, lines 371–431](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L371-L431): sleep/poll loop, routing, save/reload and quit.
[^init]: [main.cpp, lines 489–536](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/main.cpp#L489-L536): curses initialization, signal handling, cleanup and resize.
[^device-screen]: [device.cpp, lines 67–124](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/device.cpp#L67-L124): header, warning, graph layout and scales.
[^reader]: [devreader.cpp, lines 53–105](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/devreader.cpp#L53-L105): wall-clock timestamps and IPv4 lookup.
[^stats-insert]: [statistics.cpp, lines 24–74](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/statistics.cpp#L24-L74): insertion, pruning, endpoints, extrema initialization and reset.
[^stats-header]: [statistics.h, lines 29–97](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/statistics.h#L29-L97): unit enum, validity and field getters.
[^frame]: [dataframe.h, lines 26–45](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/dataframe.h#L26-L45): zero initialization and byte-counter types.
[^device-fields]: [device.cpp, lines 191–225](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/device.cpp#L191-L225): field names/getters and lifetime comments.
[^device-update]: [device.cpp, lines 38–65](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/device.cpp#L38-L65): correction, statistics and graph updates, invalid-read resets.
[^stats-window]: [statistics.cpp, lines 201–215](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/statistics.cpp#L201-L215): frame-count window formulas.
[^stats-math]: [statistics.cpp, lines 144–199](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/statistics.cpp#L144-L199): elapsed-time division and min/max comparison.
[^device-format]: [device.cpp, lines 171–189](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/device.cpp#L171-L189): float conversion, two-decimal rendering and suffixes.
[^units]: [statistics.cpp, lines 76–142](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/statistics.cpp#L76-L142): numeric unit factors, prefix thresholds and names.
[^graph]: [graph.cpp, lines 25–109](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/graph.cpp#L25-L109): capacity, history ordering, thresholds and reset.
[^window]: [window.cpp, lines 66–122 and 172–259](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/window.cpp#L66-L259): curses window lifecycle, cursor tracking and unchecked writes.
[^configure]: [configure.in, lines 98–99](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/configure.in#L98-L99): compile-time sysconfdir forwarding.
[^man-files]: [docs/nload.1.in, lines 164–178](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/docs/nload.1.in#L164-L178): documented config and Linux file locations.
[^store]: [settingstore.cpp, lines 52–97](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/settingstore.cpp#L52-L97): file application/writing, unknown keys and Version output.
[^setting]: [setting.cpp, lines 74–142](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/setting.cpp#L74-L142): filter order and configuration parsing/serialization.
[^strings]: [stringutils.cpp, lines 22–96](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/stringutils.cpp#L22-L96): trimming and quoted/space splitting.
[^filters]: [settingfilter.cpp, lines 47–181](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/settingfilter.cpp#L47-L181): default, exclusive-token, mapping and minimum filters.
[^setting-header]: [setting.h, lines 54–58](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/setting.h#L54-L58): raw typed conversion/assignment.
[^overflow]: [device.cpp, lines 139–169](https://github.com/rolandriegel/nload/blob/a5715a77eb2c11702a7ffc7664240b9250cafa4d/src/device.cpp#L139-L169): counter correction heuristic.
