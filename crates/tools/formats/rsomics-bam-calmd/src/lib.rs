//! `samtools calmd` port: recompute the MD and NM aux tags of every alignment
//! against a reference FASTA, then re-emit the BAM.
//!
//! The MD/NM walk mirrors `bam_fillmd1_core` in samtools `bam_md.c` exactly,
//! including its nibble-level base comparison (htslib `seq_nt16_table` /
//! `bam_seqi`), its run-length MD string with `^`-prefixed deletions, and its
//! "only rewrite the tag when the value differs" guard so unchanged records
//! stay byte-for-byte identical (matching aux ordering and integer subtype).
//!
//! Reading uses the shared [`rsomics_bamio`] reader (libdeflate BGZF, parallel
//! at `workers >= 2`); records are processed on the raw BAM byte level —
//! seq/qual/cigar are never decoded into noodles types. MD/NM are written
//! directly into the raw aux tail via `RawRecord::set_aux`. This avoids the
//! full `RecordBuf` decode+re-encode round-trip (the former bottleneck at
//! `-t4` and above, accounting for 67% of wall time). Output goes through the
//! bamio parallel BGZF writer.
//!
//! The reference is read from an indexed FASTA (`.fai`); each contig is fetched
//! once on first use and cached for the run. Coordinate-sorted input therefore
//! touches each contig once and never re-reads it — the common calmd case.
//!
//! At `workers >= 2` the MD/NM computation is parallelised with rayon: records
//! are collected into a batch, the needed contigs are fetched serially into a
//! shared read-only map (`Arc<Vec<u8>>` per contig), then `par_iter_mut` runs
//! the raw MD/NM pass on every record simultaneously. Output is written in
//! original batch order so the byte stream is identical to the serial path.

use std::collections::HashMap;
use std::num::NonZero;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use noodles::bam;
use noodles::fasta;
use noodles::sam::alignment::RecordBuf;
use noodles::sam::alignment::io::Write as AlignmentWrite;
use noodles::sam::alignment::record::cigar::op::Kind;
use noodles::sam::alignment::record::data::field::Tag;
use noodles::sam::alignment::record_buf::data::field::Value;
use rayon::prelude::*;
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

const TAG_NM: Tag = Tag::EDIT_DISTANCE;
const TAG_MD: Tag = Tag::MISMATCHED_POSITIONS;

/// BAM aux type codes for MD (Z string) and NM (i32).
const AUX_TYPE_Z: u8 = b'Z';
const AUX_TYPE_I: u8 = b'i';

/// Tag bytes for the raw aux path.
const NM_TAG: [u8; 2] = [b'N', b'M'];
const MD_TAG: [u8; 2] = [b'M', b'D'];

/// Number of raw records processed per rayon batch.
///
/// A `RawRecord` holds the on-disk payload bytes (~350 bytes for 150 bp).
/// 4096 records ≈ 1.4 MB per batch, fitting in L3. 4096 gives rayon workers
/// ~1-4 ms of compute per batch — well above scheduler granularity while
/// keeping the serial read/write phase short enough to maintain pipelining.
const BATCH_SIZE: usize = 4096;

/// htslib `seq_nt16_table` (htslib `hts.c`): ASCII base → 4-bit nucleotide code.
/// `=`→0, A→1 … N→15, with the IUPAC ambiguity codes and the digit aliases
/// `0123`→`ACGT`; any unrecognised byte → 15. The MD/NM match test compares
/// these codes, not ASCII, exactly as `bam_fillmd1_core` does via `bam_seqi`.
#[rustfmt::skip]
const SEQ_NT16_TABLE: [u8; 256] = [
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
     1, 2, 4, 8, 15,15,15,15, 15,15,15,15, 15, 0,15,15,
    15, 1,14, 2, 13,15,15, 4, 11,15,15,12, 15, 3,15,15,
    15,15, 5, 6,  8, 8, 7, 9, 15,10,15,15, 15,15,15,15,
    15, 1,14, 2, 13,15,15, 4, 11,15,15,12, 15, 3,15,15,
    15,15, 5, 6,  8, 8, 7, 9, 15,10,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
    15,15,15,15, 15,15,15,15, 15,15,15,15, 15,15,15,15,
];

#[derive(Debug, Clone, Default)]
pub struct CalmdOpts {
    /// `-e`: rewrite reference-matching read bases as `=` in the output SEQ.
    pub use_equal: bool,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CalmdStats {
    pub records: u64,
    /// Mapped records whose MD/NM were (re)computed.
    pub computed: u64,
    /// Mapped records whose contig was missing from the reference, left untouched.
    pub missing_ref: u64,
    /// Records skipped because they carry no stored sequence (`l_qseq == 0`).
    pub no_sequence: u64,
}

/// Port of the MD/NM walk in `bam_fillmd1_core` (samtools `bam_md.c`).
///
/// `seq` is the read's ASCII bases (mutated in place to `=` on matches when
/// `use_equal`), `cigar` the (kind, len) op list, `ref_seq` the contig bases,
/// and `pos` the 0-based reference start. The constructed MD string is written
/// into `md` (cleared by the caller); the return value is the recomputed NM.
///
/// M/=/X consume read+ref and compare nucleotide codes (`SEQ_NT16_TABLE`): a
/// match needs equal non-N codes, or a read code of 0 (`=`); a mismatch flushes
/// the current match run then the uppercased ref base. D emits `^` + uppercased
/// ref bases; I/S advance the read (I also bumps NM); N advances the reference
/// only. Out-of-bounds / NUL-padded ref ends the walk early, exactly as the C
/// `break` does.
fn compute_md_nm(
    seq: &mut [u8],
    cigar: &[(Kind, usize)],
    ref_seq: &[u8],
    pos: usize,
    use_equal: bool,
    md: &mut Vec<u8>,
) -> i32 {
    let qual_len = seq.len();
    let ref_len = ref_seq.len();
    let mut nm: i32 = 0;
    let mut matched: i64 = 0;
    let mut qpos: usize = 0;
    let mut rpos: usize = pos;

    'outer: for &(kind, oplen) in cigar {
        match kind {
            Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                let mut j = 0;
                while j < oplen {
                    let z = qpos + j;
                    let rp = rpos + j;
                    if rp >= ref_len || z >= qual_len || ref_seq[rp] == 0 {
                        break;
                    }
                    let c1 = SEQ_NT16_TABLE[seq[z] as usize];
                    let c2 = SEQ_NT16_TABLE[ref_seq[rp] as usize];
                    let is_match = (c1 == c2 && c1 != 15 && c2 != 15) || c1 == 0;
                    if is_match {
                        if use_equal {
                            seq[z] = b'=';
                        }
                        matched += 1;
                    } else {
                        append_int(md, matched);
                        md.push(ref_seq[rp].to_ascii_uppercase());
                        matched = 0;
                        nm += 1;
                    }
                    j += 1;
                }
                if j < oplen {
                    break 'outer;
                }
                rpos += oplen;
                qpos += oplen;
            }
            Kind::Deletion => {
                append_int(md, matched);
                md.push(b'^');
                let mut j = 0;
                while j < oplen {
                    let rp = rpos + j;
                    if rp >= ref_len || ref_seq[rp] == 0 {
                        break;
                    }
                    md.push(ref_seq[rp].to_ascii_uppercase());
                    j += 1;
                }
                matched = 0;
                rpos += j;
                nm += j as i32;
                if j < oplen {
                    break 'outer;
                }
            }
            Kind::Insertion => {
                qpos += oplen;
                nm += oplen as i32;
            }
            Kind::SoftClip => {
                qpos += oplen;
            }
            Kind::Skip => {
                rpos += oplen;
            }
            Kind::HardClip | Kind::Pad => {}
        }
    }
    append_int(md, matched);

    nm
}

/// BAM CIGAR op-code to `noodles::sam::alignment::record::cigar::op::Kind`.
/// The op codes in the BAM 4-bit encoding are: 0=M 1=I 2=D 3=N 4=S 5=H 6=P 7== 8=X.
fn bam_op_to_kind(op: u8) -> Kind {
    match op {
        0 => Kind::Match,
        1 => Kind::Insertion,
        2 => Kind::Deletion,
        3 => Kind::Skip,
        4 => Kind::SoftClip,
        5 => Kind::HardClip,
        6 => Kind::Pad,
        7 => Kind::SequenceMatch,
        8 => Kind::SequenceMismatch,
        _ => Kind::Match,
    }
}

/// Port of the MD/NM walk operating directly on packed BAM nibble SEQ.
///
/// The BAM packed SEQ format stores two bases per byte: high nibble = even
/// index, low nibble = odd index. Nibble codes are the `seq_nt16` values
/// directly (`=`→0, A→1, C→2, G→4, T→8, N→15), so no table lookup is needed
/// for the read side — the nibble IS the `c1` value in `bam_fillmd1_core`.
///
/// `seq_bytes` is the packed SEQ field (mutated in place when `use_equal` sets
/// matched bases to 0). `seq_len` is the number of query bases (not byte count).
/// `cigar` is the raw BAM CIGAR op list as `(op_code, len)` pairs. `ref_seq`
/// and `pos` are the reference contig and 0-based start. `md` is cleared and
/// filled with the MD string bytes; the return value is the recomputed NM.
fn compute_md_nm_raw(
    seq_bytes: &mut [u8],
    seq_len: usize,
    cigar: &[(u8, u32)],
    ref_seq: &[u8],
    pos: usize,
    use_equal: bool,
    md: &mut Vec<u8>,
) -> i32 {
    let ref_len = ref_seq.len();
    let mut nm: i32 = 0;
    let mut matched: i64 = 0;
    let mut qpos: usize = 0;
    let mut rpos: usize = pos;

    'outer: for &(op, oplen_u32) in cigar {
        let oplen = oplen_u32 as usize;
        let kind = bam_op_to_kind(op);
        match kind {
            Kind::Match | Kind::SequenceMatch | Kind::SequenceMismatch => {
                let mut j = 0;
                while j < oplen {
                    let z = qpos + j;
                    let rp = rpos + j;
                    if rp >= ref_len || z >= seq_len || ref_seq[rp] == 0 {
                        break;
                    }
                    // BAM nibble codes are already seq_nt16 values: no table lookup.
                    let byte_idx = z / 2;
                    let c1 = if z.is_multiple_of(2) {
                        seq_bytes[byte_idx] >> 4
                    } else {
                        seq_bytes[byte_idx] & 0x0f
                    };
                    let c2 = SEQ_NT16_TABLE[ref_seq[rp] as usize];
                    let is_match = (c1 == c2 && c1 != 15 && c2 != 15) || c1 == 0;
                    if is_match {
                        if use_equal {
                            // Set nibble to 0 (the `=` code in seq_nt16).
                            if z.is_multiple_of(2) {
                                seq_bytes[byte_idx] &= 0x0f;
                            } else {
                                seq_bytes[byte_idx] &= 0xf0;
                            }
                        }
                        matched += 1;
                    } else {
                        append_int(md, matched);
                        md.push(ref_seq[rp].to_ascii_uppercase());
                        matched = 0;
                        nm += 1;
                    }
                    j += 1;
                }
                if j < oplen {
                    break 'outer;
                }
                rpos += oplen;
                qpos += oplen;
            }
            Kind::Deletion => {
                append_int(md, matched);
                md.push(b'^');
                let mut j = 0;
                while j < oplen {
                    let rp = rpos + j;
                    if rp >= ref_len || ref_seq[rp] == 0 {
                        break;
                    }
                    md.push(ref_seq[rp].to_ascii_uppercase());
                    j += 1;
                }
                matched = 0;
                rpos += j;
                nm += j as i32;
                if j < oplen {
                    break 'outer;
                }
            }
            Kind::Insertion => {
                qpos += oplen;
                nm += oplen as i32;
            }
            Kind::SoftClip => {
                qpos += oplen;
            }
            Kind::Skip => {
                rpos += oplen;
            }
            Kind::HardClip | Kind::Pad => {}
        }
    }
    append_int(md, matched);

    nm
}

/// `kputw`: append a base-10 match-run length as ASCII to the MD buffer. The run
/// length is a non-negative count, so this writes digits straight into the Vec
/// (most significant first) with no temporary allocation or formatter overhead.
fn append_int(buf: &mut Vec<u8>, value: i64) {
    debug_assert!(value >= 0, "MD match run lengths are non-negative");
    if value == 0 {
        buf.push(b'0');
        return;
    }
    let mut digits = [0u8; 20];
    let mut i = digits.len();
    let mut v = value;
    while v > 0 {
        i -= 1;
        digits[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    buf.extend_from_slice(&digits[i..]);
}

/// Apply the MD/NM update rules of `bam_fillmd1_core` to a decoded record:
/// append the tag if absent, replace-and-move-to-end if the value differs, and
/// leave it in place (preserving aux order + integer subtype) if it is already
/// correct. Mirrors samtools' `bam_aux_get` / `bam_aux_del` / `bam_aux_append`.
/// `md` is taken by value to avoid copying the MD bytes into the tag.
fn apply_tags(record: &mut RecordBuf, nm: i32, md: &[u8]) {
    let data = record.data_mut();

    let nm_same = data
        .get(&TAG_NM)
        .and_then(Value::as_int)
        .is_some_and(|old| old == i64::from(nm));
    if !nm_same {
        replace_or_append(data, TAG_NM, Value::Int32(nm));
    }

    let md_same = data.get(&TAG_MD).is_some_and(|v| match v {
        Value::String(s) => s.eq_ignore_ascii_case(md),
        _ => false,
    });
    if !md_same {
        replace_or_append(data, TAG_MD, Value::String(md.into()));
    }
}

/// Apply MD/NM tags directly to a `RawRecord`'s aux tail, bypassing full
/// decode+re-encode. This is the hot path for the raw parallel pipeline.
///
/// NM is written as BAM type `i` (signed 32-bit). MD is written as BAM type
/// `Z` (NUL-terminated string). `set_aux` removes the old field (if any) and
/// appends the new value at the end, matching samtools' `bam_aux_del` +
/// `bam_aux_append` behaviour.
fn apply_tags_raw(record: &mut RawRecord, nm: i32, md: &[u8]) {
    let nm_same = record
        .aux_value(NM_TAG)
        .and_then(|v| {
            if v.len() == 4 {
                Some(i32::from_le_bytes(v.try_into().unwrap()))
            } else {
                None
            }
        })
        .is_some_and(|old| old == nm);

    if !nm_same {
        record.set_aux(NM_TAG, AUX_TYPE_I, &nm.to_le_bytes());
    }

    let md_same = record
        .aux_value(MD_TAG)
        .and_then(|v| {
            // The stored Z value includes a NUL terminator; strip it for comparison.
            let stored = v.strip_suffix(&[0]).unwrap_or(v);
            if stored.eq_ignore_ascii_case(md) {
                Some(())
            } else {
                None
            }
        })
        .is_some();

    if !md_same {
        // Z values are NUL-terminated on disk.
        let mut md_z = Vec::with_capacity(md.len() + 1);
        md_z.extend_from_slice(md);
        md_z.push(0);
        record.set_aux(MD_TAG, AUX_TYPE_Z, &md_z);
    }
}

/// samtools' `bam_aux_del` + `bam_aux_append`: when the tag already exists with
/// a different value it is deleted and re-appended at the end of the aux block;
/// when it is absent it is appended. The absent case (the overwhelmingly common
/// one — a fresh calmd run) is a plain `insert`, which appends in O(1) amortised.
/// Only an existing tag triggers the order-preserving rebuild (noodles'
/// `Data::remove` is a swap-remove, which would scramble the surviving order).
fn replace_or_append(data: &mut noodles::sam::alignment::record_buf::Data, tag: Tag, value: Value) {
    if data.get(&tag).is_none() {
        data.insert(tag, value);
        return;
    }
    let kept: Vec<(Tag, Value)> = data
        .iter()
        .filter(|(t, _)| *t != tag)
        .map(|(t, v)| (t, v.clone()))
        .collect();
    let mut rebuilt: noodles::sam::alignment::record_buf::Data = kept.into_iter().collect();
    rebuilt.insert(tag, value);
    *data = rebuilt;
}

/// A reference contig loaded once and reused across consecutive records on it.
///
/// The sequence is stored as `Arc<Vec<u8>>` so that cloning for the parallel
/// batch path is an atomic refcount bump (O(1), no copy), not a full chromosome
/// memcpy. The `None` sentinel means the contig was not found in the reference.
struct RefCache<R> {
    reader: fasta::io::IndexedReader<R>,
    current: Option<(usize, Arc<Vec<u8>>)>,
}

impl<R> RefCache<R>
where
    R: std::io::BufRead + std::io::Seek,
{
    /// Fetch the contig for `tid` (header-resolved name), reusing the held one
    /// when the tid is unchanged. `None` means the contig is absent from the
    /// reference — calmd leaves such records untouched (matching samtools).
    fn get(&mut self, tid: usize, name: &[u8]) -> Result<Option<&[u8]>> {
        if self.current.as_ref().is_none_or(|(t, _)| *t != tid) {
            let region = noodles::core::Region::new(name.to_vec(), ..);
            match self.reader.query(&region) {
                Ok(record) => {
                    self.current = Some((tid, Arc::new(record.sequence().as_ref().to_vec())));
                }
                Err(_) => {
                    self.current = None;
                    return Ok(None);
                }
            }
        }
        Ok(self.current.as_ref().map(|(_, seq)| seq.as_slice()))
    }

    /// Return an `Arc` handle to the contig for `tid`. On a cache hit this is
    /// a single atomic refcount bump — no chromosome copy.
    fn get_arc(&mut self, tid: usize, name: &[u8]) -> Result<Option<Arc<Vec<u8>>>> {
        if self.current.as_ref().is_none_or(|(t, _)| *t != tid) {
            let region = noodles::core::Region::new(name.to_vec(), ..);
            match self.reader.query(&region) {
                Ok(record) => {
                    self.current = Some((tid, Arc::new(record.sequence().as_ref().to_vec())));
                }
                Err(_) => {
                    self.current = None;
                    return Ok(None);
                }
            }
        }
        Ok(self.current.as_ref().map(|(_, arc)| Arc::clone(arc)))
    }
}

pub fn calmd(
    input: &Path,
    reference: &Path,
    output_path: Option<&Path>,
    opts: &CalmdOpts,
    workers: NonZero<usize>,
) -> Result<CalmdStats> {
    let fasta_reader = fasta::io::indexed_reader::Builder::default()
        .build_from_path(reference)
        .map_err(|e| {
            RsomicsError::InvalidInput(format!("reference {}: {e}", reference.display()))
        })?;
    let mut refs = RefCache {
        reader: fasta_reader,
        current: None,
    };

    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    match output_path {
        Some(path) => {
            let mut writer = rsomics_bamio::create_with_workers(path, workers)?;
            if workers.get() == 1 {
                run_serial_raw(reader.get_mut(), &mut writer, &header, &mut refs, opts)
            } else {
                run_parallel_raw(
                    reader.get_mut(),
                    &mut writer,
                    &header,
                    &mut refs,
                    opts,
                    workers,
                )
            }
        }
        None => {
            let mut writer = bam::io::Writer::new(std::io::stdout().lock());
            run_serial_fallback(&mut reader, &mut writer, &header, &mut refs, opts)
        }
    }
}

/// Serial raw path: read raw BAM records, compute MD/NM from nibble SEQ, write
/// raw bytes with aux tags patched in-place. No noodles codec round-trip.
fn run_serial_raw<R, W, F>(
    inner: &mut R,
    writer: &mut bam::io::Writer<W>,
    header: &noodles::sam::Header,
    refs: &mut RefCache<F>,
    opts: &CalmdOpts,
) -> Result<CalmdStats>
where
    R: std::io::BufRead,
    W: std::io::Write,
    F: std::io::BufRead + std::io::Seek,
{
    use rsomics_bamio::raw::write_record;

    writer.write_header(header).map_err(RsomicsError::Io)?;
    let writer_inner = writer.get_mut();

    let mut stats = CalmdStats::default();
    let mut record = RawRecord::default();
    // Per-record scratch reused across the whole stream — no allocation in the
    // hot loop once warmed up.
    let mut cigar_buf: Vec<(u8, u32)> = Vec::new();
    let mut md: Vec<u8> = Vec::new();

    loop {
        let n = raw::read_record(inner, &mut record)?;
        if n == 0 {
            break;
        }
        stats.records += 1;
        process_record_raw(
            &mut record,
            header,
            refs,
            opts,
            &mut stats,
            &mut cigar_buf,
            &mut md,
        )?;
        write_record(writer_inner, &record)?;
    }

    Ok(stats)
}

/// Fallback serial path for stdout output using noodles RecordBuf (stdout has no
/// parallel BGZF writer, so we use the original decoded path for correctness).
fn run_serial_fallback<R, W, F>(
    reader: &mut bam::io::Reader<R>,
    writer: &mut bam::io::Writer<W>,
    header: &noodles::sam::Header,
    refs: &mut RefCache<F>,
    opts: &CalmdOpts,
) -> Result<CalmdStats>
where
    R: std::io::Read,
    W: std::io::Write,
    F: std::io::BufRead + std::io::Seek,
{
    writer.write_header(header).map_err(RsomicsError::Io)?;

    let mut stats = CalmdStats::default();
    let mut record = RecordBuf::default();
    let mut cigar: Vec<(Kind, usize)> = Vec::new();
    let mut md: Vec<u8> = Vec::new();
    while reader
        .read_record_buf(header, &mut record)
        .map_err(RsomicsError::Io)?
        != 0
    {
        stats.records += 1;
        process_record(
            &mut record,
            header,
            refs,
            opts,
            &mut stats,
            &mut cigar,
            &mut md,
        )?;
        writer
            .write_alignment_record(header, &record)
            .map_err(RsomicsError::Io)?;
    }

    Ok(stats)
}

/// Parallel raw path: read `BATCH_SIZE` raw records serially, prefetch contigs,
/// rayon-compute MD/NM on packed nibble SEQ in parallel, write raw bytes in order.
///
/// The raw path avoids the noodles RecordBuf decode+re-encode round-trip that
/// was the bottleneck at `-t4` and above (67% of wall time at t4 in the old
/// path). Raw records hold on-disk payload bytes (~350 bytes for 150 bp);
/// `par_iter_mut` modifies them in place with zero per-record allocation beyond
/// the MD string (~20 bytes). The BGZF writer deflates blocks asynchronously in
/// its worker pool while the main thread continues the next batch.
fn run_parallel_raw<R, W, F>(
    inner: &mut R,
    writer: &mut bam::io::Writer<W>,
    header: &noodles::sam::Header,
    refs: &mut RefCache<F>,
    opts: &CalmdOpts,
    workers: NonZero<usize>,
) -> Result<CalmdStats>
where
    R: std::io::BufRead,
    W: std::io::Write,
    F: std::io::BufRead + std::io::Seek,
{
    use rsomics_bamio::raw::write_record;

    writer.write_header(header).map_err(RsomicsError::Io)?;
    let writer_inner = writer.get_mut();

    let timing = std::env::var("CALMD_PHASE_TIMING").is_ok();

    // Rayon pool sized to `workers`. The BGZF reader uses `workers` inflate
    // threads and the writer uses `workers` deflate threads in separate pools;
    // at any point only one batch's worth of records is being computed, so the
    // three pools are not simultaneously contending for the same data.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers.get())
        .build()
        .map_err(|e| RsomicsError::InvalidInput(format!("rayon pool: {e}")))?;

    let mut stats = CalmdStats::default();
    let mut batch: Vec<RawRecord> = Vec::with_capacity(BATCH_SIZE);
    // Persistent contig cache keyed by TID. Coordinate-sorted input fetches
    // each contig exactly once.
    let mut contig_map: HashMap<usize, Arc<Vec<u8>>> = HashMap::new();

    let mut t_read = std::time::Duration::ZERO;
    let mut t_compute = std::time::Duration::ZERO;
    let mut t_write = std::time::Duration::ZERO;

    loop {
        // --- Phase 1: fill batch (serial raw read) ---
        let t0 = Instant::now();
        batch.clear();
        for _ in 0..BATCH_SIZE {
            let mut record = RawRecord::default();
            let n = raw::read_record(inner, &mut record)?;
            if n == 0 {
                break;
            }
            batch.push(record);
        }
        if timing {
            t_read += t0.elapsed();
        }
        if batch.is_empty() {
            break;
        }

        // --- Phase 2: populate contig cache for any TID not yet seen ---
        for record in &batch {
            let flags = record.flags();
            if flags & 0x4 != 0 {
                continue;
            }
            let tid = record.reference_sequence_id();
            if tid < 0 {
                continue;
            }
            let tid_usize = tid as usize;
            if contig_map.contains_key(&tid_usize) {
                continue;
            }
            let Some((name, _)) = header.reference_sequences().get_index(tid_usize) else {
                continue;
            };
            if let Some(seq) = refs.get_arc(tid_usize, name.as_ref())? {
                contig_map.insert(tid_usize, seq);
            }
        }

        // --- Phase 3: compute MD/NM in parallel (rayon, in-place) ---
        let t1 = Instant::now();
        let contig_map_ref = &contig_map;
        let opts_ref = opts;
        let header_ref = header;

        let batch_computed = std::sync::atomic::AtomicU64::new(0);
        let batch_missing = std::sync::atomic::AtomicU64::new(0);
        let batch_noseq = std::sync::atomic::AtomicU64::new(0);

        pool.install(|| {
            batch.par_iter_mut().for_each(|record| {
                process_record_raw_parallel(
                    record,
                    header_ref,
                    contig_map_ref,
                    opts_ref,
                    &batch_computed,
                    &batch_missing,
                    &batch_noseq,
                );
            });
        });
        if timing {
            t_compute += t1.elapsed();
        }

        // --- Phase 4: write in original order (serial raw write) ---
        let t2 = Instant::now();
        stats.records += batch.len() as u64;
        stats.computed += batch_computed.load(std::sync::atomic::Ordering::Relaxed);
        stats.missing_ref += batch_missing.load(std::sync::atomic::Ordering::Relaxed);
        stats.no_sequence += batch_noseq.load(std::sync::atomic::Ordering::Relaxed);
        for record in &batch {
            write_record(writer_inner, record)?;
        }
        if timing {
            t_write += t2.elapsed();
        }
    }

    if timing {
        eprintln!(
            "PHASE TIMING: read={:.3}s compute={:.3}s write={:.3}s total_phase={:.3}s",
            t_read.as_secs_f64(),
            t_compute.as_secs_f64(),
            t_write.as_secs_f64(),
            (t_read + t_compute + t_write).as_secs_f64()
        );
    }

    Ok(stats)
}

/// Per-record raw MD/NM pass for the parallel path. Reads nibble SEQ and raw
/// CIGAR from `RawRecord`, updates MD/NM aux in-place. No noodles decode.
fn process_record_raw_parallel(
    record: &mut RawRecord,
    header: &noodles::sam::Header,
    contig_map: &HashMap<usize, Arc<Vec<u8>>>,
    opts: &CalmdOpts,
    computed: &std::sync::atomic::AtomicU64,
    missing_ref: &std::sync::atomic::AtomicU64,
    no_sequence: &std::sync::atomic::AtomicU64,
) {
    use std::sync::atomic::Ordering::Relaxed;

    let flags = record.flags();
    if flags & 0x4 != 0 {
        return;
    }
    let tid = record.reference_sequence_id();
    if tid < 0 {
        return;
    }
    let tid_usize = tid as usize;
    let pos_raw = record.alignment_start();
    if pos_raw < 0 {
        return;
    }
    if header.reference_sequences().get_index(tid_usize).is_none() {
        return;
    }

    let Some(ref_seq) = contig_map.get(&tid_usize) else {
        missing_ref.fetch_add(1, Relaxed);
        return;
    };

    let seq_len = record.sequence_len();
    if seq_len == 0 {
        no_sequence.fetch_add(1, Relaxed);
        return;
    }

    let pos = pos_raw as usize;

    // Collect CIGAR ops from the raw payload into a small inline buffer.
    let mut cigar: Vec<(u8, u32)> = record.cigar_ops().collect();

    let mut md: Vec<u8> = Vec::new();

    // Compute MD/NM directly on the raw nibble SEQ, mutating the record's
    // packed SEQ bytes in place for `use_equal`. `seq_bytes_mut()` accesses
    // the packed [(l_seq+1)/2] bytes starting after name+cigar in the payload.
    let nm = {
        let seq_bytes = record.seq_bytes_mut();
        compute_md_nm_raw(
            seq_bytes,
            seq_len,
            &cigar,
            ref_seq,
            pos,
            opts.use_equal,
            &mut md,
        )
    };

    // Patch the raw aux tail: set NM and MD without decoding the rest of the record.
    apply_tags_raw(record, nm, &md);
    computed.fetch_add(1, Relaxed);
    cigar.clear();
}

/// Per-record raw MD/NM pass for the serial path (reuses scratch buffers).
fn process_record_raw<F>(
    record: &mut RawRecord,
    header: &noodles::sam::Header,
    refs: &mut RefCache<F>,
    opts: &CalmdOpts,
    stats: &mut CalmdStats,
    cigar_buf: &mut Vec<(u8, u32)>,
    md: &mut Vec<u8>,
) -> Result<()>
where
    F: std::io::BufRead + std::io::Seek,
{
    let flags = record.flags();
    if flags & 0x4 != 0 {
        return Ok(());
    }
    let tid = record.reference_sequence_id();
    if tid < 0 {
        return Ok(());
    }
    let tid_usize = tid as usize;
    let pos_raw = record.alignment_start();
    if pos_raw < 0 {
        return Ok(());
    }
    let Some((name, _)) = header.reference_sequences().get_index(tid_usize) else {
        return Ok(());
    };

    let ref_seq = match refs.get(tid_usize, name.as_ref())? {
        Some(seq) => seq,
        None => {
            stats.missing_ref += 1;
            return Ok(());
        }
    };

    let seq_len = record.sequence_len();
    if seq_len == 0 {
        stats.no_sequence += 1;
        return Ok(());
    }

    let pos = pos_raw as usize;

    cigar_buf.clear();
    cigar_buf.extend(record.cigar_ops());
    md.clear();

    let nm = {
        let seq_bytes = record.seq_bytes_mut();
        compute_md_nm_raw(
            seq_bytes,
            seq_len,
            cigar_buf,
            ref_seq,
            pos,
            opts.use_equal,
            md,
        )
    };

    apply_tags_raw(record, nm, md);
    stats.computed += 1;
    Ok(())
}

/// The per-record body of samtools' `bam_fillmd` loop: skip unmapped/refless
/// records, fetch the contig (held by reference — never copied), run the MD/NM
/// walk in place, and apply the tag updates. Used by the stdout fallback path.
fn process_record<F>(
    record: &mut RecordBuf,
    header: &noodles::sam::Header,
    refs: &mut RefCache<F>,
    opts: &CalmdOpts,
    stats: &mut CalmdStats,
    cigar: &mut Vec<(Kind, usize)>,
    md: &mut Vec<u8>,
) -> Result<()>
where
    F: std::io::BufRead + std::io::Seek,
{
    if record.flags().is_unmapped() {
        return Ok(());
    }
    let Some(tid) = record.reference_sequence_id() else {
        return Ok(());
    };
    let Some(start) = record.alignment_start() else {
        return Ok(());
    };
    let Some((name, _)) = header.reference_sequences().get_index(tid) else {
        return Ok(());
    };

    // The contig slice borrows the cache and is only read in `compute_md_nm`;
    // the record mutation that follows borrows `record`, a disjoint object. The
    // contig is therefore never copied per record (the MD pass stays
    // O(records + contig_len), not O(records × contig_len)).
    let ref_seq = match refs.get(tid, name.as_ref())? {
        Some(seq) => seq,
        None => {
            stats.missing_ref += 1;
            return Ok(());
        }
    };

    if record.sequence().is_empty() {
        stats.no_sequence += 1;
        return Ok(());
    }

    let pos = start.get() - 1;
    cigar.clear();
    cigar.extend(
        record
            .cigar()
            .as_ref()
            .iter()
            .map(|op| (op.kind(), op.len())),
    );
    md.clear();

    let seq = record.sequence_mut().as_mut();
    let nm = compute_md_nm(seq, cigar, ref_seq, pos, opts.use_equal, md);

    apply_tags(record, nm, md);
    stats.computed += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_walk(seq: &[u8], cigar: &[(Kind, usize)], rf: &[u8], pos: usize) -> (String, i32) {
        let mut s = seq.to_vec();
        let mut md = Vec::new();
        let nm = compute_md_nm(&mut s, cigar, rf, pos, false, &mut md);
        (String::from_utf8(md).unwrap(), nm)
    }

    #[test]
    fn perfect_match() {
        let (md, nm) = run_walk(b"ACGTACGT", &[(Kind::Match, 8)], b"ACGTACGT", 0);
        assert_eq!((md.as_str(), nm), ("8", 0));
    }

    #[test]
    fn two_mismatches() {
        let (md, nm) = run_walk(b"ATGTACAT", &[(Kind::Match, 8)], b"ACGTACGT", 0);
        assert_eq!((md.as_str(), nm), ("1C4G1", 2));
    }

    #[test]
    fn deletion() {
        // 4M2D2M over ref ACGT GG AC, read = ACGT AC.
        let (md, nm) = run_walk(
            b"ACGTAC",
            &[(Kind::Match, 4), (Kind::Deletion, 2), (Kind::Match, 2)],
            b"ACGTGGAC",
            0,
        );
        assert_eq!((md.as_str(), nm), ("4^GG2", 2));
    }

    #[test]
    fn insertion_adds_only_nm() {
        // 3M2I3M; the inserted bases never appear in MD but bump NM by 2.
        let (md, nm) = run_walk(
            b"ACGTTACG",
            &[(Kind::Match, 3), (Kind::Insertion, 2), (Kind::Match, 3)],
            b"ACGACG",
            0,
        );
        assert_eq!((md.as_str(), nm), ("6", 2));
    }

    #[test]
    fn soft_clip_and_skip() {
        // 2S3M3N3M: soft-clip consumes read only; N consumes ref only.
        let (md, nm) = run_walk(
            b"NNACGTAC",
            &[
                (Kind::SoftClip, 2),
                (Kind::Match, 3),
                (Kind::Skip, 3),
                (Kind::Match, 3),
            ],
            b"ACGXXXTAC",
            0,
        );
        assert_eq!((md.as_str(), nm), ("6", 0));
    }

    #[test]
    fn n_in_reference_is_mismatch() {
        // A read base over an N reference base is a mismatch emitting the ref N.
        let (md, nm) = run_walk(b"ACGT", &[(Kind::Match, 4)], b"ACNT", 0);
        assert_eq!((md.as_str(), nm), ("2N1", 1));
    }

    #[test]
    fn equal_base_in_read_is_match() {
        // A read code of 0 (`=`) is always a match regardless of the ref base.
        let (md, nm) = run_walk(b"=CGT", &[(Kind::Match, 4)], b"ACGT", 0);
        assert_eq!((md.as_str(), nm), ("4", 0));
    }

    #[test]
    fn use_equal_rewrites_matches() {
        let mut s = b"ATGT".to_vec();
        let mut md = Vec::new();
        // Position 1 mismatches (T vs C); the other three match → become `=`.
        let nm = compute_md_nm(&mut s, &[(Kind::Match, 4)], b"ACGT", 0, true, &mut md);
        assert_eq!(&s, b"=T==");
        assert_eq!((String::from_utf8(md).unwrap().as_str(), nm), ("1C2", 1));
    }
}
