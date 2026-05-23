//! Revert aligner-applied changes on BAM records, like `samtools reset`.
//!
//! Each primary record is reverted to its pre-alignment state: the FLAG is
//! reduced to read/mate pairing + unmapped bits, RNAME/POS/MAPQ/CIGAR/RNEXT/
//! PNEXT/TLEN go to their unmapped defaults, aligner aux tags are dropped, and a
//! record stored reverse-complemented (the `0x10` reverse bit) has its SEQ/QUAL
//! restored to original read orientation. Secondary (`0x100`) and supplementary
//! (`0x800`) records are dropped. The output header is rebuilt from scratch:
//! `@HD VN:1.6`, the input `@RG` lines (unless `--no-RG`) and `@PG` chain (unless
//! `--reject-PG`), plus a provenance `@PG` for this command (unless `--no-PG`).
//! All `@SQ` lines are dropped — the reads are unmapped, so the output references
//! nothing.
//!
//! The record path never decodes CIGAR/SEQ/QUAL into typed structures: the output
//! payload is assembled directly from the input's raw bytes (read_name copied
//! verbatim, SEQ nibbles relocated, only reverse-strand records repacked). This
//! is why it beats samtools' full `RecordBuf` decode + re-encode.
//!
//! ## Origin
//!
//! This crate is an independent Rust port of `samtools reset` based on the
//! samtools source (`reset.c`, MIT licence, commit 1.23). Specifically the FLAG
//! reduction (`reset.c:354-362`), the secondary/supplementary drop
//! (`reset.c:350-352`), the reverse-complement restore using the
//! `=TGKCYSBAWRDMHVN` table (`reset.c:377-391`), the default aux-tag removal set
//! and `--keep-tag`/`--remove-tag` interaction (`reset.c:80-159`), and the
//! rebuilt-header `@HD`/`@RG`/`@PG` logic (`reset.c:307-323`, `getRGlines`,
//! `getPGlines`). samtools source is MIT, so reading and citing it is permitted.
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: samtools <https://github.com/samtools/samtools> (MIT).

use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::sam;
use rsomics_bamio::raw;
use rsomics_common::{Result, RsomicsError};

const FLAG_PAIRED: u16 = 0x1;
const FLAG_PROPER_PAIR: u16 = 0x2;
const FLAG_UNMAP: u16 = 0x4;
const FLAG_MUNMAP: u16 = 0x8;
const FLAG_REVERSE: u16 = 0x10;
const FLAG_MREVERSE: u16 = 0x20;
const FLAG_SECONDARY: u16 = 0x100;
const FLAG_DUP: u16 = 0x400;
const FLAG_SUPPLEMENTARY: u16 = 0x800;

/// BAM record field offsets (SAMv1 §4.2), from the start of the payload.
const REF_ID: usize = 0;
const L_READ_NAME: usize = 8;
const MAPQ: usize = 9;
const N_CIGAR: usize = 12;
const FLAG: usize = 14;
const L_SEQ: usize = 16;
const NEXT_REF_ID: usize = 20;
const TLEN: usize = 28;
const FIXED_HEAD: usize = 32;

/// `seq_nt16` complement table: index by the 4-bit base code to get the
/// complemented base code's printable character. `reset.c:380`.
const SEQ_NT16_COMP_STR: &[u8; 16] = b"=TGKCYSBAWRDMHVN";

/// Forward `seq_nt16` printable table (`=ACMGRSVTWYHKDBN`), used to re-encode the
/// non-reverse path's SEQ in read order. `reset.c:388`.
const SEQ_NT16_STR: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

/// Inverse of [`SEQ_NT16_STR`]: printable base character → 4-bit code, for
/// re-packing a SEQ that was decoded to characters.
fn nt16_code(printable: u8) -> u8 {
    SEQ_NT16_STR
        .iter()
        .position(|&c| c == printable)
        .map(|p| p as u8)
        .unwrap_or(15)
}

/// The default aligner aux tags removed by `samtools reset` (`reset.c:83-84`).
const DEFAULT_REMOVE_TAGS: [[u8; 2]; 15] = [
    *b"AS", *b"CC", *b"CG", *b"CP", *b"H1", *b"H2", *b"HI", *b"H0", *b"IH", *b"MC", *b"MD", *b"MQ",
    *b"NM", *b"SA", *b"TS",
];

/// Which aux tags survive the reset, mirroring `removeauxtags` + `update_aux_conf`
/// (`reset.c:80-159`).
#[derive(Debug, Clone)]
enum AuxFilter {
    /// Remove every tag in this set. Built from `-x`/--remove-tag plus the
    /// default removal set plus RG (if `!keep_rgs`).
    Remove(Vec<[u8; 2]>),
    /// Keep only tags in this set. Built from `--keep-tag` (`-x ^STR`); RG is
    /// dropped from the keep set if `!keep_rgs`.
    Keep(Vec<[u8; 2]>),
}

impl AuxFilter {
    fn survives(&self, tag: [u8; 2]) -> bool {
        match self {
            AuxFilter::Remove(set) => !set.contains(&tag),
            AuxFilter::Keep(set) => set.contains(&tag),
        }
    }
}

/// Options for [`reset`].
#[derive(Debug, Clone, Default)]
pub struct ResetOpts {
    /// Explicit `-x`/--remove-tag tags (additive to the default set).
    pub remove_tags: Vec<[u8; 2]>,
    /// `--keep-tag` tags; when present, switches to keep-mode.
    pub keep_tags: Vec<[u8; 2]>,
    /// `--no-RG`: drop `@RG` header lines and the RG aux tag.
    pub no_rg: bool,
    /// `--no-PG`: do not add a provenance `@PG` line for this command.
    pub no_pg: bool,
    /// `--reject-PG ID`: drop the `@PG` with this ID and every line after it.
    pub reject_pg: Option<String>,
    /// `--dupflag`: keep the duplicate (`0x400`) bit instead of clearing it.
    pub keep_dupflag: bool,
}

/// Build the aux filter from the options, applying the same default-set and RG
/// rules as `update_aux_conf` (`reset.c:80-121`): keep-mode takes priority and
/// does not auto-add defaults; remove-mode unions the user tags with the default
/// set; RG joins the removal set (or leaves the keep set) only when `--no-RG`.
fn build_aux_filter(opts: &ResetOpts) -> Result<AuxFilter> {
    if !opts.keep_tags.is_empty() && !opts.remove_tags.is_empty() {
        return Err(RsomicsError::InvalidInput(
            "--keep-tag and -x/--remove-tag are mutually exclusive".to_owned(),
        ));
    }
    if !opts.keep_tags.is_empty() {
        let mut keep = opts.keep_tags.clone();
        if opts.no_rg {
            keep.retain(|t| t != b"RG");
        }
        Ok(AuxFilter::Keep(keep))
    } else {
        let mut remove = opts.remove_tags.clone();
        if opts.no_rg && !remove.contains(b"RG") {
            remove.push(*b"RG");
        }
        for t in DEFAULT_REMOVE_TAGS {
            if !remove.contains(&t) {
                remove.push(t);
            }
        }
        Ok(AuxFilter::Remove(remove))
    }
}

/// Rebuild the output SAM header text: `@HD VN:1.6`, then `@RG` lines (unless
/// `--no-RG`), then the `@PG` chain (truncated by `--reject-PG`), then a
/// provenance `@PG` for this command (unless `--no-PG`). All `@SQ` lines are
/// dropped — the output references no sequences. `reset.c:307-323`.
fn rebuild_header(input_text: &str, opts: &ResetOpts, args_cl: &str) -> String {
    let mut out: Vec<String> = vec!["@HD\tVN:1.6".to_owned()];

    if !opts.no_rg {
        for line in input_text.lines().filter(|l| l.starts_with("@RG\t")) {
            out.push(line.to_owned());
        }
    }

    let pg_lines: Vec<&str> = input_text
        .lines()
        .filter(|l| l.starts_with("@PG\t"))
        .collect();
    let kept_pg: Vec<&str> = match &opts.reject_pg {
        Some(id) => {
            let needle = format!("ID:{id}");
            pg_lines
                .into_iter()
                .take_while(|l| !l.split('\t').any(|f| f == needle))
                .collect()
        }
        None => pg_lines,
    };
    let last_pg_id = kept_pg.last().and_then(|l| {
        l.split('\t')
            .find_map(|f| f.strip_prefix("ID:").map(str::to_owned))
    });
    for line in &kept_pg {
        out.push((*line).to_owned());
    }

    if !opts.no_pg {
        let id = unique_pg_id("rsomics-bam-reset", &kept_pg);
        let mut pg = format!("@PG\tID:{id}\tPN:rsomics-bam-reset");
        if let Some(pp) = &last_pg_id {
            pg.push_str(&format!("\tPP:{pp}"));
        }
        pg.push_str(&format!("\tVN:{}\tCL:{args_cl}", env!("CARGO_PKG_VERSION")));
        out.push(pg);
    }

    let mut text = out.join("\n");
    text.push('\n');
    text
}

/// Produce a `@PG` ID not already present in `existing`, suffixing `.1`, `.2`, …
/// on collision (the htslib convention `reset.c` relies on via `sam_hdr_add_pg`).
fn unique_pg_id(base: &str, existing: &[&str]) -> String {
    let ids: Vec<&str> = existing
        .iter()
        .filter_map(|l| l.split('\t').find_map(|f| f.strip_prefix("ID:")))
        .collect();
    if !ids.contains(&base) {
        return base.to_owned();
    }
    let mut n = 1;
    loop {
        let candidate = format!("{base}.{n}");
        if !ids.contains(&candidate.as_str()) {
            return candidate;
        }
        n += 1;
    }
}

fn header_to_text(header: &sam::Header) -> Result<String> {
    let mut buf: Vec<u8> = Vec::new();
    let mut writer = sam::io::Writer::new(&mut buf);
    writer.write_header(header).map_err(RsomicsError::Io)?;
    String::from_utf8(buf)
        .map_err(|e| RsomicsError::InvalidInput(format!("header contains non-UTF-8: {e}")))
}

fn reparse_header(text: &str) -> Result<sam::Header> {
    let mut reader = sam::io::Reader::new(text.as_bytes());
    reader.read_header().map_err(RsomicsError::Io)
}

/// Build the reverted output payload from `input` into `out` (cleared first).
/// Returns `false` for secondary/supplementary records, which are dropped.
///
/// The output is assembled byte-for-byte rather than decoded: read_name is copied
/// verbatim, CIGAR is omitted (`n_cigar = 0`), the non-reverse SEQ/QUAL are
/// memcpy'd straight across (their nibble/byte layout is unchanged, only their
/// offset shifts now that CIGAR is gone), and only a reverse-strand record pays
/// for a nibble-level reverse-complement repack.
fn revert_record(
    input: &raw::RecordRef<'_>,
    filter: &AuxFilter,
    keep_dupflag: bool,
    out: &mut Vec<u8>,
) -> bool {
    let flag = input.flags();
    if flag & FLAG_SECONDARY != 0 || flag & FLAG_SUPPLEMENTARY != 0 {
        return false;
    }

    let mut new_flag = flag & !FLAG_PROPER_PAIR;
    new_flag |= FLAG_UNMAP;
    if flag & FLAG_PAIRED != 0 {
        new_flag |= FLAG_MUNMAP;
    }
    new_flag &= !FLAG_MREVERSE;
    if !keep_dupflag {
        new_flag &= !FLAG_DUP;
    }
    let reverse = flag & FLAG_REVERSE != 0;
    if reverse {
        new_flag &= !FLAG_REVERSE;
    }

    let payload = input.payload();
    let name = &payload[FIXED_HEAD..FIXED_HEAD + name_len(payload)];
    let l_seq = base_count(payload);
    let seq_in_start = FIXED_HEAD + name_len(payload) + cigar_op_count(payload) * 4;
    let seq_in = &payload[seq_in_start..seq_in_start + l_seq.div_ceil(2)];
    let qual_in_start = seq_in_start + l_seq.div_ceil(2);
    let qual_in = &payload[qual_in_start..qual_in_start + l_seq];

    out.clear();
    out.resize(FIXED_HEAD, 0);
    write_i32(out, REF_ID, -1);
    write_i32(out, REF_ID + 4, -1); // POS
    out[L_READ_NAME] = payload[L_READ_NAME];
    out[MAPQ] = 0;
    write_u16(out, N_CIGAR, 0);
    write_u16(out, FLAG, new_flag);
    write_u32(out, L_SEQ, l_seq as u32);
    write_i32(out, NEXT_REF_ID, -1);
    write_i32(out, NEXT_REF_ID + 4, -1); // PNEXT
    write_i32(out, TLEN, 0);

    out.extend_from_slice(name);

    if reverse {
        // Restore original read orientation: reverse the base order and
        // complement each base via the seq_nt16 complement table, then re-pack
        // the printable characters back into nibbles. `reset.c:379-382`.
        let mut packed = vec![0u8; l_seq.div_ceil(2)];
        for (out_i, in_i) in (0..l_seq).rev().enumerate() {
            let code = seq_nibble(seq_in, in_i);
            let comp = nt16_code(SEQ_NT16_COMP_STR[code as usize]);
            set_seq_nibble(&mut packed, out_i, comp);
        }
        out.extend_from_slice(&packed);
        out.extend(qual_in.iter().rev());
    } else {
        out.extend_from_slice(seq_in);
        out.extend_from_slice(qual_in);
    }

    let aux_in = &payload[qual_in_start + l_seq..];
    copy_filtered_aux(aux_in, filter, out);

    true
}

/// Write one assembled payload to a BGZF stream, prefixed with its `block_size`
/// (u32 LE) — the same framing as `rsomics_bamio::raw::write_record`, inlined
/// because the payload is built as a plain `Vec<u8>` here, not a `RawRecord`.
fn write_payload<W: Write>(writer: &mut W, payload: &[u8]) -> Result<()> {
    let block_size = u32::try_from(payload.len())
        .map_err(|e| RsomicsError::InvalidInput(format!("record too large: {e}")))?;
    writer
        .write_all(&block_size.to_le_bytes())
        .map_err(RsomicsError::Io)?;
    writer.write_all(payload).map_err(RsomicsError::Io)?;
    Ok(())
}

/// Append every aux field from `aux_in` that survives `filter` onto `out`.
fn copy_filtered_aux(aux_in: &[u8], filter: &AuxFilter, out: &mut Vec<u8>) {
    let mut pos = 0;
    while pos + 3 <= aux_in.len() {
        let tag = [aux_in[pos], aux_in[pos + 1]];
        let type_code = aux_in[pos + 2];
        let value_len = aux_value_len(aux_in, pos + 3, type_code);
        let field_end = pos + 3 + value_len;
        if filter.survives(tag) {
            out.extend_from_slice(&aux_in[pos..field_end]);
        }
        pos = field_end;
    }
}

fn name_len(p: &[u8]) -> usize {
    usize::from(p[L_READ_NAME])
}

fn cigar_op_count(p: &[u8]) -> usize {
    usize::from(u16::from_le_bytes([p[N_CIGAR], p[N_CIGAR + 1]]))
}

fn base_count(p: &[u8]) -> usize {
    u32::from_le_bytes(p[L_SEQ..L_SEQ + 4].try_into().unwrap()) as usize
}

fn seq_nibble(seq: &[u8], i: usize) -> u8 {
    let byte = seq[i / 2];
    if i.is_multiple_of(2) {
        byte >> 4
    } else {
        byte & 0x0f
    }
}

fn set_seq_nibble(seq: &mut [u8], i: usize, code: u8) {
    if i.is_multiple_of(2) {
        seq[i / 2] = (seq[i / 2] & 0x0f) | (code << 4);
    } else {
        seq[i / 2] = (seq[i / 2] & 0xf0) | (code & 0x0f);
    }
}

fn aux_value_len(bytes: &[u8], pos: usize, type_code: u8) -> usize {
    match type_code {
        b'A' | b'c' | b'C' => 1,
        b's' | b'S' => 2,
        b'i' | b'I' | b'f' => 4,
        b'Z' | b'H' => bytes[pos..].iter().position(|&b| b == 0).unwrap() + 1,
        b'B' => {
            let sub = bytes[pos];
            let count = u32::from_le_bytes(bytes[pos + 1..pos + 5].try_into().unwrap()) as usize;
            let elem = match sub {
                b'c' | b'C' => 1,
                b's' | b'S' => 2,
                _ => 4,
            };
            1 + 4 + count * elem
        }
        _ => panic!("malformed aux field: unknown type code {type_code}"),
    }
}

fn write_i32(bytes: &mut [u8], off: usize, v: i32) {
    bytes[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], off: usize, v: u32) {
    bytes[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn write_u16(bytes: &mut [u8], off: usize, v: u16) {
    bytes[off..off + 2].copy_from_slice(&v.to_le_bytes());
}

/// Revert aligner changes on every primary record of `input`, writing the result
/// to `output_path` (`None` = stdout). `args_cl` is the command-line text written
/// into the provenance `@PG` line. Returns the number of records written.
pub fn reset(
    input: &Path,
    output_path: Option<&Path>,
    opts: &ResetOpts,
    args_cl: &str,
    workers: NonZero<usize>,
) -> Result<u64> {
    let filter = build_aux_filter(opts)?;

    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;
    let input_text = header_to_text(&header)?;
    let out_text = rebuild_header(&input_text, opts, args_cl);
    let out_header = reparse_header(&out_text)?;

    match output_path {
        Some(path) => {
            let mut writer = rsomics_bamio::create_with_workers(path, workers)?;
            run(
                &mut reader,
                &mut writer,
                &out_header,
                &filter,
                opts.keep_dupflag,
            )
        }
        None => {
            let mut writer = bam::io::Writer::new(std::io::stdout().lock());
            run(
                &mut reader,
                &mut writer,
                &out_header,
                &filter,
                opts.keep_dupflag,
            )
        }
    }
}

fn run<W: Write>(
    reader: &mut rsomics_bamio::ParallelBamReader,
    writer: &mut bam::io::Writer<W>,
    header: &sam::Header,
    filter: &AuxFilter,
    keep_dupflag: bool,
) -> Result<u64> {
    writer.write_header(header).map_err(RsomicsError::Io)?;

    let mut out: Vec<u8> = Vec::new();
    let mut count: u64 = 0;
    let mut scanner = raw::RecordReader::new(reader.get_mut());
    while let Some(rec) = scanner.next()? {
        if revert_record(&rec, filter, keep_dupflag, &mut out) {
            write_payload(writer.get_mut(), &out)?;
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_includes_defaults_not_rg() {
        let opts = ResetOpts::default();
        let f = build_aux_filter(&opts).unwrap();
        assert!(!f.survives(*b"NM"));
        assert!(!f.survives(*b"MD"));
        assert!(!f.survives(*b"AS"));
        assert!(f.survives(*b"RG"));
        assert!(f.survives(*b"XS"));
    }

    #[test]
    fn no_rg_removes_rg() {
        let opts = ResetOpts {
            no_rg: true,
            ..Default::default()
        };
        let f = build_aux_filter(&opts).unwrap();
        assert!(!f.survives(*b"RG"));
    }

    #[test]
    fn remove_tag_adds_to_defaults() {
        let opts = ResetOpts {
            remove_tags: vec![*b"XS"],
            ..Default::default()
        };
        let f = build_aux_filter(&opts).unwrap();
        assert!(!f.survives(*b"XS"));
        assert!(!f.survives(*b"NM"));
        assert!(f.survives(*b"RG"));
    }

    #[test]
    fn keep_tag_drops_everything_else() {
        let opts = ResetOpts {
            keep_tags: vec![*b"RG"],
            ..Default::default()
        };
        let f = build_aux_filter(&opts).unwrap();
        assert!(f.survives(*b"RG"));
        assert!(!f.survives(*b"NM"));
        assert!(!f.survives(*b"XS"));
    }

    #[test]
    fn keep_tag_drops_rg_with_no_rg() {
        let opts = ResetOpts {
            keep_tags: vec![*b"RG", *b"BC"],
            no_rg: true,
            ..Default::default()
        };
        let f = build_aux_filter(&opts).unwrap();
        assert!(!f.survives(*b"RG"));
        assert!(f.survives(*b"BC"));
    }

    #[test]
    fn rebuild_header_drops_sq_keeps_rg_pg() {
        let input = "@HD\tVN:1.6\tSO:coordinate\n@SQ\tSN:chr1\tLN:100\n@RG\tID:rg1\tSM:s1\n@PG\tID:bwa\tPN:bwa\n";
        let text = rebuild_header(input, &ResetOpts::default(), "rsomics-bam-reset in.bam");
        assert!(text.starts_with("@HD\tVN:1.6\n"));
        assert!(!text.contains("@SQ"));
        assert!(text.contains("@RG\tID:rg1\tSM:s1"));
        assert!(text.contains("@PG\tID:bwa\tPN:bwa"));
        assert!(text.contains("PN:rsomics-bam-reset"));
        assert!(text.contains("PP:bwa"));
    }

    #[test]
    fn reject_pg_truncates_chain() {
        let input = "@HD\tVN:1.6\n@PG\tID:a\tPN:a\n@PG\tID:b\tPN:b\tPP:a\n@PG\tID:c\tPN:c\tPP:b\n";
        let opts = ResetOpts {
            reject_pg: Some("b".to_owned()),
            no_pg: true,
            ..Default::default()
        };
        let text = rebuild_header(input, &opts, "");
        assert!(text.contains("ID:a"));
        assert!(!text.contains("ID:b"));
        assert!(!text.contains("ID:c"));
    }

    #[test]
    fn unique_pg_id_suffixes_on_collision() {
        let existing = ["@PG\tID:rsomics-bam-reset\tPN:x"];
        assert_eq!(
            unique_pg_id("rsomics-bam-reset", &existing),
            "rsomics-bam-reset.1"
        );
    }

    #[test]
    fn nt16_roundtrip() {
        for (code, &ch) in SEQ_NT16_STR.iter().enumerate() {
            assert_eq!(nt16_code(ch), code as u8);
        }
    }
}
