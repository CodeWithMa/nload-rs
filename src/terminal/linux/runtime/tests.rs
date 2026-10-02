// SPDX-License-Identifier: GPL-2.0-only

use super::*;
use crate::terminal::linux::{SignalPipe, duplicate};
use crate::terminal::{Event, Runtime, Shutdown};

fn pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [-1; 2];
    // SAFETY: writable two-FD array; wrap each successful owned FD once.
    cvt(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC) }).unwrap();
    unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) }
}

fn write(fd: &OwnedFd, bytes: &[u8]) -> io::Result<usize> {
    // SAFETY: owned live nonblocking FD and initialized slice.
    let result = unsafe { libc::write(fd.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result as usize)
    }
}

#[test]
fn checked_timespec_conversion_rejects_invalid_fields_and_overflow() {
    for time in [
        libc::timespec {
            tv_sec: -1,
            tv_nsec: 0,
        },
        libc::timespec {
            tv_sec: 0,
            tv_nsec: -1,
        },
        libc::timespec {
            tv_sec: 0,
            tv_nsec: 1_000_000_000,
        },
        libc::timespec {
            tv_sec: i64::MAX,
            tv_nsec: 0,
        },
        libc::timespec {
            tv_sec: (u64::MAX / 1_000_000_000) as i64,
            tv_nsec: (u64::MAX % 1_000_000_000 + 1) as i64,
        },
    ] {
        assert!(timespec_to_nanoseconds(time).is_err());
    }
    for nanos in [
        0,
        1,
        999_999_999,
        1_000_000_000,
        (1_u64 << 53) + 1,
        u64::MAX,
    ] {
        assert_eq!(
            timespec_to_nanoseconds(nanoseconds_to_timespec(nanos).unwrap()).unwrap(),
            nanos
        );
    }
}

#[test]
fn owned_boottime_timer_is_absolute_nonblocking_and_consumes_due_zero_deadline() {
    let mut clock = LinuxClock::new().unwrap();
    let before = clock.now().unwrap();
    assert!(clock.now().unwrap() >= before);
    // SAFETY: live owned timerfd, no pointer fcntl arguments.
    assert_ne!(
        unsafe { libc::fcntl(clock.timer.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
        0
    );
    assert_ne!(
        unsafe { libc::fcntl(clock.timer.as_raw_fd(), libc::F_GETFL) } & libc::O_NONBLOCK,
        0
    );
    clock.arm(0).unwrap();
    let timer_fd = clock.timer.as_raw_fd();
    let mut fds = [
        pollfd(-1, 0),
        pollfd(-1, 0),
        pollfd(timer_fd, libc::POLLIN),
        pollfd(-1, 0),
    ];
    assert!(poll(&mut fds, Some(1_000_000_000)).unwrap().timer);
    clock.consume().unwrap();
    assert!(!poll(&mut fds, Some(0)).unwrap().timer);
    let future = clock.now().unwrap().checked_add(60_000_000_000).unwrap();
    clock.arm(future).unwrap();
    let mut value = MaybeUninit::<libc::itimerspec>::uninit();
    // SAFETY: owned timer and writable output structure initialized on success.
    cvt(unsafe { libc::timerfd_gettime(timer_fd, value.as_mut_ptr()) }).unwrap();
    let value = unsafe { value.assume_init() };
    assert_eq!(
        timespec_to_nanoseconds(value.it_interval).unwrap(),
        0,
        "one shot, no catch-up timer"
    );
    assert!(timespec_to_nanoseconds(value.it_value).unwrap() > 0);
    assert!(!poll(&mut fds, Some(0)).unwrap().timer);
}

#[test]
fn poll_detects_each_owned_fd_hangup_error_and_invalid_descriptor() {
    for index in 0..4 {
        let (reader, writer) = pipe();
        drop(writer);
        let mut fds = [pollfd(-1, 0); 4];
        fds[index] = pollfd(reader.as_raw_fd(), libc::POLLIN);
        assert!(poll(&mut fds, Some(0)).is_err(), "hangup index {index}");
        // Use a number that is not open, without closing a live OwnedFd.
        fds[index] = pollfd(i32::MAX, libc::POLLIN);
        assert!(poll(&mut fds, Some(0)).is_err(), "invalid index {index}");
    }
    let (reader, writer) = pipe();
    drop(reader);
    let mut fds = [pollfd(-1, 0); 4];
    fds[3] = pollfd(writer.as_raw_fd(), libc::POLLOUT);
    assert!(poll(&mut fds, Some(0)).is_err(), "broken output poll");
}

#[test]
fn timer_effect_errors_and_incomplete_reads_are_fatal() {
    let mut clock = LinuxClock::new().unwrap();
    let (reader, writer) = pipe();
    // Replacing ownership closes the real timer without dangling/double-close.
    clock.timer = reader;
    assert!(clock.arm(1).is_err(), "pipe is not a timerfd");
    clock.consume().unwrap(); // EAGAIN is only a spurious wake.
    write(&writer, b"short").unwrap();
    assert!(clock.consume().is_err());
    drop(writer);
    assert!(clock.consume().is_err()); // EOF, not unavailable counters.
}

#[test]
fn linux_runtime_reports_ready_output_transient_input_and_actual_eof() {
    let (input, input_writer) = pipe();
    let (_output_reader, output) = pipe();
    let mut terminal = LinuxTerminal {
        input: Some(input),
        output: Some(output),
        signals: Some(SignalPipe::new().unwrap()),
    };
    let mut clock = LinuxClock::new().unwrap();
    let deadline = clock.now().unwrap().checked_add(60_000_000_000).unwrap();
    let mut effects = LinuxRuntime::new(&mut terminal, &mut clock);
    assert_eq!(
        effects.wait(deadline, Some(0), false).unwrap(),
        Ready::default()
    );
    assert_eq!(
        effects.wait(deadline, Some(0), true).unwrap(),
        Ready {
            output: true,
            ..Ready::default()
        }
    );
    let mut bytes = [0; 4];
    assert_eq!(
        effects.read(&mut bytes).unwrap_err().kind(),
        io::ErrorKind::WouldBlock
    );
    write(&input_writer, b"\x03\x1aQ").unwrap();
    assert!(effects.wait(deadline, Some(0), false).unwrap().input);
    assert_eq!(effects.read(&mut bytes).unwrap(), 3);
    assert_eq!(&bytes[..3], b"\x03\x1aQ");
    drop(input_writer);
    assert_eq!(effects.read(&mut bytes).unwrap(), 0);
    assert!(effects.wait(deadline, Some(0), false).is_err());
}

#[test]
fn real_nonblocking_backpressure_does_not_hide_input_signal_or_timer_readiness() {
    let (input, input_writer) = pipe();
    let (output_reader, output) = pipe();
    let signals = SignalPipe::new().unwrap();
    let signal_writer = duplicate(signals.write.as_raw_fd()).unwrap();
    let flags = signals.flags.interrupt.clone();
    let mut terminal = LinuxTerminal {
        input: Some(input),
        output: Some(output),
        signals: Some(signals),
    };
    let mut clock = LinuxClock::new().unwrap();
    let future = clock.now().unwrap().checked_add(60_000_000_000).unwrap();
    // Fill the real output pipe to capacity before a frame is submitted.
    let output = terminal.output.as_ref().unwrap();
    while write(output, &[b'x'; 4096]).is_ok() {}
    write(&input_writer, b"\x1b[").unwrap();
    write(&signal_writer, b"not signal identities").unwrap();
    flags.store(true, std::sync::atomic::Ordering::SeqCst);
    let mut runtime = Runtime::new(LinuxRuntime::new(&mut terminal, &mut clock));
    runtime.queue_frame(&["latest".into()], 6, 1).unwrap();
    runtime.output_batch().unwrap(); // Actual EAGAIN, no wait.
    let ready = runtime.wait(future).unwrap();
    assert_eq!(
        ready,
        Ready {
            input: true,
            signals: true,
            timer: false,
            output: false
        }
    );
    assert!(runtime.input_batch(ready.input).unwrap().is_empty());
    write(&input_writer, b"C").unwrap();
    assert_eq!(runtime.input_batch(true).unwrap(), [Event::Forward]);
    assert_eq!(
        runtime.notifications().unwrap().shutdown,
        Some(Shutdown::Interrupt)
    );
    assert!(runtime.has_pending_output());
    assert!(runtime.now().is_ok());
    // A due timer still wakes while output is full; expiration count is consumed.
    assert!(runtime.wait(0).unwrap().timer);
    drop(runtime);
    drop(output_reader);
}
