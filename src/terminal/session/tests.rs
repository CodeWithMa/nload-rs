// SPDX-License-Identifier: GPL-2.0-only

use super::*;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

// guarded owns a process-global hook. Serialize tests that install hooks; the
// production application invokes it once in its single main thread.
static HOOK_LOCK: Mutex<()> = Mutex::new(());
const INPUT_FLAGS: i32 = libc::O_APPEND;
const OUTPUT_FLAGS: i32 = libc::O_ASYNC;

#[derive(Clone, Copy)]
enum Step {
    Error,
    Panic,
    Short(usize),
    Interrupted,
    WouldBlock,
    Zero,
}

struct State {
    log: Vec<&'static str>,
    faults: HashMap<&'static str, VecDeque<Step>>,
    termios: usize,
    input_flags: i32,
    output_flags: i32,
    registered: bool,
    emitted: Vec<u8>,
    dimensions: (usize, usize),
    pending: Notifications,
}

impl Default for State {
    fn default() -> Self {
        Self {
            log: Vec::new(),
            faults: HashMap::new(),
            termios: 42,
            input_flags: INPUT_FLAGS,
            output_flags: OUTPUT_FLAGS,
            registered: false,
            emitted: Vec::new(),
            dimensions: (80, 24),
            pending: Notifications::default(),
        }
    }
}

struct Script {
    state: Rc<RefCell<State>>,
    support: Option<Support>,
    write_kind: &'static str,
}

impl Script {
    fn new() -> (Session<Self>, Rc<RefCell<State>>) {
        let state = Rc::new(RefCell::new(State::default()));
        let script = Self {
            state: Rc::clone(&state),
            support: Some(Support {
                input_terminal: true,
                output_terminal: true,
                term: Some("xterm-256color".into()),
            }),
            write_kind: "write",
        };
        (Session::new(script), state)
    }

    fn step(&mut self, name: &'static str) -> io::Result<Option<Step>> {
        let step = {
            let mut state = self.state.borrow_mut();
            state.log.push(name);
            state.faults.get_mut(name).and_then(VecDeque::pop_front)
        };
        match step {
            Some(Step::Error) => Err(io::Error::other(format!("scripted {name} failure"))),
            Some(Step::Panic) => panic!("scripted {name} panic"),
            Some(Step::Interrupted) => Err(io::ErrorKind::Interrupted.into()),
            Some(Step::WouldBlock) => Err(io::ErrorKind::WouldBlock.into()),
            other => Ok(other),
        }
    }
}

impl SessionOps for Script {
    type Termios = usize;

    fn support(&mut self) -> Support {
        self.state.borrow_mut().log.push("support");
        self.support.take().unwrap()
    }

    fn snapshot(&mut self) -> io::Result<Saved<Self::Termios>> {
        self.step("snapshot")?;
        let state = self.state.borrow();
        Ok(Saved {
            termios: state.termios,
            input_flags: state.input_flags,
            output_flags: state.output_flags,
        })
    }

    fn prepare_notifications(&mut self) -> io::Result<()> {
        // Even a partially successful registration is owned for cleanup.
        self.state.borrow_mut().registered = true;
        self.step("prepare")?;
        Ok(())
    }

    fn raw(&mut self, _: &Self::Termios) -> io::Result<()> {
        self.state.borrow_mut().termios = 99;
        self.step("raw")?;
        Ok(())
    }

    fn input_flags(&mut self, flags: i32) -> io::Result<()> {
        self.state.borrow_mut().input_flags = flags;
        self.step(if flags & libc::O_NONBLOCK != 0 {
            "input"
        } else {
            "restore-input"
        })?;
        Ok(())
    }

    fn output_flags(&mut self, flags: i32) -> io::Result<()> {
        self.state.borrow_mut().output_flags = flags;
        self.step(if flags & libc::O_NONBLOCK != 0 {
            "output"
        } else {
            "restore-output"
        })?;
        Ok(())
    }

    fn restore_termios(&mut self, saved: &Self::Termios) -> io::Result<()> {
        self.step("restore-termios")?;
        self.state.borrow_mut().termios = *saved;
        Ok(())
    }

    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.write_kind = match bytes {
            ENTER_SCREEN => "enter",
            HIDE_CURSOR => "hide",
            SHOW_CURSOR => "show",
            LEAVE_SCREEN => "leave",
            _ => self.write_kind,
        };
        let count = match self.step(self.write_kind)? {
            Some(Step::Short(count)) => count.min(bytes.len()),
            Some(Step::Zero) => 0,
            _ => bytes.len(),
        };
        self.state
            .borrow_mut()
            .emitted
            .extend_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn wait_writable(&mut self, _: Duration) -> io::Result<()> {
        self.step("wait")?;
        Ok(())
    }

    fn dimensions(&mut self) -> io::Result<(usize, usize)> {
        self.step("dimensions")?;
        Ok(self.state.borrow().dimensions)
    }

    fn notifications(&mut self) -> io::Result<Notifications> {
        self.step("drain-before-flags")?;
        Ok(self.state.borrow().pending)
    }

    fn release_notifications(&mut self) {
        self.state.borrow_mut().registered = false;
        let _ = self.step("release").unwrap();
    }
}

fn fault(state: &Rc<RefCell<State>>, name: &'static str, steps: &[Step]) {
    state
        .borrow_mut()
        .faults
        .insert(name, steps.iter().copied().collect());
}

fn assert_restored(state: &Rc<RefCell<State>>) {
    let state = state.borrow();
    assert_eq!(state.termios, 42);
    assert_eq!(state.input_flags, INPUT_FLAGS);
    assert_eq!(state.output_flags, OUTPUT_FLAGS);
    assert!(!state.registered);
}

const RESTORE: &[&str] = &[
    "show",
    "leave",
    "restore-termios",
    "restore-input",
    "restore-output",
    "release",
];

#[test]
fn support_rejects_each_stream_and_term_before_snapshot_or_mutation() {
    for (input, output, term) in [
        (false, true, Some("xterm")),
        (true, false, Some("xterm")),
        (false, false, Some("xterm")),
        (true, true, None),
        (true, true, Some("")),
        (true, true, Some("dumb")),
        (true, true, Some("linux")),
    ] {
        let (mut session, state) = Script::new();
        session.ops.support = Some(Support {
            input_terminal: input,
            output_terminal: output,
            term: term.map(OsString::from),
        });
        assert!(session.initialize().is_err());
        assert!(session.cleanup().is_empty());
        assert_eq!(state.borrow().log, ["support"]);
        assert_restored(&state);
    }
}

#[test]
fn ownership_snapshot_and_notifications_precede_every_mutation() {
    let (mut session, state) = Script::new();
    assert_eq!(session.initialize().unwrap(), (80, 24));
    assert_eq!(
        state.borrow().log,
        [
            "support",
            "snapshot",
            "prepare",
            "raw",
            "input",
            "output",
            "dimensions",
            "enter",
            "hide"
        ]
    );
    {
        let state = state.borrow();
        assert_eq!(state.termios, 99);
        assert_eq!(state.input_flags, INPUT_FLAGS | libc::O_NONBLOCK);
        assert_eq!(state.output_flags, OUTPUT_FLAGS | libc::O_NONBLOCK);
        assert!(state.registered);
        assert_eq!(state.emitted, [ENTER_SCREEN, HIDE_CURSOR].concat());
    }
    state.borrow_mut().log.clear();
    assert!(session.cleanup().is_empty());
    assert_eq!(state.borrow().log, RESTORE);
    assert_restored(&state);
    assert_eq!(
        state.borrow().emitted,
        [ENTER_SCREEN, HIDE_CURSOR, SHOW_CURSOR, LEAVE_SCREEN].concat()
    );
    let old_log = state.borrow().log.clone();
    assert!(session.cleanup().is_empty());
    drop(session);
    assert_eq!(state.borrow().log, old_log);
}

#[test]
fn existing_nonblocking_flags_are_preserved_without_claiming_changes() {
    let (mut session, state) = Script::new();
    state.borrow_mut().input_flags |= libc::O_NONBLOCK;
    state.borrow_mut().output_flags |= libc::O_NONBLOCK;
    session.initialize().unwrap();
    assert!(!state.borrow().log.contains(&"input"));
    assert!(!state.borrow().log.contains(&"output"));
    state.borrow_mut().log.clear();
    assert!(session.cleanup().is_empty());
    assert_eq!(
        state.borrow().log,
        ["show", "leave", "restore-termios", "release"]
    );
    assert_eq!(state.borrow().input_flags, INPUT_FLAGS | libc::O_NONBLOCK);
    assert_eq!(state.borrow().output_flags, OUTPUT_FLAGS | libc::O_NONBLOCK);
}

#[test]
fn every_initialization_failure_restores_only_applicable_attempted_effects() {
    let _lock = HOOK_LOCK.lock().unwrap();
    for (stage, expected) in [
        ("snapshot", &[][..]),
        ("prepare", &["release"][..]),
        ("raw", &["restore-termios", "release"][..]),
        (
            "input",
            &["restore-termios", "restore-input", "release"][..],
        ),
        (
            "output",
            &[
                "restore-termios",
                "restore-input",
                "restore-output",
                "release",
            ][..],
        ),
        (
            "dimensions",
            &[
                "restore-termios",
                "restore-input",
                "restore-output",
                "release",
            ][..],
        ),
        (
            "enter",
            &[
                "leave",
                "restore-termios",
                "restore-input",
                "restore-output",
                "release",
            ][..],
        ),
        ("hide", RESTORE),
    ] {
        let (mut session, state) = Script::new();
        fault(&state, stage, &[Step::Error]);
        let report = guarded(&mut session, |session| {
            session.initialize().map_err(|error| error.to_string())?;
            panic!("failed initialization must not execute runtime");
        });
        assert_eq!(report.status, 1, "{stage}");
        assert_eq!(report.diagnostics.len(), 1, "{stage}");
        let log = &state.borrow().log;
        let failure_index = log.iter().position(|name| *name == stage).unwrap();
        assert_eq!(&log[failure_index + 1..], expected, "{stage}");
        assert_restored(&state);
    }
}

#[test]
fn partial_screen_commands_are_recorded_even_when_the_remainder_fails() {
    let _lock = HOOK_LOCK.lock().unwrap();
    for stage in ["enter", "hide"] {
        let (mut session, state) = Script::new();
        fault(&state, stage, &[Step::Short(4), Step::WouldBlock]);
        let report = guarded(&mut session, |session| {
            session.initialize().map_err(|error| error.to_string())?;
            Ok(Shutdown::Quit)
        });
        assert_eq!(report.status, 1);
        assert!(state.borrow().log.contains(&"leave"));
        assert_eq!(state.borrow().log.contains(&"show"), stage == "hide");
        assert!(state.borrow().emitted.ends_with(LEAVE_SCREEN));
        assert_restored(&state);
    }
}

#[test]
fn interrupted_and_short_setup_writes_resume_without_repeating_emitted_bytes() {
    let (mut session, state) = Script::new();
    fault(
        &state,
        "enter",
        &[Step::Interrupted, Step::Short(2), Step::Short(1)],
    );
    fault(&state, "hide", &[Step::Short(1)]);
    session.initialize().unwrap();
    assert_eq!(state.borrow().emitted, [ENTER_SCREEN, HIDE_CURSOR].concat());
    assert!(session.cleanup().is_empty());
    assert_restored(&state);
}

#[test]
fn zero_write_is_fatal_and_still_restores_the_attempted_screen_change() {
    let (mut session, state) = Script::new();
    fault(&state, "enter", &[Step::Zero]);
    assert_eq!(
        session.initialize().unwrap_err().kind(),
        io::ErrorKind::WriteZero
    );
    assert!(session.cleanup().is_empty());
    assert!(state.borrow().log.contains(&"leave"));
    assert_restored(&state);
}

#[test]
fn small_and_resized_dimensions_are_not_session_failures() {
    use std::os::unix::ffi::OsStringExt;
    let (mut session, state) = Script::new();
    // Other declarations, even non-UTF-8, are declarations, not capability probes.
    session.ops.support.as_mut().unwrap().term = Some(OsString::from_vec(b"XTERM\xff".to_vec()));
    state.borrow_mut().dimensions = (0, 0);
    assert_eq!(session.initialize().unwrap(), (0, 0));
    state.borrow_mut().dimensions = (59, 15);
    assert_eq!(session.dimensions().unwrap(), (59, 15));
    state.borrow_mut().dimensions = (80, 24);
    assert_eq!(session.dimensions().unwrap(), (80, 24));
    assert!(session.cleanup().is_empty());
    assert_restored(&state);
}

#[test]
fn shutdown_statuses_and_runtime_failures_report_only_after_cleanup() {
    let _lock = HOOK_LOCK.lock().unwrap();
    for shutdown in [Shutdown::Quit, Shutdown::Interrupt, Shutdown::Terminate] {
        let (mut session, state) = Script::new();
        let report = guarded(&mut session, |session| {
            session.initialize().map_err(|error| error.to_string())?;
            state.borrow_mut().pending = Notifications {
                shutdown: Some(shutdown),
                resized: true,
            };
            assert_eq!(session.notifications().unwrap().shutdown, Some(shutdown));
            Ok(shutdown)
        });
        assert_eq!(
            report,
            ExitReport {
                status: shutdown.status(),
                diagnostics: Vec::new()
            }
        );
        assert_restored(&state);
        assert!(state.borrow().log.ends_with(RESTORE));
    }
    for error in [
        "input EOF",
        "input hangup",
        "input error",
        "output error",
        "clock error",
    ] {
        let (mut session, state) = Script::new();
        let report = guarded(&mut session, |session| {
            session.initialize().map_err(|error| error.to_string())?;
            Err(error.to_owned().into())
        });
        assert_eq!(
            report,
            ExitReport {
                status: 1,
                diagnostics: vec![error.to_owned()]
            }
        );
        assert!(state.borrow().log.ends_with(RESTORE));
        assert_restored(&state);
    }
}

#[test]
fn multiple_cleanup_failures_preserve_selected_status_and_attempt_every_operation() {
    let _lock = HOOK_LOCK.lock().unwrap();
    for result in [
        Ok(Shutdown::Quit),
        Ok(Shutdown::Interrupt),
        Ok(Shutdown::Terminate),
        Err(Failure::from("runtime failure".to_owned())),
    ] {
        let (mut session, state) = Script::new();
        let selected = result.as_ref().map_or(1, |shutdown| shutdown.status());
        let report = guarded(&mut session, |session| {
            session.initialize().map_err(|error| error.to_string())?;
            state.borrow_mut().log.clear();
            for &operation in &RESTORE[..RESTORE.len() - 1] {
                fault(&state, operation, &[Step::Error]);
            }
            result
        });
        assert_eq!(report.status, if selected == 0 { 1 } else { selected });
        assert_eq!(state.borrow().log, RESTORE);
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|error| error.starts_with("terminal cleanup:"))
                .count(),
            5
        );
        assert!(!state.borrow().registered);
        let log = state.borrow().log.clone();
        drop(session);
        assert_eq!(state.borrow().log, log);
    }
}

#[test]
fn cleanup_backpressure_is_bounded_and_does_not_skip_os_restoration() {
    let (mut session, state) = Script::new();
    session.initialize().unwrap();
    state.borrow_mut().log.clear();
    fault(&state, "show", &[Step::Short(2), Step::WouldBlock]);
    fault(&state, "wait", &[Step::Error]);
    let errors = session.cleanup();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("show cursor:"));
    assert_eq!(
        state.borrow().log,
        [
            "show",
            "show",
            "wait",
            "leave",
            "restore-termios",
            "restore-input",
            "restore-output",
            "release"
        ]
    );
    assert_restored(&state);
}

#[test]
fn drop_fallback_restores_an_owner_without_explicit_cleanup() {
    let (mut session, state) = Script::new();
    session.initialize().unwrap();
    state.borrow_mut().log.clear();
    drop(session);
    assert_eq!(state.borrow().log, RESTORE);
    assert_restored(&state);
}

#[test]
fn drop_during_unwinding_survives_an_additional_restoration_panic() {
    let _lock = HOOK_LOCK.lock().unwrap();
    let hook = QuietHook::install();
    let (mut session, state) = Script::new();
    let result = catch_unwind(AssertUnwindSafe(|| {
        session.initialize().unwrap();
        state.borrow_mut().log.clear();
        fault(&state, "show", &[Step::Panic]);
        let _owner = session;
        panic!("unwind through session owner");
    }));
    drop(hook);
    assert!(result.is_err());
    assert_eq!(state.borrow().log, RESTORE);
    assert_restored(&state);
}

#[test]
fn pre_session_startup_errors_keep_argument_and_failure_statuses_without_effects() {
    let _lock = HOOK_LOCK.lock().unwrap();
    for (failure, expected_status) in [
        (
            Failure::argument("requested interface is absent".to_owned()),
            2,
        ),
        (Failure::from("enumeration failed".to_owned()), 1),
    ] {
        let (mut session, state) = Script::new();
        let report = guarded(&mut session, |_| Err(failure));
        assert_eq!(report.status, expected_status);
        assert_eq!(report.diagnostics.len(), 1);
        assert!(state.borrow().log.is_empty());
        assert_restored(&state);
    }
}

#[test]
fn interrupted_cleanup_wait_resumes_within_the_same_bounded_write() {
    let (mut session, state) = Script::new();
    session.initialize().unwrap();
    fault(
        &state,
        "show",
        &[Step::WouldBlock, Step::Short(2), Step::WouldBlock],
    );
    fault(&state, "wait", &[Step::Interrupted]);
    assert!(session.cleanup().is_empty());
    assert_eq!(
        state.borrow().emitted,
        [ENTER_SCREEN, HIDE_CURSOR, SHOW_CURSOR, LEAVE_SCREEN].concat()
    );
    assert_restored(&state);
}

#[test]
fn caught_initialization_runtime_and_cleanup_panics_restore_the_prior_hook() {
    let _lock = HOOK_LOCK.lock().unwrap();
    let calls = std::sync::Arc::new(AtomicUsize::new(0));
    let original = std::panic::take_hook();
    let observed = std::sync::Arc::clone(&calls);
    std::panic::set_hook(Box::new(move |_| {
        observed.fetch_add(1, Ordering::SeqCst);
    }));
    let restore_hook = QuietHook(Some(original));
    for initialization_panic in [true, false] {
        let (mut session, state) = Script::new();
        if initialization_panic {
            fault(&state, "hide", &[Step::Panic]);
        }
        let report = guarded(&mut session, |session| {
            session.initialize().map_err(|error| error.to_string())?;
            panic!("runtime panic");
        });
        assert_eq!(report.status, 1);
        assert!(report.diagnostics[0].starts_with("caught panic:"));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(state.borrow().log.ends_with(RESTORE));
        assert_restored(&state);
    }
    let (mut session, state) = Script::new();
    let report = guarded(&mut session, |session| {
        session.initialize().map_err(|error| error.to_string())?;
        fault(&state, "show", &[Step::Panic]);
        fault(&state, "restore-termios", &[Step::Panic]);
        Ok(Shutdown::Interrupt)
    });
    assert_eq!(report.status, 130);
    assert_eq!(report.diagnostics.len(), 2);
    assert!(state.borrow().log.ends_with(RESTORE));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let _ = catch_unwind(|| panic!("probe restored hook"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(restore_hook);
}
