// SPDX-License-Identifier: GPL-2.0-only

//! Session effects for complete application startup/cleanup scripts.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::rc::Rc;
use std::time::Duration;

use super::{Notifications, Saved, SessionOps, Support};
use crate::terminal::scripted::ScriptClock;

#[derive(Default)]
pub(crate) struct State {
    pub(crate) calls: Vec<&'static str>,
    pub(crate) log: Option<Rc<RefCell<Vec<&'static str>>>>,
    pub(crate) faults: BTreeMap<&'static str, bool>, // true panics, false errors
    pub(crate) dimensions: Option<(usize, usize)>,
    pub(crate) init_delay: u64,
    pub(crate) unsupported: bool,
    pub(crate) emitted: Vec<u8>,
}

#[derive(Clone, Default)]
pub(crate) struct ScriptSession {
    pub(crate) state: Rc<RefCell<State>>,
    pub(crate) clock: ScriptClock,
}

impl ScriptSession {
    fn step(&mut self, name: &'static str) -> io::Result<()> {
        let fault = {
            let mut state = self.state.borrow_mut();
            state.calls.push(name);
            if let Some(log) = &state.log {
                log.borrow_mut().push(name);
            }
            state.faults.remove(name)
        };
        match fault {
            Some(true) => panic!("scripted {name} panic"),
            Some(false) => Err(io::Error::other(format!("scripted {name} failure"))),
            None => Ok(()),
        }
    }
}

impl SessionOps for ScriptSession {
    type Termios = ();
    fn support(&mut self) -> Support {
        self.step("support").unwrap();
        Support {
            input_terminal: !self.state.borrow().unsupported,
            output_terminal: true,
            term: Some("xterm".into()),
        }
    }
    fn snapshot(&mut self) -> io::Result<Saved<()>> {
        self.step("snapshot")?;
        Ok(Saved {
            termios: (),
            input_flags: 0,
            output_flags: 0,
        })
    }
    fn prepare_notifications(&mut self) -> io::Result<()> {
        self.step("prepare")
    }
    fn raw(&mut self, _: &()) -> io::Result<()> {
        self.step("raw")
    }
    fn input_flags(&mut self, flags: i32) -> io::Result<()> {
        self.step(if flags == 0 { "restore-input" } else { "input" })
    }
    fn output_flags(&mut self, flags: i32) -> io::Result<()> {
        self.step(if flags == 0 {
            "restore-output"
        } else {
            "output"
        })
    }
    fn restore_termios(&mut self, _: &()) -> io::Result<()> {
        self.step("restore-termios")
    }
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let name = match bytes {
            super::ENTER_SCREEN => "enter",
            super::HIDE_CURSOR => "hide",
            super::SHOW_CURSOR => "show",
            super::LEAVE_SCREEN => "leave",
            _ => "write",
        };
        self.step(name)?;
        self.state.borrow_mut().emitted.extend_from_slice(bytes);
        if name == "hide" {
            self.clock.0.borrow_mut().time += self.state.borrow().init_delay;
        }
        Ok(bytes.len())
    }
    fn wait_writable(&mut self, _: Duration) -> io::Result<()> {
        self.step("wait-output")
    }
    fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        self.step("dimensions")?;
        Ok(self.state.borrow().dimensions.unwrap_or((80, 24)))
    }
    fn notifications(&mut self) -> io::Result<Notifications> {
        self.step("notifications")?;
        Ok(Notifications::default())
    }
    fn release_notifications(&mut self) {
        self.step("release").unwrap();
    }
}
