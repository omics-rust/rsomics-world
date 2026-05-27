use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub mod gtcheck;

pub use gtcheck::{GtcheckArgs, GtcheckMode, run_gtcheck};

fn open_maybe_gz(path: &Path) -> Result<Box<dyn Read>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut peek = [0u8; 2];
    let mut buf = BufReader::new(file);
    let n = buf.read(&mut peek).map_err(RsomicsError::Io)?;
    let is_gz = n == 2 && peek[0] == 0x1f && peek[1] == 0x8b;

    // Prepend the already-read bytes before the still-buffered remainder
    let chain = std::io::Cursor::new(peek[..n].to_vec()).chain(buf);
    if is_gz {
        Ok(Box::new(flate2::read::GzDecoder::new(chain)))
    } else {
        Ok(Box::new(chain))
    }
}

pub(crate) fn open_vcf_reader(path: &Path) -> Result<Box<dyn BufRead>> {
    Ok(Box::new(BufReader::new(open_maybe_gz(path)?)))
}

pub(crate) fn write_all(w: &mut impl Write, buf: &[u8]) -> Result<()> {
    w.write_all(buf).map_err(RsomicsError::Io)
}
