# Implement the pure monitor and statistics runs

Status: open
Labels: wayfinder:task
Type: task
Mode: AFK
Parent: [Implement and verify the minimal Rust port of nload](../map.md)
Assignee:
Blocked by: 01

## Question

Can a pure Monitor module implement the fixed startup list, navigation, paired observation/reset semantics, exact-delta rate windows and five-minute graph histories specified in sections 3, 4 and 8, exposing only the selected read-only view?

Implement construction from ordered names, selection, ceilings and initial baselines; explicit-time observations for retained names; forward/backward navigation; and the selected view contract. Keep counters/endpoints/deltas exact, window calculations and extrema unrounded, baseline/unavailable states explicit, and histories observation-owned. Preserve boundary-crossing intervals, reset both directions on discontinuity, reject nonpositive observation intervals without advancing statistics/history, and never fabricate catch-up columns or bridge unavailable gaps.

Cover applicable A11–A12 and S03–S12 using explicit-time fixtures through public-to-the-application monitor/view operations rather than private queues. Include the specification's numerical examples, paired reset cases, off-screen history, boundary expiry and resize-independent history. Document the view interface for screen/runtime integration and link test mappings/results in the verification record. Clock/deadline effect behavior belongs to integration, not this pure module.
