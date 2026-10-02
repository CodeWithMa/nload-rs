// SPDX-License-Identifier: GPL-2.0-only

//! Linux terminal/runtime adapters and normalized terminal events.
//!
//! Owns terminal FDs, BOOTTIME/timerfd, signal notifications, incremental input,
//! nonblocking output and staged restoration. Unsafe Linux calls stay inside
//! concrete adapters. The thin application loop connects these effects to pure
//! modules. Session restoration also covers incomplete initialization.

mod input;
mod linux;
mod runtime;
mod session;

pub(crate) use linux::{LinuxClock, LinuxRuntime, LinuxTerminal};
pub(crate) use runtime::{Clock, Event, Ready, Runtime, RuntimeEffects};
pub(crate) use session::{ExitReport, Failure, Notifications, Session, Shutdown, guarded};

#[cfg(test)]
pub(crate) use runtime::scripted;
