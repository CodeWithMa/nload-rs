# Dynamic interface discovery and one-time startup-request selection

Status: open
Labels: ready-for-agent
Confirmed: 2026-10-08, from the agreed behavior and application-session test seam

## Problem Statement

An interface that appears after nload-rs starts cannot currently be viewed without restarting the application. This affects interfaces created during a running session, such as a VPN interface brought up after monitoring begins.

Supplying that interface's name at launch does not help: when it is absent from the initial discovery, nload-rs falls back to the first interface and forgets the request. The user must restart and select the interface again once it exists.

## Solution

Automatically add interfaces observed during a running session to the existing arrow-key navigable collection. Keep the complete retained collection naturally sorted, and preserve the selected interface when new entries change its position.

Honor the optional startup interface request once. If it exists at startup, select it immediately. If it does not, initially display the first naturally sorted interface and automatically switch to the requested interface when it is first observed. Manual navigation while waiting does not cancel this switch. Once honored, the request never causes another automatic switch during that session.

Continue retaining disappeared interfaces as unavailable entries. Returning names reuse their entries and begin fresh statistics runs. Starting with no interfaces remains an error.

## User Stories

1. As a network-monitor user, I want an interface created after launch to become navigable automatically, so that I can monitor it without restarting nload-rs.
2. As a network-monitor user, I want new interfaces added to the existing single-interface view's collection, so that I can continue using familiar arrow-key navigation.
3. As a network-monitor user, I want the collection to remain naturally sorted after additions, so that I can find interfaces predictably.
4. As a network-monitor user, I want names such as eth2 to precede eth10, so that numeric name suffixes behave consistently with startup ordering.
5. As a network-monitor user, I want multiple interfaces discovered in one observation to be added together, so that the next displayed collection reflects the complete successful observation.
6. As a network-monitor user, I want interfaces included regardless of activity, address assignment, or whether they are loopback interfaces, so that dynamic discovery matches startup discovery.
7. As a network-monitor user, I want a new interface that sorts before my selection to leave the same interface selected, so that discovery does not unexpectedly change what I am watching.
8. As a network-monitor user, I want the displayed position and interface count to reflect additions, so that navigation accurately describes the current retained collection.
9. As a network-monitor user, I want all four arrow keys to navigate the updated collection with wrapping, so that new interfaces are reachable in either direction.
10. As a network-monitor user, I want navigation to remain independent of counter acquisition, so that moving through the collection does not distort traffic statistics.
11. As a network-monitor user, I want existing interfaces to retain their ongoing statistics and graph history when another interface is added, so that discovery does not interrupt monitoring.
12. As a network-monitor user, I want a newly observed interface to begin with a baseline observation, so that nload-rs does not invent traffic rates for a period it did not observe.
13. As a network-monitor user, I want off-screen new interfaces to be monitored too, so that they have available statistics when I navigate to them later.
14. As a network-monitor user, I want configured incoming and outgoing graph ceilings to apply to new interfaces, so that the session's settings remain consistent.
15. As a network-monitor user, I want a requested interface present at startup to be selected immediately, so that the existing launch workflow continues to work.
16. As a network-monitor user, I want an absent startup-requested interface to fall back silently to the first naturally sorted interface, so that I can start monitoring while waiting for it.
17. As a network-monitor user, I want nload-rs to remember an absent startup-requested interface, so that it can select that interface when it appears.
18. As a network-monitor user, I want the delayed automatic switch to occur even if I navigated elsewhere while waiting, so that my explicit launch request is eventually honored.
19. As a network-monitor user, I want startup selection to match the exact original interface-name bytes, so that case differences and non-UTF-8 names do not select a different interface.
20. As a network-monitor user, I want my startup request to select an interface without filtering the collection, so that other interfaces remain accessible.
21. As a network-monitor user, I want the startup request honored only once, so that disappearance and return do not repeatedly pull me away from another interface.
22. As a network-monitor user, I want ordinary additions not matching a pending startup request to preserve my selection, so that unrelated interfaces do not interrupt my view.
23. As a network-monitor user, I want disappeared interfaces retained as unavailable entries, so that I can see their absence and navigate away when I choose.
24. As a network-monitor user, I want a returning interface name to reuse its retained entry, so that repeated appearance does not create duplicate navigation entries.
25. As a network-monitor user, I want a returning interface to start a fresh statistics run, so that traffic across an observation gap is not presented as continuous measurement.
26. As a network-monitor user, I want counter acquisition failures to remain nonfatal and not erase the collection or a pending startup request, so that monitoring can recover when acquisition succeeds again.
27. As a network-monitor user, I want malformed snapshots rejected as a whole, so that partial data cannot create spurious interfaces or trigger an incorrect automatic switch.
28. As a network-monitor user, I want monitoring to continue if a later successful observation contains no interfaces, so that transient disappearance does not end an established session.
29. As a network-monitor user, I want an initially empty discovery to keep its existing clear startup error, so that launching with nothing to monitor does not silently enter a waiting mode.
30. As a network-monitor user, I want documentation to describe dynamic discovery and one-time selection accurately, so that I know how launch arguments behave when interfaces appear later.

## Implementation Decisions

- **Existing module responsibilities:** Extend the counter-snapshot/discovery module, pure monitor state, and application integration. Argument parsing continues to provide the same optional raw-byte interface name. The screen continues rendering a selected-interface view; the terminal runtime continues handling input, timing, output, and cleanup.
- **One acquisition source:** Reuse each existing complete counter snapshot to discover names and update statistics. Do not introduce a second acquisition, a separate discovery timer, a background watcher, or per-interface reads. Runtime discovery occurs on the existing 500-ms sampling schedule, using acquisition-completion timestamps and the existing missed-slot policy.
- **Observation contract:** The monitor's round-ingestion boundary must receive enough information to incorporate all names from a successful snapshot as well as update every retained entry. Its current lookup-only boundary cannot reveal names outside the retained collection. Keep counter acquisition and clock reads outside the pure monitor; exact internal types are left to implementation.
- **Shared ordering:** Startup and runtime membership use the same existing natural-order contract. Digit runs compare by numeric value without bounded integer conversion; other bytes compare case-sensitively and independently of locale. Complete original bytes break natural ties, including eth02 before eth2. The entire retained collection, including unavailable entries, remains sorted.
- **Identity and uniqueness:** Original interface-name bytes remain the identity for matching, ordering, observation lookup, and selection. Add only names not already retained. A rename exposes the old name as unavailable and adds the newly observed name independently; do not infer kernel-interface identity or transfer history between names.
- **Selection preservation:** Membership updates preserve the selected name, rather than preserving its old numeric position. Recompute the selected entry's displayed one-based position and total count after additions. The only addition-driven exception is honoring a pending startup request.
- **Initial request handling:** A requested name present in the initial nonempty snapshot is selected and immediately considered honored. An absent requested name silently falls back to the first naturally sorted interface and remains pending. With no request, select the first interface and carry no pending selection. Do not create an unavailable placeholder for a name never observed.
- **One-time delayed selection:** During the first accepted successful runtime observation containing the pending exact name, add its entry, initialize its baseline, and select it as part of the same membership/statistics update. Permanently clear the pending request for the session. Earlier manual navigation does not cancel it. Unrelated additions, failed acquisitions, and observations without that name leave it pending.
- **No repeated request:** An honored request never rearms after manual navigation, counter acquisition failure, disappearance, return, or name reuse. This applies both to requests honored at startup and to requests honored later. Without a pending request, appearances never automatically change the selected interface.
- **Retained lifecycle:** Missing names remain in the collection and follow the existing unavailable-interface rules. A successful empty runtime snapshot marks retained entries absent without emptying the collection or ending the session. Return reuses the entry and starts a fresh baseline observation rather than recovering pre-gap measurements.
- **Statistics continuity:** New entries receive baseline observations using their first successful snapshot's cumulative totals and completion timestamp, with no measured rates or graph samples yet. Subsequent observations use the existing statistics rules. Additions do not reconstruct existing entries, reset their statistics runs, or discard their graph history. Existing missing-name, failure, and counter-decrease resets still apply normally.
- **Settings and off-screen monitoring:** The session's incoming/outgoing graph ceilings apply to new entries. Continue observing every retained interface, not just the selected one, including entries added while off-screen.
- **Failure and timestamp semantics:** Global read or parse failure supplies no discoverable names, cannot satisfy a pending request, and preserves membership and selection while applying the existing read-failed statistics reset. Snapshots remain all-or-nothing validated. Preserve the existing rule that observations at equal or earlier timestamps are ignored entirely, including membership and pending-selection effects.
- **Startup failures:** Initial empty discovery and initial acquisition failure retain their distinct diagnostics, status 1, and failure before terminal initialization. Waiting with an initially empty collection is not introduced.
- **Navigation and presentation:** Keep existing arrow-key mappings, wrapping, input ordering, shutdown priority, redraw behavior, and output-backpressure rules. Navigation and resize do not acquire counters. Updated views expose the new count and selected position to the existing renderer. An automatic switch does not prevent later manual input from changing selection.
- **Documentation:** Update user-facing descriptions and internal contract comments that currently promise startup-only membership or forgotten missing-name requests. Preserve the historical specifications and identify this specification as the superseding authority for the changed behavior.

## Testing Decisions

- **Approved primary seam:** Use the existing scripted application-session harness as the single primary acceptance seam. The user confirmed this boundary on 2026-10-08. Drive normal argument handling, startup, and the application loop with scripted complete snapshots, acquisition failures, clock advances, terminal input, and terminal effects; inspect observable selected views, emitted frames, diagnostics, and exit status.
- **Modules exercised:** Application integration, counter snapshot parsing and discovery, monitor membership/selection/statistics, and selected-interface rendering are exercised together through the existing session boundary. No new production test seam is required.
- **Good tests:** Assert externally observable behavior, not container choice, insertion strategy, internal pending-state representation, comparator visibility, or private call sequences. Use names, position/count, navigation order, availability, cumulative totals, rates, and graph history to establish the contract. Avoid duplicating an implementation algorithm in test expectations.
- **Prior art:** Existing scripted application tests already cover startup fallback, exact raw-byte selection, navigation without acquisition, slow acquisition scheduling, off-screen statistics, quiet runtime failure/absence/return, and frame backpressure. Extend that style. Replace the existing fixed-membership expectation for runtime additions rather than retaining contradictory assertions. Existing natural-order and lifecycle tests remain useful regression coverage.
- **Harness scope:** Reuse scripted terminal output and the existing selected-view capture. If multiple intermediate states need inspection, extend test-only capture at this same session boundary or use separate sessions ending at those states; do not introduce production callbacks solely for tests.

Acceptance scenarios:

1. Start with a nonempty collection, then observe new names that sort before, between, and after existing names. Check the selected name is unchanged, displayed position/count are updated, and arrow navigation visits the complete naturally sorted collection with wrapping.
2. Add several names in a single snapshot, vary snapshot row order, and repeat those names in later snapshots. Verify deterministic navigation order and no duplicate entries. Include numeric ordering and natural ties such as eth02, eth2, and eth10.
3. Add loopback, inactive, or zero-counter interfaces and verify they are navigable; acquisition does not filter by activity or address presence.
4. Launch with no interface request and add an interface sorting before the current selection. Verify selection stays on the same name, not the same index or the newly first name.
5. Launch with an exact requested name already present, manually navigate away, then remove and return the requested name. Verify there is no automatic reselection and its return uses the retained entry with a fresh baseline.
6. Launch with an absent requested name. Verify silent first-interface fallback and no placeholder; then make the exact name appear and verify automatic selection, updated position/count, and baseline-only statistics.
7. While that request is pending, navigate to a different interface and add unrelated names. Verify the pending request is still honored when its exact name subsequently appears.
8. After delayed selection has occurred, navigate away and remove/return the requested name repeatedly. Verify count does not grow through returns and selection is never pulled back automatically.
9. Keep a request pending through successful snapshots lacking its name, a successful empty snapshot, and global read/parse failures. Verify membership is retained, failures are quiet and nonfatal, and the first subsequent successful matching observation still selects it.
10. Use case-distinct and non-UTF-8 interface names, including names whose escaped presentation could be confused. Verify matching uses original bytes and a near-match does not satisfy the request. Exercise a leading-dash positional name through the existing option terminator.
11. Add an ordinary interface while existing interfaces have measured statistics and graph history. Verify ongoing observations continue their statistics runs without insertion-induced resets. Verify the new interface initially has cumulative totals but no rate/sample, then acquires valid rates on the next observation.
12. Add an off-screen interface, observe it further, and navigate to it. Verify it has the expected accumulated observations and configured incoming/outgoing graph ceilings.
13. Remove the selected interface while adding another. Verify the selected name is retained and shown unavailable unless that observation also fulfills a pending request; verify navigation can reach the new entry. On return, verify a fresh baseline and no duplicate.
14. Observe an old name disappearing and a new name appearing. Verify separate retained entries rather than rename inference or history transfer.
15. Feed a malformed complete snapshot containing an apparent new/requested name. Verify the whole observation is rejected, count does not grow, the request is not honored, and retained entries show the existing read-failed state.
16. Verify additions and delayed selection use the regular acquisition schedule, without extra discovery reads. Retain navigation/resize-only no-acquisition and delayed-slot no-catch-up coverage.
17. Verify observations at nonpositive elapsed time have no membership, selection, or statistics effects, using the existing clock/observation test support.
18. Launch with empty discovery, both with and without an interface request. Verify status 1 and no terminal initialization. Retain the distinct initial acquisition-failure diagnostic and startup sequence coverage.

Run the repository's formatting, lint, and automated-test checks during implementation. Existing terminal cleanup and rendering regression tests should continue passing; this feature does not require a new terminal/emulator verification campaign or privileged creation of live network interfaces.

## Out of Scope

- A visible interface-list widget, multi-interface view, dashboard, or navigation redesign.
- Removing retained entries, automatically abandoning a disappeared selected interface, or inferring identity across interface renames.
- Repeated automatic selection, rearming the startup request, or adding an option that cancels pending selection through navigation.
- Waiting for the first interface when startup discovery is empty.
- Interface filters, address discovery, activity-based inclusion, or new CLI options.
- New acquisition backends, netlink/watchers, polling cadence changes, statistics formula changes, graph design changes, or terminal lifecycle changes.
- Implementation code, build tickets, and execution evidence in this planning artifact.

## Further Notes

- This specification records the human's accepted recommendations for dynamic additions, selection preservation, one-time request fulfillment even after manual navigation, retained unavailable entries, and unchanged empty-startup failure. The test seam was explicitly confirmed separately.
- Use the domain vocabulary in [Network Traffic Monitor glossary](../../GLOSSARY.md). No new domain terms or hard-to-reverse architectural decisions are needed for this change.
- This specification supersedes the startup-only membership, ignored runtime additions/rename targets, and forgotten absent request clauses in [Minimal Rust port of nload — implementation specification](../nload-rust-port/spec.md), specifically the affected portions of sections 2.1, 3.1, 3.3, and 4.1. Its graph-ceiling and sampling scope extends from startup entries to all retained entries. Other startup, acquisition, timing, statistics, and navigation contracts remain inherited.
- Presentation and terminal behavior follow [btop-style graph spec for nload-rs — implementation handoff](../btop-graph-spec/spec.md). This change updates which interface and collection position/count that existing single-interface view displays.
- The route is fully specified and fits a focused implementation effort; no wayfinding map was created. This local tracker specification carries the ready-for-agent label and is the implementation handoff.
