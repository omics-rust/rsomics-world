use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_intervals::bed;

pub fn merge(input: &Path, output: &mut dyn Write) -> Result<()> {
    let w = BufWriter::new(output);
    let f = File::open(input).map_err(RsomicsError::Io)?;
    bed::merge_sorted(f, w)
}

pub fn merge_stdin(output: &mut dyn Write) -> Result<()> {
    let w = BufWriter::new(output);
    bed::merge_sorted(io::stdin().lock(), w)
}
