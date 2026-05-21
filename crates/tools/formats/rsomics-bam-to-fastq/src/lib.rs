use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use noodles::bam;
use rsomics_common::{Result, RsomicsError};

pub fn bam_to_fastq(input: &Path, output: &mut dyn Write, include_secondary: bool) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = bam::io::Reader::new(file);
    let _header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut count: u64 = 0;

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();

        if !include_secondary && (flags.is_secondary() || flags.is_supplementary()) {
            continue;
        }

        let name: &[u8] = record.name().map_or(&b"*"[..], AsRef::as_ref);
        let seq = record.sequence();
        let qual = record.quality_scores();

        out.write_all(b"@").map_err(RsomicsError::Io)?;
        out.write_all(name).map_err(RsomicsError::Io)?;
        out.write_all(b"\n").map_err(RsomicsError::Io)?;

        let mut bases: Vec<u8> = seq.iter().collect();

        if flags.is_reverse_complemented() {
            bases.reverse();
            for b in &mut bases {
                *b = complement(*b);
            }
        }
        out.write_all(&bases).map_err(RsomicsError::Io)?;
        out.write_all(b"\n+\n").map_err(RsomicsError::Io)?;

        let qual_bytes: &[u8] = qual.as_ref();
        if flags.is_reverse_complemented() {
            let rqual: Vec<u8> = qual_bytes.iter().rev().map(|&q| q + 33).collect();
            out.write_all(&rqual).map_err(RsomicsError::Io)?;
        } else {
            let fqual: Vec<u8> = qual_bytes.iter().map(|&q| q + 33).collect();
            out.write_all(&fqual).map_err(RsomicsError::Io)?;
        }
        out.write_all(b"\n").map_err(RsomicsError::Io)?;

        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn complement(base: u8) -> u8 {
    match base {
        b'A' => b'T',
        b'T' | b'U' => b'A',
        b'C' => b'G',
        b'G' => b'C',
        b'W' | b'S' => base,
        b'M' => b'K',
        b'K' => b'M',
        b'R' => b'Y',
        b'Y' => b'R',
        b'B' => b'V',
        b'D' => b'H',
        b'H' => b'D',
        b'V' => b'B',
        _ => b'N',
    }
}
