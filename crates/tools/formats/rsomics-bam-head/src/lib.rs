//! `samtools head` port: print a BAM's header and the first N alignment records
//! as SAM text.
//!
//! Unlike cat/reheader, head's output is uncompressed SAM on stdout, so there
//! is no BGZF block copy to do — the work is decode the header text and the
//! first `nrecords` records, then stop. Reading stops the instant the record
//! budget is met, so on a multi-GB BAM head touches only the first block or two
//! regardless of file size (sam_view.c `main_head`).
//!
//! Defaults mirror samtools (sam_view.c): all header lines, zero records. `-h N`
//! prints the first N header lines (by counting newlines in the raw header
//! text, exactly as `main_head` does); `-n N` then prints the first N records.
//! The header text is emitted verbatim from the BAM's stored text — not
//! re-serialised — so it is byte-identical to `samtools head` / `sam_hdr_str`.

mod sam_format;

use std::io::{BufWriter, Read, Write};
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::sam;
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

const WRITE_BUFFER: usize = 256 * 1024;

#[derive(Debug, Default, Clone, Serialize)]
pub struct HeadStats {
    pub header_lines: u64,
    pub records: u64,
}

#[derive(Debug, Clone, Default)]
pub struct HeadOpts {
    /// `-h N`: print only the first N header lines. `None` = all (the default).
    pub header_lines: Option<u64>,
    /// `-n N`: also print the first N alignment records. Default 0.
    pub records: u64,
}

/// Truncate the raw header text to its first `n` newline-terminated lines,
/// matching sam_view.c `main_head`: scan for the nth `\n` and emit up to and
/// including it; if fewer than `n` lines exist, emit the whole text. The bytes
/// are the BAM's stored header text verbatim, so output matches `samtools head`.
fn truncate_header_lines(text: &[u8], n: u64) -> &[u8] {
    let mut end = 0usize;
    let mut seen = 0u64;
    while seen < n {
        match text[end..].iter().position(|&b| b == b'\n') {
            Some(rel) => {
                end += rel + 1;
                seen += 1;
            }
            None => return text,
        }
    }
    &text[..end]
}

pub fn head(input: &Path, output_path: Option<&Path>, opts: &HeadOpts) -> Result<HeadStats> {
    match output_path {
        Some(path) => {
            let file = std::fs::File::create(path).map_err(|e| {
                RsomicsError::InvalidInput(format!("creating {}: {e}", path.display()))
            })?;
            let mut out = BufWriter::with_capacity(WRITE_BUFFER, file);
            let stats = run(input, &mut out, opts)?;
            out.flush().map_err(RsomicsError::Io)?;
            Ok(stats)
        }
        None => {
            let stdout = std::io::stdout();
            let mut out = BufWriter::with_capacity(WRITE_BUFFER, stdout.lock());
            let stats = run(input, &mut out, opts)?;
            out.flush().map_err(RsomicsError::Io)?;
            Ok(stats)
        }
    }
}

fn run<W: Write>(input: &Path, out: &mut W, opts: &HeadOpts) -> Result<HeadStats> {
    let mut reader = rsomics_bamio::open_with_workers(input, NonZero::<usize>::MIN)?;
    let (header, header_text) = read_header_and_text(&mut reader)?;

    let emitted = match opts.header_lines {
        Some(n) => truncate_header_lines(&header_text, n),
        None => &header_text,
    };
    out.write_all(emitted).map_err(RsomicsError::Io)?;
    let header_lines = emitted.iter().filter(|&&b| b == b'\n').count() as u64;

    let mut written = 0u64;
    if opts.records > 0 {
        // Reference names indexed by refID for RNAME/RNEXT rendering.
        let ref_names: Vec<Vec<u8>> = header
            .reference_sequences()
            .keys()
            .map(|name| name.to_vec())
            .collect();

        // Format raw record payloads straight to SAM bytes via our own
        // formatter, reusing a single `RawRecord` and appending into a batch
        // buffer flushed every ~64 KiB — no per-record allocation and one write
        // per batch, beating noodles' generic record writer.
        const FLUSH_AT: usize = 64 * 1024;
        let mut record = RawRecord::default();
        let mut batch = Vec::with_capacity(FLUSH_AT + 1024);
        let inner = reader.get_mut();
        while written < opts.records {
            if raw::read_record(inner, &mut record)? == 0 {
                break;
            }
            sam_format::format_record(&mut batch, record.as_bytes(), &ref_names)?;
            written += 1;
            if batch.len() >= FLUSH_AT {
                out.write_all(&batch).map_err(RsomicsError::Io)?;
                batch.clear();
            }
        }
        if !batch.is_empty() {
            out.write_all(&batch).map_err(RsomicsError::Io)?;
        }
    }

    Ok(HeadStats {
        header_lines,
        records: written,
    })
}

/// Read the BAM header twice over: once as raw stored SAM text (for verbatim
/// output, matching `samtools head` / `sam_hdr_str`) and as a parsed model (for
/// record formatting). The single forward pass reads magic → raw text → refs;
/// the raw text bytes are both captured and parsed, so neither the stored text
/// nor the reference dictionary is re-serialised away from what the file holds.
fn read_header_and_text<R: Read>(
    reader: &mut bam::io::Reader<R>,
) -> Result<(sam::Header, Vec<u8>)> {
    let mut header_reader = reader.header_reader();
    let magic = header_reader
        .read_magic_number()
        .map_err(RsomicsError::Io)?;
    if magic != *b"BAM\x01" {
        return Err(RsomicsError::InvalidInput("not a BAM file".to_string()));
    }

    let mut raw_text = Vec::new();
    {
        let mut sam_reader = header_reader
            .raw_sam_header_reader()
            .map_err(RsomicsError::Io)?;
        sam_reader
            .read_to_end(&mut raw_text)
            .map_err(RsomicsError::Io)?;
        sam_reader.discard_to_end().map_err(RsomicsError::Io)?;
    }

    // `parse_partial` consumes one header line at a time (sans trailing
    // newline), mirroring noodles' own line-based header reader.
    let mut parser = sam::header::Parser::default();
    for line in raw_text.split(|&b| b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            continue;
        }
        parser
            .parse_partial(line)
            .map_err(|e| RsomicsError::InvalidInput(format!("parsing SAM header: {e}")))?;
    }
    let mut header = parser.finish();

    let reference_sequences = header_reader
        .read_reference_sequences()
        .map_err(RsomicsError::Io)?;
    if header.reference_sequences().is_empty() {
        *header.reference_sequences_mut() = reference_sequences;
    }

    Ok((header, raw_text))
}
