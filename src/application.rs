// SPDX-License-Identifier: GPL-2.0-only

//! Startup orchestration and the single-threaded sampling/redraw loop.
//! Effects stay in the counter, clock and terminal adapters; statistics and rows
//! remain pure. The same orchestration runs with scripted adapters in tests.

use std::ffi::OsString;
use std::io::{self, Write};

use crate::arguments::{self, Invocation, RunConfig};
use crate::monitor::Monitor;
use crate::proc_sampler::{CounterSource, ProcSampler, counter_outcome};
use crate::screen;
use crate::terminal::{
    Clock, Event, ExitReport, Failure, LinuxClock, LinuxTerminal, Ready, Runtime, RuntimeEffects,
    Session, SessionOps, Shutdown, guarded,
};

const SAMPLE_NS: u64 = 500_000_000;

/// Parse before invoking any interactive effects. Diagnostics are emitted only
/// after the interactive owner has restored terminal state and the panic hook.
pub(crate) fn invoke(
    tokens: impl IntoIterator<Item = OsString>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    interactive: impl FnOnce(RunConfig) -> ExitReport,
) -> u8 {
    let report = match arguments::parse(tokens) {
        Ok(Invocation::Help) => informational(stdout, arguments::help_text()),
        Ok(Invocation::Version) => informational(stdout, arguments::version_text()),
        Ok(Invocation::Run(config)) => interactive(config),
        Err(error) => ExitReport {
            status: 2,
            diagnostics: vec![error.to_string()],
        },
    };
    for diagnostic in report.diagnostics {
        let _ = writeln!(stderr, "nload-rs: {diagnostic}");
    }
    report.status
}

fn informational(stdout: &mut impl Write, text: String) -> ExitReport {
    match stdout
        .write_all(text.as_bytes())
        .and_then(|()| stdout.flush())
    {
        Ok(()) => ExitReport {
            status: 0,
            diagnostics: Vec::new(),
        },
        Err(error) => ExitReport {
            status: 1,
            diagnostics: vec![format!("informational output: {error}")],
        },
    }
}

pub(crate) fn linux(config: RunConfig) -> ExitReport {
    let mut counters = ProcSampler::default();
    let mut session = Session::new(LinuxTerminal::new());
    interactive(
        config,
        &mut counters,
        &mut session,
        LinuxClock::new,
        |session, clock, monitor, counters, deadline, dimensions| {
            drive(
                monitor,
                counters,
                &mut session.runtime(clock),
                deadline,
                dimensions,
            )
            .map_err(Failure::from)
        },
    )
}

/// One guarded startup path, including clock creation, discovery and execution.
/// The execution closure only selects the concrete borrowing runtime adapter.
fn interactive<C: CounterSource, K: Clock, O: SessionOps>(
    config: RunConfig,
    counters: &mut C,
    session: &mut Session<O>,
    initialize_clock: impl FnOnce() -> io::Result<K>,
    execute: impl FnOnce(
        &mut Session<O>,
        &mut K,
        &mut Monitor,
        &mut C,
        u64,
        (usize, usize),
    ) -> Result<Shutdown, Failure>,
) -> ExitReport {
    guarded(session, |session| {
        let mut clock = initialize_clock()?;
        let snapshot = counters
            .acquire()
            .map_err(|error| Failure::from(format!("interface enumeration: {error}")))?;
        let completed = clock.now()?;
        let startup = snapshot
            .startup(config.interface.as_deref())
            .map_err(|error| Failure::from(error.to_string()))?;
        let deadline = completed
            .checked_add(SAMPLE_NS)
            .ok_or_else(|| io::Error::other("sample deadline overflow"))?;
        let round = Ok(snapshot);
        let mut monitor = Monitor::new(startup, config.ceilings, completed, |name| {
            counter_outcome(&round, name)
        });
        let dimensions = session.initialize()?;
        execute(
            session,
            &mut clock,
            &mut monitor,
            counters,
            deadline,
            dimensions,
        )
    })
}

/// At each batch boundary drain/inspect shutdown before consulting deadlines.
/// Completion timestamps advance missed slots without catch-up observations.
fn checkpoint<C: CounterSource, O: RuntimeEffects>(
    monitor: &mut Monitor,
    counters: &mut C,
    runtime: &mut Runtime<O>,
    deadline: &mut u64,
    dirty: &mut bool,
    resized: &mut bool,
) -> io::Result<Option<Shutdown>> {
    let notifications = runtime.notifications()?;
    if notifications.shutdown.is_some() {
        return Ok(notifications.shutdown);
    }
    *resized |= notifications.resized;
    if runtime.now()? >= *deadline {
        let round = counters.acquire();
        let completed = runtime.now()?;
        monitor.observe(completed, |name| counter_outcome(&round, name));
        let slots = completed
            .checked_sub(*deadline)
            .and_then(|elapsed| (elapsed / SAMPLE_NS).checked_add(1))
            .and_then(|slots| slots.checked_mul(SAMPLE_NS))
            .ok_or_else(|| io::Error::other("sample deadline overflow or clock regression"))?;
        *deadline = deadline
            .checked_add(slots)
            .ok_or_else(|| io::Error::other("sample deadline overflow"))?;
        *dirty = true;
        let notifications = runtime.notifications()?;
        if notifications.shutdown.is_some() {
            return Ok(notifications.shutdown);
        }
        *resized |= notifications.resized;
    }
    Ok(None)
}

fn drive<C: CounterSource, O: RuntimeEffects>(
    monitor: &mut Monitor,
    counters: &mut C,
    runtime: &mut Runtime<O>,
    mut deadline: u64,
    mut dimensions: (usize, usize),
) -> io::Result<Shutdown> {
    let mut dirty = true;
    let mut resized = false;
    let mut ready = Ready::default();
    loop {
        if let Some(shutdown) = checkpoint(
            monitor,
            counters,
            runtime,
            &mut deadline,
            &mut dirty,
            &mut resized,
        )? {
            return Ok(shutdown);
        }
        let events = runtime.input_batch(ready.input)?;
        // A shutdown in this bounded read prevents navigation/rendering as well.
        let typed_shutdown = events
            .iter()
            .filter_map(|event| match event {
                Event::Shutdown(shutdown) => Some(*shutdown),
                _ => None,
            })
            .min_by_key(|shutdown| match shutdown {
                Shutdown::Interrupt => 0,
                Shutdown::Terminate => 1,
                Shutdown::Quit => 2,
            });
        if let Some(shutdown) = typed_shutdown {
            return Ok(runtime.notifications()?.shutdown.unwrap_or(shutdown));
        }
        for event in events {
            match event {
                Event::Forward => monitor.forward(),
                Event::Backward => monitor.backward(),
                Event::Shutdown(_) => unreachable!(),
            }
            dirty = true;
        }
        if let Some(shutdown) = checkpoint(
            monitor,
            counters,
            runtime,
            &mut deadline,
            &mut dirty,
            &mut resized,
        )? {
            return Ok(shutdown);
        }
        if std::mem::take(&mut resized) {
            dimensions = runtime.dimensions()?;
            dirty = true;
        }
        // Check signals again after dimension I/O and before any screen work.
        let notifications = runtime.notifications()?;
        if let Some(shutdown) = notifications.shutdown {
            return Ok(shutdown);
        }
        resized |= notifications.resized;
        if dirty {
            let rows = screen::render(monitor.selected_view(), dimensions.0, dimensions.1);
            if runtime.queue_frame(&rows, dimensions.0, dimensions.1)? {
                dirty = false;
            }
        }
        if ready.output {
            runtime.output_batch()?;
            if dirty && !runtime.has_pending_output() {
                // The emitted old frame is now complete; take the latest view on
                // the next batch, after another shutdown/deadline checkpoint.
                ready = Ready::default();
                continue;
            }
        }
        if resized {
            ready = Ready::default();
        } else {
            ready = runtime.wait(deadline)?;
        }
    }
}

#[cfg(test)]
mod tests;
