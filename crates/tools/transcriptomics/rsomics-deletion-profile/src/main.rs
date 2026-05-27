mod cli;

use std::process::ExitCode;

use clap::Parser;
use rsomics_common::{ToolMeta, run};

use cli::Cli;

const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

fn main() -> ExitCode {
    let args = Cli::parse();
    let common = args.common.clone();
    run(&common, META, || args.execute())
}
