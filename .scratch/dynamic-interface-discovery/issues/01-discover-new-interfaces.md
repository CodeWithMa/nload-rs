# 01: Discover new interfaces while preserving selection

**What to build:** Interfaces appearing during an established monitoring session become available automatically through the existing arrow-key navigation, in natural order. Keep the same interface selected when additions change its position, retain unavailable entries, and monitor new interfaces with the session's existing statistics and graph settings.

**Blocked by:** None (can start immediately).

**Status:** closed

**Assignee:** OpenCode

**Labels:** ready-for-agent

**Specification:** [Dynamic interface discovery and one-time startup-request selection](../spec.md). This slice establishes runtime membership; one-time delayed startup-request selection is delivered by the subsequent ticket.

- [x] Every accepted successful sampling round incorporates all previously unseen interface-name bytes from its complete snapshot, without activity or address filtering. Repeated names reuse retained entries rather than creating duplicates.
- [x] Startup and runtime membership share the existing natural ordering: numeric digit runs, case-sensitive locale-independent non-digit bytes, and full original-byte tie-breaking. Verify additions before, between, and after existing entries, multiple additions together, varying snapshot row order, and names such as eth02, eth2, and eth10.
- [x] Insertion preserves the selected interface's original name bytes, not its previous numeric index. The displayed one-based position and total count reflect the updated collection, including unavailable entries.
- [x] Right/Down and Left/Up navigate the complete updated collection with existing symmetric wrapping. Navigation and resize remain redraw operations and do not acquire counters or reset statistics.
- [x] New interfaces begin with baseline observations containing cumulative totals but no measured rates or graph samples. Further observations produce statistics normally, including while those interfaces are off-screen. Both configured graph ceilings apply to new entries.
- [x] Adding interfaces does not restart existing statistics runs or discard existing graph history. Ordinary missing-name, acquisition-failure, and counter-decrease reset rules continue to apply independently.
- [x] Disappeared names remain navigable unavailable entries, and a disappeared selected interface stays selected. A successful empty runtime snapshot marks retained entries absent without terminating the session or emptying the retained collection.
- [x] Returning names reuse their existing entries and start fresh statistics runs with baseline observations, even if their counters are higher. A rename retains the old name as unavailable and adds the new name separately, without inferred identity or transferred history.
- [x] Global read or parse failures remain quiet and nonfatal during monitoring. They preserve membership and selected identity, supply no new names, and apply existing read-failed statistics behavior. A malformed snapshot cannot partially add interfaces.
- [x] Discovery reuses the existing bulk acquisition on the 500-ms sampling schedule, with acquisition-completion timestamps and missed-slot behavior. It adds no extra counter acquisitions, timers, or per-interface reads. Observations at equal or earlier timestamps have no membership, selection, or statistics effects.
- [x] Initial empty discovery and initial acquisition failure retain their distinct diagnostics, status 1, and failure before terminal initialization. Startup fallback, exact original-byte name matching, and argument handling remain compatible while the delayed-request behavior awaits the subsequent slice.
- [x] Acceptance coverage uses the approved existing scripted application-session seam to drive snapshots, failures, clock advances, and input, and assert selected views and emitted frames. Replace obsolete fixed-membership expectations and retain relevant navigation, statistics, scheduling, rendering, and cleanup regressions. Extend only test-side capture when needed; no new production test seam is introduced.
- [x] User-facing documentation and internal contract descriptions explain runtime discovery, retained unavailable entries, and selection preservation accurately, with historical specifications preserved as superseded authority where applicable.
- [x] Repository formatting, lint, and automated-test checks pass for this slice.

## Comments

- OpenCode: Runtime membership is implemented. The discovery tracer test failed
  with the old count/position (1/2 instead of 4/8), then passed after extending
  round ingestion with complete snapshot names. Focused application-session tests
  and `cargo check --locked --all-targets` pass. The implementation dependency for
  ticket 02 is satisfied; integrated verification and review remain before closure.
- Resolution: Delivered and verified together with ticket 02. All acceptance
  criteria are complete; the user-approved supplementary monitor regression
  covers nonpositive observation times. See the [implementation record](../implementation.md)
  for test coverage, passing checks, existing Clippy warning sites and completed
  standards/specification review.
