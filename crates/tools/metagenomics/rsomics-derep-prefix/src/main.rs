mod cli;

use clap::Parser;
use rsomics_common::Tool;

fn main() {
    cli::Cli::parse().run();
}
