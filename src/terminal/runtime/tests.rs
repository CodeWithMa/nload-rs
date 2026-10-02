// SPDX-License-Identifier: GPL-2.0-only

use super::scripted::{Call, ScriptedEffects};
use super::*;

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
    runtime.queue_frame(&["obsolete".into()], 8, 1).unwrap();
    effects
        .script
        .borrow_mut()
        .writes
        .push_back(Err(io::ErrorKind::WouldBlock.into()));
    runtime.output_batch().unwrap();
    assert!(runtime.queue_frame(&["current".into()], 8, 1).unwrap());
    effects.script.borrow_mut().writes.push_back(Ok(2));
    runtime.output_batch().unwrap();
    assert!(!runtime.queue_frame(&["latest".into()], 8, 1).unwrap());
    effects
        .script
        .borrow_mut()
        .writes
        .push_back(Err(io::ErrorKind::Interrupted.into()));
    runtime.output_batch().unwrap();
    assert!(!runtime.queue_frame(&["latest".into()], 8, 1).unwrap());
    effects.script.borrow_mut().writes.push_back(Ok(3));
    runtime.output_batch().unwrap();
    runtime.output_batch().unwrap();
    assert!(!runtime.has_pending_output());
    assert_eq!(
        effects.script.borrow().emitted,
        b"\x1b[H\x1b[2J\x1b[1;1Hcurrent\x1b[H"
    );
    assert!(runtime.queue_frame(&["latest".into()], 8, 1).unwrap());
    runtime.output_batch().unwrap();
    assert_eq!(
        effects.script.borrow().emitted,
        b"\x1b[H\x1b[2J\x1b[1;1Hcurrent\x1b[H\x1b[H\x1b[2J\x1b[1;1Hlatest\x1b[H"
    );
}

#[test]
fn backpressure_keeps_clock_input_signals_resize_and_timer_effects_available() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    runtime.queue_frame(&["frame".into()], 5, 1).unwrap();
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
fn output_batches_are_finite_and_frames_reject_out_of_bounds_protocol_injection() {
    let effects = ScriptedEffects::default();
    let mut runtime = Runtime::new(effects.clone());
    for rows in [
        vec!["1234".into()],
        vec!["x".into(), "y".into()],
        vec!["\x1b".into()],
        vec!["é".into()],
        vec!["x\n".into()],
    ] {
        assert!(runtime.queue_frame(&rows, 3, 1).is_err());
    }
    runtime.queue_frame(&[], 0, 0).unwrap();
    runtime.output_batch().unwrap();
    assert_eq!(effects.script.borrow().emitted, b"\x1b[H\x1b[2J\x1b[H");
    effects.script.borrow_mut().emitted.clear();
    runtime
        .queue_frame(&["x".repeat(20_000), "y ".into()], 20_000, 2)
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
            .ends_with(b"\x1b[2;1Hy\x1b[H")
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
        runtime.queue_frame(&["x".into()], 1, 1).unwrap();
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
