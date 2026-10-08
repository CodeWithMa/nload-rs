# 02: Honor an absent startup interface request once

**What to build:** When the optional startup-requested interface is absent, initially display the first naturally sorted interface, then automatically switch to the exact requested interface when it is first observed during monitoring. Honor this request once per session, even after earlier manual navigation, without repeating the switch when the interface later disappears and returns.

**Blocked by:** 01 — [Discover new interfaces while preserving selection](01-discover-new-interfaces.md).

**Status:** closed

**Assignee:** OpenCode

**Labels:** ready-for-agent

**Specification:** [Dynamic interface discovery and one-time startup-request selection](../spec.md).

- [x] A requested interface present in the initial nonempty snapshot is selected immediately and considered permanently honored for the session. With no interface request, startup selects the first naturally sorted interface and no later appearance causes automatic selection.
- [x] An absent requested name silently falls back to the first naturally sorted interface and remains pending. It does not create an unavailable placeholder, filter the navigable collection, or cause an extra acquisition.
- [x] The first accepted successful runtime observation containing the exact pending name incorporates its entry, initializes its baseline observation, and selects it in the same update. The displayed name, position/count, cumulative totals, and baseline-only statistics reflect that observation.
- [x] Manual navigation before the requested interface appears does not cancel the pending request. Unrelated additions preserve the selected name and leave the request pending until its exact match appears.
- [x] Matching uses the original name bytes, case-sensitively, without lossy conversion or presentation escaping. Scripted sessions verify case-distinct names, non-UTF-8 names, misleadingly similar escaped presentations, and a leading-dash positional name accepted through the existing option terminator.
- [x] Successful observations lacking the target, successful empty runtime snapshots, global read failures, and global parse failures preserve the pending request. Failed or malformed snapshots cannot trigger selection or partially add its entry; later successful exact observation can still fulfill the request.
- [x] Observations at equal or earlier timestamps neither honor the request nor alter membership or selection, consistent with the existing ignored-observation contract.
- [x] Once honored, the request never rearms after manual navigation, acquisition failure, disappearance, return, or name reuse. Verify this for requests fulfilled both at startup and later during runtime, including repeated disappearance/return cycles.
- [x] Returning requested names reuse retained entries and start fresh baseline observations without duplicating entries or pulling selection back. Ordinary additions continue preserving selection after request fulfillment.
- [x] Existing input ordering and shutdown priority remain intact. Automatic selection does not prevent later manual input from changing selection, add navigation aliases, or change rendering and output-backpressure behavior.
- [x] Initial empty discovery still exits with status 1 before terminal initialization, both with and without a request. Initial acquisition failure retains its distinct diagnostic. The feature introduces no initially empty waiting mode or new CLI option.
- [x] Acceptance tests use the approved scripted application-session seam for absent-name fallback, delayed matching, earlier manual navigation, failure recovery, exact-byte identity, and no reselection after fulfillment. Assert observable views, frames, diagnostics, and exit behavior rather than internal pending-state representation.
- [x] User-facing documentation and internal contract descriptions explain the one-time delayed switch, its persistence through manual navigation, and its permanent consumption once honored. Replace claims that absent startup requests are forgotten, while preserving historical specifications and identifying the current specification as superseding authority.
- [x] Repository formatting, lint, and automated-test checks pass with both slices integrated.

## Comments

- Resolution: Delivered with ticket 01. The delayed-selection tracer first
  failed with `lo` still selected, then passed after pending-request consumption.
  Exact-byte matching, navigation while waiting, failure recovery and no rearming
  are covered through scripted application sessions. All criteria are complete;
  see the [implementation record](../implementation.md) for coverage, verification,
  the user-approved supplementary monitor seam and completed review.
