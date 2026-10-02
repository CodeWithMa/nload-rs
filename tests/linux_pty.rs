// SPDX-License-Identifier: GPL-2.0-only
#![cfg(target_os = "linux")]

//! Product tests, run by `cargo test --locked`. No product instrumentation.
//! NLOAD_RS_PTY_BINARY may select a separately built release binary for this
//! harness; otherwise Cargo supplies the real debug executable.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

const LIMIT: Duration = Duration::from_secs(5);
const START: &[u8] = b"\x1b[H\x1b[2J";
const END: &[u8] = b"\x1b[H";
const ENTER: &[u8] = b"\x1b[?1049h\x1b[?25l";
const RESTORE: &[u8] = b"\x18\x1b[?25h\x18\x1b[?1049l";

fn checked(result: i32) -> i32 {
    assert!(result >= 0, "{}", io::Error::last_os_error());
    result
}

fn flags(file: &File) -> i32 {
    // SAFETY: live descriptor; F_GETFL requires no third argument.
    checked(unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) })
}

fn set_flags(file: &File, value: i32) {
    // SAFETY: live descriptor and integer F_SETFL argument.
    checked(unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, value) });
}

fn mode(file: &File) -> libc::termios {
    let mut value = MaybeUninit::uninit();
    // SAFETY: writable storage; initialized only on success.
    checked(unsafe { libc::tcgetattr(file.as_raw_fd(), value.as_mut_ptr()) });
    unsafe { value.assume_init() }
}

fn same_mode(actual: libc::termios, expected: libc::termios) {
    // Compare defined fields, not struct padding.
    assert_eq!(actual.c_iflag, expected.c_iflag);
    assert_eq!(actual.c_oflag, expected.c_oflag);
    assert_eq!(actual.c_cflag, expected.c_cflag);
    assert_eq!(actual.c_lflag, expected.c_lflag);
    assert_eq!(actual.c_line, expected.c_line);
    assert_eq!(actual.c_cc, expected.c_cc);
    assert_eq!(actual.c_ispeed, expected.c_ispeed);
    assert_eq!(actual.c_ospeed, expected.c_ospeed);
}

struct Pty {
    master: Option<File>,
    slave: File,
    saved_mode: libc::termios,
    saved_flags: i32,
}

impl Pty {
    fn new(width: u16, height: u16) -> Self {
        let (mut master, mut slave) = (-1, -1);
        let size = libc::winsize {
            ws_col: width,
            ws_row: height,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: writable FD outputs and initialized size; wrap each new FD once.
        checked(unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                &size,
            )
        });
        let master = unsafe { File::from_raw_fd(master) };
        let slave = unsafe { File::from_raw_fd(slave) };
        // Do not leak retained observers through exec. Stdio duplicates only the
        // explicitly selected descriptions into the child.
        for fd in [&master, &slave] {
            // SAFETY: live descriptor, integer flags.
            checked(unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) });
        }
        set_flags(&master, flags(&master) | libc::O_NONBLOCK);
        let saved_mode = mode(&slave);
        let saved_flags = flags(&slave);
        Self {
            master: Some(master),
            slave,
            saved_mode,
            saved_flags,
        }
    }

    fn resize(&self, width: u16, height: u16) {
        let size = libc::winsize {
            ws_col: width,
            ws_row: height,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: live PTY and initialized winsize.
        checked(unsafe { libc::ioctl(self.slave.as_raw_fd(), libc::TIOCSWINSZ, &size) });
    }

    fn assert_restored(&self) {
        same_mode(mode(&self.slave), self.saved_mode);
        self.assert_flags_restored();
    }

    fn assert_flags_restored(&self) {
        // This observer shares the child's OFD and stays open after child exit.
        assert_eq!(flags(&self.slave), self.saved_flags);
    }

    fn read_only_slave(&self) -> File {
        // Open a new read-only description of the same terminal, not a duplicate
        // of the read/write description. isatty succeeds, but writes get EBADF.
        let path = std::fs::read_link(format!("/proc/self/fd/{}", self.slave.as_raw_fd())).unwrap();
        OpenOptions::new().read(true).open(path).unwrap()
    }
}

struct Product {
    child: Child,
    input: Pty,
    output: Pty,
    bytes: Vec<u8>,
    pending: Vec<u8>,
}

fn command() -> Command {
    let binary = std::env::var_os("NLOAD_RS_PTY_BINARY")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_nload-rs").into());
    let mut command = Command::new(binary);
    command.env("TERM", "xterm-256color");
    command
}

impl Product {
    fn spawn(width: u16, height: u16, setup: impl FnOnce(&mut Command, &Pty, &Pty)) -> Self {
        // Independent terminals and OFDs: input restoration cannot accidentally
        // stand in for output restoration, and either master can hang up alone.
        let input = Pty::new(width, height);
        let output = Pty::new(width, height);
        let mut command = command();
        command
            .stdin(input.slave.try_clone().unwrap())
            .stdout(output.slave.try_clone().unwrap())
            .stderr(Stdio::piped());
        setup(&mut command, &input, &output);
        Self {
            child: command.spawn().unwrap(),
            input,
            output,
            bytes: Vec::new(),
            pending: Vec::new(),
        }
    }

    fn start(width: u16, height: u16) -> Self {
        Self::spawn(width, height, |_, _, _| {})
    }

    fn signal(&self, signal: i32) {
        // SAFETY: child's PID remains owned until wait; no process-global hooks.
        checked(unsafe { libc::kill(self.child.id() as i32, signal) });
    }

    fn send(&mut self, bytes: &[u8]) {
        self.input
            .master
            .as_mut()
            .unwrap()
            .write_all(bytes)
            .unwrap();
    }

    fn collect(&mut self, timeout_ms: i32) {
        let Some(master) = self.output.master.as_mut() else {
            return;
        };
        let mut poll = libc::pollfd {
            fd: master.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one initialized pollfd, finite timeout.
        let result = unsafe { libc::poll(&mut poll, 1, timeout_ms) };
        if result < 0 && io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
            return;
        }
        checked(result);
        let mut buffer = [0; 16384];
        loop {
            match master.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    self.bytes.extend_from_slice(&buffer[..n]);
                    self.pending.extend_from_slice(&buffer[..n]);
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.raw_os_error() == Some(libc::EIO) => break,
                Err(error) => panic!("PTY read: {error}"),
            }
        }
    }

    fn frame(
        &mut self,
        width: usize,
        height: usize,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Vec<String> {
        let deadline = Instant::now() + LIMIT;
        loop {
            while let Some(start) = find(&self.pending, START) {
                let body = start + START.len();
                let Some(end) = find(&self.pending[body..], END).map(|offset| body + offset) else {
                    break;
                };
                let rows = decode_rows(&self.pending[body..end], width, height);
                self.pending.drain(..end + END.len());
                if predicate(&rows) {
                    return rows;
                }
            }
            assert!(
                Instant::now() < deadline,
                "frame timeout: {}",
                String::from_utf8_lossy(&self.bytes)
            );
            if let Some(status) = self.child.try_wait().unwrap() {
                panic!("product exited before expected frame: {status}");
            }
            self.collect(20);
        }
    }

    fn finish(&mut self, expected: i32, drain: bool) -> String {
        let deadline = Instant::now() + LIMIT;
        let status: ExitStatus = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "product exit timeout");
            if drain {
                self.collect(10);
            } else {
                std::thread::sleep(Duration::from_millis(5));
            }
        };
        if drain {
            self.collect(0);
        }
        let mut stderr = String::new();
        self.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        assert_eq!(status.code(), Some(expected), "stderr: {stderr}");
        stderr
    }

    fn assert_cleanup(&self) {
        self.input.assert_restored();
        self.output.assert_restored();
        assert!(self.bytes.starts_with(ENTER));
        assert!(self.bytes.ends_with(RESTORE));
    }
}

impl Drop for Product {
    fn drop(&mut self) {
        // Failure/timeouts cannot orphan a monitor or leave it stopped.
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Validate the emitted row protocol, not visual emulator behavior. In
/// particular, no LF/CR, unknown controls, row overflow or right-margin writes.
fn decode_rows(mut bytes: &[u8], width: usize, height: usize) -> Vec<String> {
    let mut rows = vec![String::new(); height];
    while !bytes.is_empty() {
        assert!(bytes.starts_with(b"\x1b["));
        let final_h = bytes.iter().position(|&byte| byte == b'H').unwrap();
        let address = std::str::from_utf8(&bytes[2..final_h]).unwrap();
        let (row, column) = address.split_once(';').unwrap();
        let row = row.parse::<usize>().unwrap();
        assert_eq!(column, "1");
        assert!((1..=height).contains(&row));
        bytes = &bytes[final_h + 1..];
        let length = bytes
            .iter()
            .position(|&byte| byte == 27)
            .unwrap_or(bytes.len());
        assert!(length <= width);
        assert!(
            bytes[..length]
                .iter()
                .all(|byte| (b' '..=b'~').contains(byte))
        );
        rows[row - 1] = std::str::from_utf8(&bytes[..length]).unwrap().to_owned();
        bytes = &bytes[length..];
    }
    rows
}

fn position(rows: &[String]) -> (usize, usize) {
    let header = rows
        .iter()
        .find(|row| row.starts_with("Interface "))
        .unwrap();
    let (_, suffix) = header.rsplit_once('(').unwrap();
    let (index, count) = suffix.trim_end_matches(')').split_once('/').unwrap();
    (index.parse().unwrap(), count.parse().unwrap())
}

fn assert_raw(product: &Product) {
    let raw = mode(&product.input.slave);
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
    for pty in [&product.input, &product.output] {
        assert_eq!(flags(&pty.slave), pty.saved_flags | libc::O_NONBLOCK);
    }
}

#[test]
fn normal_controls_restore_real_modes_shared_flags_and_protocol() {
    for (typed, signal, status) in [
        (Some(b'q'), None, 0),
        (Some(b'Q'), None, 0),
        (Some(3), None, 130),
        (None, Some(libc::SIGINT), 130),
        (None, Some(libc::SIGTERM), 143),
    ] {
        let mut product = Product::start(80, 24);
        product.frame(80, 24, |_| true);
        assert_raw(&product);
        // Ctrl+Z is a byte, not a stop signal; a following navigation redraw
        // proves that the child continues processing, even for singleton lists.
        product.send(b"\x1a\x1b[C");
        product.frame(80, 24, |_| true);
        if let Some(byte) = typed {
            product.send(&[byte]);
        }
        if let Some(signal) = signal {
            product.signal(signal);
        }
        assert!(product.finish(status, true).is_empty());
        product.assert_cleanup();
    }
}

#[test]
fn shared_terminal_description_and_existing_nonblocking_flags_are_preserved() {
    for already_nonblocking in [false, true] {
        let mut product = Product::spawn(80, 24, |command, input, _| {
            if already_nonblocking {
                set_flags(&input.slave, input.saved_flags | libc::O_NONBLOCK);
            }
            command.stdout(input.slave.try_clone().unwrap());
        });
        if already_nonblocking {
            product.input.saved_flags |= libc::O_NONBLOCK;
        }
        // Observe the same terminal/OFD used for both standard streams, while
        // retaining master and slave observers outside the subprocess.
        product.output = Pty {
            master: Some(product.input.master.as_ref().unwrap().try_clone().unwrap()),
            slave: product.input.slave.try_clone().unwrap(),
            saved_mode: product.input.saved_mode,
            saved_flags: product.input.saved_flags,
        };
        product.frame(80, 24, |_| true);
        assert_raw(&product);
        product.send(b"q");
        assert!(product.finish(0, true).is_empty());
        product.assert_cleanup();
    }
}

#[test]
fn queued_external_interrupt_and_terminate_choose_interrupt() {
    let mut product = Product::start(80, 24);
    product.frame(80, 24, |_| true);
    product.signal(libc::SIGSTOP);
    let deadline = Instant::now() + LIMIT;
    loop {
        let mut status = 0;
        // SAFETY: owned child PID and writable status; only consume stop
        // notification, never its final exit status. No fixed sleep oracle.
        let result = unsafe {
            libc::waitpid(
                product.child.id() as i32,
                &mut status,
                libc::WUNTRACED | libc::WNOHANG,
            )
        };
        checked(result);
        if result != 0 {
            assert!(libc::WIFSTOPPED(status));
            break;
        }
        assert!(Instant::now() < deadline, "child did not stop");
        std::thread::sleep(Duration::from_millis(1));
    }
    // Both are kernel-pending before ordinary flow resumes; SIGINT must win.
    product.signal(libc::SIGTERM);
    product.signal(libc::SIGINT);
    product.signal(libc::SIGCONT);
    assert!(product.finish(130, true).is_empty());
    product.assert_cleanup();
}

#[test]
fn fragmented_repeated_arrows_and_ignored_sequences_reach_expected_selection() {
    let mut product = Product::start(80, 24);
    let first = product.frame(80, 24, |_| true);
    let (mut index, count) = position(&first);
    for introducer in [b'[', b'O'] {
        for (final_byte, forward) in [(b'A', false), (b'B', true), (b'C', true), (b'D', false)] {
            for split in [1, 2] {
                let sequence = [27, introducer, final_byte];
                product.send(&sequence[..split]);
                // A finite readiness wait lets the incomplete fragment reach
                // the child without crossing its 100-ms idle timeout.
                product.collect(10);
                product.send(&sequence[split..]);
                index = if forward {
                    index % count + 1
                } else {
                    (index + count - 2) % count + 1
                };
                product.frame(80, 24, |rows| position(rows) == (index, count));
            }
        }
    }
    // Command-looking suffixes, aliases and typed Ctrl+Z must not quit or move.
    product.send(b"\x1b[1;2C\x1b[5~\x1b[6~\t\r\x1a\x1b[q\x1b[Q\x1b]qQ\x07");
    // A subsequent valid event supplies an observable processing checkpoint.
    product.send(b"\x1b[C\x1b[C\x1bOD");
    index = index % count + 1;
    product.frame(80, 24, |rows| position(rows) == (index, count));
    product.send(b"q");
    assert!(product.finish(0, true).is_empty());
    product.assert_cleanup();
}

#[test]
fn small_startup_resize_sampling_navigation_and_quit_keep_bounded_rows() {
    for (width, height) in [(59, 16), (60, 15), (1, 1)] {
        let mut product = Product::start(width, height);
        product.frame(width as usize, height as usize, |_| true);
        // No redundant startup sample; small mode later leaves baseline status.
        if width > 1 {
            product.frame(width as usize, height as usize, |rows| {
                rows[0].contains("too small") && !rows.iter().any(|row| row.contains("Waiting"))
            });
        }
        product.send(b"\x1b[C");
        product.output.resize(60, 16);
        product.signal(libc::SIGWINCH);
        let grown = product.frame(60, 16, |rows| rows[2].starts_with("Incoming:"));
        let (index, count) = position(&grown);
        assert_eq!(index, 1 % count + 1);
        product.frame(60, 16, |rows| rows[1].starts_with('='));
        product.output.resize(80, 24);
        product.signal(libc::SIGWINCH);
        product.frame(80, 24, |rows| rows[13].starts_with("Outgoing:"));
        product.output.resize(1, 1);
        product.signal(libc::SIGWINCH);
        product.frame(1, 1, |_| true);
        product.send(b"q");
        assert!(product.finish(0, true).is_empty());
        product.assert_cleanup();
    }
}

#[test]
fn rejected_term_and_independent_nonterminal_streams_do_not_mutate() {
    for term in [None, Some(""), Some("dumb"), Some("linux")] {
        let mut product = Product::spawn(80, 24, |command, _, _| {
            if let Some(term) = term {
                command.env("TERM", term);
            } else {
                command.env_remove("TERM");
            }
        });
        assert!(product.finish(1, true).contains("TERM"));
        assert!(product.bytes.is_empty());
        product.input.assert_restored();
        product.output.assert_restored();
    }
    for nonterminal_input in [false, true] {
        let mut product = Product::spawn(80, 24, |command, _, _| {
            if nonterminal_input {
                command.stdin(Stdio::null());
            } else {
                command.stdout(Stdio::null());
            }
        });
        assert!(
            product
                .finish(1, true)
                .contains("stdin and stdout must be terminals")
        );
        assert!(product.bytes.is_empty());
        product.input.assert_restored();
        product.output.assert_restored();
    }
    for argument in ["--help", "--version"] {
        for term in [None, Some(""), Some("dumb"), Some("linux")] {
            let mut product = Product::spawn(80, 24, |command, _, _| {
                command.arg(argument);
                if let Some(term) = term {
                    command.env("TERM", term);
                } else {
                    command.env_remove("TERM");
                }
            });
            assert!(product.finish(0, true).is_empty());
            assert!(find(&product.bytes, ENTER).is_none());
            assert!(!product.bytes.is_empty());
            product.input.assert_restored();
            product.output.assert_restored();
            let mut command = command();
            command.arg(argument).stdin(Stdio::null());
            if let Some(term) = term {
                command.env("TERM", term);
            } else {
                command.env_remove("TERM");
            }
            let result = command.output().unwrap();
            assert!(result.status.success());
            assert!(result.stderr.is_empty());
            if argument == "--version" {
                assert_eq!(
                    result.stdout,
                    format!("nload-rs {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
                );
            } else {
                assert!(String::from_utf8(result.stdout).unwrap().contains("Usage:"));
            }
        }
    }
}

#[test]
fn read_only_terminal_output_fails_setup_and_restores_changed_os_state() {
    let mut observer = None;
    let mut original_flags = 0;
    let mut product = Product::spawn(80, 24, |command, _, output| {
        let read_only = output.read_only_slave();
        original_flags = flags(&read_only);
        command.stdout(read_only.try_clone().unwrap());
        observer = Some(read_only);
    });
    let stderr = product.finish(1, true);
    assert!(stderr.contains("Bad file descriptor"));
    assert!(stderr.contains("leave alternate screen"));
    assert!(product.bytes.is_empty());
    product.input.assert_restored();
    product.output.assert_restored();
    assert_eq!(flags(observer.as_ref().unwrap()), original_flags);
}

#[test]
fn runtime_input_and_output_hangups_fail_and_attempt_remaining_restoration() {
    for input_hangup in [true, false] {
        let mut product = Product::start(80, 24);
        product.frame(80, 24, |_| true);
        if input_hangup {
            product.input.master.take();
        } else {
            product.output.master.take();
        }
        let stderr = product.finish(1, true);
        assert!(stderr.contains("poll error/hangup") || stderr.contains("Input/output error"));
        product.input.assert_flags_restored();
        product.output.assert_flags_restored();
        if input_hangup {
            product.output.assert_restored();
            // A disconnected input PTY cannot accept tcsetattr; cleanup still
            // emits both sequences through the independently healthy output.
            assert!(stderr.contains("restore termios"));
            assert!(product.bytes.ends_with(RESTORE));
        } else {
            product.input.assert_restored();
            assert!(stderr.contains("show cursor"));
            assert!(stderr.contains("leave alternate screen"));
        }
    }
}

#[test]
fn canonical_eof_on_real_input_is_fatal_and_restores_the_saved_session() {
    let mut product = Product::start(80, 24);
    product.frame(80, 24, |_| true);
    assert_raw(&product);
    // Raw PTYs do not interpret VEOF. Deliberately change the real line
    // discipline from the observer to make an empty canonical EOF readable.
    // The binary remains unmodified and must detect read(2) returning zero.
    let canonical = product.input.saved_mode;
    assert_ne!(canonical.c_lflag & libc::ICANON, 0);
    // SAFETY: live input terminal and initialized canonical termios.
    checked(unsafe { libc::tcsetattr(product.input.slave.as_raw_fd(), libc::TCSANOW, &canonical) });
    product.send(&[canonical.c_cc[libc::VEOF]]);
    assert!(product.finish(1, true).contains("terminal input closed"));
    product.assert_cleanup();
}

fn wait_backpressure(product: &Product) {
    let deadline = Instant::now() + LIMIT;
    loop {
        let mut fd = libc::pollfd {
            fd: product.output.slave.as_raw_fd(),
            events: libc::POLLOUT,
            revents: 0,
        };
        // SAFETY: live retained output description, zero-time readiness probe.
        checked(unsafe { libc::poll(&mut fd, 1, 0) });
        if fd.revents & libc::POLLOUT == 0 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "output never became backpressured"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn flow_stopped_output_during_setup_restores_os_state_despite_cleanup_failure() {
    let mut product = Product::spawn(80, 24, |_, _, output| {
        // A filled PTY alone can asynchronously move bytes into its master queue
        // and free space for the tiny setup command. Stop output flow first so
        // kernel queue movement cannot race the fault we intend to exercise.
        // SAFETY: live output terminal and valid tcflow action.
        checked(unsafe { libc::tcflow(output.slave.as_raw_fd(), libc::TCOOFF) });
        set_flags(&output.slave, output.saved_flags | libc::O_NONBLOCK);
        let mut writer = output.slave.try_clone().unwrap();
        let buffer = [b'x'; 4096];
        loop {
            match writer.write(&buffer) {
                Ok(n) => assert!(n > 0),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => panic!("prefill: {error}"),
            }
        }
        set_flags(&output.slave, output.saved_flags);
    });
    let stderr = product.finish(1, false);
    assert!(stderr.contains("temporarily unavailable"));
    assert!(stderr.contains("leave alternate screen"));
    product.input.assert_restored();
    product.output.assert_restored();
    // SAFETY: undo the harness's flow stop, which predates the child session.
    checked(unsafe { libc::tcflow(product.output.slave.as_raw_fd(), libc::TCOON) });
}

#[test]
fn backpressured_runtime_still_handles_quit_and_signal_with_bounded_cleanup() {
    for (signal, status) in [
        (None, 1),
        (Some(libc::SIGINT), 130),
        (Some(libc::SIGTERM), 143),
    ] {
        let mut product = Product::start(80, 24);
        product.frame(80, 24, |_| true);
        // A large but ordinary winsize creates a real partially emitted frame.
        product.output.resize(1000, 1000);
        product.signal(libc::SIGWINCH);
        wait_backpressure(&product);
        if let Some(signal) = signal {
            product.signal(signal);
        } else {
            product.send(b"q");
        }
        let stderr = product.finish(status, false);
        assert!(stderr.contains("show cursor"));
        assert!(stderr.contains("leave alternate screen"));
        product.input.assert_restored();
        product.output.assert_restored();
    }
}
