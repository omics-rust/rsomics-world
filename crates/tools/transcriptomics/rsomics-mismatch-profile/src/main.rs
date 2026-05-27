mod cli;

use std::process::ExitCode;

use clap::Parser;
use rsomics_common::run;

use cli::{Cli, META};

fn main() -> ExitCode {
    let args = Cli::parse();
    let common = args.common.clone();
    run(&common, META, || args.execute())
}
