// SPDX-License-Identifier: GPL-2.0-only

use std::io;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use super::{LinuxTerminal, cvt};
use crate::terminal::runtime::{Clock, Ready, RuntimeEffects};
use crate::terminal::session::{Notifications, SessionOps};

/// Owns an absolute CLOCK_BOOTTIME timerfd and validates clock readings. Created
/// before discovery, borrowed during runtime, dropped normally on every exit.
pub(crate) struct LinuxClock {
    timer: OwnedFd,
    previous: Option<u64>,
}

impl LinuxClock {
    pub(crate) fn new() -> io::Result<Self> {
        // SAFETY: valid Linux clock id/flags; successful result is a fresh FD.
        let fd = cvt(unsafe {
            libc::timerfd_create(libc::CLOCK_BOOTTIME, libc::TFD_NONBLOCK | libc::TFD_CLOEXEC)
        })?;
        let mut clock = Self {
            timer: unsafe { OwnedFd::from_raw_fd(fd) },
            previous: None,
        };
        clock.now()?;
        Ok(clock)
    }

    fn arm(&mut self, deadline: u64) -> io::Result<()> {
        // timerfd interprets zero as disarm, so an already-due zero deadline
        // becomes the first positive absolute nanosecond instead.
        let timer = libc::itimerspec {
            it_interval: nanoseconds_to_timespec(0)?,
            it_value: nanoseconds_to_timespec(deadline.max(1))?,
        };
        // SAFETY: owned timer FD and initialized itimerspec; no old-value output.
        cvt(unsafe {
            libc::timerfd_settime(
                self.timer.as_raw_fd(),
                libc::TFD_TIMER_ABSTIME,
                &timer,
                std::ptr::null_mut(),
            )
        })?;
        Ok(())
    }

    fn consume(&mut self) -> io::Result<()> {
        let mut count = 0_u64;
        // SAFETY: owned nonblocking timerfd and writable eight-byte counter.
        let read = unsafe {
            libc::read(
                self.timer.as_raw_fd(),
                (&mut count as *mut u64).cast(),
                size_of::<u64>(),
            )
        };
        if read == size_of::<u64>() as isize {
            // Counts are wake information, never a requested number of samples.
            return Ok(());
        }
        if read < 0 {
            let error = io::Error::last_os_error();
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) {
                return Ok(());
            }
            return Err(error);
        }
        Err(io::Error::other(
            "timerfd returned an incomplete expiration count",
        ))
    }
}

impl Clock for LinuxClock {
    fn now(&mut self) -> io::Result<u64> {
        let mut time = MaybeUninit::<libc::timespec>::uninit();
        // SAFETY: valid clock id and writable local timespec, read on success.
        cvt(unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, time.as_mut_ptr()) })?;
        let time = unsafe { time.assume_init() };
        let now = timespec_to_nanoseconds(time)?;
        if self
            .previous
            .is_some_and(|previous| now.checked_sub(previous).is_none())
        {
            return Err(io::Error::other("BOOTTIME clock moved backwards"));
        }
        self.previous = Some(now);
        Ok(now)
    }
}

fn timespec_to_nanoseconds(time: libc::timespec) -> io::Result<u64> {
    let seconds =
        u64::try_from(time.tv_sec).map_err(|_| io::Error::other("negative BOOTTIME seconds"))?;
    let nanos = u64::try_from(time.tv_nsec)
        .map_err(|_| io::Error::other("negative BOOTTIME nanoseconds"))?;
    if nanos >= 1_000_000_000 {
        return Err(io::Error::other("invalid BOOTTIME nanosecond field"));
    }
    seconds
        .checked_mul(1_000_000_000)
        .and_then(|value| value.checked_add(nanos))
        .ok_or_else(|| io::Error::other("BOOTTIME nanosecond conversion overflow"))
}

fn nanoseconds_to_timespec(nanos: u64) -> io::Result<libc::timespec> {
    Ok(libc::timespec {
        tv_sec: (nanos / 1_000_000_000)
            .try_into()
            .map_err(|_| io::Error::other("timer seconds conversion overflow"))?,
        tv_nsec: (nanos % 1_000_000_000)
            .try_into()
            .map_err(|_| io::Error::other("timer nanoseconds conversion overflow"))?,
    })
}

/// Borrows session-owned FDs rather than extending their lifetimes or taking
/// restoration ownership. The application never receives raw descriptors.
pub(crate) struct LinuxRuntime<'a> {
    terminal: &'a mut LinuxTerminal,
    clock: &'a mut LinuxClock,
}

impl<'a> LinuxRuntime<'a> {
    pub(crate) fn new(terminal: &'a mut LinuxTerminal, clock: &'a mut LinuxClock) -> Self {
        Self { terminal, clock }
    }
}

impl Clock for LinuxRuntime<'_> {
    fn now(&mut self) -> io::Result<u64> {
        self.clock.now()
    }
}

impl RuntimeEffects for LinuxRuntime<'_> {
    fn wait(
        &mut self,
        sample_deadline: u64,
        idle_ns: Option<u64>,
        output: bool,
    ) -> io::Result<Ready> {
        self.clock.arm(sample_deadline)?;
        let mut fds = [
            pollfd(self.terminal.input_fd()?.as_raw_fd(), libc::POLLIN),
            pollfd(self.terminal.signal_fd()?.as_raw_fd(), libc::POLLIN),
            pollfd(self.clock.timer.as_raw_fd(), libc::POLLIN),
            pollfd(
                self.terminal.output_fd()?.as_raw_fd(),
                if output { libc::POLLOUT } else { 0 },
            ),
        ];
        let ready = poll(&mut fds, idle_ns)?;
        if ready.timer {
            self.clock.consume()?;
        }
        Ok(ready)
    }

    fn notifications(&mut self) -> io::Result<Notifications> {
        self.terminal.notifications()
    }

    fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        self.terminal.dimensions()
    }

    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        // SAFETY: owned live input FD and writable slice for the syscall.
        let read = unsafe {
            libc::read(
                self.terminal.input_fd()?.as_raw_fd(),
                bytes.as_mut_ptr().cast(),
                bytes.len(),
            )
        };
        if read < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(read as usize)
        }
    }

    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.terminal.write(bytes)
    }
}

fn pollfd(fd: i32, events: i16) -> libc::pollfd {
    libc::pollfd {
        fd,
        events,
        revents: 0,
    }
}

fn poll(fds: &mut [libc::pollfd; 4], idle_ns: Option<u64>) -> io::Result<Ready> {
    // Ceil sub-ms deadlines, cap huge waits (a harmless earlier wake). No clock
    // fallback: callers inspect BOOTTIME again after every wake/timeout/EINTR.
    let milliseconds = idle_ns.map_or(-1, |nanos| {
        nanos.div_ceil(1_000_000).min(i32::MAX as u64) as i32
    });
    // SAFETY: four initialized pollfds stay live for the synchronous syscall.
    let result = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, milliseconds) };
    if result < 0 {
        let error = io::Error::last_os_error();
        return if error.kind() == io::ErrorKind::Interrupted {
            Ok(Ready::default())
        } else {
            Err(error)
        };
    }
    for (fd, name) in fds
        .iter()
        .zip(["input", "signal pipe", "timerfd", "output"])
    {
        if fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
            return Err(io::Error::other(format!(
                "{name} poll error/hangup/invalid descriptor"
            )));
        }
    }
    Ok(Ready {
        input: fds[0].revents & libc::POLLIN != 0,
        signals: fds[1].revents & libc::POLLIN != 0,
        timer: fds[2].revents & libc::POLLIN != 0,
        output: fds[3].revents & libc::POLLOUT != 0,
    })
}

#[cfg(test)]
mod tests;
