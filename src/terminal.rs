// SPDX-License-Identifier: GPL-2.0-only

//! Linux terminal/runtime adapters and normalized terminal events.
//!
//! Owns terminal FDs, BOOTTIME/timerfd, signal notifications, incremental input,
//! nonblocking output, default-style clearing/absolute positioning of opaque
//! screen frames, and staged restoration (including SGR reset). Unsafe calls stay inside
//! concrete adapters. The thin application loop connects these effects to pure
//! modules. Session restoration also covers incomplete initialization.
//! Accepted TERM declarations assume UTF-8/Braille support (and xterm-256color
//! with --color); no locale, NO_COLOR or capability probe changes the chosen mode.

mod input;
mod linux;
mod runtime;
mod session;

pub(crate) use linux::{LinuxClock, LinuxRuntime, LinuxTerminal};
pub(crate) use runtime::{Clock, Event, Ready, Runtime, RuntimeEffects};
pub(crate) use session::SessionOps;
pub(crate) use session::{ExitReport, Failure, Notifications, Session, Shutdown, guarded};

#[cfg(test)]
pub(crate) use runtime::scripted;

#[cfg(test)]
pub(crate) use session::{HOOK_LOCK, scripted as scripted_session};
