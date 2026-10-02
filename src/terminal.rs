// SPDX-License-Identifier: GPL-2.0-only

//! Linux terminal/runtime adapters and normalized terminal events.
//!
//! Owns terminal FDs, BOOTTIME/timerfd, signal notifications, incremental input,
//! nonblocking output and staged restoration. Unsafe Linux calls stay inside
//! concrete adapters. Implementation belongs to the terminal session/events
//! tickets; the thin application loop connects these effects to pure modules.
