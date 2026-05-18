use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn concat_fasta(inputs: &[&Path], output: &mut dyn Write) -> Result<u64> {
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut total_bytes: u64 = 0;

    for path in inputs {
        let file = File::open(path)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
        let mut reader = BufReader::new(file);
        let bytes = io::copy(&mut reader, &mut out).map_err(RsomicsError::Io)?;
        total_bytes += bytes;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(total_bytes)
}
