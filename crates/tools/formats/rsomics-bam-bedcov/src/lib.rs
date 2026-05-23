//! `samtools bedcov` port: per-BED-region read depth.
//!
//! Two evaluation strategies, chosen per call by region count (see
//! [`SWEEP_REGION_THRESHOLD`]):
//!
//! - **Linear sweep** (many regions — the panel / exome / genome-window
//!   workload). The BAM is read exactly once via the [`rsomics_bamio::raw`]
//!   path (refID/pos/cigar at fixed offsets, no full record decode). Reads
//!   arrive in coordinate order; regions are pre-sorted by `(ref_id, start)`.
//!   A moving cursor over the sorted regions attributes each read's per-base
//!   reference span to every region it overlaps, so the cost is
//!   O(reads + overlap-hits) and each BGZF block is inflated once. The
//!   per-region indexed query, by contrast, re-seeks and re-inflates blocks
//!   for every region (noodles' CSI query reads to each chunk's end, vastly
//!   overshooting tiny regions) — for tens of thousands of regions it reads
//!   the file many times over.
//!
//! - **Indexed query** (few regions — sparse spot checks). Each region is
//!   served from the BAM index with a direct BGZF seek, decoding only the
//!   reads that overlap it. For a handful of regions this beats reading the
//!   whole file. The path's cost grows ~linearly with region count (noodles'
//!   CSI query reads to each chunk's end, so each region pays a near-constant
//!   seek + over-read), so it is only worth taking when there are few enough
//!   regions that the total seek cost stays below one full linear pass — see
//!   [`use_sweep`] for the file-size-aware crossover.
//!
//! Coverage semantics match `samtools bedcov` (bedcov.c) with no `-j`: each
//! read contributes, to every BED region it overlaps, the length of the
//! intersection of its reference span `[start, start+span)` with the region
//! `[beg, end)`. The reference span counts CIGAR M/D/N/=/X (the default
//! pileup counts deletion and ref-skip positions as covered; only `-j`
//! excludes them). Output preserves input BED order with one coverage column
//! appended per BAM.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::core::{Position, Region};
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

// CIGAR op codes (BAM packed encoding, low nibble): M=0 I=1 D=2 N=3 S=4 H=5 P=6 ==7 X=8.
const CIGAR_MATCH: u8 = 0;
const CIGAR_DELETION: u8 = 2;
const CIGAR_SKIP: u8 = 3;
const CIGAR_SEQ_MATCH: u8 = 7;
const CIGAR_SEQ_MISMATCH: u8 = 8;

/// BAM bytes per region at the indexed-query / linear-sweep crossover.
///
/// The sweep reads the file once: cost ≈ `file_bytes / inflate_rate`, flat in
/// region count. The indexed path pays a near-constant seek + over-read per
/// region (noodles' CSI query reads to each chunk's end): cost ≈
/// `n_regions × per_region_cost`. The two are equal when
/// `n_regions ≈ file_bytes / SWEEP_BYTES_PER_REGION`. Measured on an
/// Apple-M2 / 170 MB coord-sorted BAM the indexed path costs ~1.5 ms/region
/// and the sweep ~0.67 s, giving a crossover near 450 regions over 170 MB →
/// one region per ~380 KB. Above that many regions the sweep wins; below it the
/// per-region seeks are cheaper than a full pass. The constant is deliberately
/// conservative (slightly favouring the sweep) because the indexed path
/// degrades linearly and badly past the crossover, whereas the sweep's penalty
/// for being chosen a little early is a sub-second fixed cost.
const SWEEP_BYTES_PER_REGION: u64 = 384 * 1024;

/// Below this many regions, always take the indexed path regardless of file
/// size: a true sparse spot-check (tens of regions) is sub-100 ms either way,
/// and the indexed path avoids inflating a whole large BAM for a few windows.
const MIN_SWEEP_REGIONS: usize = 256;

/// Choose the linear sweep over per-region indexed queries. The sweep wins once
/// the indexed path's per-region seek cost would exceed one full linear pass —
/// a file-size-aware crossover (see [`SWEEP_BYTES_PER_REGION`]) — with an
/// absolute floor ([`MIN_SWEEP_REGIONS`]) so a genuinely sparse query of a few
/// windows never inflates the entire BAM.
fn use_sweep(n_regions: usize, max_bam_bytes: u64) -> bool {
    if n_regions < MIN_SWEEP_REGIONS {
        return false;
    }
    n_regions as u64 >= max_bam_bytes / SWEEP_BYTES_PER_REGION
}

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

fn load_bed(bed_path: &Path) -> Result<Vec<BedRegion>> {
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
    Ok(regions)
}

/// Reference span of a read (0-based half-open length) from its packed CIGAR:
/// the bases consumed on the reference by M/D/N/=/X. I/S/H/P do not advance the
/// reference cursor. Deletion (D) and ref-skip (N) are included because the
/// default `samtools bedcov` pileup counts those positions as covered.
fn ref_span_raw(record: &RawRecord) -> u64 {
    let mut span: u64 = 0;
    for (kind, len) in record.cigar_ops() {
        match kind {
            CIGAR_MATCH | CIGAR_DELETION | CIGAR_SKIP | CIGAR_SEQ_MATCH | CIGAR_SEQ_MISMATCH => {
                span += u64::from(len);
            }
            _ => {}
        }
    }
    span
}

/// Compute per-BED-region coverage summed across `bam_paths`, writing one
/// output line per BED region (original columns + one coverage column per BAM).
///
/// Each BAM must have a companion `.bam.bai` index — both strategies use it (the
/// sweep needs the header for ref-name → tid mapping; below
/// [`SWEEP_REGION_THRESHOLD`] regions the indexed query path is taken).
///
/// Returns the number of BED regions emitted.
pub fn bedcov(
    bed_path: &Path,
    bam_paths: &[impl AsRef<Path>],
    opts: &BedcovOpts,
    // Retained for API compatibility; both strategies read each BAM single-threaded.
    _workers: NonZero<usize>,
    output: &mut dyn Write,
) -> Result<u64> {
    let regions = load_bed(bed_path)?;
    let n_bams = bam_paths.len();
    let mut coverage: Vec<Vec<u64>> = vec![vec![0u64; n_bams]; regions.len()];

    // The crossover is per-BAM (sweep cost scales with that BAM's size). Decide
    // per BAM from its on-disk size so a mix of large and small inputs each
    // takes the right path.
    for (bam_idx, bam_path) in bam_paths.iter().enumerate() {
        let bam_path = bam_path.as_ref();
        let bam_bytes = std::fs::metadata(bam_path).map(|m| m.len()).unwrap_or(0);
        if use_sweep(regions.len(), bam_bytes) {
            sweep_bam(bam_path, &regions, opts, &mut coverage, bam_idx)?;
        } else {
            query_bam(bam_path, &regions, opts, &mut coverage, bam_idx)?;
        }
    }

    emit(output, &regions, &coverage)
}

/// Per-reference sorted view of the BED regions for the sweep: indices into the
/// original `regions` slice, sorted by `start`, grouped by reference tid.
struct SweepIndex {
    // tid -> sorted-by-start list of (start, end, original_region_index).
    by_tid: HashMap<usize, Vec<(u64, u64, usize)>>,
}

impl SweepIndex {
    fn build(regions: &[BedRegion], name_to_tid: &HashMap<&str, usize>) -> Self {
        let mut by_tid: HashMap<usize, Vec<(u64, u64, usize)>> = HashMap::new();
        for (ri, reg) in regions.iter().enumerate() {
            if let Some(&tid) = name_to_tid.get(reg.chrom.as_str()) {
                by_tid
                    .entry(tid)
                    .or_default()
                    .push((reg.start, reg.end, ri));
            }
            // A region on a chromosome absent from the BAM header gets no
            // coverage — same as samtools, which finds no reads for it.
        }
        for v in by_tid.values_mut() {
            v.sort_unstable_by_key(|&(start, _, _)| start);
        }
        Self { by_tid }
    }
}

/// Single linear pass over the BAM, attributing each read's reference span to
/// every overlapping region. Reads arrive coordinate-sorted; regions per tid are
/// pre-sorted by start. A monotone `cursor` skips regions that end before the
/// current read starts (they can never overlap this read or any later one), so
/// the work is O(reads + overlap-hits) and every BGZF block is inflated once.
fn sweep_bam(
    bam_path: &Path,
    regions: &[BedRegion],
    opts: &BedcovOpts,
    coverage: &mut [Vec<u64>],
    bam_idx: usize,
) -> Result<()> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, NonZero::<usize>::MIN)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let names: Vec<&str> = header
        .reference_sequences()
        .keys()
        .map(|n| std::str::from_utf8(n.as_ref()).unwrap_or(""))
        .collect();
    let name_to_tid: HashMap<&str, usize> =
        names.iter().enumerate().map(|(i, &n)| (n, i)).collect();

    let index = SweepIndex::build(regions, &name_to_tid);

    let mut record = RawRecord::default();
    let mut cur_tid: i32 = -1;
    // Regions on the current reference, sorted by start; empty when the current
    // reference has no regions. `cursor` is the first region that may still
    // overlap reads at or after the current position.
    let empty: Vec<(u64, u64, usize)> = Vec::new();
    let mut cur_regions: &[(u64, u64, usize)] = &empty;
    let mut cursor: usize = 0;

    while raw::read_record(reader.get_mut(), &mut record)? != 0 {
        if (record.flags() & opts.skip_flags) != 0 {
            continue;
        }
        if record.mapping_quality() < opts.min_mapq {
            continue;
        }

        let tid = record.reference_sequence_id();
        let pos0 = record.alignment_start();
        if tid < 0 || pos0 < 0 {
            continue;
        }

        if tid != cur_tid {
            cur_tid = tid;
            cur_regions = index
                .by_tid
                .get(&(tid as usize))
                .map_or(&empty[..], Vec::as_slice);
            cursor = 0;
        }
        if cur_regions.is_empty() {
            continue;
        }

        let span = ref_span_raw(&record);
        if span == 0 {
            continue;
        }
        let read_start = pos0 as u64;
        let read_end = read_start + span; // 0-based half-open

        // Drop regions that end at or before this read's start: since reads only
        // advance, those regions can overlap neither this read nor any later one.
        while cursor < cur_regions.len() && cur_regions[cursor].1 <= read_start {
            cursor += 1;
        }

        // Scan forward over regions whose start is before this read's end; once a
        // region starts at or past read_end, no later region (sorted by start)
        // can overlap either, so stop.
        let mut j = cursor;
        while j < cur_regions.len() {
            let (reg_start, reg_end, ri) = cur_regions[j];
            if reg_start >= read_end {
                break;
            }
            let lo = read_start.max(reg_start);
            let hi = read_end.min(reg_end);
            if hi > lo {
                coverage[ri][bam_idx] += hi - lo;
            }
            j += 1;
        }
    }

    Ok(())
}

/// Per-region indexed-query path for sparse region sets. Each region is served
/// from a direct BGZF seek via the BAM index, decoding only overlapping reads.
fn query_bam(
    bam_path: &Path,
    regions: &[BedRegion],
    opts: &BedcovOpts,
    coverage: &mut [Vec<u64>],
    bam_idx: usize,
) -> Result<()> {
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
        // BED is 0-based half-open [start, end); noodles Region is 1-based inclusive.
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

            let Some(aln_start_pos) = record.alignment_start().transpose().ok().flatten() else {
                continue;
            };

            // noodles Position is 1-based; convert to 0-based.
            let read_start = aln_start_pos.get() as u64 - 1;
            let span = ref_span_record(&record)?;
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

    Ok(())
}

// Reference span from a fully-decoded noodles record (indexed-query path), same
// op set as `ref_span_raw`: M/D/N/=/X advance the reference.
fn ref_span_record(record: &bam::Record) -> Result<u64> {
    use noodles::sam::alignment::record::cigar::op::Kind;
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

fn emit(output: &mut dyn Write, regions: &[BedRegion], coverage: &[Vec<u64>]) -> Result<u64> {
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
