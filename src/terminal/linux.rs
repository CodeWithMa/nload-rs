// SPDX-License-Identifier: GPL-2.0-only

use std::io;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::session::{Notifications, Saved, SessionOps, Shutdown, Support};

/// Concrete Linux effect adapter. Duplication owns descriptor lifetimes, but
/// file-status flags still share the original open-file descriptions and are
/// therefore explicitly restored by Session.
#[derive(Default)]
pub(crate) struct LinuxTerminal {
    input: Option<OwnedFd>,
    output: Option<OwnedFd>,
    signals: Option<SignalPipe>,
}

impl LinuxTerminal {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(super) fn input_fd(&self) -> io::Result<BorrowedFd<'_>> {
        use std::os::fd::AsFd;
        self.input
            .as_ref()
            .map(|fd| fd.as_fd())
            .ok_or_else(not_initialized)
    }

    pub(super) fn output_fd(&self) -> io::Result<BorrowedFd<'_>> {
        use std::os::fd::AsFd;
        self.output
            .as_ref()
            .map(|fd| fd.as_fd())
            .ok_or_else(not_initialized)
    }

    pub(super) fn signal_fd(&self) -> io::Result<BorrowedFd<'_>> {
        use std::os::fd::AsFd;
        self.signals
            .as_ref()
            .map(|signals| signals.read.as_fd())
            .ok_or_else(not_initialized)
    }
}

impl SessionOps for LinuxTerminal {
    type Termios = libc::termios;

    fn support(&mut self) -> Support {
        // SAFETY: isatty accepts arbitrary descriptors; these are standard FDs.
        Support {
            input_terminal: unsafe { libc::isatty(libc::STDIN_FILENO) == 1 },
            output_terminal: unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 },
            term: std::env::var_os("TERM"),
        }
    }

    fn snapshot(&mut self) -> io::Result<Saved<Self::Termios>> {
        self.input = Some(duplicate(libc::STDIN_FILENO)?);
        self.output = Some(duplicate(libc::STDOUT_FILENO)?);
        let mut termios = MaybeUninit::uninit();
        // SAFETY: output points to writable termios storage; initialized only
        // on successful tcgetattr. The owned input FD stays alive throughout.
        cvt(unsafe { libc::tcgetattr(self.input_fd()?.as_raw_fd(), termios.as_mut_ptr()) })?;
        Ok(Saved {
            termios: unsafe { termios.assume_init() },
            input_flags: get_flags(self.input_fd()?.as_raw_fd())?,
            output_flags: get_flags(self.output_fd()?.as_raw_fd())?,
        })
    }

    fn prepare_notifications(&mut self) -> io::Result<()> {
        // Store the owner before registering any actions. A later registration
        // failure leaves every earlier registration available for cleanup.
        self.signals = Some(SignalPipe::new()?);
        self.signals
            .as_mut()
            .ok_or_else(not_initialized)?
            .register()
    }

    fn raw(&mut self, saved: &Self::Termios) -> io::Result<()> {
        let mut raw = *saved;
        // SAFETY: initialized local termios, then a valid owned terminal FD.
        unsafe { libc::cfmakeraw(&mut raw) };
        // Nonblocking input: no minimum byte count/timeout wait inside read.
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        cvt(unsafe { libc::tcsetattr(self.input_fd()?.as_raw_fd(), libc::TCSANOW, &raw) })?;
        Ok(())
    }

    fn input_flags(&mut self, flags: i32) -> io::Result<()> {
        set_flags(self.input_fd()?.as_raw_fd(), flags)
    }

    fn output_flags(&mut self, flags: i32) -> io::Result<()> {
        set_flags(self.output_fd()?.as_raw_fd(), flags)
    }

    fn restore_termios(&mut self, saved: &Self::Termios) -> io::Result<()> {
        // SAFETY: owned input terminal and initialized saved termios. TCSANOW
        // deliberately avoids an unbounded drain of backpressured output.
        cvt(unsafe { libc::tcsetattr(self.input_fd()?.as_raw_fd(), libc::TCSANOW, saved) })?;
        Ok(())
    }

    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // SAFETY: borrowed slice is live for the syscall; FD is owned.
        let written = unsafe {
            libc::write(
                self.output_fd()?.as_raw_fd(),
                bytes.as_ptr().cast(),
                bytes.len(),
            )
        };
        if written < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(written as usize)
        }
    }

    fn wait_writable(&mut self, timeout: Duration) -> io::Result<()> {
        let mut fd = libc::pollfd {
            fd: self.output_fd()?.as_raw_fd(),
            events: libc::POLLOUT,
            revents: 0,
        };
        let milliseconds = timeout
            .as_millis()
            .saturating_add(u128::from(timeout.subsec_nanos() % 1_000_000 != 0));
        let milliseconds = milliseconds.min(i32::MAX as u128) as i32;
        // SAFETY: one initialized pollfd and finite timeout.
        let result = cvt(unsafe { libc::poll(&mut fd, 1, milliseconds) })?;
        if result == 0 {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "terminal restoration output timed out",
            ))
        } else if fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
            Err(io::Error::other(
                "terminal output unavailable during restoration",
            ))
        } else {
            Ok(())
        }
    }

    fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        let mut size = MaybeUninit::<libc::winsize>::uninit();
        // SAFETY: ioctl writes a winsize on success to valid local storage.
        cvt(unsafe {
            libc::ioctl(
                self.output_fd()?.as_raw_fd(),
                libc::TIOCGWINSZ,
                size.as_mut_ptr(),
            )
        })?;
        let size = unsafe { size.assume_init() };
        Ok((usize::from(size.ws_col), usize::from(size.ws_row)))
    }

    fn notifications(&mut self) -> io::Result<Notifications> {
        self.signals.as_mut().ok_or_else(not_initialized)?.inspect()
    }

    fn release_notifications(&mut self) {
        // Drop unregisters writers/flag actions before closing the reader.
        self.signals = None;
    }
}

struct SignalPipe {
    read: OwnedFd,
    write: OwnedFd,
    flags: SignalFlags,
    registrations: Vec<signal_hook::SigId>,
}

#[derive(Default)]
struct SignalFlags {
    interrupt: Arc<AtomicBool>,
    terminate: Arc<AtomicBool>,
    resize: Arc<AtomicBool>,
}

impl SignalFlags {
    fn inspect(&self) -> Notifications {
        // Sticky termination flags preserve arbitration across repeated checks.
        // Read INT last so it wins over a TERM already observed in this check.
        let terminate = self.terminate.load(Ordering::SeqCst);
        let interrupt = self.interrupt.load(Ordering::SeqCst);
        Notifications {
            shutdown: if interrupt {
                Some(Shutdown::Interrupt)
            } else if terminate {
                Some(Shutdown::Terminate)
            } else {
                None
            },
            resized: self.resize.swap(false, Ordering::SeqCst),
        }
    }
}

impl SignalPipe {
    fn new() -> io::Result<Self> {
        let mut fds = [-1; 2];
        // SAFETY: pipe2 initializes exactly two FDs on success. Each is wrapped
        // exactly once; both ends are nonblocking and close-on-exec from birth.
        cvt(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC) })?;
        Ok(Self {
            read: unsafe { OwnedFd::from_raw_fd(fds[0]) },
            write: unsafe { OwnedFd::from_raw_fd(fds[1]) },
            flags: SignalFlags::default(),
            registrations: Vec::with_capacity(6),
        })
    }

    fn register(&mut self) -> io::Result<()> {
        for (signal, flag) in [
            (libc::SIGINT, &self.flags.interrupt),
            (libc::SIGTERM, &self.flags.terminate),
            (libc::SIGWINCH, &self.flags.resize),
        ] {
            // signal-hook executes actions in registration order: flag first,
            // wake second. Handlers perform only atomics/nonblocking writes.
            let id = signal_hook::flag::register(signal, Arc::clone(flag))?;
            self.registrations.push(id);
            let writer = duplicate(self.write.as_raw_fd())?;
            let id = signal_hook::low_level::pipe::register(signal, writer)?;
            self.registrations.push(id);
        }
        Ok(())
    }

    fn inspect(&mut self) -> io::Result<Notifications> {
        drain(self.read.as_raw_fd())?;
        Ok(self.flags.inspect())
    }
}

impl Drop for SignalPipe {
    fn drop(&mut self) {
        for id in self.registrations.drain(..).rev() {
            signal_hook::low_level::unregister(id);
        }
    }
}

fn drain(fd: RawFd) -> io::Result<()> {
    let mut buffer = [0_u8; 256];
    loop {
        // SAFETY: writable buffer and live owned pipe reader supplied by owner.
        let read = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
        if read > 0 {
            continue;
        }
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "signal pipe closed",
            ));
        }
        let error = io::Error::last_os_error();
        match error.kind() {
            io::ErrorKind::Interrupted => continue,
            io::ErrorKind::WouldBlock => return Ok(()),
            _ => return Err(error),
        }
    }
}

fn duplicate(fd: RawFd) -> io::Result<OwnedFd> {
    // SAFETY: fcntl accepts an arbitrary FD and either returns a fresh owned FD
    // or fails without transferring ownership. F_DUPFD_CLOEXEC is atomic.
    let duplicate = cvt(unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 3) })?;
    Ok(unsafe { OwnedFd::from_raw_fd(duplicate) })
}

fn get_flags(fd: RawFd) -> io::Result<i32> {
    // SAFETY: fcntl accepts arbitrary FDs; no pointer argument for F_GETFL.
    cvt(unsafe { libc::fcntl(fd, libc::F_GETFL) })
}

fn set_flags(fd: RawFd, flags: i32) -> io::Result<()> {
    // SAFETY: integer F_SETFL argument, live owned FD.
    cvt(unsafe { libc::fcntl(fd, libc::F_SETFL, flags) })?;
    Ok(())
}

fn cvt(result: i32) -> io::Result<i32> {
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result)
    }
}

fn not_initialized() -> io::Error {
    io::Error::other("terminal effects used before initialization")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_signals_wake_and_unregister_without_owned_fd_leaks() {
        const CHILD: &str = "NLOAD_RS_SIGNAL_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            // Isolate process-global signal registrations from the parallel Rust
            // test harness. These are test-harness flags, never product flags.
            let result = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "terminal::linux::tests::registered_signals_wake_and_unregister_without_owned_fd_leaks", "--test-threads=1"])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "child stdout: {}\nchild stderr: {}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            return;
        }
        let fd_count = || std::fs::read_dir("/proc/self/fd").unwrap().count();
        let before = fd_count();
        let mut terminal = LinuxTerminal::new();
        terminal.prepare_notifications().unwrap();
        let read_fd = terminal.signal_fd().unwrap().as_raw_fd();
        let resize = Arc::clone(&terminal.signals.as_ref().unwrap().flags.resize);
        signal_hook::low_level::raise(libc::SIGWINCH).unwrap();
        signal_hook::low_level::raise(libc::SIGTERM).unwrap();
        signal_hook::low_level::raise(libc::SIGINT).unwrap();
        let mut ready = libc::pollfd {
            fd: read_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: initialized pollfd, finite zero timeout; handlers have returned.
        assert_eq!(unsafe { libc::poll(&mut ready, 1, 0) }, 1);
        assert_ne!(ready.revents & libc::POLLIN, 0);
        assert_eq!(
            terminal.notifications().unwrap(),
            Notifications {
                shutdown: Some(Shutdown::Interrupt),
                resized: true
            }
        );
        assert_eq!(
            terminal.notifications().unwrap(),
            Notifications {
                shutdown: Some(Shutdown::Interrupt),
                resized: false
            }
        );
        terminal.release_notifications();
        // SAFETY: F_GETFD accepts arbitrary FD numbers; no syscall opened another
        // descriptor between close and this check, so this number is not reused.
        assert_eq!(unsafe { libc::fcntl(read_fd, libc::F_GETFD) }, -1);
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::EBADF));
        assert_eq!(Arc::strong_count(&resize), 1);
        signal_hook::low_level::raise(libc::SIGWINCH).unwrap();
        assert!(!resize.load(Ordering::SeqCst));
        assert_eq!(fd_count(), before);
    }

    #[test]
    fn full_pipe_and_collated_tokens_do_not_replace_atomic_signal_identity() {
        let mut pipe = SignalPipe::new().unwrap();
        let tokens = [b'X'; 4096];
        loop {
            // SAFETY: owned nonblocking pipe, live slice; no real signals used.
            let written = unsafe {
                libc::write(pipe.write.as_raw_fd(), tokens.as_ptr().cast(), tokens.len())
            };
            if written < 0 {
                assert_eq!(io::Error::last_os_error().kind(), io::ErrorKind::WouldBlock);
                break;
            }
        }
        pipe.flags.terminate.store(true, Ordering::SeqCst);
        pipe.flags.interrupt.store(true, Ordering::SeqCst);
        pipe.flags.resize.store(true, Ordering::SeqCst);
        assert_eq!(
            pipe.inspect().unwrap(),
            Notifications {
                shutdown: Some(Shutdown::Interrupt),
                resized: true
            }
        );
        let mut byte = 0_u8;
        // SAFETY: valid owned nonblocking reader and writable one-byte buffer.
        assert_eq!(
            unsafe { libc::read(pipe.read.as_raw_fd(), (&mut byte as *mut u8).cast(), 1) },
            -1
        );
        assert_eq!(io::Error::last_os_error().kind(), io::ErrorKind::WouldBlock);
        // Sticky shutdown and one-shot resize are independent of wake tokens.
        assert_eq!(
            pipe.inspect().unwrap(),
            Notifications {
                shutdown: Some(Shutdown::Interrupt),
                resized: false
            }
        );
        pipe.flags.interrupt.store(false, Ordering::SeqCst);
        assert_eq!(pipe.inspect().unwrap().shutdown, Some(Shutdown::Terminate));
    }

    #[test]
    fn linux_adapter_raw_flags_and_dimensions_round_trip_on_owned_pty() {
        let (mut master, mut slave) = (-1, -1);
        let size = libc::winsize {
            ws_col: 80,
            ws_row: 24,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: openpty writes two fresh FDs; optional name/termios are null,
        // dimensions point to initialized storage. Wrap each FD exactly once.
        cvt(unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                &size,
            )
        })
        .unwrap();
        let _master = unsafe { OwnedFd::from_raw_fd(master) };
        let slave = unsafe { OwnedFd::from_raw_fd(slave) };
        let mut terminal = LinuxTerminal {
            input: Some(duplicate(slave.as_raw_fd()).unwrap()),
            output: Some(duplicate(slave.as_raw_fd()).unwrap()),
            signals: None,
        };
        let read_mode = || {
            let mut mode = MaybeUninit::<libc::termios>::uninit();
            // SAFETY: owned live PTY and writable local termios storage.
            cvt(unsafe { libc::tcgetattr(slave.as_raw_fd(), mode.as_mut_ptr()) }).unwrap();
            unsafe { mode.assume_init() }
        };
        let saved = read_mode();
        let flags = get_flags(slave.as_raw_fd()).unwrap();
        terminal.raw(&saved).unwrap();
        terminal.input_flags(flags | libc::O_NONBLOCK).unwrap();
        terminal.output_flags(flags | libc::O_NONBLOCK).unwrap();
        let raw = read_mode();
        assert_eq!(
            raw.c_lflag & (libc::ISIG | libc::ICANON | libc::ECHO | libc::IEXTEN),
            0
        );
        assert_eq!(
            raw.c_iflag & (libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON),
            0
        );
        assert_eq!(raw.c_oflag & libc::OPOST, 0);
        assert_eq!(raw.c_cflag & libc::CSIZE, libc::CS8);
        assert_eq!(raw.c_cflag & libc::PARENB, 0);
        assert_eq!(raw.c_cc[libc::VMIN], 1);
        assert_eq!(raw.c_cc[libc::VTIME], 0);
        assert_eq!(
            get_flags(slave.as_raw_fd()).unwrap(),
            flags | libc::O_NONBLOCK
        );
        assert_eq!(terminal.dimensions().unwrap(), (80, 24));
        let small = libc::winsize {
            ws_col: 1,
            ws_row: 1,
            ..size
        };
        // SAFETY: live slave and initialized winsize for TIOCSWINSZ.
        cvt(unsafe { libc::ioctl(slave.as_raw_fd(), libc::TIOCSWINSZ, &small) }).unwrap();
        assert_eq!(terminal.dimensions().unwrap(), (1, 1));
        terminal.restore_termios(&saved).unwrap();
        terminal.input_flags(flags).unwrap();
        terminal.output_flags(flags).unwrap();
        let restored = read_mode();
        assert_eq!(restored.c_iflag, saved.c_iflag);
        assert_eq!(restored.c_oflag, saved.c_oflag);
        assert_eq!(restored.c_cflag, saved.c_cflag);
        assert_eq!(restored.c_lflag, saved.c_lflag);
        assert_eq!(restored.c_line, saved.c_line);
        assert_eq!(restored.c_cc, saved.c_cc);
        assert_eq!(restored.c_ispeed, saved.c_ispeed);
        assert_eq!(restored.c_ospeed, saved.c_ospeed);
        assert_eq!(get_flags(slave.as_raw_fd()).unwrap(), flags);
    }
}
