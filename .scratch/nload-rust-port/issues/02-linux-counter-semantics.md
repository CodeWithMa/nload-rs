# Establish Linux interface and traffic-counter semantics

Status: closed
Labels: wayfinder:research
Type: research
Mode: AFK
Parent: [Find the way to a minimal Rust port of nload](../map.md)
Assignee: alice (research agent: Linux counters)
Blocked by:

## Question

Which unprivileged Linux interfaces can enumerate network interfaces and sample incoming/outgoing byte counters for this monitor, and what guarantees do those interfaces provide? Compare `/proc/net/dev`, sysfs, and rtnetlink using kernel or other authoritative primary documentation. Establish counter meaning, width, consistency, interface identity and ordering, namespace visibility, rename/removal/recreation behavior, and failure characteristics. Distinguish documented facts from assumptions and recommendations. What limitations matter when deriving rates and totals from consecutive samples?

This investigation informs the later backend and statistics decisions; it does not select a backend. Link a single research Markdown asset citing primary sources.

## Comments

### Research context

Claimed for delegated research during charting. Throwaway branch: `research/nload-linux-counters`; isolated worktree: `/tmp/nix-shell.TfI7CV/opencode/nload-linux-counters`. The branch is unborn because this repository has no commits. Findings will be linked from `../research/linux-counters.md` and mirrored in the research worktree; no commit is authorized.

### Resolution — alice, 2026-10-01

Resolved with [Linux interface enumeration and byte-counter semantics](../research/linux-counters.md), mirrored identically at `/tmp/nix-shell.TfI7CV/opencode/nload-linux-counters/research/linux-counters.md`. Proc, sysfs, and rtnetlink expose the same standard byte accounting through normally unprivileged reads/GET requests; exported 64-bit width does not guarantee driver width, atomic RX/TX consistency, or a since-boot epoch. Findings establish namespace binding, reusable names/ifindices, rename/removal/movement behavior, loopback/order limits, driver freshness/reset qualifications, and acquisition/loss/performance trade-offs. Kernel documentation and tagged upstream v6.12 implementations were cross-checked against Linux man-pages; documented guarantees, implementation observations, and inference are distinguished. Backend and human-policy decisions remain with the existing decision tickets. The asset formulates one conditional driver-specific research follow-up if a concrete deployment target requires stronger guarantees.
