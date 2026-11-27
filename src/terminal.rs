// SPDX-License-Identifier: GPL-2.0-only

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
