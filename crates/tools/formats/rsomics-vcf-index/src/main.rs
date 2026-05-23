mod cli;

use std::process::ExitCode;

use clap::Parser;
use rsomics_common::Tool;
use rsomics_help::{intercept_help, render as render_help};

use cli::{Cli, HELP};

fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().collect();
    if let Some(mode) = intercept_help(&raw_args) {
        render_help(&HELP, mode);
        return ExitCode::SUCCESS;
    }
    let args = Cli::parse();
    args.run()
}
