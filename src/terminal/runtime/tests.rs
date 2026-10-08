// SPDX-License-Identifier: GPL-2.0-only

use super::scripted::{Call, ScriptedEffects};
use super::*;

fn frame(name: &[u8], width: usize, height: usize) -> crate::screen::Frame {
    use crate::arguments::Ceilings;
    use crate::monitor::Monitor;
    use crate::proc_sampler::{CounterOutcome, Counters, StartupSelection};
    let monitor = Monitor::new(
        StartupSelection {
            names: vec![name.to_vec()],
            selected: 0,
        },
        Ceilings::default(),
        0,
        |_| {
            CounterOutcome::Readable(Counters {
                incoming: 0,
                outgoing: 0,
            })
        },
    );
    crate::screen::render(monitor.selected_view(), width, height)
}

fn colored_frame(name: &[u8], width: usize, height: usize) -> crate::screen::Frame {
    use crate::arguments::Ceilings;
    use crate::monitor::Monitor;
    use crate::proc_sampler::{CounterOutcome, Counters, StartupSelection};
    let mut monitor = Monitor::new(
        StartupSelection {
            names: vec![name.to_vec()],
            selected: 0,
        },
        Ceilings {
            incoming: 24,
            outgoing: 24,
        },
        0,
        |_| {
            CounterOutcome::Readable(Counters {
                incoming: 0,
                outgoing: 0,
            })
        },
    );
    for (at, total) in [(1_000_000_000, 24), (2_000_000_000, 48)] {
        monitor.observe(at, |_| {
            CounterOutcome::Readable(Counters {
                incoming: total,
                outgoing: total,
            })
        });
    }
    crate::screen::render(monitor.selected_view(), width, height)
}

// The transport oracle takes the screen's public encoded rows as payload, with
// literal session-independent addressing from the wire contract. Screen glyph
// and palette correctness has separate exact oracles in screen::tests.
fn wire(frame: &crate::screen::Frame) -> Vec<u8> {
    let mut bytes = b"\x1b[0m\x1b[H\x1b[2J".to_vec();
    for (row, payload) in frame.encoded_rows().iter().enumerate() {
        bytes.extend_from_slice(format!("\x1b[{};1H", row + 1).as_bytes());
        bytes.extend_from_slice(payload.as_bytes());
    }
    bytes.extend_from_slice(b"\x1b[H");
    bytes
}

#[test]
fn every_colored_frame_byte_boundary_resumes_exactly_before_the_latest_frame() {
    let first = colored_frame(b"first", 60, 16);
    let latest = colored_frame(b"latest", 80, 24);
    let expected = wire(&first);
    assert!(first.encoded_rows()[3].ends_with("\x1b[38;5;183m⣿\x1b[0m\x1b[0m\x1b[0m"));
    for index in [60, 61, 98, 140, 146, 183, 96, 132, 133, 170, 176, 219] {
        assert!(String::from_utf8_lossy(&expected).contains(&format!("\x1b[38;5;{index}m")));
    }
    let complete = [expected.as_slice(), wire(&latest).as_slice()].concat();
    for transient in [io::ErrorKind::Interrupted, io::ErrorKind::WouldBlock] {
        // Every byte offset includes each interior byte of all twelve foreground
        // SGRs, every reset, all three UTF-8 glyph bytes and positioning commands.
        for split in 1..expected.len() {
            let effects = ScriptedEffects::default();
            let mut runtime = Runtime::new(effects.clone());
            assert!(runtime.queue_frame(&first).unwrap());
            effects.script.borrow_mut().writes.push_back(Ok(split));
            runtime.output_batch().unwrap();
            assert_eq!(effects.script.borrow().emitted, expected[..split]);
            assert!(!runtime.queue_frame(&latest).unwrap());
            effects
                .script
                .borrow_mut()
                .writes
                .push_back(Err(transient.into()));
            runtime.output_batch().unwrap();
            assert_eq!(effects.script.borrow().emitted, expected[..split]);
            assert_eq!(feed(&mut runtime, &effects, b"\x1b[C", 0), [Event::Forward]);
            assert!(!runtime.queue_frame(&latest).unwrap());
            // A second short write proves the offset survives multiple batches.
            effects.script.borrow_mut().writes.push_back(Ok(1));
            runtime.output_batch().unwrap();
            runtime.output_batch().unwrap();
            assert!(!runtime.has_pending_output());
            assert_eq!(effects.script.borrow().emitted, expected);
            assert!(runtime.queue_frame(&latest).unwrap());
            runtime.output_batch().unwrap();
            assert_eq!(
                effects.script.borrow().emitted,
                complete,
                "{transient:?} at {split}"
            );
        }
    }
}

#[test]
fn untouched_colored_frames_replace_after_transients_and_large_frames_yield_each_batch() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    for (name, transient) in [
        (b"obsolete".as_slice(), io::ErrorKind::WouldBlock),
        (b"also obsolete".as_slice(), io::ErrorKind::Interrupted),
    ] {
        assert!(runtime.queue_frame(&colored_frame(name, 60, 16)).unwrap());
        effects
            .script
            .borrow_mut()
            .writes
            .push_back(Err(transient.into()));
        runtime.output_batch().unwrap();
        assert!(effects.script.borrow().emitted.is_empty());
    }
    let latest = colored_frame(b"latest", 1000, 1000);
    let expected = wire(&latest);
    assert!(expected.len() > 16 * 1024);
    assert!(runtime.queue_frame(&latest).unwrap());
    for _ in 0..expected.len().div_ceil(16 * 1024) {
        assert!(runtime.has_pending_output());
        let before = effects.script.borrow().calls.len();
        runtime.output_batch().unwrap();
        assert_eq!(effects.script.borrow().calls.len(), before + 1);
        let script = effects.script.borrow();
        let Some(Call::Write(bytes)) = script.calls.last() else {
            panic!("missing write")
        };
        assert!(bytes.len() <= 16 * 1024);
        drop(script);
        assert_eq!(
            feed(&mut runtime, &effects, b"\x1b[D", 0),
            [Event::Backward]
        );
    }
    assert!(!runtime.has_pending_output());
    assert_eq!(effects.script.borrow().emitted, expected);
}

#[test]
fn screen_frames_clear_in_default_style_and_keep_row_resets_and_cell_bounds() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    for (width, height, expected) in [
        (1, 1, "\x1b[0m\x1b[H\x1b[2J\x1b[1;1H\x1b[0mT\x1b[0m\x1b[H"),
        (0, 5, "\x1b[0m\x1b[H\x1b[2J\x1b[H"),
        (80, 0, "\x1b[0m\x1b[H\x1b[2J\x1b[H"),
    ] {
        effects.script.borrow_mut().emitted.clear();
        assert!(
            runtime
                .queue_frame(&frame(b"eth\x1b[31m", width, height))
                .unwrap()
        );
        runtime.output_batch().unwrap();
        assert_eq!(effects.script.borrow().emitted, expected.as_bytes());
    }
}

#[test]
fn every_prefactor_protocol_byte_boundary_resumes_exactly_without_interleaving() {
    let expected = b"\x1b[0m\x1b[H\x1b[2J\x1b[1;1H\x1b[0mT\x1b[0m\x1b[H";
    for split in 1..expected.len() {
        let effects = ScriptedEffects::default();
        let mut runtime = Runtime::new(effects.clone());
        runtime.queue_frame(&frame(b"eth0", 1, 1)).unwrap();
        effects.script.borrow_mut().writes.push_back(Ok(split));
        runtime.output_batch().unwrap();
        assert_eq!(effects.script.borrow().emitted, expected[..split]);
        assert!(!runtime.queue_frame(&frame(b"latest", 60, 16)).unwrap());
        effects
            .script
            .borrow_mut()
            .writes
            .push_back(Err(io::ErrorKind::WouldBlock.into()));
        runtime.output_batch().unwrap();
        assert_eq!(feed(&mut runtime, &effects, b"\x1b[C", 0), [Event::Forward]);
        runtime.output_batch().unwrap();
        assert!(!runtime.has_pending_output());
        assert_eq!(effects.script.borrow().emitted, expected);
    }
}

fn feed(
    runtime: &mut Runtime<ScriptedEffects>,
    effects: &ScriptedEffects,
    bytes: &[u8],
    time: u64,
) -> Vec<Event> {
    effects.clock.0.borrow_mut().time = time;
    effects
        .script
        .borrow_mut()
        .reads
        .push_back(Ok(bytes.to_vec()));
    runtime.input_batch(true).unwrap()
}

#[test]
fn every_csi_and_ss3_fragment_boundary_yields_one_ordered_arrow() {
    for introducer in [b'[', b'O'] {
        for (final_byte, expected) in [
            (b'A', Event::Backward),
            (b'B', Event::Forward),
            (b'C', Event::Forward),
            (b'D', Event::Backward),
        ] {
            let sequence = [27, introducer, final_byte];
            for split in 1..3 {
                let effects = ScriptedEffects::default();
                let mut runtime = Runtime::new(effects.clone());
                assert!(feed(&mut runtime, &effects, &sequence[..split], 0).is_empty());
                assert_eq!(
                    feed(&mut runtime, &effects, &sequence[split..], 99_999_999),
                    [expected]
                );
                assert!(runtime.input_batch(false).unwrap().is_empty());
            }
        }
    }
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    assert!(feed(&mut runtime, &effects, b"\x1b", 0).is_empty());
    assert!(feed(&mut runtime, &effects, b"[", 50_000_000).is_empty());
    assert_eq!(
        feed(
            &mut runtime,
            &effects,
            b"C\x1b[D\x1bOB\x1bOA\x1b[C",
            149_999_999
        ),
        [
            Event::Forward,
            Event::Backward,
            Event::Forward,
            Event::Backward,
            Event::Forward
        ]
    );
}

#[test]
fn typed_interrupt_does_not_wait_behind_an_incomplete_escape_sequence() {
    for prefix in [
        b"\x1b".as_slice(),
        b"\x1b[1;",
        b"\x1bO",
        b"\x1b]payload",
        b"\x1bPpayload",
    ] {
        let effects = ScriptedEffects::default();
        let mut runtime = Runtime::new(effects.clone());
        assert!(feed(&mut runtime, &effects, prefix, 0).is_empty());
        assert_eq!(
            feed(&mut runtime, &effects, b"\x03", 1),
            [Event::Shutdown(Shutdown::Interrupt)]
        );
    }
}

#[test]
fn unknown_modified_and_string_sequences_consume_command_looking_bytes() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    for sequence in [
        b"\x1b[1;5C".as_slice(),
        b"\x1bO1;2A",
        b"\x1b[5~",
        b"\x1b[6~",
        b"\x1b[q",
        b"\x1b[Q",
        b"\x1bq",
        b"\x1bQ",
        b"\x1b q",
        b"\t\r\n\x1a",
        b"\x1b]qQ\x1b[A\x07",
        b"\x1bPqQ\x1b[B\x1b\\",
        b"\x1bXqQ\x1b\\",
        b"\x1b^qQ\x1b\\",
        b"\x1b_qQ\x1b\\",
    ] {
        // Every byte separately, including string ST split after ESC.
        for byte in sequence {
            assert!(
                feed(&mut runtime, &effects, &[*byte], 0).is_empty(),
                "{sequence:?}"
            );
        }
    }
    assert_eq!(
        feed(&mut runtime, &effects, b"qQ\x03", 0),
        [
            Event::Shutdown(Shutdown::Quit),
            Event::Shutdown(Shutdown::Quit),
            Event::Shutdown(Shutdown::Interrupt)
        ]
    );
}

#[test]
fn idle_timeout_is_a_wait_deadline_and_resets_at_exact_boundary() {
    for gap in [99_999_999, 100_000_000, 100_000_001] {
        let effects = ScriptedEffects::default();
        let mut runtime = Runtime::new(effects.clone());
        assert!(feed(&mut runtime, &effects, b"\x1b[", 1).is_empty());
        runtime.wait(500_000_001).unwrap();
        assert_eq!(
            effects.script.borrow().calls.last(),
            Some(&Call::Wait {
                deadline: 500_000_001,
                idle_ns: Some(100_000_000),
                output: false
            })
        );
        let events = feed(&mut runtime, &effects, b"q", gap + 1);
        assert_eq!(
            events,
            if gap < 100_000_000 {
                vec![]
            } else {
                vec![Event::Shutdown(Shutdown::Quit)]
            }
        );
        runtime.wait(500_000_001).unwrap();
        assert_eq!(
            effects.script.borrow().calls.last(),
            Some(&Call::Wait {
                deadline: 500_000_001,
                idle_ns: None,
                output: false
            })
        );
    }
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    feed(&mut runtime, &effects, b"\x1b[", 0);
    effects.clock.0.borrow_mut().time = 100_000_000;
    runtime.wait(500_000_000).unwrap();
    assert_eq!(
        effects.script.borrow().calls.last(),
        Some(&Call::Wait {
            deadline: 500_000_000,
            idle_ns: Some(0),
            output: false
        })
    );
    let before = effects.script.borrow().calls.len();
    assert!(runtime.input_batch(false).unwrap().is_empty());
    assert_eq!(
        effects.script.borrow().calls.len(),
        before,
        "timeout performs no terminal I/O or read"
    );
    assert_eq!(
        feed(&mut runtime, &effects, b"Q", 100_000_000),
        [Event::Shutdown(Shutdown::Quit)]
    );
}

#[test]
fn sustained_input_and_overlong_sequences_are_bounded_and_discarded() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    let mut bytes = b"\x1b[".to_vec();
    bytes.extend(std::iter::repeat_n(b'1', 10_000));
    bytes.extend_from_slice(b"q\x1b[CQ");
    effects.script.borrow_mut().reads.push_back(Ok(bytes));
    let mut events = Vec::new();
    while !effects.script.borrow().reads.is_empty() {
        events.extend(runtime.input_batch(true).unwrap());
    }
    assert_eq!(events, [Event::Forward, Event::Shutdown(Shutdown::Quit)]);
    assert!(
        effects
            .script
            .borrow()
            .calls
            .iter()
            .all(|call| matches!(call, Call::Read(256)))
    );
    // Overflow does not restore plain mode until idle timeout, either.
    feed(
        &mut runtime,
        &effects,
        b"\x1b[1111111111111111111111111111111111111111111111111111111111111111111111",
        0,
    );
    assert_eq!(
        feed(&mut runtime, &effects, b"q", 100_000_000),
        [Event::Shutdown(Shutdown::Quit)]
    );
}

#[test]
fn untouched_frames_replace_but_partial_frames_finish_without_duplicate_bytes() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    runtime.queue_frame(&frame(b"obsolete", 32, 2)).unwrap();
    effects
        .script
        .borrow_mut()
        .writes
        .push_back(Err(io::ErrorKind::WouldBlock.into()));
    runtime.output_batch().unwrap();
    assert!(runtime.queue_frame(&frame(b"current", 32, 2)).unwrap());
    effects.script.borrow_mut().writes.push_back(Ok(2));
    runtime.output_batch().unwrap();
    assert!(!runtime.queue_frame(&frame(b"latest", 32, 2)).unwrap());
    effects
        .script
        .borrow_mut()
        .writes
        .push_back(Err(io::ErrorKind::Interrupted.into()));
    runtime.output_batch().unwrap();
    assert!(!runtime.queue_frame(&frame(b"latest", 32, 2)).unwrap());
    effects.script.borrow_mut().writes.push_back(Ok(3));
    runtime.output_batch().unwrap();
    runtime.output_batch().unwrap();
    assert!(!runtime.has_pending_output());
    assert_eq!(
        effects.script.borrow().emitted,
        b"\x1b[0m\x1b[H\x1b[2J\x1b[1;1H\x1b[0mTerminal too small (need 60x16)\x1b[0m\x1b[2;1H\x1b[0mInterface current (1/1)\x1b[0m\x1b[H"
    );
    assert!(runtime.queue_frame(&frame(b"latest", 32, 2)).unwrap());
    runtime.output_batch().unwrap();
    assert_eq!(
        effects.script.borrow().emitted,
        b"\x1b[0m\x1b[H\x1b[2J\x1b[1;1H\x1b[0mTerminal too small (need 60x16)\x1b[0m\x1b[2;1H\x1b[0mInterface current (1/1)\x1b[0m\x1b[H\x1b[0m\x1b[H\x1b[2J\x1b[1;1H\x1b[0mTerminal too small (need 60x16)\x1b[0m\x1b[2;1H\x1b[0mInterface latest (1/1)\x1b[0m\x1b[H"
    );
}

#[test]
fn backpressure_keeps_clock_input_signals_resize_and_timer_effects_available() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    runtime.queue_frame(&frame(b"eth0", 60, 16)).unwrap();
    for time in [0, 500_000_000, 1_000_000_000] {
        effects.clock.0.borrow_mut().time = time;
        effects
            .script
            .borrow_mut()
            .writes
            .push_back(Err(io::ErrorKind::WouldBlock.into()));
        runtime.output_batch().unwrap();
        effects.script.borrow_mut().waits.push_back(Ok(Ready {
            input: true,
            signals: true,
            timer: true,
            output: false,
        }));
        let ready = runtime.wait(time + 500_000_000).unwrap();
        assert!(ready.input && ready.signals && ready.timer && !ready.output);
        assert_eq!(
            feed(&mut runtime, &effects, b"\x1b[C", time),
            [Event::Forward]
        );
        assert_eq!(runtime.dimensions().unwrap(), (80, 24));
        assert_eq!(runtime.now().unwrap(), time);
    }
    effects
        .script
        .borrow_mut()
        .notifications
        .push_back(Ok(Notifications {
            shutdown: Some(Shutdown::Interrupt),
            resized: true,
        }));
    assert_eq!(
        runtime.notifications().unwrap(),
        Notifications {
            shutdown: Some(Shutdown::Interrupt),
            resized: true
        }
    );
    assert!(runtime.has_pending_output());
    assert!(effects.script.borrow().emitted.is_empty());
    // Every wait includes pending output readiness, without a write-only wait.
    assert!(
        effects
            .script
            .borrow()
            .calls
            .iter()
            .filter_map(|call| match call {
                Call::Wait { output, .. } => Some(*output),
                _ => None,
            })
            .all(|output| output)
    );
}

#[test]
fn output_batches_are_finite_for_screen_constructed_frames() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    runtime.queue_frame(&frame(b"eth0", 0, 0)).unwrap();
    runtime.output_batch().unwrap();
    assert_eq!(
        effects.script.borrow().emitted,
        b"\x1b[0m\x1b[H\x1b[2J\x1b[H"
    );
    effects.script.borrow_mut().emitted.clear();
    runtime
        .queue_frame(&frame(&vec![b'x'; 20_000], 20_030, 2))
        .unwrap();
    runtime.output_batch().unwrap();
    assert_eq!(effects.script.borrow().emitted.len(), 16 * 1024);
    assert!(runtime.has_pending_output());
    runtime.output_batch().unwrap();
    assert!(!runtime.has_pending_output());
    assert!(
        effects
            .script
            .borrow()
            .emitted
            .ends_with(b" (1/1)\x1b[0m\x1b[H")
    );
}

#[test]
fn input_eof_and_effect_faults_are_fatal_while_transient_io_yields() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    for kind in [io::ErrorKind::Interrupted, io::ErrorKind::WouldBlock] {
        effects
            .script
            .borrow_mut()
            .reads
            .push_back(Err(kind.into()));
        assert!(runtime.input_batch(true).unwrap().is_empty());
    }
    effects.script.borrow_mut().reads.push_back(Ok(vec![]));
    assert_eq!(
        runtime.input_batch(true).unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
    effects
        .script
        .borrow_mut()
        .reads
        .push_back(Err(io::ErrorKind::BrokenPipe.into()));
    assert_eq!(
        runtime.input_batch(true).unwrap_err().kind(),
        io::ErrorKind::BrokenPipe
    );
    for result in [Ok(0), Ok(usize::MAX), Err(io::ErrorKind::BrokenPipe.into())] {
        // Each fatal result has its own owner: a started frame cannot be replaced.
        let mut runtime = Runtime::new(effects.clone());
        runtime.queue_frame(&frame(b"eth0", 1, 1)).unwrap();
        effects.script.borrow_mut().writes.push_back(result);
        assert!(runtime.output_batch().is_err());
    }
    effects
        .script
        .borrow_mut()
        .waits
        .push_back(Err(io::Error::other("poll fault")));
    assert!(runtime.wait(500_000_000).is_err());
    effects
        .script
        .borrow_mut()
        .notifications
        .push_back(Err(io::Error::other("signal drain fault")));
    assert!(runtime.notifications().is_err());
    effects
        .script
        .borrow_mut()
        .dimensions
        .push_back(Err(io::Error::other("ioctl fault")));
    assert!(runtime.dimensions().is_err());
}

#[test]
fn clock_failure_regression_and_idle_deadline_overflow_have_no_fallback() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    assert_eq!(runtime.now().unwrap(), 0);
    effects
        .clock
        .0
        .borrow_mut()
        .readings
        .push_back(Err(io::Error::other("clock fault")));
    assert!(runtime.wait(500_000_000).is_err());
    assert!(
        effects.script.borrow().calls.is_empty(),
        "failed clock never enters wait"
    );
    effects.clock.0.borrow_mut().time = 10;
    assert_eq!(runtime.now().unwrap(), 10);
    effects.clock.0.borrow_mut().time = 9;
    assert!(runtime.now().is_err());
    effects.clock.0.borrow_mut().time = u64::MAX;
    effects.script.borrow_mut().reads.push_back(Ok(vec![27]));
    assert!(runtime.input_batch(true).is_err());
}
