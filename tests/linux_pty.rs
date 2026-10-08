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
const START: &[u8] = b"\x1b[0m\x1b[H\x1b[2J";
const END: &[u8] = b"\x1b[H";
const ENTER: &[u8] = b"\x1b[?1049h\x1b[?25l";
const RESTORE: &[u8] = b"\x18\x1b[0m\x18\x1b[?25h\x18\x1b[?1049l";

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
    color: bool,
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
        let color = command
            .get_args()
            .take_while(|arg| *arg != "--")
            .any(|arg| arg == "--color");
        Self {
            child: command.spawn().unwrap(),
            input,
            output,
            bytes: Vec::new(),
            pending: Vec::new(),
            color,
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
        self.frame_with_geometry(|_| (width, height), predicate)
    }

    fn frame_with_geometry(
        &mut self,
        geometry: impl Fn(&[u8]) -> (usize, usize),
        predicate: impl Fn(&[String]) -> bool,
    ) -> Vec<String> {
        let deadline = Instant::now() + LIMIT;
        loop {
            while let Some(start) = find(&self.pending, START) {
                assert!(
                    start == 0 || &self.pending[..start] == ENTER,
                    "unknown protocol before frame"
                );
                let body = start + START.len();
                let Some(end) = find(&self.pending[body..], END).map(|offset| body + offset) else {
                    break;
                };
                let payload = &self.pending[body..end];
                let (width, height) = geometry(payload);
                let rows = decode_rows(payload, width, height, self.color);
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

#[test]
fn decoder_counts_styled_braille_as_cells_and_preserves_default_text() {
    let body = "\x1b[1;1H\x1b[0m\x1b[38;5;183m⢸⣿\x1b[0m  Curr: 1\x1b[0m\x1b[2;1H\x1b[0m\x1b[38;5;96m⡇\x1b[0m\x1b[0m";
    assert_eq!(
        decode_rows(body.as_bytes(), 11, 2, true),
        ["⢸⣿  Curr: 1", "⡇"]
    );
    assert_eq!(decode_rows(b"", 0, 3, false), ["", "", ""]);
    let plain = "\x1b[1;1H\x1b[0m⢸⣿  Curr: 1\x1b[0m\x1b[2;1H\x1b[0m⡇\x1b[0m";
    assert_eq!(
        decode_rows(plain.as_bytes(), 11, 2, false),
        ["⢸⣿  Curr: 1", "⡇"]
    );
    assert!(std::panic::catch_unwind(|| decode_rows(body.as_bytes(), 11, 2, false)).is_err());
}

#[test]
fn decoder_rejects_unknown_protocol_style_leaks_and_cell_overflow() {
    for body in [
        "\x1b[+1;1H\x1b[0mx\x1b[0m",                // not a numeric CSI address
        "\x1b[1;2H\x1b[0mx\x1b[0m",                 // unexpected column
        "\x1b[2;1H\x1b[0mx\x1b[0m",                 // outside logical height
        "\x1b[1;1H\x1b[0m\x1b[2J\x1b[0m",           // screen cannot clear
        "\x1b[1;1H\x1b[0m\x1b[?25h\x1b[0m",         // screen cannot change session
        "\x1b[1;1H\x1b[0m\x1b[31m⣿\x1b[0m",         // unsupported SGR
        "\x1b[1;1H\x1b[0m\x1b[38;5;1m⣿\x1b[0m",     // outside palette
        "\x1b[1;1H\x1b[0m\x1b[48;5;60m⣿\x1b[0m",    // background
        "\x1b[1;1H\x1b[0m\x1b[38;5;60mCurr\x1b[0m", // styled text
        "\x1b[1;1H\x1b[0m\x1b[38;5;60m \x1b[0m",    // styled blank
        "\x1b[1;1H\x1b[0m⣿\x1b[0m",                 // unstyled Braille
        "\x1b[1;1H\x1b[0m\x1b[38;5;60m⠀\x1b[0m",    // U+2800
        "\x1b[1;1H\x1b[0mé\x1b[0m",                 // arbitrary Unicode
        "\x1b[1;1H\x1b[0mx\n\x1b[0m",
        "\x1b[1;1H\x1b[0mx\r\x1b[0m",
        "\x1b[1;1H\x1b[0m\x07\x1b[0m",
        "\x1b[1;1Hx\x1b[0m",                      // missing row-start reset
        "\x1b[1;1H\x1b[0mx",                      // missing final reset
        "\x1b[1;1H\x1b[0m\x1b[38;5;60m⣿⢸\x1b[0m", // two cells in width one
    ] {
        assert!(
            std::panic::catch_unwind(|| decode_rows(body.as_bytes(), 1, 1, true)).is_err(),
            "accepted {body:?}"
        );
    }
    for index in [60, 61, 98, 140, 146, 183, 96, 132, 133, 170, 176, 219] {
        let body = format!("\x1b[1;1H\x1b[0m\x1b[38;5;{index}m⣿\x1b[0m");
        assert_eq!(decode_rows(body.as_bytes(), 1, 1, true), ["⣿"]);
    }
}

/// Validate protocol rather than visual appearance. This is deliberately a
/// whitelist, not a general ANSI stripper: SGR is zero-width, allowed Braille
/// consumes one cell, and all other controls/Unicode/escapes are rejected.
fn decode_rows(mut bytes: &[u8], width: usize, height: usize, color: bool) -> Vec<String> {
    let mut rows = vec![String::new(); height];
    let mut next_row = 1;
    while !bytes.is_empty() {
        let (address, command, rest) = csi(bytes);
        assert_eq!(command, b'H');
        let (row, column) = address.split_once(';').unwrap();
        assert!(!row.is_empty() && row.bytes().all(|byte| byte.is_ascii_digit()));
        let row = row.parse::<usize>().unwrap();
        assert_eq!(column, "1");
        assert!((1..=height).contains(&row));
        assert_eq!(row, next_row);
        next_row += 1;
        bytes = rest;
        assert!(
            bytes.starts_with(b"\x1b[0m"),
            "row must start in default style"
        );
        let mut foreground = false;
        let mut final_reset = false;
        let mut cells = 0;
        while !bytes.is_empty() {
            if bytes[0] == 27 {
                let (parameters, command, rest) = csi(bytes);
                if command == b'H' {
                    break;
                }
                assert_eq!(command, b'm', "unknown row command");
                if parameters == "0" {
                    foreground = false;
                    final_reset = true;
                } else {
                    assert!(color, "color is disabled");
                    assert!(
                        matches!(
                            parameters,
                            "38;5;60"
                                | "38;5;61"
                                | "38;5;98"
                                | "38;5;140"
                                | "38;5;146"
                                | "38;5;183"
                                | "38;5;96"
                                | "38;5;132"
                                | "38;5;133"
                                | "38;5;170"
                                | "38;5;176"
                                | "38;5;219"
                        ),
                        "unknown SGR: {parameters}"
                    );
                    foreground = true;
                    final_reset = false;
                }
                bytes = rest;
            } else {
                let end = bytes
                    .iter()
                    .position(|&byte| byte == 27)
                    .unwrap_or(bytes.len());
                let text = std::str::from_utf8(&bytes[..end]).unwrap();
                for character in text.chars() {
                    if ('\u{2801}'..='\u{28ff}').contains(&character) {
                        assert_eq!(
                            foreground, color,
                            "Braille must match the chosen color mode"
                        );
                    } else {
                        assert!(
                            (' '..='~').contains(&character),
                            "unknown display character"
                        );
                        assert!(!foreground, "text and spaces must use default style");
                    }
                    cells += 1;
                    assert!(cells <= width, "row exceeds terminal-cell width");
                    rows[row - 1].push(character);
                }
                final_reset = false;
                bytes = &bytes[end..];
            }
        }
        assert!(final_reset, "row must end with reset");
    }
    assert_eq!(next_row - 1, if width == 0 { 0 } else { height });
    rows
}

fn csi(bytes: &[u8]) -> (&str, u8, &[u8]) {
    assert!(bytes.starts_with(b"\x1b["), "unknown escape");
    let end = bytes[2..]
        .iter()
        .position(|byte| (0x40..=0x7e).contains(byte))
        .unwrap()
        + 2;
    let parameters = std::str::from_utf8(&bytes[2..end]).unwrap();
    (parameters, bytes[end], &bytes[end + 1..])
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

fn loopback_burst() {
    use std::net::UdpSocket;
    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    receiver.set_read_timeout(Some(LIMIT)).unwrap();
    let payload = [0x5a; 4096];
    let mut received = [0; 4096];
    for _ in 0..16 {
        assert_eq!(
            sender
                .send_to(&payload, receiver.local_addr().unwrap())
                .unwrap(),
            payload.len()
        );
        assert_eq!(receiver.recv_from(&mut received).unwrap().0, payload.len());
    }
}

fn colored_product() -> Product {
    let mut product = Product::spawn(60, 16, |command, _, _| {
        command.args(["--color", "-i", "1", "-o", "1", "lo"]);
    });
    product.frame(60, 16, |_| true);
    loopback_burst();
    product.frame(60, 16, |rows| {
        rows[3]
            .chars()
            .nth(29)
            .is_some_and(|glyph| ('\u{2801}'..='\u{28ff}').contains(&glyph))
    });
    assert!(find(&product.bytes, b"\x1b[38;5;183m").is_some());
    assert!(find(&product.bytes, b"\x1b[38;5;219m").is_some());
    product
}

#[test]
fn colored_normal_signal_and_runtime_eof_exits_restore_os_state_and_protocol() {
    enum Exit {
        Typed(&'static [u8]),
        Signal(libc::c_int),
        InputEof,
    }
    for (name, cause, status, diagnostic) in [
        ("q", Exit::Typed(b"q"), 0, None),
        ("Q", Exit::Typed(b"Q"), 0, None),
        ("typed Ctrl+C", Exit::Typed(b"\x03"), 130, None),
        ("SIGINT", Exit::Signal(libc::SIGINT), 130, None),
        ("SIGTERM", Exit::Signal(libc::SIGTERM), 143, None),
        (
            "input EOF",
            Exit::InputEof,
            1,
            Some("terminal input closed"),
        ),
    ] {
        let mut product = colored_product();
        assert_raw(&product);
        match cause {
            Exit::Typed(bytes) => product.send(bytes),
            Exit::Signal(signal) => product.signal(signal),
            Exit::InputEof => {
                let canonical = product.input.saved_mode;
                // SAFETY: real retained input terminal and initialized termios.
                checked(unsafe {
                    libc::tcsetattr(product.input.slave.as_raw_fd(), libc::TCSANOW, &canonical)
                });
                product.send(&[canonical.c_cc[libc::VEOF]]);
            }
        }
        let stderr = product.finish(status, true);
        if let Some(expected) = diagnostic {
            assert!(stderr.contains(expected), "{name}: {stderr}");
        } else {
            assert!(stderr.is_empty(), "{name}: {stderr}");
        }
        product.assert_cleanup();
    }
}

#[test]
fn colored_backpressure_resumes_a_complete_large_frame_then_recovers_latest_resize() {
    let mut product = colored_product();
    product.output.resize(1000, 1000);
    product.signal(libc::SIGWINCH);
    wait_backpressure(&product);
    // Input and resize arrive while a colored frame is partially written. Drain
    // only after requesting the latest small view; the started large frame must
    // complete first. Different heights identify each permitted fixture geometry.
    product.send(b"\x1b[C\x1b[D\x1b[C\x1b[D");
    product.output.resize(59, 15);
    product.signal(libc::SIGWINCH);
    let mut saw_large = false;
    let rows = product.frame_with_geometry(
        |body| {
            if find(body, b"\x1b[1000;1H").is_some() {
                (1000, 1000)
            } else if find(body, b"\x1b[16;1H").is_some() {
                (60, 16)
            } else {
                (59, 15)
            }
        },
        |rows| rows[0].starts_with("Terminal too small"),
    );
    // The strict decoder has consumed the complete large frame on its way to the
    // warning. Audit the captured bytes independently for a colored tall frame.
    let mut remaining = product.bytes.as_slice();
    while let Some(start) = find(remaining, START) {
        remaining = &remaining[start + START.len()..];
        let Some(end) = find(remaining, END) else {
            break;
        };
        let body = &remaining[..end];
        if find(body, b"\x1b[1000;1H").is_some() {
            assert!(find(body, b"\x1b[38;5;").is_some());
            decode_rows(body, 1000, 1000, true);
            saw_large = true;
        } else if find(body, b"\x1b[16;1H").is_none() {
            assert!(
                find(body, b"\x1b[38;5;").is_none(),
                "small view must not select a palette"
            );
        }
        remaining = &remaining[end + END.len()..];
    }
    assert!(saw_large, "started large frame was lost or interleaved");
    assert!(rows.iter().all(|row| row.is_ascii()));
    product.output.resize(80, 24);
    product.signal(libc::SIGWINCH);
    let recovered = product.frame_with_geometry(
        |body| {
            if find(body, b"\x1b[24;1H").is_some() {
                (80, 24)
            } else {
                (59, 15)
            }
        },
        |rows| rows.len() == 24 && rows[1].starts_with('='),
    );
    assert!(recovered.iter().any(|row| {
        row.chars()
            .any(|glyph| ('\u{2801}'..='\u{28ff}').contains(&glyph))
    }));
    product.send(b"q");
    assert!(product.finish(0, true).is_empty());
    product.assert_cleanup();
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
    for introducer in *b"[O" {
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
fn graph_colors_are_opt_in_under_no_color_and_locale_declarations() {
    for (width, height, color, locale, no_color) in [
        (60, 16, false, None, None),
        (60, 16, false, Some("C"), Some("1")),
        (80, 24, false, Some("C.UTF-8"), Some("")),
        (80, 24, false, Some("not-a-probed-locale"), Some("1")),
        (60, 16, true, None, None),
        (60, 16, true, Some("C"), Some("1")),
        (80, 24, true, Some("C.UTF-8"), Some("")),
        (80, 24, true, Some("not-a-probed-locale"), Some("1")),
    ] {
        let mut product = Product::spawn(width, height, |command, _, _| {
            command.args(["-i", "1", "-o", "1", "lo"]);
            if color {
                command.arg("--color");
            }
            if let Some(locale) = locale {
                command.env("LC_ALL", locale).env("LANG", locale);
            } else {
                command.env_remove("LC_ALL").env_remove("LANG");
            }
            if let Some(value) = no_color {
                command.env("NO_COLOR", value);
            } else {
                command.env_remove("NO_COLOR");
            }
        });
        let baseline = product.frame(width as usize, height as usize, |_| true);
        assert!(baseline[0].starts_with("Interface lo "));
        assert_eq!(baseline[1], "Waiting for rate sample");
        assert!(find(&product.bytes, b"\x1b[38;5;").is_none());

        // Ordinary nonprivileged loopback traffic after the baseline. Receive
        // each datagram immediately so queue capacity is not a timing oracle.
        loopback_burst();
        let graph_width = width as usize - 30;
        let rows = product.frame(width as usize, height as usize, |rows| {
            matches!(rows[3].chars().nth(graph_width - 1), Some('⢸' | '⣿'))
        });
        let incoming_end = 3 + (height as usize - 4) / 2;
        for row in rows[3..incoming_end]
            .iter()
            .chain(&rows[incoming_end + 1..])
        {
            assert!(matches!(row.chars().nth(graph_width - 1), Some('⢸' | '⣿')));
            assert_eq!(row.chars().nth(graph_width).unwrap_or(' '), ' ');
            assert_eq!(row.chars().nth(graph_width + 1).unwrap_or(' '), ' ');
        }
        if color {
            for index in [60, 61, 98, 140, 146, 183, 96, 132, 133, 170, 176, 219] {
                assert!(find(&product.bytes, format!("\x1b[38;5;{index}m").as_bytes()).is_some());
            }
        } else {
            assert!(find(&product.bytes, b"\x1b[38;5;").is_none());
        }
        assert!(!product.bytes.contains(&b'\n'));
        assert!(!product.bytes.contains(&b'\r'));
        product.send(b"q");
        assert!(product.finish(0, true).is_empty()); // redirected stderr supported
        if !color {
            assert!(find(&product.bytes, b"\x1b[38;5;").is_none());
        }
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
    assert!(stderr.contains("reset styling"));
    assert!(stderr.contains("leave alternate screen"));
    assert!(product.bytes.is_empty());
    product.input.assert_restored();
    product.output.assert_restored();
    assert_eq!(flags(observer.as_ref().unwrap()), original_flags);
}

#[test]
fn runtime_input_and_output_hangups_fail_and_attempt_remaining_restoration() {
    for input_hangup in [true, false] {
        let mut product = colored_product();
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
            assert!(stderr.contains("reset styling"));
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
    assert!(stderr.contains("reset styling"));
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
        let mut product = colored_product();
        // A large but ordinary winsize creates a real partially emitted frame.
        product.output.resize(1000, 1000);
        product.signal(libc::SIGWINCH);
        wait_backpressure(&product);
        // No POLLOUT does not guarantee the tiny restoration writes will fail:
        // the kernel can move queued bytes to the master and free slave space.
        // Freeze output after observing real runtime backpressure so cleanup
        // must exercise its bounded failure path on every kernel/schedule.
        // SAFETY: live output terminal and valid tcflow action.
        checked(unsafe { libc::tcflow(product.output.slave.as_raw_fd(), libc::TCOOFF) });
        if let Some(signal) = signal {
            product.signal(signal);
        } else {
            product.send(b"q");
        }
        let stderr = product.finish(status, false);
        assert!(stderr.contains("reset styling"), "stderr: {stderr}");
        assert!(stderr.contains("show cursor"), "stderr: {stderr}");
        assert!(
            stderr.contains("leave alternate screen"),
            "stderr: {stderr}"
        );
        product.input.assert_restored();
        product.output.assert_restored();
        // SAFETY: undo the harness's flow stop after observing child cleanup.
        checked(unsafe { libc::tcflow(product.output.slave.as_raw_fd(), libc::TCOON) });
    }
}
