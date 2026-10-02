// SPDX-License-Identifier: GPL-2.0-only

mod application;
mod arguments;
mod monitor;
mod proc_sampler;
mod screen;
mod terminal;

use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(application::invoke(
        std::env::args_os().skip(1),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
        application::linux,
    ))
}
