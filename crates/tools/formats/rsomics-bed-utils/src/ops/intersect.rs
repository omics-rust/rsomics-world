use std::fs::File;
use std::io::Write;
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_intervals::{IntervalSet, bed, intersect as iv_intersect};

pub fn intersect(a_path: &Path, b_path: &Path, output: &mut dyn Write) -> Result<()> {
    let a_ivs = bed::read(File::open(a_path).map_err(RsomicsError::Io)?)?;
    let b_ivs = bed::read(File::open(b_path).map_err(RsomicsError::Io)?)?;
    let a_set: IntervalSet = a_ivs.into_iter().collect();
    let b_set: IntervalSet = b_ivs.into_iter().collect();
    let out = iv_intersect(&a_set, &b_set);
    bed::write_bed3(output, out.iter().cloned())
}
