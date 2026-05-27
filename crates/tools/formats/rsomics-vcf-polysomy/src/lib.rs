pub mod fitting;
pub mod histogram;
pub mod io;
pub mod model;

use std::io::Write;
use std::path::Path;

pub use model::{CnEstimate, PolysomyArgs};

/// Run the full polysomy analysis pipeline.
///
/// Reads BAF values from `input` (VCF/BCF), builds per-chromosome histograms,
/// fits CN2/CN3/CN4 Gaussian mixture models, and writes `dist.dat` into `output_dir`.
pub fn run_polysomy(
    input: &Path,
    output_dir: &Path,
    args: &PolysomyArgs,
    version_str: &str,
    cmd_str: &str,
) -> rsomics_common::Result<()> {
    let dists = io::read_baf_distributions(input, args)?;
    std::fs::create_dir_all(output_dir)?;
    let dat_path = output_dir.join("dist.dat");
    let mut w = std::io::BufWriter::new(std::fs::File::create(&dat_path)?);
    write_header(&mut w, version_str, cmd_str)?;
    for dist in &dists {
        model::fit_and_write(&mut w, dist, args)?;
    }
    Ok(())
}

fn write_header(w: &mut impl Write, version_str: &str, cmd_str: &str) -> std::io::Result<()> {
    writeln!(
        w,
        "# This file was produced by: bcftools polysomy({version_str}), the command line was:"
    )?;
    writeln!(w, "# \t bcftools {cmd_str}")?;
    writeln!(w, "#")?;
    writeln!(w, "# DIST\t[2]Chrom\t[3]BAF\t[4]Normalized Count")?;
    writeln!(
        w,
        "# FIT\t[2]Goodness of Fit\t[3]iFrom\t[4]iTo\t[5]The Fitted Function"
    )?;
    writeln!(
        w,
        "# CN\t[2]Chrom\t[3]Estimated Copy Number\t[4]Absolute fit deviation"
    )
}
