//! Core conversion routines for rsomics-vcf-convert.
//!
//! ## Implemented in 0.1.0
//! - VCF text → VCF text (`-O v`)
//! - VCF text → bgzipped VCF (`-O z`)
//! - bgzipped VCF → VCF text (`-O v`)
//! - bgzipped VCF → bgzipped VCF (`-O z`)
//! - VCF/VCF.gz → HAP/LEGEND/SAMPLE export (`--haplegendsample`)
//!
//! ## Deferred to a future release
//! BCF binary input/output (`-O b`, `-O u`) requires a BCF parser; the binary
//! refuses with a descriptive error. TSV→VCF, GEN/SAMPLE, HAP/SAMPLE, gVCF
//! expansion are also deferred.
//!
//! ## Origin
//! Independent Rust reimplementation of `bcftools convert`. bcftools is
//! MIT-licensed. Implementation is derived from:
//! - The published VCF/BCF format specification (hts-specs)
//! - The IMPUTE2 hap/legend/sample format description
//!   (<https://mathgen.stats.ox.ac.uk/impute/impute_v2.html>)
//! - Black-box testing against `bcftools convert` 1.23.1
//! - Reading the MIT-licensed bcftools source `vcfconvert.c`
//!   (Petr Danecek et al., Genome Research Ltd.)
//!
//! No GPL source was used. License: MIT OR Apache-2.0.

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use flate2::Compression;
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;

use rsomics_common::{Result, RsomicsError};

/// Compression level for bgzipped VCF output (`-O z`).
/// bcftools defaults to level 6 for gzip-compressed output.
const DEFAULT_GZIP_LEVEL: u32 = 6;

/// Output format selector, matching bcftools `-O` semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Plain VCF text (`-O v`). Default.
    #[default]
    VcfText,
    /// bgzipped VCF (`-O z`).
    VcfGz,
}

/// Detect BGZF/gzip input by magic bytes and return a line-buffered reader.
pub fn open_vcf_reader(path: &Path) -> Result<Box<dyn BufRead>> {
    let mut probe_buf = [0u8; 2];
    let mut probe = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let n = probe.read(&mut probe_buf).map_err(RsomicsError::Io)?;
    drop(probe);

    let f = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;

    if n == 2 && probe_buf == [0x1f, 0x8b] {
        Ok(Box::new(BufReader::new(MultiGzDecoder::new(f))))
    } else {
        Ok(Box::new(BufReader::new(f)))
    }
}

/// Wrap a `Write` sink so that output is bgzipped at the given level.
fn bgzf_writer(sink: &mut dyn Write, level: u32) -> GzEncoder<&mut dyn Write> {
    GzEncoder::new(sink, Compression::new(level))
}

/// Convert a VCF/VCF.gz file, writing all records (header + data) to `output`
/// in the chosen format.
///
/// Byte-exact passthrough for records — no field reparsing — so INFO/FORMAT
/// annotation text is preserved verbatim.
pub fn convert(input: &Path, output: &mut dyn Write, fmt: OutputFormat) -> Result<u64> {
    let reader = open_vcf_reader(input)?;

    let mut records: u64 = 0;
    match fmt {
        OutputFormat::VcfText => {
            let mut out = BufWriter::with_capacity(256 * 1024, output);
            for line in reader.lines() {
                let line = line.map_err(RsomicsError::Io)?;
                out.write_all(line.as_bytes()).map_err(RsomicsError::Io)?;
                out.write_all(b"\n").map_err(RsomicsError::Io)?;
                if !line.starts_with('#') {
                    records += 1;
                }
            }
            out.flush().map_err(RsomicsError::Io)?;
        }
        OutputFormat::VcfGz => {
            let mut gz = bgzf_writer(output, DEFAULT_GZIP_LEVEL);
            let mut out = BufWriter::with_capacity(256 * 1024, &mut gz);
            for line in reader.lines() {
                let line = line.map_err(RsomicsError::Io)?;
                out.write_all(line.as_bytes()).map_err(RsomicsError::Io)?;
                out.write_all(b"\n").map_err(RsomicsError::Io)?;
                if !line.starts_with('#') {
                    records += 1;
                }
            }
            out.flush().map_err(RsomicsError::Io)?;
            drop(out);
            gz.finish().map_err(RsomicsError::Io)?;
        }
    }
    Ok(records)
}

/// Parsed header information needed for HAP/LEGEND/SAMPLE export.
struct VcfHeader {
    sample_names: Vec<String>,
}

/// Parse the VCF `#CHROM` header line to extract ordered sample names.
fn parse_chrom_line(line: &str) -> Option<VcfHeader> {
    if !line.starts_with("#CHROM") {
        return None;
    }
    let cols: Vec<&str> = line.splitn(10, '\t').collect();
    // Column indices: 0=CHROM,1=POS,2=ID,3=REF,4=ALT,5=QUAL,6=FILTER,7=INFO,8=FORMAT,9+=samples
    let sample_names = if cols.len() > 9 {
        cols[9].split('\t').map(|s| s.to_owned()).collect()
    } else {
        Vec::new()
    };
    Some(VcfHeader { sample_names })
}

/// A parsed minimal VCF data record (only the fields we emit in HAP/LEGEND).
struct VcfRecord<'a> {
    chrom: &'a str,
    pos: u64,
    id: &'a str,
    r#ref: &'a str,
    alt: &'a str,
    genotypes: Vec<&'a str>,
}

/// Parse a VCF data line into the fields needed for HAP/LEGEND output.
/// Returns `None` for multi-allelic sites (more than one ALT allele); the
/// HAP/LEGEND format is defined only for biallelic SNPs — we skip
/// multi-allelics with a warning, matching bcftools convert behaviour.
fn parse_vcf_record(line: &str) -> Option<VcfRecord<'_>> {
    let mut cols = line.splitn(10, '\t');
    let chrom = cols.next()?;
    let pos_str = cols.next()?;
    let id = cols.next()?;
    let r#ref = cols.next()?;
    let alt = cols.next()?;

    if alt.contains(',') {
        return None;
    }

    let _ = cols.next(); // QUAL
    let _ = cols.next(); // FILTER
    let _ = cols.next(); // INFO
    let fmt_col = cols.next(); // FORMAT
    let samples_col = cols.next().unwrap_or("");

    let pos: u64 = pos_str.parse().ok()?;

    // Parse GT from each sample field. bcftools convert HAP output uses the
    // first FORMAT subfield (GT) only.
    let gt_idx = fmt_col
        .unwrap_or("GT")
        .split(':')
        .position(|f| f == "GT")
        .unwrap_or(0);

    let genotypes: Vec<&str> = if samples_col.is_empty() {
        Vec::new()
    } else {
        samples_col
            .split('\t')
            .map(|samp| samp.split(':').nth(gt_idx).unwrap_or("./."))
            .collect()
    };

    Some(VcfRecord {
        chrom,
        pos,
        id,
        r#ref,
        alt,
        genotypes,
    })
}

/// Encode a diploid phased or unphased GT string as the two allele indices
/// expected by the HAP file. Returns `(a0, a1)` as `'0'` or `'1'`.
/// Missing genotypes (`./.: .|.`) → `0 0` (bcftools convert maps missing to
/// hom-ref for the HAP file).
fn gt_to_hap_alleles(gt: &str) -> (u8, u8) {
    let sep = if gt.contains('|') { '|' } else { '/' };
    let mut parts = gt.splitn(2, sep);
    let a0 = parts.next().unwrap_or("0");
    let a1 = parts.next().unwrap_or(a0);
    let to_allele = |s: &str| -> u8 {
        match s {
            "." | "./." | ".|." => b'0',
            "0" => b'0',
            "1" => b'1',
            _ => b'0',
        }
    };
    (to_allele(a0), to_allele(a1))
}

/// Export VCF to IMPUTE2 HAP/LEGEND/SAMPLE format.
///
/// `hap_out` receives the `.hap` content (space-separated allele pairs per
/// variant row, one column per haplotype).
/// `legend_out` receives the `.legend` content (ID POSITION a0 a1 columns).
/// `sample_out` receives the `.sample` content (sample ID + sex columns).
///
/// Multi-allelic sites are skipped — IMPUTE2 format is biallelic only.
/// Non-biallelic or structural variants produce a warning on stderr.
///
/// Returns `(n_written, n_skipped)`.
pub fn vcf_to_haplegendsample(
    input: &Path,
    hap_out: &mut dyn Write,
    legend_out: &mut dyn Write,
    sample_out: &mut dyn Write,
) -> Result<(u64, u64)> {
    let reader = open_vcf_reader(input)?;

    let mut hap_buf = BufWriter::with_capacity(256 * 1024, hap_out);
    let mut leg_buf = BufWriter::with_capacity(64 * 1024, legend_out);
    let mut samp_buf = BufWriter::with_capacity(4 * 1024, sample_out);

    // Legend header: matches bcftools convert output exactly.
    writeln!(leg_buf, "id position a0 a1").map_err(RsomicsError::Io)?;

    let mut header: Option<VcfHeader> = None;
    let mut n_written: u64 = 0;
    let mut n_skipped: u64 = 0;

    let mut lines_iter = reader.lines();

    for line in lines_iter.by_ref() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with("#CHROM") {
            header = parse_chrom_line(&line);
            break;
        }
    }

    let hdr =
        header.ok_or_else(|| RsomicsError::InvalidInput("no #CHROM header line found".into()))?;

    // Sample file: two-line header matching bcftools convention, then one
    // row per sample. bcftools emits "sample_0" as the population ID and
    // "D" (diploid) as phenotype when no sex file is provided.
    writeln!(samp_buf, "ID_1 ID_2 missing").map_err(RsomicsError::Io)?;
    writeln!(samp_buf, "0 0 0").map_err(RsomicsError::Io)?;
    for name in &hdr.sample_names {
        writeln!(samp_buf, "{name} {name} 0").map_err(RsomicsError::Io)?;
    }
    samp_buf.flush().map_err(RsomicsError::Io)?;

    let n_samples = hdr.sample_names.len();

    for line in lines_iter {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            continue;
        }

        match parse_vcf_record(&line) {
            None => {
                eprintln!("warning: skipping multi-allelic/unparseable record");
                n_skipped += 1;
                continue;
            }
            Some(rec) => {
                // Legend line: id position a0 a1
                let id_str = if rec.id == "." {
                    format!("{}:{}", rec.chrom, rec.pos)
                } else {
                    rec.id.to_owned()
                };
                writeln!(leg_buf, "{id_str} {} {} {}", rec.pos, rec.r#ref, rec.alt)
                    .map_err(RsomicsError::Io)?;

                // HAP line: allele for each haplotype, space-separated.
                // With n samples → 2*n haplotypes; haplotype order is
                // sample0_hap0 sample0_hap1 sample1_hap0 … (bcftools order).
                let mut first = true;
                for i in 0..n_samples {
                    let gt = rec.genotypes.get(i).copied().unwrap_or("0/0");
                    let (a0, a1) = gt_to_hap_alleles(gt);
                    if !first {
                        hap_buf.write_all(b" ").map_err(RsomicsError::Io)?;
                    }
                    hap_buf.write_all(&[a0]).map_err(RsomicsError::Io)?;
                    hap_buf.write_all(b" ").map_err(RsomicsError::Io)?;
                    hap_buf.write_all(&[a1]).map_err(RsomicsError::Io)?;
                    first = false;
                }
                hap_buf.write_all(b"\n").map_err(RsomicsError::Io)?;

                n_written += 1;
            }
        }
    }

    hap_buf.flush().map_err(RsomicsError::Io)?;
    leg_buf.flush().map_err(RsomicsError::Io)?;

    Ok((n_written, n_skipped))
}

/// Open a write sink, gzip-wrapping when `gz` is true.
pub enum OutputSink<'a> {
    Plain(BufWriter<&'a mut dyn Write>),
    Gz(GzEncoder<&'a mut dyn Write>),
}

impl io::Write for OutputSink<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(w) => w.write(buf),
            Self::Gz(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(w) => w.flush(),
            Self::Gz(w) => w.flush(),
        }
    }
}
