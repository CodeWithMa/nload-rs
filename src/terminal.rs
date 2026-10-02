// SPDX-License-Identifier: GPL-2.0-only

//! Linux terminal/runtime adapters and normalized terminal events.
//!
//! Owns terminal FDs, BOOTTIME/timerfd, signal notifications, incremental input,
//! nonblocking output and staged restoration. Unsafe Linux calls stay inside
//! concrete adapters. The thin application loop connects these effects to pure
//! modules. Session restoration also covers incomplete initialization.

mod linux;
mod session;

pub(crate) use linux::LinuxTerminal;
pub(crate) use session::{ExitReport, Failure, Notifications, Session, Shutdown, guarded};
