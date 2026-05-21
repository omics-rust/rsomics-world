use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec};

use rsomics_pdb_chain::{extract_chain, list_chains, split_chains};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-pdb-chain", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List all chain IDs
    List { input: PathBuf },
    /// Extract a single chain
    Extract {
        input: PathBuf,
        #[arg(short = 'c', long)]
        chain: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Split all chains into separate files
    Split {
        input: PathBuf,
        #[arg(short = 'o', long = "output-prefix")]
        prefix: PathBuf,
    },
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }
    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        match self.command {
            Command::List { input } => {
                list_chains(&input, &mut std::io::stdout().lock())?;
            }
            Command::Extract {
                input,
                chain,
                output,
            } => {
                let mut out: Box<dyn std::io::Write> = if output == "-" {
                    Box::new(std::io::stdout().lock())
                } else {
                    Box::new(std::fs::File::create(&output).map_err(RsomicsError::Io)?)
                };
                let n = extract_chain(&input, &chain, &mut out)?;
                if !self.common.quiet {
                    eprintln!("{n} atoms extracted for chain {chain}");
                }
            }
            Command::Split { input, prefix } => {
                let counts = split_chains(&input, &prefix)?;
                if !self.common.quiet {
                    for (chain, count) in &counts {
                        eprintln!("chain {chain}: {count} atoms");
                    }
                }
            }
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "PDB chain operations — list, extract, or split by chain ID.",
    origin: None,
    usage_lines: &[
        "list <input.pdb>",
        "extract <input.pdb> -c A [-o chain_A.pdb]",
        "split <input.pdb> -o <prefix>",
    ],
    sections: &[],
    examples: &[
        Example {
            description: "List chains",
            command: "rsomics-pdb-chain list structure.pdb",
        },
        Example {
            description: "Extract chain A",
            command: "rsomics-pdb-chain extract structure.pdb -c A -o chain_A.pdb",
        },
    ],
    json_result_schema_doc: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
