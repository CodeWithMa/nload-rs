// SPDX-License-Identifier: GPL-2.0-only

use std::io;

use super::Shutdown;
use super::runtime::Event;

const IDLE_NS: u64 = 100_000_000;
const SEQUENCE_LIMIT: usize = 64;

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Plain,
    Escape,
    Control,
    Other,
    // OSC/DCS/SOS/PM/APC payloads end at ST (OSC also BEL), not at letters.
    String {
        osc: bool,
        escape: bool,
    },
}

/// No dynamic sequence buffer: only state, a capped length and validity bit.
/// Overlong sequences stay in discard mode until their terminator/idle timeout.
#[derive(Default)]
pub(super) struct Decoder {
    state: State,
    length: usize,
    unmodified: bool,
    deadline: Option<u64>,
}

impl Decoder {
    pub(super) fn deadline(&self) -> Option<u64> {
        self.deadline
    }

    pub(super) fn expire(&mut self, now: u64) {
        if self.deadline.is_some_and(|deadline| now >= deadline) {
            self.reset();
        }
    }

    pub(super) fn feed(&mut self, bytes: &[u8], now: u64) -> io::Result<Vec<Event>> {
        self.expire(now);
        let mut events = Vec::new();
        for &byte in bytes {
            // Raw-mode Ctrl+C is an explicit shutdown control even while an
            // incomplete escape sequence is pending; it never waits for timeout.
            if byte == 3 {
                self.reset();
                events.push(Event::Shutdown(Shutdown::Interrupt));
                continue;
            }
            if matches!(self.state, State::Plain) {
                match byte {
                    b'q' | b'Q' => events.push(Event::Shutdown(Shutdown::Quit)),
                    0x1b => {
                        self.state = State::Escape;
                        self.length = 1;
                        self.unmodified = true;
                    }
                    _ => {}
                }
            } else {
                self.length = self.length.saturating_add(1).min(SEQUENCE_LIMIT + 1);
                if self.length > SEQUENCE_LIMIT {
                    self.unmodified = false;
                }
                match self.state {
                    State::Escape => match byte {
                        b'[' | b'O' => self.state = State::Control,
                        b']' | b'P' | b'X' | b'^' | b'_' => {
                            self.state = State::String {
                                osc: byte == b']',
                                escape: false,
                            };
                        }
                        0x20..=0x2f => self.state = State::Other,
                        // ESC + a final (including command-looking q/Q) is consumed.
                        _ => self.reset(),
                    },
                    State::Control => {
                        if (0x40..=0x7e).contains(&byte) {
                            if self.unmodified && self.length == 3 {
                                match byte {
                                    b'A' | b'D' => events.push(Event::Backward),
                                    b'B' | b'C' => events.push(Event::Forward),
                                    _ => {}
                                }
                            }
                            self.reset();
                        } else {
                            // Both CSI and SS3 accept parameters/intermediates
                            // for consumption, but only the bare forms navigate.
                            self.unmodified = false;
                        }
                    }
                    State::Other => {
                        if !(0x20..=0x2f).contains(&byte) {
                            self.reset();
                        }
                    }
                    State::String { osc, escape } => {
                        if (osc && byte == 7) || (escape && byte == b'\\') {
                            self.reset();
                        } else {
                            self.state = State::String {
                                osc,
                                escape: byte == 0x1b,
                            };
                        }
                    }
                    State::Plain => unreachable!(),
                }
            }
            self.deadline = if matches!(self.state, State::Plain) {
                None
            } else {
                Some(
                    now.checked_add(IDLE_NS)
                        .ok_or_else(|| io::Error::other("input idle deadline overflow"))?,
                )
            };
        }
        Ok(events)
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}
