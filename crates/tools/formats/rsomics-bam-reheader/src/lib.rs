//! `samtools reheader` port: swap a BAM's header, passing the alignment blocks
//! through verbatim.
//!
//! reheader is IO-bound for the same reason cat is — the alignment records are
//! never inflated. samtools writes the new header, then the leftover of the
//! input's first post-header block via `bgzf_write`, then `bgzf_raw_read` /
//! `bgzf_raw_write`s the rest (bam_reheader.c). This port does the same via
//! [`bgzf_copy`]: parse and drop the input header, write the replacement header,
//! re-emit the boundary block's record tail, and copy every later BGZF block
//! byte-for-byte.
//!
//! The replacement header is SAM text (samtools' `in.header.sam`). Its @SQ
//! reference dictionary must match the input's record refIDs for the output to
//! be valid — samtools does not re-map refIDs, and neither do we; reheader is
//! for fixing @RG/@PG/@CO and SN/AS/SP tags, not reordering references.
//!
//! Out of scope vs samtools: `-c CMD` (pipe the header through a program) and
//! `-i` in-place (CRAM-only; this crate is BAM-only). `-i` on BAM is rejected by
//! samtools too.

mod bgzf_copy;

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use noodles::bam;
use noodles::sam;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

use bgzf_copy::{HeaderReader, HeaderStream, copy_records, write_bgzf};

const READ_BUFFER: usize = 1024 * 1024;
const WRITE_BUFFER: usize = 1024 * 1024;

#[derive(Debug, Default, Clone, Serialize)]
pub struct ReheaderStats {
    pub header_lines: u64,
}

#[derive(Debug, Clone)]
pub struct ReheaderOpts {
    /// Replacement header source: a SAM text file. `None` only when the caller
    /// supplies the header bytes another way (unused here; always Some).
    pub header_file: PathBuf,
    /// Omit the @PG line (`--no-PG`). Always set in compat runs.
    pub no_pg: bool,
}

/// Parse a SAM-text header file into raw BAM header bytes
/// (`magic l_text text n_ref refs`) ready to re-frame as BGZF.
fn build_header_bytes(opts: &ReheaderOpts) -> Result<(Vec<u8>, u64)> {
    let text = std::fs::read(&opts.header_file)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", opts.header_file.display())))?;

    let mut parser = sam::header::Parser::default();
    for line in text.split(|&b| b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            continue;
        }
        parser
            .parse_partial(line)
            .map_err(|e| RsomicsError::InvalidInput(format!("parsing header: {e}")))?;
    }
    let header = parser.finish();

    // Serialise to BAM header bytes via the BAM writer, then inflate the BGZF it
    // produced back to raw uncompressed bytes (header-sized, negligible cost).
    let mut bgzf = Vec::new();
    {
        let mut hw = bam::io::Writer::new(&mut bgzf);
        hw.write_header(&header).map_err(RsomicsError::Io)?;
    }
    let mut raw = Vec::new();
    noodles::bgzf::io::Reader::new(&bgzf[..])
        .read_to_end(&mut raw)
        .map_err(RsomicsError::Io)?;

    let raw = if opts.no_pg {
        raw
    } else {
        splice_pg(&raw, &pg_line())?
    };

    let header_lines = count_header_lines(&raw);
    Ok((raw, header_lines))
}

fn count_header_lines(raw: &[u8]) -> u64 {
    let l_text = u32::from_le_bytes(raw[4..8].try_into().unwrap()) as usize;
    raw[8..8 + l_text].iter().filter(|&&b| b == b'\n').count() as u64
}

fn pg_line() -> String {
    format!(
        "@PG\tID:rsomics-bam-reheader\tPN:rsomics-bam-reheader\tVN:{}\n",
        env!("CARGO_PKG_VERSION")
    )
}

/// Insert `pg` after the last header line, before the binary `n_ref` block.
fn splice_pg(raw: &[u8], pg: &str) -> Result<Vec<u8>> {
    let l_text = u32::from_le_bytes(raw[4..8].try_into().unwrap()) as usize;
    let text_start = 8;
    let text_end = text_start + l_text;
    let text = &raw[text_start..text_end];
    let mut new_text = Vec::with_capacity(text.len() + pg.len());
    new_text.extend_from_slice(text);
    if !new_text.is_empty() && !new_text.ends_with(b"\n") {
        new_text.push(b'\n');
    }
    new_text.extend_from_slice(pg.as_bytes());

    let mut out = Vec::with_capacity(raw.len() + pg.len());
    out.extend_from_slice(&raw[..4]);
    out.extend_from_slice(&u32::try_from(new_text.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&new_text);
    out.extend_from_slice(&raw[text_end..]);
    Ok(out)
}

pub fn reheader(
    input: &Path,
    output_path: Option<&Path>,
    opts: &ReheaderOpts,
) -> Result<ReheaderStats> {
    let (header_bytes, header_lines) = build_header_bytes(opts)?;

    match output_path {
        Some(path) => {
            let file = File::create(path).map_err(|e| {
                RsomicsError::InvalidInput(format!("creating {}: {e}", path.display()))
            })?;
            let mut out = std::io::BufWriter::with_capacity(WRITE_BUFFER, file);
            write_reheadered(input, &mut out, &header_bytes)?;
            out.flush().map_err(RsomicsError::Io)?;
        }
        None => {
            let stdout = std::io::stdout();
            let mut out = std::io::BufWriter::with_capacity(WRITE_BUFFER, stdout.lock());
            write_reheadered(input, &mut out, &header_bytes)?;
            out.flush().map_err(RsomicsError::Io)?;
        }
    }

    Ok(ReheaderStats { header_lines })
}

fn write_reheadered<W: Write>(input: &Path, out: &mut W, header_bytes: &[u8]) -> Result<()> {
    // New header first.
    write_bgzf(out, header_bytes)?;

    // Skip the input's own header, then block-copy the alignment records.
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut reader = BufReader::with_capacity(READ_BUFFER, file);
    let mut state = HeaderReader::new();
    {
        let stream = HeaderStream::new(&mut reader, &mut state);
        let mut bam_reader = bam::io::Reader::from(stream);
        bam_reader.read_header().map_err(RsomicsError::Io)?;
    }
    copy_records(state, &mut reader, out)?;

    // Single trailing BGZF EOF marker.
    out.write_all(&bgzf_copy::BGZF_EOF)
        .map_err(RsomicsError::Io)?;
    Ok(())
}
