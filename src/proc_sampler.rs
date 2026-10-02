// SPDX-License-Identifier: GPL-2.0-only

//! Whole-list acquisition of original-byte interface names and paired counters.
//!
//! Owns fresh `/proc/net/dev` reads through EOF and complete snapshot validation.
//! Runtime snapshots do not change the monitor's fixed startup membership.
//! Implementation belongs to the arguments/proc/startup ticket.
