use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin};

use rsomics_bed_utils::ops;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser)]
#[command(
    name = "rsomics-bed-utils",
    version,
    about = "BED utility toolkit",
    disable_help_flag = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Annotate BED intervals with nearest GFF features
    Annotate {
        bed: PathBuf,
        gff: PathBuf,
        #[arg(long = "type", default_value = "gene")]
        feature_type: String,
        #[arg(long, default_value = "gene_name")]
        attribute: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sort BED by chrom + start (interval-set based)
    Sort {
        #[arg(short = 'i', long, default_value = "-")]
        input: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// List unique chromosomes
    Chroms {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Chromosome-level base-pair totals
    ChromsSizes {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Find closest feature in B for each interval in A
    Closest {
        a: PathBuf,
        b: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Complement: regions NOT covered by input BED
    Complement {
        #[arg(short = 'i', long)]
        input: PathBuf,
        #[arg(short = 'g', long)]
        genome: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Count BED records
    Count { input: PathBuf },
    /// Coverage histogram
    CoverageHist {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Add flanking regions (genome-aware)
    Flank {
        input: PathBuf,
        #[arg(short = 'g', long)]
        genome: PathBuf,
        #[arg(short = 'l', long, default_value_t = 0)]
        left: u64,
        #[arg(short = 'r', long, default_value_t = 0)]
        right: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Add fixed-bp flanks (no genome file)
    FlankBp {
        input: PathBuf,
        #[arg(short = 'b', long)]
        bp: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Genome coverage (per-base)
    Genomecov {
        input: PathBuf,
        #[arg(short = 'g', long)]
        genome: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract FASTA sequences for BED intervals
    Getfasta {
        bed: PathBuf,
        #[arg(short = 'f', long)]
        fasta: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Intersect two BED files
    Intersect {
        #[arg(short = 'a', long)]
        a: PathBuf,
        #[arg(short = 'b', long)]
        b: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Jaccard similarity between two BED files
    Jaccard { a: PathBuf, b: PathBuf },
    /// Print interval lengths
    Len {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Create windows across genome
    Makewindows {
        #[arg(short = 'g', long)]
        genome: PathBuf,
        #[arg(short = 'w', long)]
        window: u64,
        #[arg(short = 's', long, default_value_t = 0)]
        step: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Map values from B onto A intervals
    Map {
        a: PathBuf,
        b: PathBuf,
        #[arg(long, default_value = "mean")]
        operation: String,
        #[arg(short = 'c', long, default_value_t = 4)]
        column: usize,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Merge overlapping intervals (pre-sorted input)
    Merge {
        #[arg(short = 'i', long, default_value = "-")]
        input: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Merge intervals sharing the same name (col 4)
    MergeByName {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Merge overlapping intervals (standalone)
    MergeOverlaps {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Replace each interval with its midpoint
    Midpoint {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Nucleotide content per BED interval
    Nuc {
        bed: PathBuf,
        #[arg(short = 'f', long)]
        fasta: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Overlap statistics between two BED files
    Overlap { a: PathBuf, b: PathBuf },
    /// Generate promoter regions from gene BED
    Promoters {
        input: PathBuf,
        #[arg(short = 'u', long, default_value_t = 2000)]
        upstream: u64,
        #[arg(short = 'd', long, default_value_t = 200)]
        downstream: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Generate random BED intervals
    Random {
        #[arg(short = 'g', long)]
        genome: PathBuf,
        #[arg(short = 'n', long)]
        n: u64,
        #[arg(short = 'l', long, default_value_t = 1000)]
        length: u64,
        #[arg(long, default_value_t = 42)]
        seed: u64,
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
    /// Resize all intervals to fixed size
    Resize {
        input: PathBuf,
        #[arg(short = 's', long)]
        size: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Random sample of N intervals
    Sample {
        input: PathBuf,
        #[arg(short = 'n', long)]
        n: usize,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Shift intervals by offset
    Shift {
        input: PathBuf,
        #[arg(short = 'g', long)]
        genome: PathBuf,
        #[arg(short = 's', long)]
        amount: i64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extend intervals (genome-aware slop)
    Slop {
        input: PathBuf,
        #[arg(short = 'g', long)]
        genome: PathBuf,
        #[arg(short = 'l', long, default_value_t = 0)]
        left: u64,
        #[arg(short = 'r', long, default_value_t = 0)]
        right: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sort by name (col 4)
    SortName {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sort by interval size
    SortSize {
        input: PathBuf,
        #[arg(long)]
        descending: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Inter-interval spacing
    Spacing {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Aggregate BED statistics
    Stats { input: PathBuf },
    /// Subtract B regions from A
    Subtract {
        #[arg(short = 'a', long)]
        a: PathBuf,
        #[arg(short = 'b', long)]
        b: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Print last N intervals
    Tail {
        input: PathBuf,
        #[arg(short = 'n', default_value_t = 10)]
        n: usize,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract FASTA for BED regions
    ToFasta {
        bed: PathBuf,
        #[arg(short = 'f', long)]
        fasta: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert BED to GFF
    ToGff {
        input: PathBuf,
        #[arg(long, default_value = "rsomics")]
        source: String,
        #[arg(long = "type", default_value = "region")]
        feature_type: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert BED to GFF3
    ToGff3 {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert BED to IGV
    ToIgv {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert BED to WIG
    ToWig {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Total base-pairs covered
    TotalBp { input: PathBuf },
    /// Total span (max end - min start per chrom)
    TotalSpan {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Union of two BED files
    Union {
        a: PathBuf,
        b: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Deduplicate identical intervals
    Unique {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Validate BED format
    Validate { input: PathBuf },
    /// Window-based overlap between A and B
    Window {
        a: PathBuf,
        b: PathBuf,
        #[arg(short = 'w', long, default_value_t = 1000)]
        window: u64,
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

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    #[allow(clippy::too_many_lines)]
    fn execute(self) -> Result<()> {
        match self.command {
            Command::Annotate {
                bed,
                gff,
                feature_type,
                attribute,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::annotate::annotate_bed(&bed, &gff, &mut out, &feature_type, &attribute)?;
            }
            Command::Sort { input, output } => {
                let mut out = open_output(&output)?;
                if input == "-" {
                    ops::bed_sort::sort_bed_stdin(&mut out)?;
                } else {
                    ops::bed_sort::sort_bed(std::path::Path::new(&input), &mut out)?;
                }
            }
            Command::Chroms { input, output } => {
                let mut out = open_output(&output)?;
                ops::chroms::bed_chroms(&input, &mut out)?;
            }
            Command::ChromsSizes { input, output } => {
                let mut out = open_output(&output)?;
                ops::chroms_sizes::bed_chroms_sizes(&input, &mut out)?;
            }
            Command::Closest { a, b, output } => {
                let mut out = open_output(&output)?;
                ops::closest::closest(&a, &b, &mut out)?;
            }
            Command::Complement {
                input,
                genome,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::complement::complement(&input, &genome, &mut out)?;
            }
            Command::Count { input } => {
                let n = ops::count::count(&input)?;
                println!("{n}");
            }
            Command::CoverageHist { input, output } => {
                let mut out = open_output(&output)?;
                ops::coverage_hist::bed_coverage_hist(&input, &mut out)?;
            }
            Command::Flank {
                input,
                genome,
                left,
                right,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::flank::flank(&input, &genome, left, right, &mut out)?;
            }
            Command::FlankBp { input, bp, output } => {
                let mut out = open_output(&output)?;
                ops::flank_bp::bed_flank_bp(&input, &mut out, bp)?;
            }
            Command::Genomecov {
                input,
                genome,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::genomecov::genomecov(&input, &genome, &mut out)?;
            }
            Command::Getfasta { bed, fasta, output } => {
                let mut out = open_output(&output)?;
                ops::getfasta::getfasta(&bed, &fasta, &mut out)?;
            }
            Command::Intersect { a, b, output } => {
                let mut out = open_output(&output)?;
                ops::intersect::intersect(&a, &b, &mut out)?;
            }
            Command::Jaccard { a, b } => {
                let r = ops::jaccard::jaccard(&a, &b)?;
                println!(
                    "intersection\t{}\nunion\t{}\njaccard\t{:.6}\nn_intersections\t{}",
                    r.intersection, r.union, r.jaccard, r.n_intersections
                );
            }
            Command::Len { input, output } => {
                let mut out = open_output(&output)?;
                ops::len::lengths(&input, &mut out)?;
            }
            Command::Makewindows {
                genome,
                window,
                step,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::makewindows::makewindows(&genome, window, step, &mut out)?;
            }
            Command::Map {
                a,
                b,
                operation,
                column,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::map::map_bed(&a, &b, &mut out, &operation, column)?;
            }
            Command::Merge { input, output } => {
                let mut out = open_output(&output)?;
                if input == "-" {
                    ops::merge::merge_stdin(&mut out)?;
                } else {
                    ops::merge::merge(std::path::Path::new(&input), &mut out)?;
                }
            }
            Command::MergeByName { input, output } => {
                let mut out = open_output(&output)?;
                ops::merge_by_name::merge_by_name(&input, &mut out)?;
            }
            Command::MergeOverlaps { input, output } => {
                let mut out = open_output(&output)?;
                ops::merge_overlaps::bed_merge_overlaps(&input, &mut out)?;
            }
            Command::Midpoint { input, output } => {
                let mut out = open_output(&output)?;
                ops::midpoint::bed_midpoint(&input, &mut out)?;
            }
            Command::Nuc { bed, fasta, output } => {
                let mut out = open_output(&output)?;
                ops::nuc::bed_nuc(&bed, &fasta, &mut out)?;
            }
            Command::Overlap { a, b } => {
                let r = ops::overlap::compute_overlap(&a, &b)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&r)
                        .map_err(|e| { RsomicsError::InvalidInput(format!("json: {e}")) })?
                );
            }
            Command::Promoters {
                input,
                upstream,
                downstream,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::promoters::bed_promoters(&input, &mut out, upstream, downstream)?;
            }
            Command::Random {
                genome,
                n,
                length,
                seed,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::random::random_bed(&genome, n, length, seed, &mut out)?;
            }
            Command::Rename { input, map, output } => {
                let mut out = open_output(&output)?;
                ops::rename::rename_bed(&input, &map, &mut out)?;
            }
            Command::Resize {
                input,
                size,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::resize::bed_resize(&input, &mut out, size)?;
            }
            Command::Sample {
                input,
                n,
                seed,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::sample::sample_bed(&input, &mut out, n, seed)?;
            }
            Command::Shift {
                input,
                genome,
                amount,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::shift::shift(&input, &genome, amount, &mut out)?;
            }
            Command::Slop {
                input,
                genome,
                left,
                right,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::slop::slop(&input, &genome, left, right, &mut out)?;
            }
            Command::SortName { input, output } => {
                let mut out = open_output(&output)?;
                ops::sort_name::bed_sort_name(&input, &mut out)?;
            }
            Command::SortSize {
                input,
                descending,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::sort_size::sort_by_size(&input, &mut out, descending)?;
            }
            Command::Spacing { input, output } => {
                let mut out = open_output(&output)?;
                ops::spacing::bed_spacing(&input, &mut out)?;
            }
            Command::Stats { input } => {
                let s = ops::stats::stats(&input)?;
                println!("count\t{}", s.count);
                println!("total_bases\t{}", s.total_bases);
                println!("mean_len\t{:.1}", s.mean_len);
                println!("median_len\t{:.1}", s.median_len);
                println!("min_len\t{}", s.min_len);
                println!("max_len\t{}", s.max_len);
            }
            Command::Subtract { a, b, output } => {
                let mut out = open_output(&output)?;
                ops::subtract::subtract(&a, &b, &mut out)?;
            }
            Command::Tail { input, n, output } => {
                let mut out = open_output(&output)?;
                ops::tail::tail(&input, &mut out, n)?;
            }
            Command::ToFasta { bed, fasta, output } => {
                let mut out = open_output(&output)?;
                ops::to_fasta::bed_to_fasta(&bed, &fasta, &mut out)?;
            }
            Command::ToGff {
                input,
                source,
                feature_type,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::to_gff::bed_to_gff(&input, &source, &feature_type, &mut out)?;
            }
            Command::ToGff3 { input, output } => {
                let mut out = open_output(&output)?;
                ops::to_gff3::bed_to_gff3(&input, &mut out)?;
            }
            Command::ToIgv { input, output } => {
                let mut out = open_output(&output)?;
                ops::to_igv::bed_to_igv(&input, &mut out)?;
            }
            Command::ToWig { input, output } => {
                let mut out = open_output(&output)?;
                ops::to_wig::bed_to_wig(&input, &mut out)?;
            }
            Command::TotalBp { input } => {
                let n = ops::total_bp::bed_total_bp(&input)?;
                println!("{n}");
            }
            Command::TotalSpan { input, output } => {
                let mut out = open_output(&output)?;
                ops::total_span::bed_total_span(&input, &mut out)?;
            }
            Command::Union { a, b, output } => {
                let mut out = open_output(&output)?;
                ops::union::bed_union(&a, &b, &mut out)?;
            }
            Command::Unique { input, output } => {
                let mut out = open_output(&output)?;
                let (total, unique) = ops::unique::bed_unique(&input, &mut out)?;
                eprintln!("{total} total, {unique} unique");
            }
            Command::Validate { input } => {
                let r = ops::validate::validate_bed(&input)?;
                if r.is_valid {
                    eprintln!("OK: {} records, no errors", r.records);
                } else {
                    eprintln!("INVALID: {} records, {} errors:", r.records, r.errors.len());
                    for err in &r.errors {
                        eprintln!("  {err}");
                    }
                    return Err(RsomicsError::InvalidInput("validation failed".into()));
                }
            }
            Command::Window {
                a,
                b,
                window,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::window::window_bed(&a, &b, &mut out, window)?;
            }
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "BED utility toolkit — sort, merge, intersect, subtract, complement, stats, convert, and 40+ more operations.",
    origin: Some(Origin {
        upstream: "bedtools",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<COMMAND> [OPTIONS] <input>"],
    sections: &[],
    examples: &[
        Example {
            description: "Count intervals",
            command: "rsomics-bed-utils count peaks.bed",
        },
        Example {
            description: "Merge overlapping intervals",
            command: "rsomics-bed-utils merge -i peaks.bed -o merged.bed",
        },
        Example {
            description: "Intersect two BED files",
            command: "rsomics-bed-utils intersect -a a.bed -b b.bed -o out.bed",
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
