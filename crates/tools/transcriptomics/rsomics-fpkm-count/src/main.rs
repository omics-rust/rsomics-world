mod cli;

use clap::Parser;
use rsomics_common::{ToolMeta, run};

const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

fn main() -> std::process::ExitCode {
    let args = cli::Cli::parse();
    let common = args.common.clone();
    run(&common, META, || args.execute())
}
