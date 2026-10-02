# Linux interface enumeration and byte-counter semantics

Research for [Establish Linux interface and traffic-counter semantics](../issues/02-linux-counter-semantics.md), verified 2026-10-01. Scope: an unprivileged Linux, no-argument, single-interface monitor. This is evidence for later decisions, not a backend selection or a startup/statistics policy.

Evidence labels:

- **Documented**: kernel ABI documentation, UAPI definitions, or Linux man-pages state the behavior.
- **Implementation (v6.12)**: verified in the upstream Linux v6.12 source; an observation about that version, not a promise for every kernel/driver.
- **Inference**: a consequence of the cited facts, with assumptions stated.

Kernel v6.12 is an examination reference, **not a proposed minimum supported kernel**. Linux man-pages were read through the project's man7.org rendering (6.19). Source files were read from the upstream repository's GitHub mirror and kernel.googlesource.com mirror; citations below identify the upstream tag and symbols.

## Findings in brief

All three mechanisms can enumerate interfaces and expose the standard incoming/outgoing byte counters without routine administrative privileges. They differ mainly in identity metadata, namespace binding, collection granularity, and failure handling. `/proc/net/dev` and sysfs obtain the same standard statistics as rtnetlink; choosing a different transport does not fix a driver's accounting semantics. A 64-bit exported value does not prove a 64-bit underlying counter, an atomic RX/TX snapshot, or a counter epoch extending back to boot. [S1][S2][S3][S4][S5]

| Mechanism | Enumeration and identity | RX/TX representation | Collection granularity |
| --- | --- | --- | --- |
| `/proc/net/dev` | Two header lines, then interface names and statistics; no ifindex or flags in this file | Decimal unsigned 64-bit values in the examined implementation; RX is numeric field 1 and TX field 9 after the name/colon | One statistics retrieval per emitted interface row; one file traversal covers all interfaces |
| `/sys/class/net` | Class entries named for interfaces, normally symlinks; separate `ifindex`, `flags`, and `type` attributes | `statistics/rx_bytes` and `statistics/tx_bytes`: separate decimal unsigned 64-bit values | Each attribute read independently obtains a full standard statistics structure and exports one field |
| rtnetlink (`NETLINK_ROUTE`) | `RTM_GETLINK` dump returns `RTM_NEWLINK` records containing `ifi_index`, `ifi_flags`, and `IFLA_IFNAME` | `IFLA_STATS64` contains `rtnl_link_stats64`; legacy `IFLA_STATS` contains 32-bit `rtnl_link_stats`. `RTM_GETSTATS` can return `IFLA_STATS_LINK_64` | A link record bundles identity and one statistics retrieval; targeted requests or multipart dumps are available |

Table evidence: **Documented** API shapes and sysfs cost [S1][S3][S6][S7]; **implementation (v6.12)** proc field positions/format, common `dev_get_stats()` path, and rtnetlink filling [S2][S4][S5][S8]. `RTM_GETSTATS` uses `if_stats_msg` and a filter mask (`IFLA_STATS_FILTER_BIT(IFLA_STATS_LINK_64)`), and does not supply an interface name in the basic statistics record. [S1][S6][S8]

## 1. What the byte counters mean

**Documented:** `rx_bytes` counts good received bytes corresponding to `rx_packets`; `tx_bytes` counts good transmitted bytes corresponding to `tx_packets`. For IEEE 802.3 interfaces the documented expectation is Ethernet frame length **excluding FCS**. RX packets include good packets delivered from hardware to the host that the host subsequently drops. For hardware TX, successful handoff acknowledged by the interface can count even though successful transmission onto the medium is not established. The documentation says drivers must ensure the best possible compliance; this is not a guarantee of identical accounting at every virtual or hardware interface. [S1]

The sysfs ABI explicitly directs readers to the driver for exactly when RX increments and whether TX means successful transmission or packets queued for transmission. These qualifications concern accounting, not a separate kind of sysfs counter. [S3][S4][S5]

**Inference:** these are interface-accounted traffic bytes, not application payload, unique delivered data, or full physical-medium occupancy. The Ethernet expectation alone excludes FCS; none of these API values establishes a general count including preamble/inter-frame gaps or validates remote delivery. Nor do these interfaces make a host-wide sum across stacked interfaces a deduplicated total. A monitor displaying one interface has a well-defined observation point without assuming all interface types account at the same layer. [S1][S3]

**Implementation (v6.12):** proc prints `rx_bytes` and `tx_bytes` directly. Its documented folding of some error/drop fields does **not** fold the byte columns. For example, proc RX `drop` combines `rx_dropped + rx_missed_errors`; that issue does not change the two byte fields. [S1][S2]

### Width versus actual accounting

- **Documented:** the two fields in `rtnl_link_stats64` are `__u64`, independently of userspace pointer width. `rtnl_link_stats` fields are `__u32`; widening `IFLA_STATS` in userspace cannot recover high bits. [S1][S6]
- **Implementation (v6.12):** proc formats the 64-bit statistics with `%llu`; sysfs reads a `u64` member and formats `%llu\n`. Rtnetlink fills `IFLA_STATS64` once via `dev_get_stats()` and constructs legacy `IFLA_STATS` by narrowing the same result. [S2][S4][S8]
- **Implementation (v6.12):** `dev_get_stats()` can fall back to legacy `ndo_get_stats`/`net_device_stats`. `netdev_stats_to_stats64()` widens unsigned-long-sized accounting to `u64`; on a 32-bit kernel this cannot reconstruct previous wraps. Drivers can also provide their own `ndo_get_stats64()` implementation. Thus exported width alone is not an assertion about hardware width or driver rollover extension. [S5]
- **Inference:** modulo arithmetic only reconstructs a delta if the true counter width and epoch are known and fewer than one complete wrap's worth of bytes passed between observations. Two samples cannot distinguish several wraps. For scale, a 32-bit byte counter wraps after `2^32` bytes: about 34.36 s at an accounting rate of 1 Gbit/s, or 3.44 s at 10 Gbit/s. These are arithmetic examples, not a measured interface rate. [S5][S6]

## 2. Read consistency and freshness

**Documented:** drivers whose statistics retrieval would sleep must update asynchronously and return a recent cached copy from `ndo_get_stats64()`. The ABI does not prescribe a universal refresh period. A successful read therefore need not account for traffic up to that exact syscall instant. [S1]

**Implementation (v6.12):** all three paths ultimately call `dev_get_stats()`, which invokes driver accounting or aggregates software statistics. There is no transport-level lock freezing packet accounting for the entire returned structure. The kernel's `u64_stats_sync` comments explicitly distinguish protection against an individual 64-bit value tearing on 32-bit architectures from consistency among multiple counters; its synchronization is a no-op on 64-bit architectures. [S2][S4][S5][S9]

Consequences, with evidence boundaries:

- **Proc:** RX/TX in one row come from one `dev_get_stats()` retrieval (**implementation**); that is a narrower collection window than two sysfs attribute retrievals, but **not a documented atomic RX/TX transaction**. Each interface row is collected separately. The RCU iterator protects object access, not a frozen topology or all-interface traffic snapshot. [S2][S5][S9]
- **Sysfs:** RX and TX files trigger two full retrievals (**documented**); neither those reads nor separate `ifindex`/name/flags reads form a transaction. A rename/removal/recreation can occur between pathname lookups (**inference**). [S1][S4][S10]
- **Rtnetlink:** a link record bundles ifindex, name, flags, and one standard-statistics retrieval (**implementation**); it still lacks an atomic traffic-counter snapshot guarantee. A dump may span multiple datagrams and kernel iterations. `NLM_F_DUMP_INTR` signals that an enumeration may be inconsistent and the kernel documentation tells userspace to retry. Its absence does not promise simultaneous traffic accounting. [S8][S9][S11]
- **Implementation (v6.12):** link and statistics dumps use `dev_base_seq`/`nl_dump_check_consistent()`, and interface insertion/removal changes that sequence. That is topology consistency machinery, not a counter-update generation or timestamp. [S5][S8]

**Inference:** timestamping before and after acquisition can bracket the userspace collection window; it cannot bound the age of a driver's cached hardware counts. None of the basic byte-counter records supplies the accounting timestamp. A short poll interval can observe cached plateaus followed by catch-up jumps. [S1][S2][S4][S6]

## 3. Identity, enumeration order, and loopback

### Identity and lifecycle

**Documented:** rtnetlink has a unique interface index and an interface name; userspace can explicitly request an ifindex for creation (since Linux 3.7). Sysfs documents `ifindex` as the identifier mapping to an interface name. Its older phrase “system-wide” must be read alongside namespace isolation: the examined index tables and allocation are per-network-namespace. `iflink` describes linkage to another interface, not this interface's own identity. [S6][S7][S8][S5][S12]

**Implementation (v6.12):**

- A rename changes the name and sysfs object name without changing this interface's ifindex or replacing its accounting object (`dev_change_name`). The sysfs uevent comment specifically says ifindex does not change when the name changes. Rtnetlink emits `RTM_NEWLINK` on `NETDEV_CHANGENAME`. [S4][S5][S8]
- Removal unlinks the interface from its namespace's index/name collections and removes its sysfs entries. Subsequent targeted `RTM_GETLINK` or `RTM_GETSTATS` can return `ENODEV`. Proc can cease listing the old name rather than returning a per-interface error. [S2][S4][S5][S8]
- Index allocation is cyclic for automatic allocation and allows explicit reservation of a free index. Removal erases the reservation. Neither an ifindex nor a name is a globally unique, never-reused lifetime identifier. [S5][S7]
- Moving an interface to another network namespace removes it from the old view and adds it to the new one; name/index conflicts can cause a changed name or index. Source emits `RTM_DELLINK` for the old namespace and `RTM_NEWLINK` for the new one. The same kernel object can move, so disappearance is not proof of hardware destruction or zeroed counters. [S5]

**Inference:** `(network namespace, ifindex)` distinguishes currently existing interfaces and follows ordinary renames, but does not identify an incarnation forever. Name-only proc records cannot themselves distinguish rename from deletion plus creation. Even identical namespace/index/name at two polls does not prove continuity if a complete remove/recreate cycle occurred between them. MAC address is not a general substitute: sysfs documents assignable addresses and stacked interfaces sharing an address. [S2][S5][S7]

Rtnetlink link notifications expose lifecycle changes; they are **not a lossless lifecycle journal**. Kernel-to-user messages can be dropped when buffers fill, with `ENOBUFS` requiring resynchronization. Periodic enumeration cannot retrospectively prove that no event was missed. [S8][S13]

### Enumeration and ordering

**Documented:** proc includes interfaces without IP addresses or `IFF_RUNNING`; enumeration is not restricted to an IPv4-address list. Sysfs class entries include symlinks and should not be interpreted as only ordinary directories. Rtnetlink link dumps enumerate links and expose interface flags, including loopback; they do not inherently select the default-route or busiest interface. [S10][S14][S6]

**Implementation (v6.12):** proc and unfiltered rtnetlink link/statistics dumps walk `for_each_netdev_dump()`, using the per-namespace index XArray in increasing index order. Neither iterator filters out loopback, down, virtual, or addressless interfaces. This examined order is not a documented cross-version ordering contract or startup preference. Sysfs directory entry order is filesystem-dependent, not an alphabetic or ifindex-order guarantee. [S2][S8][S15][S16]

**Implementation (v6.12):** loopback is created as `lo` for each successfully initialized network namespace and checked against `LOOPBACK_IFINDEX`. Its statistics callback assigns the same accounted bytes to both RX and TX from one aggregate. Separately timed sysfs RX/TX reads can nevertheless differ when traffic is moving. Loopback traffic is local interface traffic, not external-link utilization. [S17][S4]

**Boundary:** whether loopback is initially selected, excluded, or navigable, and whether navigation sorts by name/index or retains a startup list, are application decisions. No examined Linux ABI determines those policies.

## 4. Namespaces and unprivileged visibility

**Documented:** since Linux 2.6.25 `/proc/net` resolves through `/proc/self/net`, exposing the calling process's network namespace. Network namespaces isolate interfaces, `/proc/net`, and `/sys/class/net`; a physical interface lives in exactly one network namespace at a time. A container's visible interfaces need not be the host's physical interfaces. [S12][S18]

**Implementation (v6.12):** namespace binding differs by mechanism:

- Proc networking open stores a reference to the inode's network namespace (`seq_open_net`); an already-open proc FD does not switch namespaces merely because the task later changes namespace. [S19]
- Sysfs captures a network-namespace tag when its mount context is initialized; the network class supplies per-interface namespace tags. An inherited/bind-mounted sysfs view may therefore differ from a task's current namespace after a namespace change. Reading `/sys/class/net` does not automatically remount it for the task. This qualification is supported by mount/class implementation, beyond the simplified namespace overview. [S4][S20]
- Rtnetlink operations normally use `sock_net()` for the socket's namespace. Target-namespace queries are a distinct mechanism: `rtnl_get_net_ns_capable()` requires `CAP_NET_ADMIN` in the user namespace owning the target namespace. A request in one's own namespace is not a host-wide query. [S8][S13]

**Implementation/documented access:** proc's `dev` entry is created with mode `0444`. Sysfs statistics use read-only attributes (`DEVICE_ATTR_RO`, documented as `0444`). Normal rtnetlink GET link/statistics operations do not pass through the mutation `CAP_NET_ADMIN` requirement; route link multicast reception is permitted to non-root listeners. Raw in `SOCK_RAW` here describes netlink framing and does not imply an IP raw-socket privilege requirement. [S2][S4][S10][S8][S13]

**Limit:** unprivileged ABI access presupposes accessible mounts/paths and an allowed socket API in the execution environment. File opens can fail for missing paths, denied search/read access, or resource exhaustion. A read-only sysfs mount alone does not prevent reading statistics. These APIs do not bypass namespace isolation or filesystem access control. [S10][S12][S13][S21]

## 5. Failure, syscall, and performance trade-offs

These are **operation-count inferences**, not benchmarks or a backend ranking. Kernel work, text/binary decoding, interface count, driver collection cost, buffers, and polling interval all affect actual cost. [S1][S2][S4][S8][S11]

| Path | Ordinary work per sample | Important consistency/failure costs |
| --- | --- | --- |
| Proc, reopened | An open, reads through EOF, and close; parser scans rows. Kernel retrieves statistics for emitted rows, even if userspace needs just one. | Output may need several reads; no fixed-size or one-read guarantee. Enumeration can change while traversing; the file has no ifindex/lifecycle sequence or per-row error channel. |
| Sysfs, reopened | For one interface, two open/read/close sequences for RX/TX; enumeration and ifindex/flags reads add operations. Each byte read retrieves all standard statistics. | Two acquisition times and multiple pathname lookups; rename/removal races. Sampling all interfaces scales attribute operations with interface count. |
| Rtnetlink, retained socket | One request send and one or more receives. A targeted link/statistics request avoids collecting every interface; a dump batches many records. | Length/alignment-aware binary decoding, request sequence matching, control/error messages, multipart completion, truncation/loss detection, and dump retries. Link requests carry more metadata than just two counters. |

**Documented:** seq_file, used by proc, supports partial reads and seeks; iterator sessions need not encompass the whole output, and buffer overflow can cause regeneration. Reusing an FD requires returning to the start for another traversal rather than repeatedly reading an EOF position. Sysfs explicitly supports rearming `show()` by seeking to zero or `pread(..., offset=0)`; two retained FDs can thus avoid repeated opens, but still trigger two statistics retrievals. Retained file references do not retarget themselves to newly created objects at the same pathname. [S2][S10][S22][S21]

**Lifecycle errors:** sysfs paths can disappear or change during lookup (`ENOENT`); attribute callbacks can fail after the represented interface is no longer alive (`netstat_show()` returns `EINVAL` if it reaches that check in v6.12). Generic sysfs documentation warns that a pinned object does not imply the physical entity remains present. This does not promise that all removal races return one particular errno. Rtnetlink explicitly returns `ENODEV` for an absent targeted interface and can also report invalid/unsupported requests and allocation errors. [S4][S8][S10][S21]

**Documented netlink transport obligations:**

- Dumps end with `NLMSG_DONE`; errors/ACKs use `NLMSG_ERROR` (zero means ACK, not failure). Completion/control messages and their return codes matter alongside link records. [S11][S13]
- A receive buffer that is too small sets `MSG_TRUNC` and discards the rest of that datagram. A single receive can contain multiple messages, and a dump can require multiple receives. [S11]
- `ENOBUFS` indicates lost kernel-to-user messages and the need to resynchronize; ACK requests do not make notifications lossless. `NLM_F_DUMP_INTR` indicates possible inconsistent enumeration and the docs specify retrying. [S11][S13]
- `NLM_F_ATOMIC` is **not** a way to obtain a frozen RX/TX snapshot: the modern kernel handbook says this flag is never used. The older man-page description of an atomic table flag must not be promoted into an RTM_GETLINK statistics guarantee. [S11][S13]

**Inference:** missing interface, failed acquisition, malformed/incomplete input, and a successful observation of zero traffic are different states. Failure or missing data supplies no evidence for a zero delta. Keeping handles can reduce syscall overhead but requires lifecycle-aware invalidation. No cost comparison here establishes which mechanism is fastest for the eventual target environment.

## 6. Limits on rates and totals from consecutive samples

The following are **mathematical/engineering consequences**, not chosen display rules:

1. With the same accounting epoch, no wrap/reset, and observations `C0 <= C1`, the accounted delta is `C1 - C0`; a rate additionally needs a positive elapsed interval. A configured timer period is not evidence of actual elapsed time after scheduling delays or read retries. [S1][S11]
2. These counters are accumulated interface accounting, not per-read deltas or counters reset when a monitor starts. The docs require persistence across routine down/up operations. They do **not** establish a universal “since boot” epoch, persistence across destruction/recreation or reboot, or identical behavior under every driver/hardware recovery. A label or total whose meaning depends on such an epoch needs an application definition. [S1][S5]
3. A decrease can be evidence of a discontinuity, narrower underlying wrap, or driver behavior; a lower value alone does not identify which. The standard byte fields have no reset generation or incarnation token. A reset/recreation followed by enough traffic can also leave a value greater than the previous sample, so nondecrease is not proof of continuity. [S5][S6][S8]
4. A first observation has no previous observation interval. A changed identity or unknown continuity cannot yield a justified ordinary consecutive delta merely by subtracting integers. Missing observations can still permit a longer-interval delta **if** epoch continuity is otherwise established; they cannot recover a lost incarnation or several unknown wraps. [S5][S6][S13]
5. The collection window and asynchronous driver updates limit precision; RX/TX and different interfaces need not be sampled at one instant. A long-interval average can smooth cached refresh bursts, but no transport supplies the exact traffic-time distribution between samples. [S1][S9]
6. Monotonic elapsed time avoids wall-clock jumps. Linux `CLOCK_MONOTONIC` excludes suspend and permits consecutive identical timestamps; `CLOCK_BOOTTIME` includes suspend. Those are documented clock facts, leaving suspend handling and clock choice to the statistics/engineering contracts. [S23]

## 7. Open boundaries and follow-up

The ABI research resolves this ticket. Remaining product decisions belong to the existing startup/navigation, statistics, engineering, architecture/backend, screen, and acceptance tickets: initial interface/order/loopback treatment; identity continuity and disappearance response; reset/wrap/gap/total definitions; namespace/support assumptions; and backend/error-handling implementation. No additional ticket is needed just to repeat those decisions.

No universal answer was found for **driver-specific accounting freshness, exact virtual-interface byte layer, underlying hardware width/extension, or exceptional-reset behavior**. The authoritative general docs explicitly defer some of these to drivers. If a later requirement demands exact behavior on a named deployment target, a distinct, precisely scoped research ticket would be:

> **Verify byte-accounting behavior for the required deployment interface.** For the specified kernel version, interface kind, driver version, and relevant offload settings, trace the RX/TX standard-counter update and recovery paths to establish byte layer, underlying width/rollover extension, refresh cadence, and exceptional-reset behavior; report any driver-specific limits with source citations. Do not generalize the result to all Linux interfaces.

That conditional question needs a concrete target from the coordinator; no new ticket is created by this investigation. Likewise there is no measured latency/CPU-cost result here; a benchmark is only a separate investigation if the eventual backend has a concrete performance requirement.

## Source verification performed

Read kernel statistics documentation and checked its accounting, persistence, and sysfs-cost claims against proc, sysfs, rtnetlink, and common statistics code in **upstream v6.12**. Checked namespace binding in sysfs mount/class and proc-open code; checked rename, per-namespace cyclic index allocation, removal, and namespace movement in networking core; checked loopback's RX/TX callback. Cross-checked rtnetlink record widths/operations against the generated kernel family specification and Linux man-pages, and dump/loss/truncation behavior against the kernel netlink handbook. Read seq_file/sysfs read-rearming documentation and clock/directory/open semantics from authoritative manuals. These were source/document checks, not privileged runtime experiments or performance measurements.

GitHub raw fetches became rate-limited and git.kernel.org presented a bot challenge; those responses supplied no evidence. The needed tagged source was successfully retrieved and decoded from kernel.googlesource.com instead. A preliminary unversioned rt-link documentation URL returned 404; the cited v6.12 specification was successfully read. A codebrowser rendering was consulted during retrieval, then substantive source claims were verified against the tagged upstream mirror.

## Primary sources

- **S1** — [Linux v6.12 Interface statistics](https://docs.kernel.org/6.12/networking/statistics.html): uAPIs, `rtnl_link_stats64`, notes for driver authors.
- **S2** — [Linux v6.12 `net/core/net-procfs.c`](https://github.com/torvalds/linux/blob/v6.12/net/core/net-procfs.c): `dev_seq_printf_stats`, `dev_seq_*` iterator, `dev_proc_net_init`.
- **S3** — [Linux v6.12 sysfs statistics ABI](https://github.com/torvalds/linux/blob/v6.12/Documentation/ABI/testing/sysfs-class-net-statistics): `rx_bytes` and `tx_bytes`.
- **S4** — [Linux v6.12 `net/core/net-sysfs.c`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/net/core/net-sysfs.c): `netstat_show`, `NETSTAT_ENTRY`, namespace class, uevent, unregister.
- **S5** — [Linux v6.12 `net/core/dev.c`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/net/core/dev.c): `dev_get_stats`, `netdev_stats_to_stats64`, `dev_change_name`, `dev_index_reserve`, list/unlist, `__dev_change_net_namespace`.
- **S6** — [Linux v6.12 rt-link specification](https://docs.kernel.org/6.12/networking/netlink_spec/rt_link.html): getlink/getstats, structures, flags, attributes.
- **S7** — [Linux v6.12 sysfs network-class ABI](https://github.com/torvalds/linux/blob/v6.12/Documentation/ABI/testing/sysfs-class-net): ifindex, iflink, flags, address, name assignment.
- **S8** — [Linux v6.12 `net/core/rtnetlink.c`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/net/core/rtnetlink.c): `rtnl_fill_stats`, getlink/stats handlers and dumps, capability checks, events.
- **S9** — [Linux v6.12 `include/linux/u64_stats_sync.h`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/include/linux/u64_stats_sync.h): introductory counter-tearing and multi-counter-consistency comments.
- **S10** — [Kernel sysfs documentation](https://docs.kernel.org/filesystems/sysfs.html): class symlinks, RO modes, read rearming, object presence.
- **S11** — [Linux v6.12 Netlink handbook](https://docs.kernel.org/6.12/userspace-api/netlink/intro.html): dump consistency, buffer sizing, controls, classic flags.
- **S12** — [Linux man-pages `network_namespaces(7)`](https://man7.org/linux/man-pages/man7/network_namespaces.7.html): namespace visibility and interface ownership.
- **S13** — [Linux man-pages `netlink(7)`](https://man7.org/linux/man-pages/man7/netlink.7.html): datagrams, controls, ENOBUFS/resynchronization, multicast access.
- **S14** — [Linux man-pages `netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html): flags and addressless/non-running proc enumeration.
- **S15** — [Linux v6.12 `include/linux/netdevice.h`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/include/linux/netdevice.h): `for_each_netdev_dump`.
- **S16** — [Linux man-pages `readdir(3)`](https://man7.org/linux/man-pages/man3/readdir.3.html): filesystem-dependent directory order.
- **S17** — [Linux v6.12 `drivers/net/loopback.c`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/drivers/net/loopback.c): `loopback_xmit`, `loopback_get_stats64`, `loopback_net_init`.
- **S18** — [Linux man-pages `proc_pid_net(5)`](https://man7.org/linux/man-pages/man5/proc_pid_net.5.html): `/proc/net` namespace indirection and `/proc/net/dev` format.
- **S19** — [Linux v6.12 `fs/proc/proc_net.c`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/fs/proc/proc_net.c): `seq_open_net` captures the proc inode network namespace.
- **S20** — [Linux v6.12 `fs/sysfs/mount.c`](https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux/+/refs/tags/v6.12/fs/sysfs/mount.c): `sysfs_init_fs_context` captures network namespace tag.
- **S21** — [Linux man-pages `open(2)`](https://man7.org/linux/man-pages/man2/open.2.html): FD/path identity, openat directory references, permission and missing-path errors.
- **S22** — [Linux v6.12 seq_file documentation](https://docs.kernel.org/6.12/filesystems/seq_file.html): sessions, partial reads/seeks, regeneration.
- **S23** — [Linux man-pages `clock_gettime(2)`](https://man7.org/linux/man-pages/man2/clock_gettime.2.html): monotonic and boottime clock behavior.
