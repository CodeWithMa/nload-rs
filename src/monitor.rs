// SPDX-License-Identifier: GPL-2.0-only

//! Pure traffic statistics, fixed interface membership and navigation.
//!
//! Owns statistics runs, availability, exact deltas, windows, extrema and history.
//! Exposes selected read-only views; performs no I/O, clock reads or sizing.
//! Implementation belongs to the monitor/statistics ticket.
