// SPDX-License-Identifier: GPL-2.0-only

use std::ffi::OsString;
use std::io;
use std::panic::{AssertUnwindSafe, PanicHookInfo, catch_unwind};
use std::time::{Duration, Instant};

pub(super) const ENTER_SCREEN: &[u8] = b"\x1b[?1049h";
pub(super) const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
// CAN cancels a possibly unfinished control sequence before restoration.
pub(super) const SHOW_CURSOR: &[u8] = b"\x18\x1b[?25h";
pub(super) const LEAVE_SCREEN: &[u8] = b"\x18\x1b[?1049l";
const CLEANUP_WRITE_BUDGET: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shutdown {
    Quit,
    Interrupt,
    Terminate,
}

impl Shutdown {
    pub(crate) fn status(self) -> u8 {
        match self {
            Self::Quit => 0,
            Self::Interrupt => 130,
            Self::Terminate => 143,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Notifications {
    pub(crate) shutdown: Option<Shutdown>,
    pub(crate) resized: bool,
}

pub(crate) struct Support {
    pub(super) input_terminal: bool,
    pub(super) output_terminal: bool,
    pub(super) term: Option<OsString>,
}

pub(crate) struct Saved<T> {
    pub(super) termios: T,
    pub(super) input_flags: i32,
    pub(super) output_flags: i32,
}

/// Private effect seam: the Linux adapter and the failure-script adapter both
/// exercise the same staged owner. Callers do not coordinate these operations.
pub(crate) trait SessionOps {
    type Termios;

    fn support(&mut self) -> Support;
    fn snapshot(&mut self) -> io::Result<Saved<Self::Termios>>;
    fn prepare_notifications(&mut self) -> io::Result<()>;
    fn raw(&mut self, saved: &Self::Termios) -> io::Result<()>;
    fn input_flags(&mut self, flags: i32) -> io::Result<()>;
    fn output_flags(&mut self, flags: i32) -> io::Result<()>;
    fn restore_termios(&mut self, saved: &Self::Termios) -> io::Result<()>;
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize>;
    fn wait_writable(&mut self, timeout: Duration) -> io::Result<()>;
    fn dimensions(&mut self) -> io::Result<(usize, usize)>;
    /// Drain wake tokens before inspecting flags; bytes have no signal identity.
    fn notifications(&mut self) -> io::Result<Notifications>;
    fn release_notifications(&mut self);
}

#[derive(Default)]
struct Stages {
    notifications: bool,
    raw: bool,
    input_flags: bool,
    output_flags: bool,
    screen: bool,
    cursor: bool,
}

/// Own before initialization and retain until `guarded` has cleaned up. Attempts
/// are recorded *before* each effect: even a failed/partial effect may need undo.
pub(crate) struct Session<O: SessionOps> {
    ops: O,
    saved: Option<Saved<O::Termios>>,
    stages: Stages,
    initialized: bool,
}

impl<O: SessionOps> Session<O> {
    pub(crate) fn new(ops: O) -> Self {
        Self {
            ops,
            saved: None,
            stages: Stages::default(),
            initialized: false,
        }
    }

    /// Validate, save, prepare notification, then mutate. Small/zero dimensions
    /// are returned normally. A session is single-use, including failed setup.
    pub(crate) fn initialize(&mut self) -> io::Result<(usize, usize)> {
        if self.initialized {
            return Err(io::Error::other("terminal session already initialized"));
        }
        self.initialized = true;
        validate_support(self.ops.support())?;
        self.saved = Some(self.ops.snapshot()?);
        self.stages.notifications = true;
        self.ops.prepare_notifications()?;

        // All saved state and notification ownership exist before mutation.
        let saved = self.saved.as_ref().expect("snapshot saved before mutation");
        self.stages.raw = true;
        self.ops.raw(&saved.termios)?;
        if saved.input_flags & libc::O_NONBLOCK == 0 {
            self.stages.input_flags = true;
            self.ops.input_flags(saved.input_flags | libc::O_NONBLOCK)?;
        }
        if saved.output_flags & libc::O_NONBLOCK == 0 {
            self.stages.output_flags = true;
            self.ops
                .output_flags(saved.output_flags | libc::O_NONBLOCK)?;
        }
        let dimensions = self.ops.dimensions()?;
        self.stages.screen = true;
        write_sequence(&mut self.ops, ENTER_SCREEN, None)?;
        self.stages.cursor = true;
        write_sequence(&mut self.ops, HIDE_CURSOR, None)?;
        Ok(dimensions)
    }

    pub(crate) fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        self.ops.dimensions()
    }

    pub(crate) fn notifications(&mut self) -> io::Result<Notifications> {
        self.ops.notifications()
    }

    // Runtime extensions stay inside terminal; application callers do not get
    // FDs or manage staged state themselves.
    pub(super) fn effects_mut(&mut self) -> &mut O {
        &mut self.ops
    }

    /// Exhaustive, nonpanicking and idempotent. Each attempted restoration is
    /// consumed, even on failure; Drop is for paths missing explicit cleanup.
    pub(crate) fn cleanup(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        if std::mem::take(&mut self.stages.cursor) {
            restoration(&mut errors, "show cursor", || {
                write_sequence(&mut self.ops, SHOW_CURSOR, Some(CLEANUP_WRITE_BUDGET))
            });
        }
        if std::mem::take(&mut self.stages.screen) {
            restoration(&mut errors, "leave alternate screen", || {
                write_sequence(&mut self.ops, LEAVE_SCREEN, Some(CLEANUP_WRITE_BUDGET))
            });
        }
        // Keep output nonblocking while attempting screen restoration. Neither
        // tcdrain nor switching to blocking writes can stall shutdown forever.
        if let Some(saved) = &self.saved {
            if std::mem::take(&mut self.stages.raw) {
                restoration(&mut errors, "restore termios", || {
                    self.ops.restore_termios(&saved.termios)
                });
            }
            if std::mem::take(&mut self.stages.input_flags) {
                restoration(&mut errors, "restore input flags", || {
                    self.ops.input_flags(saved.input_flags)
                });
            }
            if std::mem::take(&mut self.stages.output_flags) {
                restoration(&mut errors, "restore output flags", || {
                    self.ops.output_flags(saved.output_flags)
                });
            }
        }
        if std::mem::take(&mut self.stages.notifications) {
            restoration(&mut errors, "release signal notification", || {
                self.ops.release_notifications();
                Ok(())
            });
        }
        errors
    }
}

impl<O: SessionOps> Drop for Session<O> {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn validate_support(support: Support) -> io::Result<()> {
    if !support.input_terminal || !support.output_terminal {
        return Err(io::Error::other(
            "interactive stdin and stdout must be terminals",
        ));
    }
    match support.term.as_deref() {
        None => Err(io::Error::other("TERM is unset")),
        Some(term) if term.is_empty() || term == "dumb" || term == "linux" => Err(
            io::Error::other("TERM does not declare an xterm-compatible terminal"),
        ),
        Some(_) => Ok(()),
    }
}

fn write_sequence<O: SessionOps>(
    ops: &mut O,
    mut bytes: &[u8],
    budget: Option<Duration>,
) -> io::Result<()> {
    let started = Instant::now();
    while !bytes.is_empty() {
        if budget.is_some_and(|budget| started.elapsed() >= budget) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "terminal restoration output timed out",
            ));
        }
        match ops.write(bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "terminal write returned zero",
                ));
            }
            Ok(n) if n <= bytes.len() => bytes = &bytes[n..],
            Ok(_) => return Err(io::Error::other("terminal write exceeded supplied bytes")),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                let Some(budget) = budget else { return Err(e) };
                match ops.wait_writable(budget.saturating_sub(started.elapsed())) {
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    result => result?,
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn restoration(errors: &mut Vec<String>, operation: &str, effect: impl FnOnce() -> io::Result<()>) {
    // A scripted adapter can panic too; one failed operation must never prevent
    // later attempts or cause a double panic from Drop during unwinding.
    match catch_unwind(AssertUnwindSafe(|| {
        effect().map_err(|error| error.to_string())
    })) {
        Ok(Ok(())) => {}
        Ok(Err(error)) => errors.push(format!("{operation}: {error}")),
        Err(payload) => {
            std::mem::forget(payload); // A user-supplied panic payload may itself panic on drop.
            errors.push(format!("{operation}: restoration panicked"));
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ExitReport {
    pub(crate) status: u8,
    /// Returned only after restoration and hook restoration; the application
    /// reports these on stderr and returns ExitCode normally.
    pub(crate) diagnostics: Vec<String>,
}

/// Startup selection errors need status 2 inside the same caught-panic boundary
/// as discovery. All I/O/runtime failures use status 1.
#[derive(Debug)]
pub(crate) struct Failure {
    status: u8,
    diagnostic: String,
}

impl Failure {
    pub(crate) fn argument(diagnostic: String) -> Self {
        Self {
            status: 2,
            diagnostic,
        }
    }
}

impl From<String> for Failure {
    fn from(diagnostic: String) -> Self {
        Self {
            status: 1,
            diagnostic,
        }
    }
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        error.to_string().into()
    }
}

/// Wrap *all* fallible startup and execution in this closure. Construct the
/// uninitialized owner first, initialize it only after discovery/selection.
/// Process-global hook: use once from the single-threaded application entry.
pub(crate) fn guarded<O: SessionOps>(
    session: &mut Session<O>,
    execute: impl FnOnce(&mut Session<O>) -> Result<Shutdown, Failure>,
) -> ExitReport {
    let hook = QuietHook::install();
    let result = catch_unwind(AssertUnwindSafe(|| execute(session)));
    let cleanup_errors = session.cleanup();
    drop(hook);
    let (mut status, mut diagnostics) = match result {
        Ok(Ok(shutdown)) => (shutdown.status(), Vec::new()),
        Ok(Err(error)) => (error.status, vec![error.diagnostic]),
        Err(payload) => {
            let message = if let Some(message) = payload.downcast_ref::<String>() {
                message.clone()
            } else if let Some(message) = payload.downcast_ref::<&str>() {
                (*message).to_owned()
            } else {
                "non-text panic payload".to_owned()
            };
            std::mem::forget(payload);
            (1, vec![format!("caught panic: {message}")])
        }
    };
    if !cleanup_errors.is_empty() {
        if status == 0 {
            status = 1;
        }
        diagnostics.extend(
            cleanup_errors
                .into_iter()
                .map(|error| format!("terminal cleanup: {error}")),
        );
    }
    ExitReport {
        status,
        diagnostics,
    }
}

type Hook = Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>;

struct QuietHook(Option<Hook>);

impl QuietHook {
    fn install() -> Self {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        Self(Some(previous))
    }
}

impl Drop for QuietHook {
    fn drop(&mut self) {
        if let Some(previous) = self.0.take() {
            std::panic::set_hook(previous);
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod scripted;

#[cfg(test)]
pub(crate) static HOOK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
