// SPDX-License-Identifier: GPL-2.0-only

//! Pure selected-monitor-view and dimensions to an opaque, cell-bounded frame.
//!
//! Owns geometry, escaping, formatting, graph glyphs and small-terminal mode.
//! Owns row encoding and default-style resets; terminal owns positioning, clearing
//! and session commands. Adjacent observations share bottom-filled Braille cells;
//! graph-row position selects an incoming or outgoing xterm-256color foreground.
//! Two samples per cell nominally span one second at the unchanged 500-ms cadence;
//! delayed observations remain observation-spaced. Only the fitting suffix is read.
//! Interactive output assumes UTF-8, Braille font coverage and xterm-256color.

use crate::monitor::{DirectionView, SelectedView, ViewState};

const MIN_WIDTH: usize = 60;
const MIN_HEIGHT: usize = 16;
const STAT_WIDTH: usize = 28;
const GAP: usize = 2;
const RATE_UNITS: [&str; 4] = ["Bit/s", "kBit/s", "MBit/s", "GBit/s"];
const TOTAL_UNITS: [&str; 4] = ["Byte", "kByte", "MByte", "GByte"];
const INCOMING_PALETTE: [u8; 6] = [60, 61, 98, 140, 146, 183];
const OUTGOING_PALETTE: [u8; 6] = [96, 132, 133, 170, 176, 219];

/// Only screen construction can establish the logical cell bounds and row protocol.
/// Encoded rows omit trailing blank cells but retain start/end and statistics resets.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Frame {
    width: usize,
    height: usize,
    rows: Vec<String>,
}

impl Frame {
    pub(crate) fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub(crate) fn encoded_rows(&self) -> &[String] {
        &self.rows
    }
}

#[derive(Clone)]
struct Row {
    cells: Vec<char>,
    statistics: Option<usize>,
    graph_foreground: Option<u8>,
}

impl Row {
    fn encode(self) -> String {
        let end = self
            .cells
            .iter()
            .rposition(|&cell| cell != ' ')
            .map_or(0, |i| i + 1);
        let mut encoded = String::from("\x1b[0m");
        let mut foreground = None;
        for (column, cell) in self.cells[..end].iter().enumerate() {
            if self.statistics == Some(column) {
                encoded.push_str("\x1b[0m");
                foreground = None;
            }
            let style = if ('\u{2801}'..='\u{28ff}').contains(cell) {
                self.graph_foreground
            } else {
                None
            };
            if style != foreground {
                if let Some(index) = style {
                    encoded.push_str(&format!("\x1b[38;5;{index}m"));
                } else {
                    encoded.push_str("\x1b[0m");
                }
                foreground = style;
            }
            encoded.push(*cell);
        }
        if foreground.is_some() {
            encoded.push_str("\x1b[0m");
        }
        if self.statistics.is_some_and(|column| column >= end) {
            encoded.push_str("\x1b[0m");
        }
        encoded.push_str("\x1b[0m");
        encoded
    }
}

/// Exactly `height` logical rows of `width` terminal cells, including zero sizes.
/// Views come from Monitor (positive ceilings, finite nonnegative rates and paired
/// availability invariants). Dimensions affect presentation only, never history.
pub(crate) fn render(view: SelectedView<'_>, width: usize, height: usize) -> Frame {
    let mut rows = vec![
        Row {
            cells: vec![' '; width],
            statistics: None,
            graph_foreground: None,
        };
        height
    ];
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
            rows[1].cells.fill('=');
        }
        let incoming_height = (height - 4) / 2;
        let outgoing_label = 3 + incoming_height;
        direction(
            &mut rows[2..outgoing_label],
            "Incoming",
            view.incoming,
            &INCOMING_PALETTE,
        );
        direction(
            &mut rows[outgoing_label..],
            "Outgoing",
            view.outgoing,
            &OUTGOING_PALETTE,
        );
    }
    Frame {
        width,
        height,
        rows: rows.into_iter().map(Row::encode).collect(),
    }
}

fn status(state: ViewState) -> Option<&'static str> {
    match state {
        ViewState::Baseline => Some("Waiting for rate sample"),
        ViewState::Absent => Some("Unavailable: interface absent"),
        ViewState::ReadFailed => Some("Unavailable: counter read failed"),
        ViewState::Measured => None,
    }
}

fn direction(rows: &mut [Row], label: &str, view: DirectionView<'_>, palette: &[u8; 6]) {
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
    let graph_width = graph[0].cells.len() - STAT_WIDTH - GAP;
    for (index, row) in graph.iter_mut().enumerate() {
        row.statistics = Some(graph_width + GAP);
        let k = height - 1 - index;
        // Integer nearest-grade selection, including exact halfway-up ties.
        let grade = (10 * k + height - 1) / (2 * (height - 1));
        row.graph_foreground = Some(palette[grade]);
    }
    let capacity = 2 * graph_width;
    let fitted = view.samples.len().min(capacity);
    for (offset, sample) in view
        .samples
        .range(view.samples.len() - fitted..)
        .enumerate()
    {
        let position = capacity - fitted + offset;
        let fraction = (sample.rate / view.ceiling as f64).clamp(0.0, 1.0);
        // Positive ties round upward without an addition that could promote the
        // representable value immediately below a half step to that step.
        let dots = ((fraction * (4 * height) as f64).round() as usize).min(4 * height);
        // Standard Unicode dot numbers, bottom-to-top in each independent half.
        let numbers = if position % 2 == 0 {
            [7, 3, 2, 1]
        } else {
            [8, 6, 5, 4]
        };
        for (index, row) in graph.iter_mut().enumerate() {
            let count = dots.saturating_sub(4 * (height - 1 - index)).min(4);
            let cell = &mut row.cells[position / 2];
            let mut mask = if *cell == ' ' {
                0
            } else {
                *cell as u32 - 0x2800
            };
            for number in &numbers[..count] {
                mask |= 1 << (number - 1);
            }
            // U+2800 is deliberately never emitted: zero is ordinary space.
            *cell = if mask == 0 {
                ' '
            } else {
                char::from_u32(0x2800 + mask).unwrap()
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
fn put(row: &mut Row, column: usize, text: &str) {
    for (cell, character) in row.cells.iter_mut().skip(column).zip(text.chars()) {
        assert!(
            character.is_ascii() && !character.is_ascii_control(),
            "screen text must be safe ASCII"
        );
        *cell = character;
    }
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

    #[test]
    fn paired_saturated_halves_and_real_zeros_use_direction_foregrounds() {
        for (rates, glyphs) in [
            (vec![1_310_720.0], "⢸"),
            (vec![1_310_720.0, 0.0], "⡇"),
            (vec![1_310_720.0, 0.0, 1_310_720.0], "⢸⢸"),
            (vec![1_310_720.0, 0.0, 1_310_720.0, 2_621_440.0], "⡇⣿"),
        ] {
            let history = samples(&rates);
            let frame = super::render(view(&history), 60, 16);
            let padding = " ".repeat(30 - glyphs.chars().count());
            for (row, foreground) in [(3, 183), (10, 219)] {
                assert_eq!(
                    frame.encoded_rows()[row],
                    format!("\x1b[0m{padding}\x1b[38;5;{foreground}m{glyphs}\x1b[0m\x1b[0m\x1b[0m")
                );
            }
        }
    }

    #[test]
    fn opaque_frame_retains_dimensions_and_encodes_trimmed_default_rows() {
        let history = samples(&[]);
        let frame = super::render(view(&history), 60, 16);
        assert_eq!(frame.dimensions(), (60, 16));
        assert_eq!(frame.encoded_rows().len(), 16);
        assert_eq!(
            frame.encoded_rows()[0],
            "\x1b[0mInterface eth0 (1/3)\x1b[0m"
        );
        assert_eq!(frame.encoded_rows()[3], "\x1b[0m\x1b[0m\x1b[0m");
        assert_eq!(
            frame.encoded_rows()[4],
            format!("\x1b[0m{}\x1b[0mCurr: 1.00 kBit/s\x1b[0m", " ".repeat(32))
        );
        for (width, height) in [(0, 0), (0, 5), (80, 0)] {
            let frame = super::render(view(&history), width, height);
            assert_eq!(frame.dimensions(), (width, height));
            assert_eq!(frame.encoded_rows(), vec!["\x1b[0m\x1b[0m"; height]);
        }
    }

    fn bounded(rows: &[String], width: usize, height: usize) {
        assert_eq!(rows.len(), height);
        for row in rows {
            assert_eq!(row.chars().count(), width);
            assert!(
                row.chars()
                    .all(|c| (' '..='~').contains(&c) || ('\u{2801}'..='\u{28ff}').contains(&c))
            );
        }
    }

    // Test-side whitelist at the frame seam, independent of production encoding.
    // Only nonblank Braille is colored; all text/space cells must be default.
    fn decoded(frame: &Frame) -> Vec<Vec<(char, Option<u8>)>> {
        let (width, height) = frame.dimensions();
        assert_eq!(frame.encoded_rows().len(), height);
        frame
            .encoded_rows()
            .iter()
            .map(|encoded| {
                assert!(encoded.starts_with("\x1b[0m"));
                assert!(encoded.ends_with("\x1b[0m"));
                let mut remaining = encoded.as_str();
                let mut foreground = None;
                let mut cells = Vec::new();
                while !remaining.is_empty() {
                    if let Some(sgr) = remaining.strip_prefix("\x1b[") {
                        let (parameters, rest) = sgr.split_once('m').unwrap();
                        foreground = if parameters == "0" {
                            None
                        } else {
                            let index = parameters
                                .strip_prefix("38;5;")
                                .unwrap()
                                .parse::<u8>()
                                .unwrap();
                            assert!(matches!(
                                index,
                                60 | 61 | 98 | 140 | 146 | 183 | 96 | 132 | 133 | 170 | 176 | 219
                            ));
                            Some(index)
                        };
                        remaining = rest;
                    } else {
                        let c = remaining.chars().next().unwrap();
                        if ('\u{2801}'..='\u{28ff}').contains(&c) {
                            assert!(foreground.is_some());
                        } else {
                            assert!((' '..='~').contains(&c));
                            assert_eq!(foreground, None, "styled text or blank");
                        }
                        cells.push((c, foreground));
                        remaining = &remaining[c.len_utf8()..];
                    }
                }
                assert_eq!(foreground, None);
                assert!(cells.len() <= width);
                assert!(cells.last().is_none_or(|&(c, _)| c != ' '));
                cells.resize(width, (' ', None));
                cells
            })
            .collect()
    }

    fn render(view: SelectedView<'_>, width: usize, height: usize) -> Vec<String> {
        let frame = super::render(view, width, height);
        assert_eq!(frame.dimensions(), (width, height));
        decoded(&frame)
            .into_iter()
            .map(|row| row.into_iter().map(|(c, _)| c).collect())
            .collect()
    }

    fn cells(row: &str, start: usize, end: usize) -> String {
        row.chars().skip(start).take(end - start).collect()
    }

    fn stats(rows: &[String], end: usize, start: usize) -> Vec<String> {
        rows[end - 5..end]
            .iter()
            .map(|row| cells(row, start, row.chars().count()).trim_end().to_owned())
            .collect()
    }

    fn rendered_rate(rate: f64) -> String {
        let history = samples(&[rate]);
        let mut selected = view(&history);
        selected.incoming.rates.as_mut().unwrap().current = rate;
        stats(&render(selected, 60, 16), 9, 32)[0]
            .strip_prefix("Curr: ")
            .unwrap()
            .to_owned()
    }

    fn rendered_total(total: u128) -> String {
        let history = samples(&[]);
        let mut selected = view(&history);
        selected.incoming.total = Some(u64::try_from(total).unwrap());
        stats(&render(selected, 60, 16), 9, 32)[4]
            .strip_prefix("Ttl: ")
            .unwrap()
            .to_owned()
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
                *row = format!("{}⣿", " ".repeat(49));
            }
            for (row, field) in expected[end - 5..end].iter_mut().zip([
                "Curr: 1.00 kBit/s",
                "Avg: 2.00 kBit/s",
                "Min: 0.00 Bit/s",
                "Max: 4.00 kBit/s",
                "Ttl: 1.00 kByte",
            ]) {
                *row = format!("{}⢸⣿  {field}", " ".repeat(48));
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
                    assert_eq!(cells(row, width - 31, width - 30), "⢸");
                    assert_eq!(cells(row, width - 30, width - 28), "  ");
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
    fn adjacent_rates_at_half_steps_do_not_gain_a_dot_from_additive_rounding() {
        // Literal full columns from the Unicode dot diagram. At H=10 the
        // default ceiling's first threshold is exactly 16384 bytes/s. The H=6
        // power-of-two ceiling keeps normalization exact before scaling.
        for (width, height, ceiling, halfway, below, at) in [
            (80, 24, 1_310_720, 16_384.0_f64, "          ", "         ⢀"),
            (80, 24, 1_310_720, 49_152.0, "         ⢀", "         ⢠"),
            (80, 24, 1_310_720, 147_456.0, "         ⢸", "        ⢀⢸"),
            (80, 24, 1_310_720, 638_976.0, "     ⢰⢸⢸⢸⢸", "     ⢸⢸⢸⢸⢸"),
            (60, 16, 128, 8.0 / 3.0, "      ", "     ⢀"),
            (60, 16, 128, 8.0, "     ⢀", "     ⢠"),
            (60, 16, 128, 24.0, "     ⢸", "    ⢀⢸"),
            (60, 16, 128, 104.0, " ⢰⢸⢸⢸⢸", " ⢸⢸⢸⢸⢸"),
        ] {
            for (name, rate, expected) in [
                ("below", halfway.next_down(), below),
                ("at", halfway, at),
                ("above", halfway.next_up(), at),
            ] {
                let history = samples(&[rate]);
                let mut selected = view(&history);
                selected.incoming.ceiling = ceiling;
                selected.outgoing.ceiling = ceiling;
                let frame = super::render(selected, width, height);
                let rows = decoded(&frame);
                let graph_height = (height - 4) / 2;
                for top in [3, 4 + graph_height] {
                    let actual: String = rows[top..top + graph_height]
                        .iter()
                        .map(|row| row[width - 31].0)
                        .collect();
                    assert_eq!(
                        actual, expected,
                        "H={graph_height}, c={ceiling}, {name}: {rate:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn intermediate_normalization_ties_use_the_normalized_rate_not_a_rate_threshold() {
        // Division/multiplication themselves can land on a half step for
        // neighboring input rates. Preserve the contract's r/c then 4H order,
        // rather than changing it to compare r with a reconstructed threshold.
        for (width, height, ceiling, halfway, expected) in [
            (80, 24, 1_310_720, 114_688.0_f64, "         ⢸"),
            (60, 16, 48, 1.0, "     ⢀"),
            (60, 16, 1_310_720, 81_920.0 / 3.0, "     ⢀"),
        ] {
            for rate in [halfway.next_down(), halfway, halfway.next_up()] {
                let history = samples(&[rate]);
                let mut selected = view(&history);
                selected.incoming.ceiling = ceiling;
                selected.outgoing.ceiling = ceiling;
                let rows = decoded(&super::render(selected, width, height));
                let graph_height = (height - 4) / 2;
                for top in [3, 4 + graph_height] {
                    let actual: String = rows[top..top + graph_height]
                        .iter()
                        .map(|row| row[width - 31].0)
                        .collect();
                    assert_eq!(
                        actual, expected,
                        "H={graph_height}, c={ceiling}, r={rate:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn nearest_dot_half_steps_are_exact_bottom_up_without_minimum_or_tolerance() {
        // H=6, c=24 makes one byte/s one dot. Literal Unicode masks are
        // worked from the standard dot diagram, not the production algorithm.
        let right = [' ', '⢀', '⢠', '⢰', '⢸'];
        for boundary in 0_usize..24 {
            let halfway = boundary as f64 + 0.5;
            for (rate, dots) in [
                (halfway - 1e-12, boundary),
                (halfway, boundary + 1),
                (halfway + 1e-12, boundary + 1),
            ] {
                let history = samples(&[rate]);
                let mut selected = view(&history);
                selected.incoming.ceiling = 24;
                selected.outgoing.ceiling = 24;
                let rows = render(selected, 60, 16);
                for (top, end) in [(3, 9), (10, 16)] {
                    let actual: String = rows[top..end]
                        .iter()
                        .map(|row| row.chars().nth(29).unwrap())
                        .collect();
                    let expected: String = (0..6)
                        .rev()
                        .map(|k| right[dots.saturating_sub(4 * k).min(4)])
                        .collect();
                    assert_eq!(actual, expected, "rate {rate:?}, dots {dots}");
                }
            }
        }
        for (rate, expected) in [
            (0.0, "      "),
            (1e-15, "      "),
            (12.0, "   ⢸⢸⢸"),
            (24.0, "⢸⢸⢸⢸⢸⢸"),
            (48.0, "⢸⢸⢸⢸⢸⢸"),
        ] {
            let history = samples(&[rate]);
            let mut selected = view(&history);
            selected.incoming.ceiling = 24;
            let rows = render(selected, 60, 16);
            let actual: String = rows[3..9]
                .iter()
                .map(|row| row.chars().nth(29).unwrap())
                .collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn standard_braille_masks_cover_every_bottom_filled_half_combination() {
        let masks = [
            [' ', '⢀', '⢠', '⢰', '⢸'],
            ['⡀', '⣀', '⣠', '⣰', '⣸'],
            ['⡄', '⣄', '⣤', '⣴', '⣼'],
            ['⡆', '⣆', '⣦', '⣶', '⣾'],
            ['⡇', '⣇', '⣧', '⣷', '⣿'],
        ];
        for (left, expected) in masks.iter().enumerate() {
            for (right, &glyph) in expected.iter().enumerate() {
                let history = samples(&[left as f64, right as f64]);
                let mut selected = view(&history);
                selected.incoming.ceiling = 24;
                let rows = render(selected, 60, 16);
                assert_eq!(cells(&rows[8], 29, 30), glyph.to_string());
                assert!(
                    rows[3..8]
                        .iter()
                        .all(|row| cells(row, 0, 30) == " ".repeat(30))
                );
            }
        }
    }

    #[test]
    fn independent_half_and_direction_quantization_preserves_uncapped_text() {
        let history = samples(&[2.0, 28.0]);
        let mut selected = view(&history);
        selected.incoming.ceiling = 24;
        selected.outgoing.ceiling = 48;
        selected.incoming.rates.as_mut().unwrap().current = 28.0;
        let rows = render(selected, 60, 16);
        let incoming: String = rows[3..9]
            .iter()
            .map(|row| row.chars().nth(29).unwrap())
            .collect();
        let outgoing: String = rows[10..16]
            .iter()
            .map(|row| row.chars().nth(29).unwrap())
            .collect();
        assert_eq!(incoming, "⢸⢸⢸⢸⢸⣼");
        assert_eq!(outgoing, "  ⢠⢸⢸⣸");
        assert_eq!(stats(&rows, 9, 32)[0], "Curr: 224.00 Bit/s");
        assert_eq!(rows[2].trim_end(), "Incoming: (ceiling 192.00 Bit/s)");
        assert_eq!(rows[9].trim_end(), "Outgoing: (ceiling 384.00 Bit/s)");
    }

    #[test]
    fn odd_graph_heights_quantize_half_ceiling_and_first_half_dot_independently() {
        for (width, height, ceiling, incoming, outgoing) in [
            (61, 17, 48, "   ⢸⢸⢸", "   ⢠⢸⢸⢸"),
            (81, 25, 80, "     ⢸⢸⢸⢸⢸", "     ⢠⢸⢸⢸⢸⢸"),
        ] {
            let history = samples(&[0.0, ceiling as f64 / 2.0]);
            let mut selected = view(&history);
            selected.incoming.ceiling = ceiling;
            selected.outgoing.ceiling = ceiling;
            let rows = render(selected, width, height);
            let incoming_end = 3 + (height - 4) / 2;
            let column = width - 31;
            for (start, end, expected) in [
                (3, incoming_end, incoming),
                (incoming_end + 1, height, outgoing),
            ] {
                let actual: String = rows[start..end]
                    .iter()
                    .map(|row| row.chars().nth(column).unwrap())
                    .collect();
                assert_eq!(actual, expected);
            }
            // With c=8H, rate 1 is the first half-dot boundary in each graph.
            // Test true normalization at distinct positive ceilings, not c=4H.
            for (rate, expected) in [(1.0 - 1e-12, ' '), (1.0, '⢀'), (1.0 + 1e-12, '⢀')] {
                let history = samples(&[rate]);
                let mut selected = view(&history);
                selected.incoming.ceiling = 8 * (incoming_end - 3) as u64;
                selected.outgoing.ceiling = 8 * (height - incoming_end - 1) as u64;
                let rows = render(selected, width, height);
                assert_eq!(rows[incoming_end - 1].chars().nth(column), Some(expected));
                assert_eq!(rows[height - 1].chars().nth(column), Some(expected));
            }
        }
    }

    #[test]
    fn positional_palettes_use_each_direction_height_and_leave_text_default() {
        let history = samples(&[1_310_720.0, 0.0, 655_360.0, 1_310_720.0]);
        for (height, incoming, outgoing) in [
            (
                16,
                vec![183, 146, 140, 98, 61, 60],
                vec![219, 176, 170, 133, 132, 96],
            ),
            (
                17,
                vec![183, 146, 140, 98, 61, 60],
                vec![219, 176, 170, 170, 133, 132, 96],
            ),
            (
                24,
                vec![183, 146, 146, 140, 140, 98, 98, 61, 61, 60],
                vec![219, 176, 176, 170, 170, 133, 133, 132, 132, 96],
            ),
            // H=11 has exact half-grade ties; ties rise rather than to even.
            (
                25,
                vec![183, 146, 146, 140, 140, 98, 98, 61, 61, 60],
                vec![219, 219, 176, 176, 170, 170, 133, 133, 132, 132, 96],
            ),
        ] {
            let width = if height < 24 { 60 } else { 80 };
            let frame = super::render(view(&history), width, height);
            let rows = decoded(&frame);
            let outgoing_label = 3 + incoming.len();
            for (start, palette) in [(3, incoming), (outgoing_label + 1, outgoing)] {
                for (row, color) in rows[start..start + palette.len()].iter().zip(palette) {
                    assert!(row.iter().any(|&(c, _)| c != ' ' && !c.is_ascii()));
                    for &(c, foreground) in row {
                        assert_eq!(foreground, if c.is_ascii() { None } else { Some(color) });
                    }
                }
            }
            // Explicit reset at the gap and again at the statistics boundary.
            assert!(frame.encoded_rows()[outgoing_label - 5].contains("\x1b[0m  \x1b[0mCurr:"));
            for row in [0, 1, 2, outgoing_label] {
                assert!(!frame.encoded_rows()[row].contains("38;5;"));
            }
        }
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
            assert_eq!(rendered_rate(rate), expected, "{rate:?}");
        }
        for index in 1..=3 {
            let boundary = 1024_f64.powi(index as i32);
            assert_eq!(
                rendered_rate((boundary - boundary * 1e-7) / 8.0),
                format!("1024.00 {}", RATE_UNITS[index - 1])
            );
            assert_eq!(
                rendered_rate(boundary / 8.0),
                format!("1.00 {}", RATE_UNITS[index])
            );
            assert_eq!(
                rendered_rate((boundary + boundary * 1e-7) / 8.0),
                format!("1.00 {}", RATE_UNITS[index])
            );
            assert_eq!(
                rendered_rate(1.125 * boundary / 8.0),
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
            assert_eq!(rendered_total(u128::from(total)), expected);
        }
        for index in 1..=3 {
            let boundary = 1024_u128.pow(index as u32);
            let below = if index == 1 { "1023.00" } else { "1024.00" };
            assert_eq!(
                rendered_total(boundary - 1),
                format!("{below} {}", TOTAL_UNITS[index - 1])
            );
            assert_eq!(
                rendered_total(boundary),
                format!("1.00 {}", TOTAL_UNITS[index])
            );
            assert_eq!(
                rendered_total(boundary + 1),
                format!("1.00 {}", TOTAL_UNITS[index])
            );
            assert_eq!(
                rendered_total(boundary * 9 / 8),
                format!("1.13 {}", TOTAL_UNITS[index])
            );
        }
        // A value whose f64 conversion would lose the distinguishing exact bytes
        // straddles a hundredth rounding boundary at the capped G prefix.
        let base = 16_777_216_u128 * 1024_u128.pow(3);
        let halfway = (1024_u128.pow(3) * 5).div_ceil(1000);
        assert_eq!(rendered_total(base + halfway - 1), "16777216.00 GByte");
        assert_eq!(rendered_total(base + halfway), "16777216.01 GByte");
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
        assert_eq!(stats(&rows, 24, 52)[0], "Curr: 20.00 MBit/s");
        for row in rows[3..13].iter().chain(&rows[14..24]) {
            assert_eq!(cells(row, 49, 50), "⢸");
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
            assert_eq!(cells(row, 29, 30), "⢸");
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
                            .all(|row| cells(row, 0, 50) == " ".repeat(50))
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
                    .all(|row| cells(row, 0, 50) == " ".repeat(50))
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
        monitor.observe(3_000_000_000, |_| {
            CounterOutcome::Readable(Counters {
                incoming: 8192,
                outgoing: 16384,
            })
        });
        monitor.observe(4_000_000_000, |_| {
            CounterOutcome::Readable(Counters {
                incoming: 1024,
                outgoing: 20000,
            })
        });
        check(
            &monitor,
            "Waiting for rate sample",
            "Ttl: 1.00 kByte",
            "Ttl: 19.53 kByte",
        );
    }

    #[test]
    fn measured_idle_shifts_older_traffic_by_real_blank_half_cells() {
        let mut monitor = monitor(
            b"eth0".to_vec(),
            Ceilings {
                incoming: 100,
                outgoing: 100,
            },
            0,
            0,
        );
        for (seconds, expected) in [(2, " ⢸"), (4, " ⡇"), (6, "⢸ ")] {
            monitor.observe(seconds * 1_000_000_000, |_| {
                CounterOutcome::Readable(Counters {
                    incoming: 200,
                    outgoing: 200,
                })
            });
            let rows = render(monitor.selected_view(), 60, 16);
            for row in rows[3..9].iter().chain(&rows[10..16]) {
                assert_eq!(cells(row, 28, 30), expected);
            }
            if seconds > 2 {
                assert_eq!(stats(&rows, 9, 32)[0], "Curr: 0.00 Bit/s");
                assert_eq!(rows[1], "=".repeat(60));
            }
            assert_eq!(
                monitor.selected_view().incoming.samples.len(),
                seconds as usize / 2
            );
        }
    }

    #[test]
    fn raw_names_are_ascii_escaped_shortened_and_keep_position() {
        let history = samples(&[1_310_720.0]);
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
        let history = samples(&[1_310_720.0]);
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
                (1, 1000),
                (1000, 1),
            ] {
                let rows = render(selected, width, height);
                assert!(
                    super::render(selected, width, height)
                        .encoded_rows()
                        .iter()
                        .all(|row| !row.contains("38;5;"))
                );
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
            // half-cell, without fabricating catch-up samples for delayed rounds.
            monitor.observe(index * 2_000_000_000, |_| {
                CounterOutcome::Readable(Counters {
                    incoming: total,
                    outgoing: total,
                })
            });
        }
        let before = format!("{:?}", monitor.selected_view());
        let wide = render(monitor.selected_view(), 100, 24);
        assert_eq!(
            cells(&wide[12], 0, 70),
            format!("{}{}⣿", " ".repeat(35), "⡇".repeat(34))
        );
        let narrow = render(monitor.selected_view(), 60, 16);
        // The distinct last pair detects an oldest-prefix or reversed-history
        // renderer, which a purely periodic alternating fixture would miss.
        assert_eq!(cells(&narrow[8], 0, 30), format!("{}⣿", "⡇".repeat(29)));
        let small = render(monitor.selected_view(), 59, 15);
        assert!(small[0].starts_with("Terminal too small"));
        assert_eq!(render(monitor.selected_view(), 100, 24), wide);
        assert_eq!(format!("{:?}", monitor.selected_view()), before);
        assert_eq!(monitor.selected_view().incoming.samples.len(), 70);
        let standard = render(monitor.selected_view(), 80, 24);
        assert_eq!(
            cells(&standard[12], 0, 50),
            format!("{}{}⣿", " ".repeat(15), "⡇".repeat(34))
        );

        let short = samples(&[100.0, 0.0, 100.0]);
        let mut selected = view(&short);
        selected.incoming.ceiling = 100;
        selected.outgoing.ceiling = 100;
        let rows = render(selected, 60, 16);
        for row in rows[3..9].iter().chain(&rows[10..16]) {
            assert_eq!(cells(row, 0, 30), format!("{}⢸⢸", " ".repeat(28)));
        }
    }

    #[test]
    fn fitting_suffix_displays_sixty_or_one_hundred_of_actual_retained_observations() {
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
        for index in 1..=101 {
            if index == 1 || (42..=100).contains(&index) {
                total += 200;
            }
            monitor.observe(index * 2_000_000_000, |_| {
                CounterOutcome::Readable(Counters {
                    incoming: total,
                    outgoing: total,
                })
            });
        }
        let before = format!("{:?}", monitor.selected_view());
        for (width, height, expected) in [
            (60, 16, format!("{}⡇", "⣿".repeat(29))),
            (80, 24, format!("{}{}⡇", " ".repeat(20), "⣿".repeat(29))),
            (81, 25, format!("⢸{}{}⡇", " ".repeat(20), "⣿".repeat(29))),
        ] {
            let rows = render(monitor.selected_view(), width, height);
            let incoming_end = 3 + (height - 4) / 2;
            for row in rows[3..incoming_end]
                .iter()
                .chain(&rows[incoming_end + 1..height])
            {
                assert_eq!(cells(row, 0, width - 30), expected);
            }
            assert_eq!(monitor.selected_view().incoming.samples.len(), 101);
            assert_eq!(format!("{:?}", monitor.selected_view()), before);
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
        assert_eq!(cells(&rows[8], 0, 30), format!("{}⢸⣿", " ".repeat(28)));
        assert_eq!(stats(&rows, 9, 32)[0], "Curr: 800.00 Bit/s");
        assert_eq!(monitor.selected_view().incoming.samples.len(), 3);
    }
}
