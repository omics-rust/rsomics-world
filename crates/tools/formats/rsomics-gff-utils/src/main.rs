use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, RsomicsError};

use rsomics_gff_utils::ops;

#[derive(Parser)]
#[command(name = "rsomics-gff-utils", version, about = "GFF/GTF utility toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    common: CommonFlags,
}

#[derive(Subcommand)]
enum Command {
    /// Count GFF records
    Count { input: PathBuf },
    /// List unique chromosomes
    Chroms {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// List unique attribute keys
    Attributes {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract CDS features
    Cds {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Count exon features
    ExonCount { input: PathBuf },
    /// Extract exon features
    Exons {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract attribute values as TSV
    Extract {
        input: PathBuf,
        #[arg(short = 'k', long, num_args = 1..)]
        keys: Vec<String>,
        #[arg(long = "type")]
        feature_type: Option<String>,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// List feature types with counts
    Features {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter by type and/or regex
    Filter {
        input: PathBuf,
        #[arg(long = "type")]
        feature_type: Option<String>,
        #[arg(short = 'e', long)]
        pattern: Option<String>,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Count gene features
    GeneCount { input: PathBuf },
    /// Extract gene features
    Genes {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Grep by regex
    Grep {
        input: PathBuf,
        #[arg(short = 'e', long)]
        pattern: String,
        #[arg(long)]
        attr_only: bool,
        #[arg(long)]
        invert: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Infer introns from exon positions
    Introns {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Print feature lengths
    Len {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// List unique Parent attribute values
    Parents {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Rename chromosomes via mapping file
    Rename {
        input: PathBuf,
        #[arg(short = 'm', long)]
        map: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sort by chrom + position
    Sort {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// List source column values with counts
    Sources {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Split by chromosome into separate files
    Split {
        input: PathBuf,
        #[arg(short = 'o', long = "output-prefix")]
        prefix: PathBuf,
    },
    /// Aggregate stats (by type, source, chrom)
    Stats { input: PathBuf },
    /// Strand distribution counts
    StrandStats {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Subset by feature type
    Subset {
        input: PathBuf,
        #[arg(long = "type")]
        feature_type: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Overall summary
    Summary {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert GFF to BED
    ToBed {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract transcript/mRNA features
    Transcripts {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract UTR features
    Utr {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Validate GFF format
    Validate { input: PathBuf },
}

fn open_output(path: &str) -> Result<Box<dyn std::io::Write>> {
    if path == "-" {
        Ok(Box::new(std::io::stdout().lock()))
    } else {
        Ok(Box::new(
            std::fs::File::create(path).map_err(RsomicsError::Io)?,
        ))
    }
}

#[allow(clippy::too_many_lines)]
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Count { input } => {
            let n = ops::count::count(&input)?;
            println!("{n}");
        }
        Command::Chroms { input, output } => {
            let mut out = open_output(&output)?;
            ops::chroms::gff_chroms(&input, &mut out)?;
        }
        Command::Attributes { input, output } => {
            let mut out = open_output(&output)?;
            ops::attributes::gff_attributes(&input, &mut out)?;
        }
        Command::Cds { input, output } => {
            let mut out = open_output(&output)?;
            ops::cds::gff_cds(&input, &mut out)?;
        }
        Command::ExonCount { input } => {
            let n = ops::exon_count::gff_exon_count(&input)?;
            println!("{n}");
        }
        Command::Exons { input, output } => {
            let mut out = open_output(&output)?;
            ops::exons::gff_exons(&input, &mut out)?;
        }
        Command::Extract {
            input,
            keys,
            feature_type,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::extract::extract_attributes(&input, &mut out, &keys, feature_type.as_deref())?;
        }
        Command::Features { input, output } => {
            let mut out = open_output(&output)?;
            ops::features::list_features(&input, &mut out)?;
        }
        Command::Filter {
            input,
            feature_type,
            pattern,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::filter::filter_gff(
                &input,
                feature_type.as_deref(),
                pattern.as_deref(),
                &mut out,
            )?;
        }
        Command::GeneCount { input } => {
            let n = ops::gene_count::gff_gene_count(&input)?;
            println!("{n}");
        }
        Command::Genes { input, output } => {
            let mut out = open_output(&output)?;
            ops::genes::gff_genes(&input, &mut out)?;
        }
        Command::Grep {
            input,
            pattern,
            attr_only,
            invert,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::grep::grep(&input, &pattern, attr_only, invert, &mut out)?;
        }
        Command::Introns { input, output } => {
            let mut out = open_output(&output)?;
            ops::introns::gff_introns(&input, &mut out)?;
        }
        Command::Len { input, output } => {
            let mut out = open_output(&output)?;
            ops::len::gff_len(&input, &mut out)?;
        }
        Command::Parents { input, output } => {
            let mut out = open_output(&output)?;
            ops::parents::gff_parents(&input, &mut out)?;
        }
        Command::Rename { input, map, output } => {
            let mut out = open_output(&output)?;
            ops::rename::rename_gff(&input, &map, &mut out)?;
        }
        Command::Sort { input, output } => {
            let mut out = open_output(&output)?;
            ops::sort::sort_gff(&input, &mut out)?;
        }
        Command::Sources { input, output } => {
            let mut out = open_output(&output)?;
            ops::sources::gff_sources(&input, &mut out)?;
        }
        Command::Split { input, prefix } => {
            let counts = ops::split::split_gff(&input, &prefix)?;
            for (chrom, count) in &counts {
                eprintln!("{chrom}\t{count}");
            }
        }
        Command::Stats { input } => {
            let s = ops::stats::stats(&input)?;
            println!("total\t{}", s.total);
            println!("chromosomes\t{}", s.by_chrom.len());
            println!("sources\t{}", s.by_source.len());
            println!("feature_types\t{}", s.by_type.len());
            println!();
            for (t, c) in &s.by_type {
                println!("{t}\t{c}");
            }
        }
        Command::StrandStats { input, output } => {
            let mut out = open_output(&output)?;
            ops::strand_stats::gff_strand_stats(&input, &mut out)?;
        }
        Command::Subset {
            input,
            feature_type,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::subset::gff_subset(&input, &mut out, &feature_type)?;
        }
        Command::Summary { input, output } => {
            let mut out = open_output(&output)?;
            ops::summary::gff_summary(&input, &mut out)?;
        }
        Command::ToBed { input, output } => {
            let mut out = open_output(&output)?;
            ops::to_bed::gff_to_bed(&input, &mut out)?;
        }
        Command::Transcripts { input, output } => {
            let mut out = open_output(&output)?;
            ops::transcripts::gff_transcripts(&input, &mut out)?;
        }
        Command::Utr { input, output } => {
            let mut out = open_output(&output)?;
            ops::utr::gff_utr(&input, &mut out)?;
        }
        Command::Validate { input } => {
            let result = ops::validate::validate_gff(&input)?;
            if result.is_valid {
                eprintln!("OK: {} records, no errors", result.records);
            } else {
                eprintln!(
                    "INVALID: {} records, {} errors:",
                    result.records,
                    result.errors.len()
                );
                for err in &result.errors {
                    eprintln!("  {err}");
                }
                return Err(RsomicsError::InvalidInput("validation failed".into()));
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let common = cli.common.clone();
    rsomics_common::run(
        &common,
        rsomics_common::ToolMeta {
            name: "rsomics-gff-utils",
            version: env!("CARGO_PKG_VERSION"),
        },
        || run(cli),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
