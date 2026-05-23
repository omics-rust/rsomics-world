//! ATAC-seq Tn5 insertion-bias correction, matching `deeptools alignmentSieve --ATACshift`.
//!
//! ## Shift semantics (deeptools `alignmentSieve.py` `shiftRead`, MIT source)
//!
//! Only **properly paired** reads are emitted; secondary/supplementary are
//! dropped (deeptools' `shiftRead` returns `None` when `is_proper_pair` is
//! false, and the caller skips secondary/supplementary before calling it).
//!
//! The four shift constants `[4, -5, 5, -4]` map to:
//!
//! | strand  | read | field shifted | amount |
//! |---------|------|---------------|--------|
//! | forward | R1   | start         | +4     |
//! | reverse | R1   | end           | −5     |
//! | reverse | R2   | end           | −5     |
//! | forward | R2   | start         | +4     |
//!
//! TLEN is adjusted by `deltaTLen` so the pair's fragment size reflects the
//! shifted coordinates. The mate `next_reference_start` is updated on
//! reverse-strand reads only. CIGAR is rewritten to a single M-op of length
//! `end - start` (deeptools `((0, end-start),)` idiom), where `end` is
//! `pos + query_alignment_end` (pysam semantics: query-coordinate end of the
//! non-soft-clipped portion, i.e. `l_seq − trailing_soft_clips`). Seq and
//! qual are dropped, matching deeptools which constructs a fresh
//! `AlignedSegment` without copying `query_sequence`.
//!
//! ## BED output mode
//!
//! `--bed` emits the shifted start as a 1-bp insertion-site BED line:
//! `chrom  start  start+1  name  mapq  strand`. This is the Tn5 cut
//! midpoint used by downstream peak-callers.

#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

use std::collections::HashMap;
use std::io::{BufWriter, Cursor, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord, write_record};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// SAM/BAM FLAG bits (SAMv1 §1.4).
const FLAG_PROPER_PAIR: u16 = 0x2;
const FLAG_UNMAPPED: u16 = 0x4;
const FLAG_REVERSE: u16 = 0x10;
const FLAG_READ2: u16 = 0x80;
const FLAG_SECONDARY: u16 = 0x100;
const FLAG_SUPPLEMENTARY: u16 = 0x800;

// CIGAR op code (BAM packed encoding, low nibble): S=4 (soft clip — query-consuming, not reference-consuming).
const CIGAR_SOFT_CLIP: u8 = 4;

// deeptools `--ATACshift` shift array: [4, -5, 5, -4]
// shift[0]=4  shift[1]=-5  shift[2]=5  shift[3]=-4
const SHIFT_0: i32 = 4;
const SHIFT_1: i32 = -5;
const SHIFT_2: i32 = 5;
const SHIFT_3: i32 = -4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Bam,
    Bed,
}

#[derive(Debug, Clone)]
pub struct ShiftOpts {
    pub output_mode: OutputMode,
    pub min_mapq: u8,
    pub skip_flags: u16,
}

impl Default for ShiftOpts {
    fn default() -> Self {
        Self {
            output_mode: OutputMode::Bam,
            min_mapq: 0,
            skip_flags: 0,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ShiftStats {
    pub records_read: u64,
    pub records_written: u64,
    pub records_skipped: u64,
}

/// Pysam `query_alignment_end`: the 0-based exclusive index of the last
/// non-soft-clipped query base. Equal to `l_seq − trailing_soft_clip_length`.
///
/// deeptools `shiftRead` computes `end = b.pos + b.query_alignment_end`, so
/// the new CIGAR span (`end − start`) is based on query coordinates, not
/// reference-consuming span. For a read with no soft clips this equals `l_seq`;
/// for `3S97M` it is 100 (3+97), not 97 (the reference span).
fn query_alignment_end(r: &RawRecord) -> i64 {
    let l_seq = r.sequence_len() as i64;
    // Trailing soft-clip length is the len of the last CIGAR op if it is S.
    let trailing_s: i64 = r
        .cigar_ops()
        .last()
        .filter(|(k, _)| *k == CIGAR_SOFT_CLIP)
        .map(|(_, len)| i64::from(len))
        .unwrap_or(0);
    l_seq - trailing_s
}

/// Build a shifted-record byte payload from a `RawRecord`.
///
/// `RawRecord` does not expose `bytes` as `pub mut`, and the CIGAR is
/// variable-length, so we cannot edit it in-place via the existing setters.
/// We build a new payload Vec by splicing around the cigar region, then reload
/// it into the record via a block-size-prefixed cursor — the same path
/// `raw::read_record` uses.
///
/// BAM record payload layout (SAMv1 §4.2):
/// ```text
/// refID@0  pos@4  l_read_name@8  mapq@9  bin@10  n_cigar@12  flag@14
/// l_seq@16  next_refID@20  next_pos@24  tlen@28
/// read_name[l_read_name]  cigar[4*n_cigar]  seq[(l_seq+1)/2]  qual[l_seq]  aux
/// ```
fn apply_shift(src: &RawRecord, chrom_lens: &HashMap<i32, i64>) -> Option<RawRecord> {
    let flags = src.flags();

    // Drop non-proper-pair and secondary/supplementary (deeptools exact).
    if flags & FLAG_UNMAPPED != 0
        || flags & FLAG_PROPER_PAIR == 0
        || flags & (FLAG_SECONDARY | FLAG_SUPPLEMENTARY) != 0
    {
        return None;
    }

    let is_reverse = flags & FLAG_REVERSE != 0;
    let is_read2 = flags & FLAG_READ2 != 0;

    let pos = src.alignment_start() as i64;
    let mut start = pos;
    let mut end = pos + query_alignment_end(src);
    let delta_tlen: i32;

    // deeptools shiftRead exact port:
    if is_reverse && !is_read2 {
        end -= i64::from(SHIFT_2);
        delta_tlen = SHIFT_3 - SHIFT_2; // -4 - 5 = -9
    } else if is_reverse && is_read2 {
        end += i64::from(SHIFT_1);
        delta_tlen = SHIFT_1 - SHIFT_0; // -5 - 4 = -9
    } else if !is_reverse && !is_read2 {
        start += i64::from(SHIFT_0);
        delta_tlen = SHIFT_1 - SHIFT_0; // -5 - 4 = -9
    } else {
        // forward, read2
        start -= i64::from(SHIFT_3); // -(-4) = +4
        delta_tlen = SHIFT_3 - SHIFT_2; // -4 - 5 = -9
    }

    // Sanity checks (deeptools exact):
    if end - start < 1 {
        if is_reverse {
            start = end - 1;
        } else {
            end = start + 1;
        }
    }
    if start < 0 {
        start = 0;
    }
    if let Some(&chrom_len) = chrom_lens.get(&src.reference_sequence_id())
        && end > chrom_len
    {
        end = chrom_len;
    }
    if end - start < 1 {
        return None;
    }

    let new_span = (end - start) as u32;

    // New TLEN (deeptools: tLen<0 → tLen-deltaTLen, else tLen+deltaTLen).
    let old_tlen = src.template_length();
    let new_tlen: i32 = if old_tlen < 0 {
        old_tlen - delta_tlen
    } else {
        old_tlen + delta_tlen
    };

    // Mate next_reference_start adjustment (deeptools exact: reverse strand only).
    let old_mate_pos = src.mate_alignment_start();
    let new_mate_pos: i32 = if is_reverse {
        if is_read2 {
            old_mate_pos + SHIFT_0 // shift[0]=4
        } else {
            old_mate_pos - SHIFT_3 // -shift[3]= -(-4)=+4
        }
    } else {
        old_mate_pos
    };

    // Build new payload: fixed_head + read_name + new_cigar(4 bytes) + aux.
    // Seq/qual are dropped (set l_seq=0, no seq/qual bytes), matching deeptools
    // which constructs a fresh AlignedSegment without copying query_sequence.
    let old_bytes = src.as_bytes();
    let l_read_name = usize::from(old_bytes[8]);
    let n_cigar_old = usize::from(u16::from_le_bytes([old_bytes[12], old_bytes[13]]));
    let l_seq_old = u32::from_le_bytes(old_bytes[16..20].try_into().unwrap()) as usize;

    let cigar_start = 32 + l_read_name;
    let seq_start_old = cigar_start + n_cigar_old * 4;
    let seq_bytes_old = l_seq_old.div_ceil(2);
    let aux_start_old = seq_start_old + seq_bytes_old + l_seq_old;

    // Single M op: (new_span << 4) | 0.
    let packed_op = (new_span << 4).to_le_bytes();

    // New layout: fixed_head + read_name + 4-byte cigar + aux (no seq/qual).
    let aux_bytes = &old_bytes[aux_start_old..];
    let new_size = 32 + l_read_name + 4 + aux_bytes.len();
    let mut new_bytes: Vec<u8> = Vec::with_capacity(new_size);
    new_bytes.extend_from_slice(&old_bytes[..cigar_start]);
    new_bytes.extend_from_slice(&packed_op);
    new_bytes.extend_from_slice(aux_bytes);

    // Patch fields in new_bytes (all little-endian):
    // pos @ 4
    new_bytes[4..8].copy_from_slice(&(start as i32).to_le_bytes());
    // n_cigar @ 12 (u16): 1
    new_bytes[12] = 1;
    new_bytes[13] = 0;
    // l_seq @ 16: 0 (seq dropped, matching deeptools).
    new_bytes[16] = 0;
    new_bytes[17] = 0;
    new_bytes[18] = 0;
    new_bytes[19] = 0;
    // tlen @ 28
    new_bytes[28..32].copy_from_slice(&new_tlen.to_le_bytes());
    // next_pos @ 24
    new_bytes[24..28].copy_from_slice(&new_mate_pos.to_le_bytes());

    // Load into a RawRecord via the standard block-prefixed path.
    let block_size = new_bytes.len() as u32;
    let mut prefixed = Vec::with_capacity(4 + new_bytes.len());
    prefixed.extend_from_slice(&block_size.to_le_bytes());
    prefixed.extend_from_slice(&new_bytes);
    let mut out = RawRecord::default();
    raw::read_record(&mut Cursor::new(prefixed), &mut out).ok()?;
    Some(out)
}

pub fn run(
    input: &Path,
    output: &Path,
    opts: &ShiftOpts,
    workers: NonZero<usize>,
) -> Result<ShiftStats> {
    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let chrom_lens: HashMap<i32, i64> = header
        .reference_sequences()
        .iter()
        .enumerate()
        .map(|(i, (_, seq))| (i as i32, usize::from(seq.length()) as i64))
        .collect();

    let chrom_names: Vec<String> = header
        .reference_sequences()
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();

    let mut stats = ShiftStats::default();
    let mut src = RawRecord::default();

    match opts.output_mode {
        OutputMode::Bam => {
            let mut writer = rsomics_bamio::create_with_workers(output, workers)?;
            writer.write_header(&header).map_err(RsomicsError::Io)?;

            while raw::read_record(reader.get_mut(), &mut src)? != 0 {
                stats.records_read += 1;
                let flags = src.flags();
                if opts.skip_flags != 0 && (flags & opts.skip_flags) != 0 {
                    stats.records_skipped += 1;
                    continue;
                }
                if opts.min_mapq > 0 && src.mapping_quality() < opts.min_mapq {
                    stats.records_skipped += 1;
                    continue;
                }
                match apply_shift(&src, &chrom_lens) {
                    Some(shifted) => {
                        write_record(writer.get_mut(), &shifted)?;
                        stats.records_written += 1;
                    }
                    None => {
                        stats.records_skipped += 1;
                    }
                }
            }
        }
        OutputMode::Bed => {
            let bed_file = std::fs::File::create(output).map_err(RsomicsError::Io)?;
            let mut out = BufWriter::with_capacity(256 * 1024, bed_file);

            while raw::read_record(reader.get_mut(), &mut src)? != 0 {
                stats.records_read += 1;
                let flags = src.flags();
                if opts.skip_flags != 0 && (flags & opts.skip_flags) != 0 {
                    stats.records_skipped += 1;
                    continue;
                }
                if opts.min_mapq > 0 && src.mapping_quality() < opts.min_mapq {
                    stats.records_skipped += 1;
                    continue;
                }
                match apply_shift(&src, &chrom_lens) {
                    Some(shifted) => {
                        let pos = shifted.alignment_start() as i64;
                        let tid = shifted.reference_sequence_id();
                        let chrom = chrom_names
                            .get(tid as usize)
                            .map(String::as_str)
                            .unwrap_or(".");
                        let strand = if shifted.flags() & FLAG_REVERSE != 0 {
                            '-'
                        } else {
                            '+'
                        };
                        let name = std::str::from_utf8(shifted.name()).unwrap_or(".");
                        let mapq = shifted.mapping_quality();
                        writeln!(out, "{chrom}\t{pos}\t{}\t{name}\t{mapq}\t{strand}", pos + 1)
                            .map_err(RsomicsError::Io)?;
                        stats.records_written += 1;
                    }
                    None => {
                        stats.records_skipped += 1;
                    }
                }
            }
            out.flush().map_err(RsomicsError::Io)?;
        }
    }

    Ok(stats)
}
