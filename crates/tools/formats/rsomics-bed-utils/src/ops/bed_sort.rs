use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_intervals::{IntervalSet, bed};

pub fn sort_bed(input: &Path, output: &mut dyn Write) -> Result<()> {
    let intervals = bed::read(File::open(input).map_err(RsomicsError::Io)?)?;
    let mut set: IntervalSet = intervals.into_iter().collect();
    set.sort();
    bed::write_bed3(output, set.iter().cloned())
}

pub fn sort_bed_stdin(output: &mut dyn Write) -> Result<()> {
    let mut buf = Vec::new();
    io::stdin()
        .read_to_end(&mut buf)
        .map_err(RsomicsError::Io)?;
    let intervals = bed::read_bytes(&buf)?;
    let mut set: IntervalSet = intervals.into_iter().collect();
    set.sort();
    bed::write_bed3(output, set.iter().cloned())
}
