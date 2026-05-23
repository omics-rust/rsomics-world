//! FASTQ → unaligned BAM conversion, porting `samtools import` (MIT).
//!
//! The hot path avoids noodles' record model entirely: each FASTQ record is
//! serialised directly to a BAM block in memory and flushed through rsomics-bamio's
//! parallel BGZF writer. This skips the decode→struct→encode round-trip that
//! noodles' `write_alignment_record` performs and means the only non-trivial work
//! per read is the 4-bit SEQ packing and one copy of the quality bytes.
//!
//! Semantics match samtools import (`bam_import.c`, MIT): FLAG, RNEXT/PNEXT/TLEN,
//! MAPQ, RNAME, CIGAR, and read-name /1 /2 stripping are all faithfully ported
//! from reading the 1.23.1 source.

use std::io::Write;
use std::num::NonZero;
use std::path::PathBuf;

use bstr::BString;
use noodles::bam;
use noodles::sam;
use noodles::sam::alignment::io::Write as AlnWrite;
use noodles::sam::header::record::value::{
    Map,
    map::{Header as MapHeader, ReadGroup},
};
use rsomics_common::{Result, RsomicsError};
use rsomics_seqio::{FastqSource, open_fastq};
use serde::Serialize;

// ── FLAG bits (SAMv1 §1.4.2) ──────────────────────────────────────────────────
const FLAG_PAIRED: u16 = 0x1; // read is paired
const FLAG_FUNMAP: u16 = 0x4; // read itself is unmapped
const FLAG_MUNMAP: u16 = 0x8; // mate is unmapped
const FLAG_READ1: u16 = 0x40; // first in template
const FLAG_READ2: u16 = 0x80; // last in template

// SE reads: 0x4 (FUNMAP)
const SE_FLAGS: u16 = FLAG_FUNMAP;
// PE R1: 0x4d (PAIRED | FUNMAP | MUNMAP | READ1)
const PE_R1_FLAGS: u16 = FLAG_PAIRED | FLAG_FUNMAP | FLAG_MUNMAP | FLAG_READ1;
// PE R2: 0x8d (PAIRED | FUNMAP | MUNMAP | READ2)
const PE_R2_FLAGS: u16 = FLAG_PAIRED | FLAG_FUNMAP | FLAG_MUNMAP | FLAG_READ2;

// ── BAM fixed record header size (SAMv1 §4.2) ─────────────────────────────────
const FIXED_HEADER: usize = 32;

// ── Public API ─────────────────────────────────────────────────────────────────

/// Which FASTQ input mode the user selected.
#[derive(Debug, Clone)]
pub enum ImportMode {
    /// Single-ended reads: no pairing flags set (samtools `-0`).
    Single(PathBuf),
    /// Interleaved: alternating R1/R2 in one file (samtools `-s`).
    Interleaved(PathBuf),
    /// Paired-end from two separate files (samtools `-1`/`-2`).
    Paired(PathBuf, PathBuf),
}

/// Options passed through from the CLI layer.
pub struct ImportOpts {
    pub mode: ImportMode,
    pub output: Option<PathBuf>,
    pub uncompressed: bool,
    pub rg_line: Option<String>,
    pub workers: NonZero<usize>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ImportStats {
    pub records_written: u64,
}

/// Convert FASTQ reads to unaligned BAM.
pub fn import(opts: &ImportOpts) -> Result<ImportStats> {
    let header = build_header(opts.rg_line.as_deref())?;
    let rg_id: Option<String> = opts
        .rg_line
        .as_deref()
        .and_then(extract_rg_id)
        .map(str::to_owned);

    if let Some(out_path) = &opts.output {
        let records_written = if opts.uncompressed {
            let file = std::fs::File::create(out_path).map_err(|e| {
                RsomicsError::InvalidInput(format!("creating {}: {e}", out_path.display()))
            })?;
            let mut writer = bam::io::Writer::new(file);
            writer.write_header(&header).map_err(RsomicsError::Io)?;
            dispatch_raw(opts, writer.get_mut(), rg_id.as_deref())?
        } else {
            let mut writer = rsomics_bamio::create_with_workers(out_path, opts.workers)?;
            writer.write_header(&header).map_err(RsomicsError::Io)?;
            dispatch_raw(opts, writer.get_mut(), rg_id.as_deref())?
        };
        Ok(ImportStats { records_written })
    } else {
        // stdout: SAM text
        let records_written = dispatch_sam(opts, &header, rg_id.as_deref())?;
        Ok(ImportStats { records_written })
    }
}

// ── Header construction ────────────────────────────────────────────────────────

fn build_header(rg_line: Option<&str>) -> Result<sam::Header> {
    use noodles::sam::header::record::value::map::header::tag;

    let mut hd =
        Map::<MapHeader>::new(noodles::sam::header::record::value::map::header::Version::new(1, 6));
    // SO:unsorted and GO:query — stored as other_fields (noodles has no enum for these)
    hd.other_fields_mut()
        .insert(tag::SORT_ORDER, BString::from("unsorted"));
    hd.other_fields_mut()
        .insert(tag::GROUP_ORDER, BString::from("query"));

    let mut builder = sam::Header::builder().set_header(hd);

    if let Some(line) = rg_line {
        let (rg_id, rg_map) = parse_rg_line(line)?;
        builder = builder.add_read_group(rg_id, rg_map);
    }

    Ok(builder.build())
}

/// Parse "@RG\tID:foo\tSM:bar" into (id, Map<ReadGroup>). The ID field is
/// returned separately because it is both the map key and needed for the RG:Z
/// aux tag. Other fields are stored as `other_fields` on the map.
fn parse_rg_line(line: &str) -> Result<(BString, Map<ReadGroup>)> {
    let body = line.strip_prefix("@RG\t").unwrap_or(line);
    let mut id: Option<String> = None;
    let mut map = Map::<ReadGroup>::default();
    for field in body.split('\t') {
        let (tag, value) = field.split_once(':').ok_or_else(|| {
            RsomicsError::InvalidInput(format!("malformed @RG field (no ':'): {field:?}"))
        })?;
        if tag == "ID" {
            id = Some(value.to_string());
        } else {
            use noodles::sam::header::record::value::map::read_group::tag::Standard;
            use noodles::sam::header::record::value::map::tag::Other;
            let tag_bytes = tag.as_bytes();
            if tag_bytes.len() != 2 {
                return Err(RsomicsError::InvalidInput(format!(
                    "RG tag must be 2 chars: {tag:?}"
                )));
            }
            let buf: [u8; 2] = [tag_bytes[0], tag_bytes[1]];
            let other: Other<Standard> = Other::try_from(buf).map_err(|_| {
                RsomicsError::InvalidInput(format!("RG tag is a reserved standard tag: {tag:?}"))
            })?;
            map.other_fields_mut().insert(other, BString::from(value));
        }
    }
    let rg_id = id.ok_or_else(|| RsomicsError::InvalidInput("@RG line missing ID field".into()))?;
    Ok((BString::from(rg_id), map))
}

/// Extract the ID value from a raw @RG line for embedding in RG:Z aux tags.
fn extract_rg_id(rg_line: &str) -> Option<&str> {
    let body = rg_line.strip_prefix("@RG\t").unwrap_or(rg_line);
    for field in body.split('\t') {
        if let Some(v) = field.strip_prefix("ID:") {
            return Some(v);
        }
    }
    None
}

// ── Raw BAM binary dispatch ────────────────────────────────────────────────────

fn dispatch_raw<W: Write>(opts: &ImportOpts, out: &mut W, rg_id: Option<&str>) -> Result<u64> {
    match &opts.mode {
        ImportMode::Single(path) => {
            let src = open_fastq(path)?;
            write_single_raw(out, src, rg_id)
        }
        ImportMode::Interleaved(path) => {
            let src = open_fastq(path)?;
            write_interleaved_raw(out, src, rg_id)
        }
        ImportMode::Paired(r1, r2) => {
            let src1 = open_fastq(r1)?;
            let src2 = open_fastq(r2)?;
            write_paired_raw(out, src1, src2, rg_id)
        }
    }
}

// ── SAM text dispatch (stdout path) ───────────────────────────────────────────

fn dispatch_sam(opts: &ImportOpts, header: &sam::Header, rg_id: Option<&str>) -> Result<u64> {
    let stdout = std::io::stdout();
    let mut buf = std::io::BufWriter::with_capacity(256 * 1024, stdout.lock());
    let mut writer = sam::io::Writer::new(&mut buf);
    writer.write_header(header).map_err(RsomicsError::Io)?;
    let count = match &opts.mode {
        ImportMode::Single(path) => {
            let src = open_fastq(path)?;
            write_single_sam(&mut writer, header, src, rg_id)?
        }
        ImportMode::Interleaved(path) => {
            let src = open_fastq(path)?;
            write_interleaved_sam(&mut writer, header, src, rg_id)?
        }
        ImportMode::Paired(r1, r2) => {
            let src1 = open_fastq(r1)?;
            let src2 = open_fastq(r2)?;
            write_paired_sam(&mut writer, header, src1, src2, rg_id)?
        }
    };
    buf.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn write_single_sam<W: Write>(
    writer: &mut sam::io::Writer<W>,
    header: &sam::Header,
    src: FastqSource,
    rg_id: Option<&str>,
) -> Result<u64> {
    let mut count = 0u64;
    for result in src {
        let rec = result?;
        let name = strip_pair_suffix(&rec.id);
        let record = build_sam_record(name, &rec.seq, &rec.qual, SE_FLAGS, rg_id);
        writer
            .write_alignment_record(header, &record)
            .map_err(RsomicsError::Io)?;
        count += 1;
    }
    Ok(count)
}

fn write_interleaved_sam<W: Write>(
    writer: &mut sam::io::Writer<W>,
    header: &sam::Header,
    src: FastqSource,
    rg_id: Option<&str>,
) -> Result<u64> {
    let mut src = src;
    let mut count = 0u64;
    while let Some(r1_res) = src.next() {
        let r1 = r1_res?;
        let r2 = src.next().ok_or_else(|| {
            RsomicsError::InvalidInput("interleaved FASTQ has odd number of records".into())
        })??;
        let name = strip_pair_suffix(&r1.id);
        let rec1 = build_sam_record(name, &r1.seq, &r1.qual, PE_R1_FLAGS, rg_id);
        let rec2 = build_sam_record(name, &r2.seq, &r2.qual, PE_R2_FLAGS, rg_id);
        writer
            .write_alignment_record(header, &rec1)
            .map_err(RsomicsError::Io)?;
        writer
            .write_alignment_record(header, &rec2)
            .map_err(RsomicsError::Io)?;
        count += 2;
    }
    Ok(count)
}

fn write_paired_sam<W: Write>(
    writer: &mut sam::io::Writer<W>,
    header: &sam::Header,
    src1: FastqSource,
    src2: FastqSource,
    rg_id: Option<&str>,
) -> Result<u64> {
    let mut src2 = src2.peekable();
    let mut count = 0u64;
    for r1_res in src1 {
        let r1 = r1_res?;
        let r2 = src2.next().ok_or_else(|| {
            RsomicsError::InvalidInput("read-2 file has fewer records than read-1".into())
        })??;
        let name = strip_pair_suffix(&r1.id);
        let rec1 = build_sam_record(name, &r1.seq, &r1.qual, PE_R1_FLAGS, rg_id);
        let rec2 = build_sam_record(name, &r2.seq, &r2.qual, PE_R2_FLAGS, rg_id);
        writer
            .write_alignment_record(header, &rec1)
            .map_err(RsomicsError::Io)?;
        writer
            .write_alignment_record(header, &rec2)
            .map_err(RsomicsError::Io)?;
        count += 2;
    }
    if src2.next().is_some() {
        return Err(RsomicsError::InvalidInput(
            "read-2 file has more records than read-1".into(),
        ));
    }
    Ok(count)
}

fn build_sam_record(
    name: &[u8],
    seq: &[u8],
    qual: &[u8],
    flags: u16,
    rg_id: Option<&str>,
) -> sam::alignment::RecordBuf {
    use sam::alignment::record::data::field::Tag;
    use sam::alignment::record::{Flags, MappingQuality};
    use sam::alignment::record_buf::data::field::Value;
    use sam::alignment::record_buf::{Data, QualityScores, Sequence};

    let sequence = Sequence::from(seq.to_vec());
    // FASTQ qual is ASCII (Phred+33); SAM stores raw Phred values
    let qual_phred: Vec<u8> = qual.iter().map(|&q| q - 33).collect();
    let quality_scores = QualityScores::from(qual_phred);

    let mut data = Data::default();
    if let Some(id) = rg_id {
        data.insert(Tag::READ_GROUP, Value::String(id.into()));
    }

    // MAPQ=0 for all unaligned reads — MappingQuality::new(255) is None (missing);
    // samtools import uses 0, not the missing sentinel.
    let mapq = MappingQuality::new(0).expect("0 is a valid MAPQ");

    let mut builder = sam::alignment::RecordBuf::builder()
        .set_flags(Flags::from(flags))
        .set_mapping_quality(mapq)
        .set_sequence(sequence)
        .set_quality_scores(quality_scores)
        .set_data(data);

    if !name.is_empty() {
        builder = builder.set_name(bstr::BString::from(name));
    }

    builder.build()
}

// ── Raw BAM serialisers (used when -o FILE is given) ─────────────────────────

fn write_single_raw<W: Write>(out: &mut W, src: FastqSource, rg_id: Option<&str>) -> Result<u64> {
    let mut buf = Vec::with_capacity(512);
    let mut count = 0u64;
    for result in src {
        let rec = result?;
        let name = strip_pair_suffix(&rec.id);
        encode_bam_record(&mut buf, name, &rec.seq, &rec.qual, SE_FLAGS, rg_id)?;
        write_block(out, &buf)?;
        count += 1;
    }
    Ok(count)
}

fn write_interleaved_raw<W: Write>(
    out: &mut W,
    src: FastqSource,
    rg_id: Option<&str>,
) -> Result<u64> {
    let mut src = src;
    let mut buf = Vec::with_capacity(512);
    let mut count = 0u64;
    while let Some(r1_res) = src.next() {
        let r1 = r1_res?;
        let r2 = src.next().ok_or_else(|| {
            RsomicsError::InvalidInput("interleaved FASTQ has odd number of records".into())
        })??;
        let name = strip_pair_suffix(&r1.id);
        encode_bam_record(&mut buf, name, &r1.seq, &r1.qual, PE_R1_FLAGS, rg_id)?;
        write_block(out, &buf)?;
        encode_bam_record(&mut buf, name, &r2.seq, &r2.qual, PE_R2_FLAGS, rg_id)?;
        write_block(out, &buf)?;
        count += 2;
    }
    Ok(count)
}

fn write_paired_raw<W: Write>(
    out: &mut W,
    src1: FastqSource,
    src2: FastqSource,
    rg_id: Option<&str>,
) -> Result<u64> {
    let mut src2 = src2.peekable();
    let mut buf = Vec::with_capacity(512);
    let mut count = 0u64;
    for r1_res in src1 {
        let r1 = r1_res?;
        let r2 = src2.next().ok_or_else(|| {
            RsomicsError::InvalidInput("read-2 file has fewer records than read-1".into())
        })??;
        let name = strip_pair_suffix(&r1.id);
        encode_bam_record(&mut buf, name, &r1.seq, &r1.qual, PE_R1_FLAGS, rg_id)?;
        write_block(out, &buf)?;
        encode_bam_record(&mut buf, name, &r2.seq, &r2.qual, PE_R2_FLAGS, rg_id)?;
        write_block(out, &buf)?;
        count += 2;
    }
    if src2.next().is_some() {
        return Err(RsomicsError::InvalidInput(
            "read-2 file has more records than read-1".into(),
        ));
    }
    Ok(count)
}

// ── Raw BAM record serialiser ─────────────────────────────────────────────────

/// Serialise one FASTQ record to BAM bytes in `buf` (cleared first).
///
/// Layout (SAM spec §4.2, field names from the spec):
/// ```text
/// refID(-1) pos(-1) l_read_name mapq(0) bin(4680) n_cigar_op(0) flag l_seq
/// next_refID(-1) next_pos(-1) tlen(0)
/// read_name NUL [no cigar] [seq nibbles] [qual raw phred] [aux RG:Z if set]
/// ```
///
/// Unmapped unaligned defaults: refID=-1, pos=-1 (BAM 0-based; SAM POS=0),
/// MAPQ=0, CIGAR empty, RNEXT=-1, PNEXT=-1, TLEN=0 — exactly what samtools
/// import emits (black-box verified against 1.23.1). htslib's FASTQ parser
/// initialises MAPQ to 0 and sets `FLAG_FUNMAP` for every read it parses.
fn encode_bam_record(
    buf: &mut Vec<u8>,
    name: &[u8],
    seq: &[u8],
    qual: &[u8],
    flags: u16,
    rg_id: Option<&str>,
) -> Result<()> {
    buf.clear();

    let l_read_name: u8 = (name.len() + 1)
        .try_into()
        .map_err(|_| RsomicsError::InvalidInput("read name too long for BAM".into()))?;
    let l_seq = u32::try_from(seq.len())
        .map_err(|_| RsomicsError::InvalidInput("sequence too long for BAM l_seq field".into()))?;
    let seq_bytes = seq.len().div_ceil(2);

    // Fixed 32-byte header
    buf.extend_from_slice(&(-1i32).to_le_bytes()); // refID = -1 (unmapped)
    buf.extend_from_slice(&(-1i32).to_le_bytes()); // pos = -1 (none)
    buf.push(l_read_name); // l_read_name
    buf.push(0); // MAPQ = 0 (unmapped, matches samtools import)
    buf.extend_from_slice(&4680u16.to_le_bytes()); // bin = 4680 (unmapped)
    buf.extend_from_slice(&0u16.to_le_bytes()); // n_cigar_op = 0
    buf.extend_from_slice(&flags.to_le_bytes()); // FLAG
    buf.extend_from_slice(&l_seq.to_le_bytes()); // l_seq
    buf.extend_from_slice(&(-1i32).to_le_bytes()); // next_refID = -1
    buf.extend_from_slice(&(-1i32).to_le_bytes()); // next_pos = -1
    buf.extend_from_slice(&0i32.to_le_bytes()); // tlen = 0

    debug_assert_eq!(buf.len(), FIXED_HEADER);

    // read_name + NUL
    buf.extend_from_slice(name);
    buf.push(0);

    // SEQ: pack two bases per byte (high nibble = earlier base)
    let full_pairs = seq.len() / 2;
    for i in 0..full_pairs {
        buf.push((nt16(seq[i * 2]) << 4) | nt16(seq[i * 2 + 1]));
    }
    if seq.len() % 2 == 1 {
        buf.push(nt16(seq[seq.len() - 1]) << 4);
    }
    debug_assert_eq!(buf.len(), FIXED_HEADER + name.len() + 1 + seq_bytes);

    // QUAL: FASTQ ASCII (Phred+33) → raw Phred
    for &q in qual {
        buf.push(q - 33);
    }

    // RG:Z aux tag if a read group was specified
    if let Some(id) = rg_id {
        buf.push(b'R');
        buf.push(b'G');
        buf.push(b'Z');
        buf.extend_from_slice(id.as_bytes());
        buf.push(0); // NUL-terminate Z fields
    }

    Ok(())
}

/// Write one BAM record block: 4-byte LE `block_size` followed by the payload.
#[inline]
fn write_block<W: Write>(out: &mut W, payload: &[u8]) -> Result<()> {
    let size = u32::try_from(payload.len())
        .map_err(|e| RsomicsError::InvalidInput(format!("record too large: {e}")))?;
    out.write_all(&size.to_le_bytes())
        .map_err(RsomicsError::Io)?;
    out.write_all(payload).map_err(RsomicsError::Io)
}

/// Encode a nucleotide character to the `seq_nt16` code (BAM spec Table 3).
/// Matches htslib's `seq_nt16_table`.
#[inline]
fn nt16(base: u8) -> u8 {
    match base {
        b'=' => 0,
        b'A' | b'a' => 1,
        b'C' | b'c' => 2,
        b'M' | b'm' => 3,
        b'G' | b'g' => 4,
        b'R' | b'r' => 5,
        b'S' | b's' => 6,
        b'V' | b'v' => 7,
        b'T' | b't' => 8,
        b'W' | b'w' => 9,
        b'Y' | b'y' => 10,
        b'H' | b'h' => 11,
        b'K' | b'k' => 12,
        b'D' | b'd' => 13,
        b'B' | b'b' => 14,
        _ => 15, // N and anything unrecognised
    }
}

// ── Read-name /1 /2 stripping ─────────────────────────────────────────────────

/// Strip the trailing `/1` or `/2` suffix from a FASTQ read name, matching
/// htslib's FASTQ parser (fastq.c): strip only the pair-marker suffix so both
/// mates share the same QNAME in BAM. The comment (space-delimited suffix) is
/// dropped — BAM QNAME is the first whitespace-delimited token only.
#[inline]
fn strip_pair_suffix(id: &[u8]) -> &[u8] {
    let name = match id.iter().position(|&b| b == b' ') {
        Some(sp) => &id[..sp],
        None => id,
    };
    if name.ends_with(b"/1") || name.ends_with(b"/2") {
        &name[..name.len() - 2]
    } else {
        name
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_suffix_removes_1_and_2() {
        assert_eq!(strip_pair_suffix(b"read1/1"), b"read1");
        assert_eq!(strip_pair_suffix(b"read1/2"), b"read1");
        assert_eq!(strip_pair_suffix(b"read1"), b"read1");
        assert_eq!(strip_pair_suffix(b"read1 comment"), b"read1");
        assert_eq!(strip_pair_suffix(b"read1/1 comment"), b"read1");
    }

    #[test]
    fn nt16_standard_bases() {
        assert_eq!(nt16(b'A'), 1);
        assert_eq!(nt16(b'C'), 2);
        assert_eq!(nt16(b'G'), 4);
        assert_eq!(nt16(b'T'), 8);
        assert_eq!(nt16(b'N'), 15);
        assert_eq!(nt16(b'n'), 15);
        assert_eq!(nt16(b'a'), 1);
    }

    #[test]
    fn bam_record_layout_single() {
        let mut buf = Vec::new();
        encode_bam_record(&mut buf, b"r1", b"ACGT", b"IIII", SE_FLAGS, None).unwrap();
        // refID = -1
        assert_eq!(&buf[0..4], &(-1i32).to_le_bytes());
        // pos = -1
        assert_eq!(&buf[4..8], &(-1i32).to_le_bytes());
        // l_read_name = 3 (len("r1") + 1 for NUL)
        assert_eq!(buf[8], 3);
        // MAPQ = 0 (unmapped, samtools import behaviour)
        assert_eq!(buf[9], 0);
        // flags = 0x4 (FUNMAP) for SE reads
        assert_eq!(
            u16::from_le_bytes(buf[14..16].try_into().unwrap()),
            SE_FLAGS
        );
        // l_seq = 4
        assert_eq!(u32::from_le_bytes(buf[16..20].try_into().unwrap()), 4);
        // read_name "r1\0" at offset 32
        assert_eq!(&buf[32..35], b"r1\0");
        // packed SEQ: A=1,C=2 → 0x12; G=4,T=8 → 0x48
        assert_eq!(buf[35], 0x12);
        assert_eq!(buf[36], 0x48);
        // QUAL: 'I' - 33 = 40
        assert_eq!(&buf[37..41], &[40u8, 40, 40, 40]);
    }

    #[test]
    fn rg_id_extracted() {
        assert_eq!(extract_rg_id("@RG\tID:lib1\tSM:s1"), Some("lib1"));
        assert_eq!(extract_rg_id("ID:lib1\tSM:s1"), Some("lib1"));
        assert_eq!(extract_rg_id("@RG\tSM:s1"), None);
    }

    #[test]
    fn bam_record_rg_aux() {
        let mut buf = Vec::new();
        encode_bam_record(&mut buf, b"r1", b"ACGT", b"IIII", SE_FLAGS, Some("lib1")).unwrap();
        // RG:Z:lib1\0 should appear at the end
        let expected_aux = b"RGZlib1\0";
        let len = buf.len();
        assert_eq!(&buf[len - expected_aux.len()..], expected_aux);
    }
}
