// SPDX-License-Identifier: GPL-2.0-only

//! Pure selected-monitor-view and dimensions to bounded ASCII screen rows.
//!
//! Owns geometry, escaping, formatting, graph glyphs and small-terminal mode.
//! Does not mutate the monitor or emit terminal escape sequences.

use crate::monitor::{DirectionView, SelectedView, ViewState};

const MIN_WIDTH: usize = 60;
const MIN_HEIGHT: usize = 16;
const STAT_WIDTH: usize = 28;
const GAP: usize = 2;
const RATE_UNITS: [&str; 4] = ["Bit/s", "kBit/s", "MBit/s", "GBit/s"];
const TOTAL_UNITS: [&str; 4] = ["Byte", "kByte", "MByte", "GByte"];

/// Owned, printable ASCII rows, exactly `height` rows of `width` bytes each.
/// No newlines, control bytes or terminal protocol; even zero dimensions are valid.
/// Views come from Monitor (positive ceilings, finite nonnegative rates and paired
/// availability invariants). Dimensions affect presentation only, never history.
pub(crate) fn render(view: SelectedView<'_>, width: usize, height: usize) -> Vec<String> {
    let mut rows = vec![vec![b' '; width]; height];
    if width < MIN_WIDTH || height < MIN_HEIGHT {
        for (row, text) in rows.iter_mut().zip([
            "Terminal too small (need 60x16)".to_owned(),
            header(view, width),
            status(view.state).unwrap_or("Measured").to_owned(),
        ]) {
            put(row, 0, &text);
        }
    } else {
        put(&mut rows[0], 0, &header(view, width));
        if let Some(text) = status(view.state) {
            put(&mut rows[1], 0, text);
        } else {
            rows[1].fill(b'=');
        }
        let incoming_height = (height - 4) / 2;
        let outgoing_label = 3 + incoming_height;
        direction(&mut rows[2..outgoing_label], "Incoming", view.incoming);
        direction(&mut rows[outgoing_label..], "Outgoing", view.outgoing);
    }
    rows.into_iter()
        .map(|row| String::from_utf8(row).expect("screen rows contain only ASCII"))
        .collect()
}

fn status(state: ViewState) -> Option<&'static str> {
    match state {
        ViewState::Baseline => Some("Waiting for rate sample"),
        ViewState::Absent => Some("Unavailable: interface absent"),
        ViewState::ReadFailed => Some("Unavailable: counter read failed"),
        ViewState::Measured => None,
    }
}

fn direction(rows: &mut [Vec<u8>], label: &str, view: DirectionView<'_>) {
    // Ceilings are exact integer bytes/s, so use widened integer bit conversion.
    put(
        &mut rows[0],
        0,
        &format!(
            "{label}: (ceiling {})",
            format_integer(u128::from(view.ceiling) * 8, &RATE_UNITS)
        ),
    );
    let graph = &mut rows[1..];
    let height = graph.len();
    let graph_width = graph[0].len() - STAT_WIDTH - GAP;
    let fitted = view.samples.len().min(graph_width);
    for (column, sample) in view
        .samples
        .iter()
        .skip(view.samples.len() - fitted)
        .enumerate()
    {
        let ceiling = view.ceiling as f64;
        let filled = if sample.rate >= ceiling {
            height as f64
        } else {
            // Scale before dividing: division first can move an exact 1.70-cell
            // fill below its threshold (e.g. 10 * (1.7 / 10)). Monitor rates and
            // terminal dimensions are far below floating-point overflow here.
            height as f64 * sample.rate / ceiling
        };
        for (row, cells) in graph.iter_mut().enumerate() {
            let level = (height - 1 - row) as f64;
            // Compare against thresholds in whole-bar coordinates. Subtracting
            // the integer level first could turn exactly 1.70 into 0.69999… .
            cells[graph_width - fitted + column] = if filled >= level + 1.0 {
                b'#'
            } else if filled >= level + 0.70 {
                b'|'
            } else if filled >= level + 0.30 {
                b'.'
            } else {
                b' '
            };
        }
    }

    let rates = view.rates;
    let values = [
        rates.map(|r| format_rate(r.current)),
        rates.map(|r| format_rate(r.average)),
        rates.map(|r| format_rate(r.minimum)),
        rates.map(|r| format_rate(r.maximum)),
        view.total
            .map(|total| format_integer(u128::from(total), &TOTAL_UNITS)),
    ];
    for ((row, label), value) in graph[height - 5..]
        .iter_mut()
        .zip(["Curr", "Avg", "Min", "Max", "Ttl"])
        .zip(values)
    {
        let field = format!("{label}: ");
        let value = value.as_deref().unwrap_or("--");
        let value = if value.len() <= STAT_WIDTH - field.len() {
            value
        } else {
            "[too wide]"
        };
        put(row, graph_width + GAP, &field);
        put(row, graph_width + GAP + field.len(), value);
    }
}

/// Copy already-safe ASCII, clipping only presentation strings, never numbers.
fn put(row: &mut [u8], column: usize, text: &str) {
    let count = text.len().min(row.len().saturating_sub(column));
    row[column..column + count].copy_from_slice(&text.as_bytes()[..count]);
}

fn header(view: SelectedView<'_>, width: usize) -> String {
    let position = format!("({}/{})", view.position, view.count);
    let prefix = "Interface ";
    let reserved = prefix.len() + 1 + position.len();
    if width >= reserved {
        format!(
            "{prefix}{} {position}",
            escaped_name(view.name, width - reserved)
        )
    } else if width >= position.len() {
        position
    } else {
        // Extremely narrow rows cannot hold the position. Keep selection bytes
        // visible where possible; escape chunks are never split by shortening.
        escaped_name(view.name, width)
    }
}

fn escaped_name(name: &[u8], width: usize) -> String {
    let escaped_length: usize = name
        .iter()
        .map(|byte| if (0x20..=0x7e).contains(byte) { 1 } else { 4 })
        .sum();
    let shortened = escaped_length > width;
    let ellipsis = shortened && width >= 3;
    let budget = if ellipsis { width - 3 } else { width };
    let mut text = String::new();
    for &byte in name {
        let chunk = if (0x20..=0x7e).contains(&byte) {
            char::from(byte).to_string()
        } else {
            format!("\\x{byte:02X}")
        };
        if text.len() + chunk.len() > budget {
            break;
        }
        text.push_str(&chunk);
    }
    if ellipsis {
        text.push_str("...");
    }
    text
}

fn prefix(value: f64) -> (usize, f64) {
    let mut factor = 1.0;
    let mut index = 0;
    while index < 3 && value / factor >= 1024.0 {
        index += 1;
        factor *= 1024.0;
    }
    (index, factor)
}

fn format_rate(bytes_per_second: f64) -> String {
    let bits = bytes_per_second * 8.0;
    let (index, factor) = prefix(bits);
    // Explicit positive halfway-up rounding, independent of the formatter's tie
    // rule. Monitor-derived rates (at most a u64 delta per nanosecond) fit u128
    // cents even at the capped G prefix. No integer bytes/s truncation occurs.
    let cents = (bits / factor * 100.0 + 0.5).floor() as u128;
    decimal(cents, RATE_UNITS[index])
}

fn format_integer(value: u128, units: &[&str; 4]) -> String {
    let mut factor = 1_u128;
    let mut index = 0;
    while index < 3 && value >= factor * 1024 {
        index += 1;
        factor *= 1024;
    }
    // Exact u64 totals and u64 ceilings * 8, including their hundredths and
    // rounding increments, remain safely within the widened integer domain.
    let cents = (value * 100 + factor / 2) / factor;
    decimal(cents, units[index])
}

fn decimal(cents: u128, unit: &str) -> String {
    format!("{}.{:02} {unit}", cents / 100, cents % 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    use crate::arguments::Ceilings;
    use crate::monitor::{GraphSample, Monitor, Rates};
    use crate::proc_sampler::{CounterOutcome, Counters, StartupSelection};

    fn samples(rates: &[f64]) -> VecDeque<GraphSample> {
        rates
            .iter()
            .enumerate()
            .map(|(index, &rate)| GraphSample {
                at: (index as u64 + 1) * 500_000_000,
                rate,
            })
            .collect()
    }

    fn view(history: &VecDeque<GraphSample>) -> SelectedView<'_> {
        let direction = DirectionView {
            total: Some(1024),
            rates: Some(Rates {
                current: 128.0,
                average: 256.0,
                minimum: 0.0,
                maximum: 512.0,
            }),
            samples: history,
            ceiling: 1_310_720,
        };
        SelectedView {
            name: b"eth0",
            position: 1,
            count: 3,
            state: ViewState::Measured,
            incoming: direction,
            outgoing: direction,
        }
    }

    fn bounded(rows: &[String], width: usize, height: usize) {
        assert_eq!(rows.len(), height);
        for row in rows {
            assert_eq!(row.len(), width);
            assert!(row.bytes().all(|byte| (0x20..=0x7e).contains(&byte)));
        }
    }

    fn stats(rows: &[String], end: usize, start: usize) -> Vec<&str> {
        rows[end - 5..end]
            .iter()
            .map(|row| row[start..].trim_end())
            .collect()
    }

    #[test]
    fn exact_80_by_24_stacked_rows_and_bottom_aligned_fields() {
        let history = samples(&[0.0, 655_360.0, 1_310_720.0, 2_621_440.0]);
        let rows = render(view(&history), 80, 24);
        let mut expected = vec![String::new(); 24];
        expected[0] = "Interface eth0 (1/3)".to_owned();
        expected[1] = "=".repeat(80);
        expected[2] = "Incoming: (ceiling 10.00 MBit/s)".to_owned();
        expected[13] = "Outgoing: (ceiling 10.00 MBit/s)".to_owned();
        for (start, end) in [(3, 13), (14, 24)] {
            for row in &mut expected[start..end - 5] {
                *row = format!("{}##", " ".repeat(48));
            }
            for (row, field) in expected[end - 5..end].iter_mut().zip([
                "Curr: 1.00 kBit/s",
                "Avg: 2.00 kBit/s",
                "Min: 0.00 Bit/s",
                "Max: 4.00 kBit/s",
                "Ttl: 1.00 kByte",
            ]) {
                *row = format!("{}###  {field}", " ".repeat(47));
            }
        }
        let expected: Vec<_> = expected
            .into_iter()
            .map(|row| format!("{row:<80}"))
            .collect();
        assert_eq!(rows, expected);
        bounded(&rows, 80, 24);
    }

    #[test]
    fn minimum_and_odd_geometry_keep_gap_fields_and_outgoing_extra_row() {
        let history = samples(&[1_310_720.0]);
        for (width, height, outgoing_label) in [(60, 16, 9), (61, 17, 9), (80, 25, 13)] {
            let rows = render(view(&history), width, height);
            bounded(&rows, width, height);
            assert_eq!(rows[2].trim_end(), "Incoming: (ceiling 10.00 MBit/s)");
            assert_eq!(
                rows[outgoing_label].trim_end(),
                "Outgoing: (ceiling 10.00 MBit/s)"
            );
            for (start, end) in [(3, outgoing_label), (outgoing_label + 1, height)] {
                for row in &rows[start..end] {
                    assert_eq!(row.as_bytes()[width - 31], b'#');
                    assert_eq!(&row[width - 30..width - 28], "  ");
                }
                assert_eq!(
                    stats(&rows, end, width - 28),
                    [
                        "Curr: 1.00 kBit/s",
                        "Avg: 2.00 kBit/s",
                        "Min: 0.00 Bit/s",
                        "Max: 4.00 kBit/s",
                        "Ttl: 1.00 kByte",
                    ]
                );
            }
        }
    }

    #[test]
    fn fractional_cell_thresholds_are_exact_bottom_up_and_saturating() {
        let fills = [
            0.0, 0.299_999, 0.30, 0.300_001, 0.699_999, 0.70, 0.700_001, 0.999_999, 1.0, 1.000_001,
            1.299_999, 1.30, 1.300_001, 1.699_999, 1.70, 1.700_001, 9.999_999, 10.0, 20.0,
        ];
        let history = samples(&fills);
        let mut selected = view(&history);
        selected.incoming.ceiling = 10;
        selected.outgoing.ceiling = 10;
        selected.incoming.rates.as_mut().unwrap().current = 20.0;
        let rows = render(selected, 80, 24);
        let bottom = "  ...|||###########";
        // One glyph per *actual* sample, including the zero and below-threshold
        // columns. Independently specified whole-bar threshold expectations.
        assert_eq!(bottom.len(), fills.len());
        for (top, end) in [(3, 13), (14, 24)] {
            assert_eq!(&rows[end - 1][50 - fills.len()..50], bottom);
            assert_eq!(&rows[end - 2][50 - fills.len()..50], "           ...||###");
            for row in &rows[top..end] {
                assert_eq!(&row[48..50], "##");
            }
            assert_eq!(&rows[top][47..48], "|");
        }
        assert_eq!(rows[8][52..].trim_end(), "Curr: 160.00 Bit/s");
    }

    #[test]
    fn rate_units_choose_prefix_before_explicit_halfway_up_rounding() {
        for (rate, expected) in [
            (0.0, "0.00 Bit/s"),
            (128.0, "1.00 kBit/s"),
            (0.125 / 8.0, "0.13 Bit/s"),
            (127.9995, "1024.00 Bit/s"),
            (1.004_999 / 8.0, "1.00 Bit/s"),
            (1.005_001 / 8.0, "1.01 Bit/s"),
            (1024_f64.powi(4) / 8.0, "1024.00 GBit/s"),
        ] {
            assert_eq!(format_rate(rate), expected, "{rate:?}");
        }
        for index in 1..=3 {
            let boundary = 1024_f64.powi(index as i32);
            assert_eq!(
                format_rate((boundary - boundary * 1e-7) / 8.0),
                format!("1024.00 {}", RATE_UNITS[index - 1])
            );
            assert_eq!(
                format_rate(boundary / 8.0),
                format!("1.00 {}", RATE_UNITS[index])
            );
            assert_eq!(
                format_rate((boundary + boundary * 1e-7) / 8.0),
                format!("1.00 {}", RATE_UNITS[index])
            );
            assert_eq!(
                format_rate(1.125 * boundary / 8.0),
                format!("1.13 {}", RATE_UNITS[index])
            );
        }
    }

    #[test]
    fn total_units_round_exact_widened_integers_including_maximum_u64() {
        for (total, expected) in [
            (0, "0.00 Byte"),
            (1, "1.00 Byte"),
            (1023, "1023.00 Byte"),
            (1024, "1.00 kByte"),
            (1152, "1.13 kByte"),
            (u64::MAX, "17179869184.00 GByte"),
        ] {
            assert_eq!(format_integer(u128::from(total), &TOTAL_UNITS), expected);
        }
        for index in 1..=3 {
            let boundary = 1024_u128.pow(index as u32);
            let below = if index == 1 { "1023.00" } else { "1024.00" };
            assert_eq!(
                format_integer(boundary - 1, &TOTAL_UNITS),
                format!("{below} {}", TOTAL_UNITS[index - 1])
            );
            assert_eq!(
                format_integer(boundary, &TOTAL_UNITS),
                format!("1.00 {}", TOTAL_UNITS[index])
            );
            assert_eq!(
                format_integer(boundary + 1, &TOTAL_UNITS),
                format!("1.00 {}", TOTAL_UNITS[index])
            );
            assert_eq!(
                format_integer(boundary * 9 / 8, &TOTAL_UNITS),
                format!("1.13 {}", TOTAL_UNITS[index])
            );
        }
        // A value whose f64 conversion would lose the distinguishing exact bytes
        // straddles a hundredth rounding boundary at the capped G prefix.
        let base = 16_777_216_u128 * 1024_u128.pow(3);
        let halfway = (1024_u128.pow(3) * 5).div_ceil(1000);
        assert_eq!(
            format_integer(base + halfway - 1, &TOTAL_UNITS),
            "16777216.00 GByte"
        );
        assert_eq!(
            format_integer(base + halfway, &TOTAL_UNITS),
            "16777216.01 GByte"
        );
    }

    #[test]
    fn independent_ceiling_labels_and_numeric_prefixes_do_not_cap_rates() {
        let history = samples(&[2_621_440.0]);
        let mut selected = view(&history);
        selected.incoming.ceiling = 2_621_440;
        selected.outgoing.ceiling = 655_360;
        selected.incoming.rates = Some(Rates {
            current: 2_621_440.0,
            average: 128.0,
            minimum: 0.015625,
            maximum: 134_217_728.0,
        });
        selected.outgoing.rates.as_mut().unwrap().current = 2_621_440.0;
        let rows = render(selected, 80, 24);
        assert_eq!(rows[2].trim_end(), "Incoming: (ceiling 20.00 MBit/s)");
        assert_eq!(rows[13].trim_end(), "Outgoing: (ceiling 5.00 MBit/s)");
        assert_eq!(
            stats(&rows, 13, 52),
            [
                "Curr: 20.00 MBit/s",
                "Avg: 1.00 kBit/s",
                "Min: 0.13 Bit/s",
                "Max: 1.00 GBit/s",
                "Ttl: 1.00 kByte",
            ]
        );
        assert_eq!(rows[19][52..].trim_end(), "Curr: 20.00 MBit/s");
        for row in rows[3..13].iter().chain(&rows[14..24]) {
            assert_eq!(&row[49..50], "#");
        }
    }

    fn monitor(name: Vec<u8>, ceilings: Ceilings, incoming: u64, outgoing: u64) -> Monitor {
        Monitor::new(
            StartupSelection {
                names: vec![name],
                selected: 0,
            },
            ceilings,
            0,
            |_| CounterOutcome::Readable(Counters { incoming, outgoing }),
        )
    }

    #[test]
    fn largest_exact_totals_and_ceilings_and_overwide_rates_keep_labels() {
        let ceilings = Ceilings {
            incoming: 18_446_744_073_709_551_488,
            outgoing: 128,
        };
        let mut monitor = monitor(b"eth0".to_vec(), ceilings, 0, 0);
        monitor.observe(1, |_| {
            CounterOutcome::Readable(Counters {
                incoming: u64::MAX,
                outgoing: u64::MAX,
            })
        });
        let rows = render(monitor.selected_view(), 60, 16);
        bounded(&rows, 60, 16);
        assert_eq!(
            rows[2].trim_end(),
            "Incoming: (ceiling 137438953472.00 GBit/s)"
        );
        assert_eq!(rows[9].trim_end(), "Outgoing: (ceiling 1.00 kBit/s)");
        for end in [9, 16] {
            assert_eq!(
                stats(&rows, end, 32),
                [
                    "Curr: [too wide]",
                    "Avg: [too wide]",
                    "Min: [too wide]",
                    "Max: [too wide]",
                    "Ttl: 17179869184.00 GByte",
                ]
            );
        }
        for row in rows[3..9].iter().chain(&rows[10..16]) {
            assert_eq!(&row[29..30], "#");
        }
    }

    #[test]
    fn baseline_absence_failure_idle_and_recovery_have_exact_rows() {
        let mut monitor = monitor(b"eth0".to_vec(), Ceilings::default(), 1024, 2048);
        let check =
            |monitor: &Monitor, status_text: &str, incoming_total: &str, outgoing_total: &str| {
                let rows = render(monitor.selected_view(), 80, 24);
                assert_eq!(rows[0].trim_end(), "Interface eth0 (1/1)");
                assert_eq!(rows[1].trim_end(), status_text);
                for (end, total) in [(13, incoming_total), (24, outgoing_total)] {
                    assert_eq!(
                        stats(&rows, end, 52),
                        ["Curr: --", "Avg: --", "Min: --", "Max: --", total]
                    );
                    assert!(
                        rows[end - 10..end]
                            .iter()
                            .all(|row| row[..50] == " ".repeat(50))
                    );
                }
                bounded(&rows, 80, 24);
            };
        check(
            &monitor,
            "Waiting for rate sample",
            "Ttl: 1.00 kByte",
            "Ttl: 2.00 kByte",
        );
        monitor.observe(500_000_000, |_| {
            CounterOutcome::Readable(Counters {
                incoming: 1024,
                outgoing: 2048,
            })
        });
        let idle = render(monitor.selected_view(), 80, 24);
        assert_eq!(idle[1], "=".repeat(80));
        for end in [13, 24] {
            assert_eq!(
                &stats(&idle, end, 52)[..4],
                [
                    "Curr: 0.00 Bit/s",
                    "Avg: 0.00 Bit/s",
                    "Min: 0.00 Bit/s",
                    "Max: 0.00 Bit/s"
                ]
            );
            assert!(
                idle[end - 10..end]
                    .iter()
                    .all(|row| row[..50] == " ".repeat(50))
            );
        }
        monitor.observe(1_000_000_000, |_| CounterOutcome::Absent);
        check(
            &monitor,
            "Unavailable: interface absent",
            "Ttl: --",
            "Ttl: --",
        );
        monitor.observe(1_500_000_000, |_| CounterOutcome::ReadFailed);
        check(
            &monitor,
            "Unavailable: counter read failed",
            "Ttl: --",
            "Ttl: --",
        );
        monitor.observe(2_000_000_000, |_| {
            CounterOutcome::Readable(Counters {
                incoming: 4096,
                outgoing: 8192,
            })
        });
        check(
            &monitor,
            "Waiting for rate sample",
            "Ttl: 4.00 kByte",
            "Ttl: 8.00 kByte",
        );
    }

    #[test]
    fn raw_names_are_ascii_escaped_shortened_and_keep_position() {
        let history = samples(&[0.0]);
        let mut selected = view(&history);
        selected.name = b"eth\x00\x1b\x7f\x80\xff";
        let rows = render(selected, 60, 16);
        assert_eq!(
            rows[0].trim_end(),
            "Interface eth\\x00\\x1B\\x7F\\x80\\xFF (1/3)"
        );
        bounded(&rows, 60, 16);
        let long = [b'X'; 100];
        selected.name = &long;
        let rows = render(selected, 60, 16);
        assert_eq!(rows[0], format!("Interface {}... (1/3)", "X".repeat(41)));
        selected.name = b"eth\xffxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let tiny = render(selected, 24, 3);
        assert_eq!(tiny[1].trim_end(), "Interface eth... (1/3)");
        assert_eq!(render(selected, 5, 2)[1], "(1/3)");
        assert_eq!(render(selected, 4, 2)[1], "e...");
        assert_eq!(
            selected.name,
            b"eth\xffxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        );
    }

    #[test]
    fn small_and_zero_dimensions_show_only_bounded_warning_selection_status() {
        let history = samples(&[0.0]);
        let mut selected = view(&history);
        for state in [
            ViewState::Baseline,
            ViewState::Absent,
            ViewState::ReadFailed,
            ViewState::Measured,
        ] {
            selected.state = state;
            for (width, height) in [
                (59, 16),
                (60, 15),
                (1, 1),
                (0, 0),
                (0, 5),
                (80, 0),
                (2, 2),
                (3, 3),
            ] {
                let rows = render(selected, width, height);
                bounded(&rows, width, height);
                if width >= 59 && height >= 3 {
                    assert_eq!(rows[0].trim_end(), "Terminal too small (need 60x16)");
                    assert_eq!(rows[1].trim_end(), "Interface eth0 (1/3)");
                    assert_eq!(rows[2].trim_end(), status(state).unwrap_or("Measured"));
                    assert!(rows[3..].iter().all(|row| row.trim().is_empty()));
                }
            }
        }
        assert_eq!(render(selected, 1, 1), ["T"]);
        assert_eq!(
            render(selected, 60, 16)[2].trim_end(),
            "Incoming: (ceiling 10.00 MBit/s)"
        );
    }

    #[test]
    fn fitting_history_resize_and_repeated_redraw_preserve_real_monitor_view() {
        let mut monitor = monitor(
            b"eth0".to_vec(),
            Ceilings {
                incoming: 100,
                outgoing: 100,
            },
            0,
            0,
        );
        let mut total = 0;
        for index in 1..=70 {
            total += if index % 2 == 0 && index != 70 {
                0
            } else {
                200
            };
            // Two-second observations give a full or genuinely zero current-rate
            // column, without fabricating catch-up samples for the delayed round.
            monitor.observe(index * 2_000_000_000, |_| {
                CounterOutcome::Readable(Counters {
                    incoming: total,
                    outgoing: total,
                })
            });
        }
        let before = format!("{:?}", monitor.selected_view());
        let wide = render(monitor.selected_view(), 100, 24);
        assert_eq!(&wide[12][..70], format!("{}##", "# ".repeat(34)));
        let narrow = render(monitor.selected_view(), 60, 16);
        // The distinct last pair detects an oldest-prefix or reversed-history
        // renderer, which a purely periodic alternating fixture would miss.
        assert_eq!(&narrow[8][..30], format!("{}##", "# ".repeat(14)));
        let small = render(monitor.selected_view(), 59, 15);
        assert!(small[0].starts_with("Terminal too small"));
        assert_eq!(render(monitor.selected_view(), 100, 24), wide);
        assert_eq!(format!("{:?}", monitor.selected_view()), before);
        assert_eq!(monitor.selected_view().incoming.samples.len(), 70);

        let short = samples(&[100.0, 0.0, 100.0]);
        let mut selected = view(&short);
        selected.incoming.ceiling = 100;
        selected.outgoing.ceiling = 100;
        let rows = render(selected, 60, 16);
        for row in rows[3..9].iter().chain(&rows[10..16]) {
            assert_eq!(&row[..30], format!("{}# #", " ".repeat(27)));
        }
    }

    #[test]
    fn sampling_and_navigation_while_small_reappear_as_actual_history_on_growth() {
        let mut monitor = Monitor::new(
            StartupSelection {
                names: vec![b"eth0".to_vec(), b"eth1".to_vec()],
                selected: 0,
            },
            Ceilings {
                incoming: 100,
                outgoing: 100,
            },
            0,
            |_| {
                CounterOutcome::Readable(Counters {
                    incoming: 0,
                    outgoing: 0,
                })
            },
        );
        assert_eq!(
            render(monitor.selected_view(), 59, 15)[1].trim_end(),
            "Interface eth0 (1/2)"
        );
        for index in 1..=3 {
            monitor.observe(index * 2_000_000_000, |_| {
                CounterOutcome::Readable(Counters {
                    incoming: index * 200,
                    outgoing: index * 200,
                })
            });
            render(monitor.selected_view(), 1, 1);
        }
        monitor.forward();
        assert_eq!(
            render(monitor.selected_view(), 59, 15)[1].trim_end(),
            "Interface eth1 (2/2)"
        );
        let rows = render(monitor.selected_view(), 60, 16);
        assert_eq!(rows[0].trim_end(), "Interface eth1 (2/2)");
        assert_eq!(&rows[8][..30], format!("{}###", " ".repeat(27)));
        assert_eq!(rows[4][32..].trim_end(), "Curr: 800.00 Bit/s");
        assert_eq!(monitor.selected_view().incoming.samples.len(), 3);
    }
}
