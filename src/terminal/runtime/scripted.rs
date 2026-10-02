// SPDX-License-Identifier: GPL-2.0-only

//! Shared scripted adapters for runtime and later application integration tests.
//! Handles observe effect requests and emitted bytes without accessing Runtime's
//! decoder or frame bookkeeping. No real-time sleeps or numerical clock oracle.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use super::{Clock, Notifications, Ready, RuntimeEffects};

#[derive(Default)]
pub(crate) struct ClockScript {
    pub(crate) time: u64,
    pub(crate) readings: VecDeque<io::Result<u64>>,
    pub(crate) calls: usize,
}

#[derive(Clone, Default)]
pub(crate) struct ScriptClock(pub(crate) Rc<RefCell<ClockScript>>);

impl Clock for ScriptClock {
    fn now(&mut self) -> io::Result<u64> {
        let mut script = self.0.borrow_mut();
        script.calls += 1;
        let time = script.time;
        script.readings.pop_front().unwrap_or(Ok(time))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Call {
    Wait {
        deadline: u64,
        idle_ns: Option<u64>,
        output: bool,
    },
    Notifications,
    Dimensions,
    Read(usize),
    Write(Vec<u8>),
}

#[derive(Default)]
pub(crate) struct EffectsScript {
    pub(crate) calls: Vec<Call>,
    pub(crate) waits: VecDeque<io::Result<Ready>>,
    pub(crate) notifications: VecDeque<io::Result<Notifications>>,
    pub(crate) dimensions: VecDeque<io::Result<(usize, usize)>>,
    pub(crate) reads: VecDeque<io::Result<Vec<u8>>>,
    pub(crate) writes: VecDeque<io::Result<usize>>,
    pub(crate) emitted: Vec<u8>,
}

#[derive(Clone, Default)]
pub(crate) struct ScriptedEffects {
    pub(crate) clock: ScriptClock,
    pub(crate) script: Rc<RefCell<EffectsScript>>,
}

impl Clock for ScriptedEffects {
    fn now(&mut self) -> io::Result<u64> {
        self.clock.now()
    }
}

impl RuntimeEffects for ScriptedEffects {
    fn wait(&mut self, deadline: u64, idle_ns: Option<u64>, output: bool) -> io::Result<Ready> {
        let mut script = self.script.borrow_mut();
        script.calls.push(Call::Wait {
            deadline,
            idle_ns,
            output,
        });
        script.waits.pop_front().unwrap_or(Ok(Ready::default()))
    }

    fn notifications(&mut self) -> io::Result<Notifications> {
        let mut script = self.script.borrow_mut();
        script.calls.push(Call::Notifications);
        script
            .notifications
            .pop_front()
            .unwrap_or(Ok(Notifications::default()))
    }

    fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        let mut script = self.script.borrow_mut();
        script.calls.push(Call::Dimensions);
        script.dimensions.pop_front().unwrap_or(Ok((80, 24)))
    }

    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let mut script = self.script.borrow_mut();
        script.calls.push(Call::Read(bytes.len()));
        let mut input = script
            .reads
            .pop_front()
            .unwrap_or_else(|| Err(io::ErrorKind::WouldBlock.into()))?;
        let count = input.len().min(bytes.len());
        bytes[..count].copy_from_slice(&input[..count]);
        if count < input.len() {
            let remaining = input.split_off(count);
            script.reads.push_front(Ok(remaining));
        }
        Ok(count)
    }

    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut script = self.script.borrow_mut();
        script.calls.push(Call::Write(bytes.to_vec()));
        let count = script.writes.pop_front().unwrap_or(Ok(bytes.len()))?;
        if count <= bytes.len() {
            script.emitted.extend_from_slice(&bytes[..count]);
        }
        Ok(count)
    }
}
