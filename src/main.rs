// SPDX-License-Identifier: GPL-2.0-only

mod arguments;
mod monitor;
mod proc_sampler;
mod screen;
mod terminal;

use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!(
        "{} {}: application foundation only; traffic monitoring is not implemented yet",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    ExitCode::FAILURE
}
