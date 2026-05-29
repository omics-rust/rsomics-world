//! Format conversion between FASTA and FASTQ.
//!
//! Implements five operations matching seqkit's conversion subcommands:
//! - `fq2fa`  — strip quality scores, emit FASTA
//! - `fa2fq`  — add dummy quality scores, emit FASTQ
//! - `fx2tab` — convert to two- or three-column TSV (name, seq[, qual])
//! - `tab2fx` — convert TSV back to FASTA or FASTQ
//! - `phred`  — re-encode quality scores between Phred+33 and Phred+64

use std::io::{BufRead, BufWriter, Write};

use anyhow::{Context, bail};
use needletail::{
    parse_fastx_reader,
    parser::{LineEnding, write_fasta, write_fastq},
};

/// Convert FASTQ → FASTA by stripping quality scores.
///
/// Each record's sequence is written as-is (raw bytes including any embedded
/// newlines for FASTA inputs); FASTA inputs pass through unchanged.
pub fn fq2fa<R, W>(input: R, output: &mut W) -> anyhow::Result<()>
where
    R: std::io::Read + Send,
    W: Write,
{
    let mut reader = parse_fastx_reader(input).context("parsing input")?;
    let le = LineEnding::Unix;
    while let Some(rec) = reader.next() {
        let rec = rec.context("reading record")?;
        write_fasta(rec.id(), rec.raw_seq(), output, le)?;
    }
    Ok(())
}

/// Convert FASTA → FASTQ by adding dummy Phred+33 quality (all `I` = Q40).
///
/// Existing FASTQ input passes through, preserving original quality.
pub fn fa2fq<R, W>(input: R, output: &mut W) -> anyhow::Result<()>
where
    R: std::io::Read + Send,
    W: Write,
{
    let mut reader = parse_fastx_reader(input).context("parsing input")?;
    let le = LineEnding::Unix;
    while let Some(rec) = reader.next() {
        let rec = rec.context("reading record")?;
        // write_fastq uses `None` quality to write dummy `I` bytes.
        write_fastq(rec.id(), rec.raw_seq(), rec.qual(), output, le)?;
    }
    Ok(())
}

/// Convert FASTA/FASTQ → tab-separated (name, seq[, qual]).
///
/// The output has two columns for FASTA records and three columns for FASTQ
/// records. Existing quality columns appear in the third column.
pub fn fx2tab<R, W>(input: R, output: &mut W) -> anyhow::Result<()>
where
    R: std::io::Read + Send,
    W: Write,
{
    let mut reader = parse_fastx_reader(input).context("parsing input")?;
    while let Some(rec) = reader.next() {
        let rec = rec.context("reading record")?;
        // Sequence may span multiple lines in FASTA; normalize to single line.
        let seq = rec.seq();
        output.write_all(rec.id())?;
        output.write_all(b"\t")?;
        output.write_all(seq.as_ref())?;
        if let Some(qual) = rec.qual() {
            output.write_all(b"\t")?;
            output.write_all(qual)?;
        }
        output.write_all(b"\n")?;
    }
    Ok(())
}

/// Convert tab-separated input back to FASTA or FASTQ.
///
/// - Two-column lines (name, seq) → FASTA.
/// - Three-column lines (name, seq, qual) → FASTQ.
/// - Lines with more than three columns: only the first three are used.
pub fn tab2fx<R, W>(input: R, output: &mut W) -> anyhow::Result<()>
where
    R: BufRead,
    W: Write,
{
    let le = LineEnding::Unix;
    for (lineno, line) in input.lines().enumerate() {
        let line = line.with_context(|| format!("reading line {}", lineno + 1))?;
        if line.is_empty() {
            continue;
        }
        let mut cols = line.splitn(4, '\t');
        let name = cols
            .next()
            .unwrap_or("")
            .trim_start_matches('>')
            .trim_start_matches('@');
        let seq = cols
            .next()
            .with_context(|| format!("line {}: missing sequence column", lineno + 1))?;
        let qual = cols.next();

        match qual {
            Some(q) => write_fastq(
                name.as_bytes(),
                seq.as_bytes(),
                Some(q.as_bytes()),
                output,
                le,
            )?,
            None => write_fasta(name.as_bytes(), seq.as_bytes(), output, le)?,
        }
    }
    Ok(())
}

/// Re-encode Phred quality scores between Phred+33 (Sanger/Illumina 1.8+) and Phred+64 (Illumina 1.3-1.7).
///
/// `from_offset` must be 33 or 64. Conversion is offset arithmetic on every
/// quality byte. Input must be FASTQ; FASTA records are rejected.
pub fn phred_recode<R, W>(input: R, output: &mut W, from_offset: u8) -> anyhow::Result<()>
where
    R: std::io::Read + Send,
    W: Write,
{
    if from_offset != 33 && from_offset != 64 {
        bail!("Phred offset must be 33 or 64, got {from_offset}");
    }
    let to_offset: u8 = if from_offset == 33 { 64 } else { 33 };
    let delta = i16::from(to_offset) - i16::from(from_offset);

    let mut reader = parse_fastx_reader(input).context("parsing input")?;
    let le = LineEnding::Unix;
    while let Some(rec) = reader.next() {
        let rec = rec.context("reading record")?;
        let qual = rec.qual().with_context(|| {
            format!(
                "record {:?}: no quality — is input FASTQ?",
                std::str::from_utf8(rec.id()).unwrap_or("?")
            )
        })?;
        let recoded: Vec<u8> = qual
            .iter()
            .map(|&b| {
                let q = i16::from(b) + delta;
                // q is always in [33,126] after clamping — the sign bit cannot
                // be set because clamp ensures q ≥ 33.
                #[allow(clippy::cast_sign_loss)]
                {
                    q.clamp(33, 126) as u8
                }
            })
            .collect();
        write_fastq(rec.id(), rec.raw_seq(), Some(&recoded), output, le)?;
    }
    Ok(())
}

/// Open an output path for writing, or return stdout if path is `-`.
pub fn open_output(path: &str) -> anyhow::Result<Box<dyn Write>> {
    if path == "-" {
        Ok(Box::new(BufWriter::new(std::io::stdout())))
    } else {
        Ok(Box::new(BufWriter::new(
            std::fs::File::create(path).with_context(|| format!("creating {path:?}"))?,
        )))
    }
}

/// Open an input path for reading, or return stdin if path is `-`.
pub fn open_input(path: &str) -> anyhow::Result<Box<dyn std::io::Read + Send>> {
    if path == "-" {
        Ok(Box::new(std::io::stdin()))
    } else {
        Ok(Box::new(
            std::fs::File::open(path).with_context(|| format!("opening {path:?}"))?,
        ))
    }
}

/// Open an input path as a buffered reader, or return stdin if path is `-`.
pub fn open_input_buffered(path: &str) -> anyhow::Result<Box<dyn BufRead>> {
    if path == "-" {
        Ok(Box::new(std::io::BufReader::new(std::io::stdin())))
    } else {
        Ok(Box::new(std::io::BufReader::new(
            std::fs::File::open(path).with_context(|| format!("opening {path:?}"))?,
        )))
    }
}
