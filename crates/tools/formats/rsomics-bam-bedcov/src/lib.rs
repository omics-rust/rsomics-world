use std::fs::File;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::core::{Position, Region};
use noodles::sam::alignment::record::cigar::op::Kind;
use rsomics_common::{Result, RsomicsError};

pub struct BedcovOpts {
    pub min_mapq: u8,
    /// Reads with any of these flags set are skipped (default 0x704 = UNMAP|SECONDARY|QCFAIL|DUP).
    pub skip_flags: u16,
}

impl Default for BedcovOpts {
    fn default() -> Self {
        Self {
            min_mapq: 0,
            skip_flags: 0x704,
        }
    }
}

// A BED region: 0-based half-open [start, end), raw original line bytes.
struct BedRegion {
    chrom: String,
    // 0-based half-open [start, end)
    start: u64,
    end: u64,
    // Original BED line (no trailing newline) preserved verbatim for output.
    raw: Vec<u8>,
}

// Parse a raw BED line and extract (chrom_str, start, end).  Returns None for
// blank / comment lines so the caller skips them while still preserving order.
fn parse_bed_line(line: &[u8]) -> Option<(String, u64, u64)> {
    if line.is_empty() || line[0] == b'#' {
        return None;
    }
    let mut it = line.split(|&c| c == b'\t');
    let chrom_bytes = it.next()?;
    let start = parse_u64(it.next()?)?;
    let end = parse_u64(it.next()?)?;
    // Silently skip degenerate (empty) regions — they can never have coverage.
    if start >= end {
        return None;
    }
    let chrom = std::str::from_utf8(chrom_bytes).ok()?.to_string();
    Some((chrom, start, end))
}

fn parse_u64(b: &[u8]) -> Option<u64> {
    if b.is_empty() {
        return None;
    }
    let mut n: u64 = 0;
    for &c in b {
        let d = c.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        n = n.checked_mul(10)?.checked_add(u64::from(d))?;
    }
    Some(n)
}

// Reference span of a read = bases consumed on the reference by M/D/N/=/X cigar ops.
// I/S/H/P do not advance the reference cursor.
fn ref_span_from_cigar(record: &bam::Record) -> Result<u64> {
    let mut span: u64 = 0;
    for op in record.cigar().iter() {
        let op = op.map_err(RsomicsError::Io)?;
        match op.kind() {
            Kind::Match
            | Kind::Deletion
            | Kind::Skip
            | Kind::SequenceMatch
            | Kind::SequenceMismatch => {
                span += op.len() as u64;
            }
            _ => {}
        }
    }
    Ok(span)
}

/// Compute per-BED-region coverage summed across `bam_paths`, writing one
/// output line per BED region (original columns + one coverage column per BAM).
///
/// Each BAM must have a companion `.bam.bai` index. The index is used to seek
/// directly to the BGZF blocks overlapping each region, so only reads that
/// actually overlap a region are decoded — O(reads_per_region × regions)
/// instead of O(all_reads × regions).
///
/// Returns the number of BED regions emitted.
pub fn bedcov(
    bed_path: &Path,
    bam_paths: &[impl AsRef<Path>],
    opts: &BedcovOpts,
    // Retained for API compatibility; indexed queries use a seekable single-threaded reader.
    _workers: NonZero<usize>,
    output: &mut dyn Write,
) -> Result<u64> {
    // --- Load BED ---
    let bed_bytes =
        std::fs::read(bed_path).map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;

    let mut regions: Vec<BedRegion> = Vec::new();

    for raw in bed_bytes.split(|&b| b == b'\n') {
        let line = match raw.last() {
            Some(b'\r') => &raw[..raw.len() - 1],
            _ => raw,
        };
        if let Some((chrom, start, end)) = parse_bed_line(line) {
            regions.push(BedRegion {
                chrom,
                start,
                end,
                raw: line.to_vec(),
            });
        }
    }

    // coverage[region_idx][bam_idx]
    let n_bams = bam_paths.len();
    let mut coverage: Vec<Vec<u64>> = vec![vec![0u64; n_bams]; regions.len()];

    // --- Query each BAM via its index ---
    for (bam_idx, bam_path) in bam_paths.iter().enumerate() {
        let bam_path = bam_path.as_ref();
        let index_path = bam_path.with_extension("bam.bai");

        let index = bam::bai::fs::read(&index_path).map_err(|e| {
            RsomicsError::InvalidInput(format!(
                "cannot open BAM index {}: {e} — run `samtools index` first",
                index_path.display()
            ))
        })?;

        // Indexed queries require a seekable reader; bgzf::io::Reader<File> satisfies the
        // bgzf::io::BufRead + bgzf::io::Seek bound that bam::io::Reader::query requires.
        let file = File::open(bam_path)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;
        let mut reader = bam::io::Reader::new(file);
        let header = reader.read_header().map_err(RsomicsError::Io)?;

        let mut record = bam::Record::default();

        for (ri, reg) in regions.iter().enumerate() {
            // BED is 0-based half-open [start, end).
            // noodles Region uses 1-based inclusive positions.
            // BED start=0 → Position 1; BED end=N → Position N (last covered base).
            let pos_start = Position::try_from(reg.start as usize + 1)
                .map_err(|e| RsomicsError::InvalidInput(format!("BED start out of range: {e}")))?;
            let pos_end = Position::try_from(reg.end as usize)
                .map_err(|e| RsomicsError::InvalidInput(format!("BED end out of range: {e}")))?;

            let noodles_region = Region::new(reg.chrom.as_bytes(), pos_start..=pos_end);

            let mut query = match reader.query(&header, &index, &noodles_region) {
                Ok(q) => q,
                // Chromosome not in BAM header — no coverage for this region.
                Err(_) => continue,
            };

            loop {
                let n = query.read_record(&mut record).map_err(RsomicsError::Io)?;
                if n == 0 {
                    break;
                }

                let flags = record.flags();
                if (flags.bits() & opts.skip_flags) != 0 {
                    continue;
                }

                let mq = record.mapping_quality().map_or(0, |q| q.get());
                if mq < opts.min_mapq {
                    continue;
                }

                let Some(aln_start_pos) = record.alignment_start().transpose().ok().flatten()
                else {
                    continue;
                };

                // noodles Position is 1-based; convert to 0-based.
                let read_start = aln_start_pos.get() as u64 - 1;
                let span = ref_span_from_cigar(&record)?;
                if span == 0 {
                    continue;
                }
                let read_end = read_start + span; // 0-based half-open

                // Clamp to the BED region to get the exact overlap length.
                let lo = read_start.max(reg.start);
                let hi = read_end.min(reg.end);
                if hi > lo {
                    coverage[ri][bam_idx] += hi - lo;
                }
            }
        }
    }

    // --- Emit output ---
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut ib = itoa::Buffer::new();
    let mut emitted: u64 = 0;

    for (ri, reg) in regions.iter().enumerate() {
        out.write_all(&reg.raw).map_err(RsomicsError::Io)?;
        for &cov in &coverage[ri] {
            out.write_all(b"\t").map_err(RsomicsError::Io)?;
            out.write_all(ib.format(cov).as_bytes())
                .map_err(RsomicsError::Io)?;
        }
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
        emitted += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(emitted)
}
