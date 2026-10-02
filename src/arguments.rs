// SPDX-License-Identifier: GPL-2.0-only

//! OS argument tokens to informational output or a validated run configuration.
//!
//! Owns argument grammar, graph ceilings and Cargo-authoritative version output.
//! Must not initialize clocks, discover interfaces or touch terminal state.
//! Implementation belongs to the arguments/proc/startup ticket.
