mod cli;
use clap::Parser;
use cli::{Cli, META};
use rsomics_common::run;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Cli::parse();
    let common = args.common.clone();
    run(&common, META, || args.execute())
}
