// SPDX-License-Identifier: GPL-2.0-only

//! Pure selected-monitor-view and dimensions to bounded ASCII screen rows.
//!
//! Owns geometry, escaping, formatting, graph glyphs and small-terminal mode.
//! Does not mutate the monitor or emit terminal escape sequences.
//! Implementation belongs to the ASCII screen ticket.
