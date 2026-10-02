// SPDX-License-Identifier: GPL-2.0-only

use std::io;

use super::input::Decoder;
use super::{Notifications, Shutdown};

const INPUT_BATCH: usize = 256;
const OUTPUT_BATCH: usize = 16 * 1024;

/// Integer CLOCK_BOOTTIME nanoseconds; errors are fatal, never counter failures.
pub(crate) trait Clock {
    fn now(&mut self) -> io::Result<u64>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Forward,
    Backward,
    Shutdown(Shutdown),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Ready {
    pub(crate) input: bool,
    pub(crate) signals: bool,
    pub(crate) timer: bool,
    pub(crate) output: bool,
}

/// Real effects vary here: LinuxRuntime borrows owned session/clock effects;
/// ScriptedEffects supplies deterministic clock, readiness, input and output.
pub(crate) trait RuntimeEffects: Clock {
    fn wait(
        &mut self,
        sample_deadline: u64,
        idle_ns: Option<u64>,
        output: bool,
    ) -> io::Result<Ready>;
    fn notifications(&mut self) -> io::Result<Notifications>;
    fn dimensions(&mut self) -> io::Result<(usize, usize)>;
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize>;
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize>;
}

struct Frame {
    bytes: Vec<u8>,
    emitted: usize,
}

/// Decoder/output state only; never acquires counters or coordinates redraws.
pub(crate) struct Runtime<O> {
    effects: O,
    decoder: Decoder,
    frame: Option<Frame>,
    previous_time: Option<u64>,
}

impl<O: RuntimeEffects> Runtime<O> {
    pub(crate) fn new(effects: O) -> Self {
        Self {
            effects,
            decoder: Decoder::default(),
            frame: None,
            previous_time: None,
        }
    }

    pub(crate) fn notifications(&mut self) -> io::Result<Notifications> {
        self.effects.notifications()
    }

    pub(crate) fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        self.effects.dimensions()
    }

    /// Arms an absolute one-shot BOOTTIME sample deadline; decoder timeout is a
    /// poll timeout. EINTR is a spurious wake, so callers recheck shutdown/time.
    pub(crate) fn wait(&mut self, sample_deadline: u64) -> io::Result<Ready> {
        let now = self.now()?;
        let idle = self
            .decoder
            .deadline()
            .map(|deadline| deadline.saturating_sub(now));
        self.effects
            .wait(sample_deadline, idle, self.has_pending_output())
    }

    /// One read of at most 256 bytes; EINTR/EAGAIN yield to the loop. Expiration
    /// also runs without readable input and never requests a counter acquisition.
    pub(crate) fn input_batch(&mut self, readable: bool) -> io::Result<Vec<Event>> {
        let now = self.now()?;
        self.decoder.expire(now);
        if !readable {
            return Ok(Vec::new());
        }
        let mut bytes = [0; INPUT_BATCH];
        match self.effects.read(&mut bytes) {
            Ok(0) => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "terminal input closed",
            )),
            Ok(n) if n <= bytes.len() => self.decoder.feed(&bytes[..n], now),
            Ok(_) => Err(io::Error::other("terminal read exceeded supplied buffer")),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                Ok(Vec::new())
            }
            Err(error) => Err(error),
        }
    }

    /// False means an emitted frame must finish: leave the application's latest
    /// redraw request dirty, and render the latest view after that frame finishes.
    /// This owns exactly one frame, not a queue of already stale views.
    pub(crate) fn queue_frame(
        &mut self,
        rows: &[String],
        width: usize,
        height: usize,
    ) -> io::Result<bool> {
        if self.frame.as_ref().is_some_and(|frame| frame.emitted != 0) {
            return Ok(false);
        }
        if rows.len() > height
            || rows.iter().any(|row| {
                row.len() > width || !row.bytes().all(|byte| (b' '..=b'~').contains(&byte))
            })
        {
            return Err(io::Error::other(
                "frame rows exceed dimensions or contain nonprintable bytes",
            ));
        }
        // No LF/CR and no character beyond the supplied right margin. Absolute
        // positioning cancels delayed autowrap without changing terminal modes.
        let mut bytes = b"\x1b[H\x1b[2J".to_vec();
        if width != 0 {
            for (index, row) in rows.iter().enumerate() {
                bytes.extend_from_slice(format!("\x1b[{};1H", index + 1).as_bytes());
                bytes.extend_from_slice(row.trim_end_matches(' ').as_bytes());
            }
        }
        bytes.extend_from_slice(b"\x1b[H");
        self.frame = Some(Frame { bytes, emitted: 0 });
        Ok(true)
    }

    pub(crate) fn has_pending_output(&self) -> bool {
        self.frame.is_some()
    }

    /// One nonblocking syscall of at most 16 KiB, never a writability wait.
    pub(crate) fn output_batch(&mut self) -> io::Result<()> {
        let Some(frame) = self.frame.as_mut() else {
            return Ok(());
        };
        let end = frame
            .bytes
            .len()
            .min(frame.emitted.saturating_add(OUTPUT_BATCH));
        let suffix = &frame.bytes[frame.emitted..end];
        match self.effects.write(suffix) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "terminal output closed",
                ));
            }
            Ok(n) if n <= suffix.len() => frame.emitted += n,
            Ok(_) => return Err(io::Error::other("terminal write exceeded supplied bytes")),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) => {}
            Err(error) => return Err(error),
        }
        if frame.emitted == frame.bytes.len() {
            self.frame = None;
        }
        Ok(())
    }
}

impl<O: RuntimeEffects> Clock for Runtime<O> {
    fn now(&mut self) -> io::Result<u64> {
        let now = self.effects.now()?;
        if self
            .previous_time
            .is_some_and(|previous| now.checked_sub(previous).is_none())
        {
            return Err(io::Error::other("BOOTTIME clock moved backwards"));
        }
        self.previous_time = Some(now);
        Ok(now)
    }
}

impl super::Session<super::LinuxTerminal> {
    pub(crate) fn runtime<'a>(
        &'a mut self,
        clock: &'a mut super::LinuxClock,
    ) -> Runtime<super::LinuxRuntime<'a>> {
        Runtime::new(super::LinuxRuntime::new(self.effects_mut(), clock))
    }
}

#[cfg(test)]
pub(crate) mod scripted;
#[cfg(test)]
mod tests;
