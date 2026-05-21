use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};

use rsomics_vcf_utils::ops;

const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser)]
#[command(name = "rsomics-vcf-utils", version, about = "VCF utility toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    common: CommonFlags,
}

#[derive(Subcommand)]
enum Command {
    /// Extract allele frequencies
    Af {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter to biallelic variants
    Biallelic {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// List unique chromosomes
    Chroms { input: PathBuf },
    /// Count VCF records
    Count { input: PathBuf },
    /// Variant density in windows
    Density {
        input: PathBuf,
        #[arg(short = 'w', long, default_value_t = 1_000_000)]
        window: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract depth (DP)
    Dp {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract genotype matrix (CHROM/POS + per-sample GT)
    Genotypes {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Grep VCF by regex
    Grep {
        input: PathBuf,
        #[arg(short = 'e', long)]
        pattern: String,
        #[arg(long)]
        invert: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Het/hom counts per sample
    HetHom {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter to indels only
    Indels {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract an INFO field value
    Info {
        input: PathBuf,
        #[arg(short = 'k', long)]
        key: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter by minor allele frequency
    Maf {
        input: PathBuf,
        #[arg(long, default_value_t = 0.01)]
        min_maf: f64,
        #[arg(long, default_value_t = 0.5)]
        max_maf: f64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Per-sample missingness report
    Missing {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter to multiallelic variants
    Multiallelic {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter to PASS variants
    Pass {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Variant counts per chromosome
    PerChrom {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter to private variants (one sample only)
    Private {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter by QUAL threshold
    QualFilter {
        input: PathBuf,
        #[arg(long)]
        min_qual: f64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// QUAL score statistics
    QualStats {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Rename chromosomes using a mapping file
    Rename {
        input: PathBuf,
        #[arg(short = 'm', long)]
        map: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// List sample names
    Samples { input: PathBuf },
    /// Filter to singleton variants (AC=1)
    Singletons {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract sites (CHROM + POS only)
    Sites {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter to SNPs only
    Snps {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sort VCF by chromosome + position
    Sort {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Print last N variants
    Tail {
        input: PathBuf,
        #[arg(short = 'n', default_value_t = 10)]
        n: usize,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert VCF to TSV
    ToTsv {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Transition/transversion ratio
    TsTv { input: PathBuf },
    /// Count variant types (SNP/indel/MNP/other)
    TypeCounts {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// View VCF with optional header/region/count filtering
    View {
        input: PathBuf,
        #[arg(short = 'H', long)]
        header_only: bool,
        #[arg(long)]
        no_header: bool,
        #[arg(short = 'c', long)]
        count_only: bool,
        #[arg(short = 'r', long)]
        region: Option<String>,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
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
        Command::Af { input, output } => {
            let mut out = open_output(&output)?;
            ops::af::vcf_af(&input, &mut out)?;
        }
        Command::Biallelic { input, output } => {
            let mut out = open_output(&output)?;
            ops::biallelic::vcf_biallelic(&input, &mut out)?;
        }
        Command::Chroms { input } => {
            let mut out = open_output("-")?;
            ops::chroms::vcf_chroms(&input, &mut out)?;
        }
        Command::Count { input } => {
            let n = ops::count::count(&input)?;
            println!("{n}");
        }
        Command::Density {
            input,
            window,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::density::vcf_density(&input, &mut out, window)?;
        }
        Command::Dp { input, output } => {
            let mut out = open_output(&output)?;
            ops::dp::vcf_dp(&input, &mut out)?;
        }
        Command::Genotypes { input, output } => {
            let mut out = open_output(&output)?;
            ops::genotypes::vcf_genotypes(&input, &mut out)?;
        }
        Command::Grep {
            input,
            pattern,
            invert,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::grep::grep(&input, &pattern, invert, &mut out)?;
        }
        Command::HetHom { input, output } => {
            let mut out = open_output(&output)?;
            ops::het_hom::vcf_het_hom(&input, &mut out)?;
        }
        Command::Indels { input, output } => {
            let mut out = open_output(&output)?;
            ops::indels::vcf_indels(&input, &mut out)?;
        }
        Command::Info { input, key, output } => {
            let mut out = open_output(&output)?;
            ops::info::extract_info(&input, &mut out, &key)?;
        }
        Command::Maf {
            input,
            min_maf,
            max_maf,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::maf::vcf_maf(&input, &mut out, min_maf, max_maf)?;
        }
        Command::Missing { input, output } => {
            let mut out = open_output(&output)?;
            ops::missing::vcf_missing(&input, &mut out)?;
        }
        Command::Multiallelic { input, output } => {
            let mut out = open_output(&output)?;
            ops::multiallelic::vcf_multiallelic(&input, &mut out)?;
        }
        Command::Pass { input, output } => {
            let mut out = open_output(&output)?;
            ops::pass::vcf_pass(&input, &mut out)?;
        }
        Command::PerChrom { input, output } => {
            let mut out = open_output(&output)?;
            ops::per_chrom::vcf_per_chrom(&input, &mut out)?;
        }
        Command::Private { input, output } => {
            let mut out = open_output(&output)?;
            ops::private::vcf_private(&input, &mut out)?;
        }
        Command::QualFilter {
            input,
            min_qual,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::qual_filter::vcf_qual_filter(&input, &mut out, min_qual)?;
        }
        Command::QualStats { input, output } => {
            let mut out = open_output(&output)?;
            ops::qual_stats::vcf_qual_stats(&input, &mut out)?;
        }
        Command::Rename { input, map, output } => {
            let mut out = open_output(&output)?;
            ops::rename::rename_vcf(&input, &map, &mut out)?;
        }
        Command::Samples { input } => {
            let mut out = open_output("-")?;
            ops::samples::vcf_samples(&input, &mut out)?;
        }
        Command::Singletons { input, output } => {
            let mut out = open_output(&output)?;
            ops::singletons::vcf_singletons(&input, &mut out)?;
        }
        Command::Sites { input, output } => {
            let mut out = open_output(&output)?;
            ops::sites::vcf_sites(&input, &mut out)?;
        }
        Command::Snps { input, output } => {
            let mut out = open_output(&output)?;
            ops::snps::vcf_snps(&input, &mut out)?;
        }
        Command::Sort { input, output } => {
            let mut out = open_output(&output)?;
            ops::sort::sort_vcf(&input, &mut out)?;
        }
        Command::Tail { input, n, output } => {
            let mut out = open_output(&output)?;
            ops::tail::tail(&input, &mut out, n)?;
        }
        Command::ToTsv { input, output } => {
            let mut out = open_output(&output)?;
            ops::to_tsv::vcf_to_tsv(&input, &mut out)?;
        }
        Command::TsTv { input } => {
            let r = ops::ts_tv::vcf_ts_tv(&input)?;
            println!("transitions\t{}", r.ts);
            println!("transversions\t{}", r.tv);
            println!("Ts/Tv\t{:.4}", r.ratio);
        }
        Command::TypeCounts { input, output } => {
            let mut out = open_output(&output)?;
            ops::type_counts::vcf_type_counts(&input, &mut out)?;
        }
        Command::View {
            input,
            header_only,
            no_header,
            count_only,
            region,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::view::view_vcf(
                &input,
                &mut out,
                header_only,
                no_header,
                count_only,
                region.as_deref(),
            )?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let common = cli.common.clone();
    rsomics_common::run(&common, META, || run(cli))
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
