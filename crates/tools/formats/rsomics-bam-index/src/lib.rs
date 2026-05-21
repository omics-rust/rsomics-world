use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};

pub fn index_bam(bam_path: &Path) -> Result<()> {
    let bai_path = bam_path.with_extension("bam.bai");

    let index = bam::fs::index(bam_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;

    bam::bai::fs::write(&bai_path, &index)
        .map_err(|e| RsomicsError::InvalidInput(format!("writing index: {e}")))?;

    Ok(())
}
