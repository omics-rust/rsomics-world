use std::io::{BufWriter, Write};
use std::path::Path;

use rand::SeedableRng;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rsomics_common::{Result, RsomicsError};

pub fn downsample(
    input: &Path,
    fraction: f64,
    seed: u64,
    output: &mut dyn Write,
) -> Result<(u64, u64)> {
    let mut reader = needletail::parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut rng = StdRng::seed_from_u64(seed);
    let dist = Uniform::new(0.0f64, 1.0);
    let mut total: u64 = 0;
    let mut kept: u64 = 0;

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        total += 1;

        if dist.sample(&mut rng) < fraction {
            let name = record.id();
            let seq = record.seq();
            let qual = record.qual();

            out.write_all(b"@").map_err(RsomicsError::Io)?;
            out.write_all(name).map_err(RsomicsError::Io)?;
            out.write_all(b"\n").map_err(RsomicsError::Io)?;
            out.write_all(&seq).map_err(RsomicsError::Io)?;
            out.write_all(b"\n+\n").map_err(RsomicsError::Io)?;
            if let Some(q) = qual {
                out.write_all(q.as_ref()).map_err(RsomicsError::Io)?;
            }
            out.write_all(b"\n").map_err(RsomicsError::Io)?;
            kept += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, kept))
}
