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
//! at `workers >= 2`); records are decoded to a `RecordBuf` because calmd needs
//! the read SEQ to compare against the reference, so the raw seq-skipping path
//! does not apply here. Output goes through the bamio parallel BGZF writer.
//!
//! The reference is read from an indexed FASTA (`.fai`); each contig is fetched
//! once on first use and held until the next contig appears. Coordinate-sorted
//! input therefore touches each contig once and never re-reads it — the common
//! calmd case. Out-of-order input re-fetches, which is correct but slower; that
//! mirrors samtools' own behaviour before its tid-cache kicks in.

use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::fasta;
use noodles::sam::alignment::RecordBuf;
use noodles::sam::alignment::io::Write as AlignmentWrite;
use noodles::sam::alignment::record::cigar::op::Kind;
use noodles::sam::alignment::record::data::field::Tag;
use noodles::sam::alignment::record_buf::data::field::Value;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

const TAG_NM: Tag = Tag::EDIT_DISTANCE;
const TAG_MD: Tag = Tag::MISMATCHED_POSITIONS;

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
struct RefCache<R> {
    reader: fasta::io::IndexedReader<R>,
    current: Option<(usize, Vec<u8>)>,
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
                    self.current = Some((tid, record.sequence().as_ref().to_vec()));
                }
                Err(_) => {
                    self.current = None;
                    return Ok(None);
                }
            }
        }
        Ok(self.current.as_ref().map(|(_, seq)| seq.as_slice()))
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
            run(&mut reader, &mut writer, &header, &mut refs, opts)
        }
        None => {
            let mut writer = bam::io::Writer::new(std::io::stdout().lock());
            run(&mut reader, &mut writer, &header, &mut refs, opts)
        }
    }
}

fn run<R, W, F>(
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
    // Per-record scratch reused across the whole stream — no allocation in the
    // hot loop. `cigar` holds the decoded op list; `md` accumulates the MD
    // string. Both are cleared, never reallocated, once warmed up.
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

/// The per-record body of samtools' `bam_fillmd` loop: skip unmapped/refless
/// records, fetch the contig (held by reference — never copied), run the MD/NM
/// walk in place, and apply the tag updates.
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
