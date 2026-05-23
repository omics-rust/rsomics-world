use std::collections::HashMap;
use std::fmt;
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{RecordReader, RecordRef};
use rsomics_common::{Result, RsomicsError};

/// Finite-field prime for multiplicative hash combination, matching biobambam2.
const PRIME: u64 = (1u64 << 31) - 1;

/// Default BAM flag bits used in checksums: PAIRED | READ1 | READ2 (0x0c1 = 193).
pub const DEFAULT_FLAG_MASK: u16 = 0x0c1;
/// Default exclude flags: SECONDARY | SUPPLEMENTARY (0x900).
pub const DEFAULT_EXCL_FLAGS: u16 = 0x900;
/// Default require flags: none.
pub const DEFAULT_REQ_FLAGS: u16 = 0;
/// Default aux tags to checksum.
pub const DEFAULT_TAGS: &[&str] = &["BC", "FI", "QT", "RT", "TC"];

/// Multiplicative hash combination in GF(PRIME).
///
/// Avoids 0 and PRIME (identity/annihilator) by mapping them to 1.
/// Multiplication mod PRIME is commutative, so the fold is order-independent.
/// Compatible with biobambam2's bamseqchksum.
#[inline(always)]
fn update_hash(hash: u64, crc: u32) -> u64 {
    let mut c = u64::from(crc) & PRIME;
    if c == 0 || c == PRIME {
        c = 1;
    }
    (hash * c) % PRIME
}

/// Per-record CRC values (one per checksum category).
#[derive(Clone, Copy, Default)]
struct Crcs {
    seq: u32,
    name: u32,
    qual: u32,
    aux: u32,
}

/// Accumulated checksum sums for one read-group (or "all" / no-RG).
#[derive(Clone)]
pub struct Sums {
    pub seq: u64,
    pub name: u64,
    pub qual: u64,
    pub aux: u64,
    pub count: u64,
}

impl Sums {
    /// Initial state: multiplicative identity 1 for hash fields, 0 for count.
    fn new() -> Self {
        Sums {
            seq: 1,
            name: 1,
            qual: 1,
            aux: 1,
            count: 0,
        }
    }

    fn update(&mut self, crcs: &Crcs) {
        self.seq = update_hash(self.seq, crcs.seq);
        self.name = update_hash(self.name, crcs.name);
        self.qual = update_hash(self.qual, crcs.qual);
        self.aux = update_hash(self.aux, crcs.aux);
        self.count += 1;
    }
}

/// Checksum configuration mirroring samtools checksum's default invocation.
pub struct ChecksumOpts {
    /// Flag bits included in each per-record CRC (default 0x0c1).
    pub flag_mask: u16,
    /// Skip records with any of these flags set (default 0x900).
    pub excl_flags: u16,
    /// Skip records missing any of these flags (default 0).
    pub req_flags: u16,
    /// Reverse-complement sequences on the reverse strand (default true).
    pub rev_comp: bool,
    /// Aux tag names to include (default ["BC","FI","QT","RT","TC"]).
    pub tags: Vec<[u8; 2]>,
    /// Number of BGZF inflate workers.
    pub workers: NonZero<usize>,
}

impl Default for ChecksumOpts {
    fn default() -> Self {
        ChecksumOpts {
            flag_mask: DEFAULT_FLAG_MASK,
            excl_flags: DEFAULT_EXCL_FLAGS,
            req_flags: DEFAULT_REQ_FLAGS,
            rev_comp: true,
            tags: DEFAULT_TAGS
                .iter()
                .map(|s| [s.as_bytes()[0], s.as_bytes()[1]])
                .collect(),
            workers: NonZero::new(1).unwrap(),
        }
    }
}

/// Checksum result: per-RG and global sums.
pub struct ChecksumResult {
    pub all: Sums,
    pub no_rg: Sums,
    /// Per-read-group sums, sorted by read-group name.
    pub rg: Vec<(String, Sums)>,
    pub filename: String,
    pub flag_mask_str: String,
    pub tags_str: String,
}

/// Combined hash of one row, for the "combined" column.
///
/// Matches the C sums_report logic: seq appears twice (seq then seq again).
fn combined_hash(s: &Sums) -> u64 {
    let mut hc: u64 = 1;
    hc = update_hash(hc, (s.count >> 32) as u32);
    hc = update_hash(hc, s.count as u32);
    hc = update_hash(hc, s.seq as u32);
    hc = update_hash(hc, s.name as u32);
    hc = update_hash(hc, s.seq as u32); // seq used twice — matches C source
    hc = update_hash(hc, s.aux as u32);
    hc
}

impl fmt::Display for ChecksumResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# Checksum 1.0 for file: {}", self.filename)?;
        writeln!(f, "# Aux tags:          {}", self.tags_str)?;
        writeln!(f, "# BAM flags:         {}", self.flag_mask_str)?;
        writeln!(
            f,
            "\n# Group    QC          count  flag+seq  +name     +qual     +aux      combined"
        )?;
        write_row(f, "all", &self.all)?;
        if self.no_rg.count > 0 || self.rg.is_empty() {
            write_row(f, "-", &self.no_rg)?;
        }
        for (name, sums) in &self.rg {
            write_row(f, name, sums)?;
        }
        Ok(())
    }
}

fn write_row(f: &mut fmt::Formatter<'_>, label: &str, s: &Sums) -> fmt::Result {
    let hc = combined_hash(s);
    writeln!(
        f,
        "{:<10} {:<4} {:>12}  {:08x}  {:08x}  {:08x}  {:08x}  {:08x}",
        label, "all", s.count, s.seq, s.name, s.qual, s.aux, hc
    )
}

/// Lookup table for forward-strand pair: byte index = packed nibble byte,
/// gives two ASCII bases (high nibble first). 512 bytes = 256 pairs.
static NT16_PAIR_FWD: [[u8; 2]; 256] = {
    const FWD: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    let mut t = [[0u8; 2]; 256];
    let mut b = 0usize;
    while b < 256 {
        t[b] = [FWD[b >> 4], FWD[b & 0x0f]];
        b += 1;
    }
    t
};

/// ASCII base for each 4-bit nibble, reverse-complement strand.
const NT16_REV: &[u8; 16] = b"=TGKCYSBAWRDMHVN";

/// Offset into the raw payload where the packed seq nibbles begin.
///
/// Uses the same layout computation as `AuxIter::new` and BAM spec §4.2.
fn seq_raw_offset(rec: RecordRef<'_>) -> usize {
    let b = rec.as_bytes();
    let l_read_name = usize::from(b[8]);
    let n_cigar = usize::from(u16::from_le_bytes([b[12], b[13]]));
    32 + l_read_name + 4 * n_cigar
}

/// Expand BAM nibble-encoded sequence to ASCII, with optional rev-comp.
///
/// Processes two nibbles per packed byte on the forward strand using a
/// precomputed pair lookup, matching the inner-loop shape of htslib's
/// `fill_seq_qual`. Qual is offset by +33 for biobambam2 compatibility.
/// `BAM_FREVERSE = 0x10`.
fn fill_seq_qual(
    rec: RecordRef<'_>,
    flags: u16,
    rev_comp: bool,
    seq_buf: &mut Vec<u8>,
    qual_buf: &mut Vec<u8>,
) {
    let l = rec.sequence_len();
    seq_buf.resize(l, 0);
    qual_buf.resize(l, 0);

    let do_rev = rev_comp && (flags & 0x10 != 0);
    let raw = rec.as_bytes();
    let seq_start = seq_raw_offset(rec);
    let seq_packed = &raw[seq_start..seq_start + l.div_ceil(2)];
    // Access qual bytes directly to avoid the O(l) sentinel scan in quality_scores().
    // The qual block immediately follows the seq block (SAMv1 §4.2).
    let qual_start = seq_start + l.div_ceil(2);
    let qual_raw = &raw[qual_start..qual_start + l];
    // BAM "missing quality" sentinel: all 0xff bytes — yield empty qual.
    let qual_present = l == 0 || qual_raw[0] != 0xff;

    if do_rev {
        // Reverse-complement: nibble[i] → rev_base at position l-1-i.
        for i in 0..l {
            let byte = seq_packed[i / 2];
            let nib = if i & 1 == 0 { byte >> 4 } else { byte & 0x0f };
            seq_buf[l - 1 - i] = NT16_REV[nib as usize];
            if qual_present {
                qual_buf[l - 1 - i] = qual_raw[i].wrapping_add(33);
            }
        }
    } else {
        // Forward: process 2 bases per packed byte.
        let pairs = l / 2;
        for (k, &byte) in seq_packed[..pairs].iter().enumerate() {
            let pair = NT16_PAIR_FWD[byte as usize];
            seq_buf[k * 2] = pair[0];
            seq_buf[k * 2 + 1] = pair[1];
        }
        if l & 1 != 0 {
            // Odd-length: last packed byte has the final base in the high nibble.
            seq_buf[l - 1] = NT16_PAIR_FWD[seq_packed[pairs] as usize][0];
        }
        if qual_present {
            for i in 0..l {
                qual_buf[i] = qual_raw[i].wrapping_add(33);
            }
        }
    }
}

/// Canonicalize integer aux tag encoding to the smallest valid representation.
///
/// Matches htslib `canonical_tag`: unsigned ≥ 0 → smallest C/S/I type;
/// negative → smallest c/s/i type. Returns the (type_code, value_bytes) pair.
fn canonical_int_tag(type_code: u8, value: &[u8]) -> (u8, [u8; 4]) {
    let val: i64 = match type_code {
        b'C' => i64::from(value[0]),
        b'c' => i64::from(value[0] as i8),
        b'S' => i64::from(u16::from_le_bytes([value[0], value[1]])),
        b's' => i64::from(i16::from_le_bytes([value[0], value[1]])),
        b'I' => i64::from(u32::from_le_bytes([value[0], value[1], value[2], value[3]])),
        b'i' => i64::from(i32::from_le_bytes([value[0], value[1], value[2], value[3]])),
        _ => unreachable!(),
    };

    let (code, len): (u8, usize) = if val >= 0 {
        if val <= 255 {
            (b'C', 1)
        } else if val <= 65535 {
            (b'S', 2)
        } else {
            (b'I', 4)
        }
    } else if val >= -128 {
        (b'c', 1)
    } else if val >= -32768 {
        (b's', 2)
    } else {
        (b'i', 4)
    };

    let mut buf = [0u8; 4];
    match len {
        1 => buf[0] = val as u8,
        2 => buf[..2].copy_from_slice(&(val as i16).to_le_bytes()),
        4 => buf[..4].copy_from_slice(&(val as i32).to_le_bytes()),
        _ => unreachable!(),
    }
    (code, buf)
}

/// Iterator over raw aux fields in a BAM record payload.
///
/// Yields `(tag_2bytes, type_code, value_slice)`. Stops on malformed data.
struct AuxIter<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> AuxIter<'a> {
    fn new(rec: RecordRef<'a>) -> Self {
        let b = rec.payload();
        // Compute aux_start from SAMv1 §4.2 record layout.
        let l_read_name = usize::from(b[8]);
        let n_cigar = usize::from(u16::from_le_bytes([b[12], b[13]]));
        let l_seq = usize::try_from(u32::from_le_bytes([b[16], b[17], b[18], b[19]])).unwrap();
        let aux_start = 32 + l_read_name + 4 * n_cigar + l_seq.div_ceil(2) + l_seq;
        AuxIter {
            bytes: b,
            pos: aux_start,
        }
    }
}

impl<'a> Iterator for AuxIter<'a> {
    type Item = ([u8; 2], u8, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 3 > self.bytes.len() {
            return None;
        }
        let tag = [self.bytes[self.pos], self.bytes[self.pos + 1]];
        let type_code = self.bytes[self.pos + 2];
        let val_start = self.pos + 3;
        let val_len = aux_val_len(self.bytes, val_start, type_code)?;
        self.pos = val_start + val_len;
        Some((tag, type_code, &self.bytes[val_start..self.pos]))
    }
}

fn aux_val_len(bytes: &[u8], pos: usize, type_code: u8) -> Option<usize> {
    match type_code {
        b'A' | b'c' | b'C' => Some(1),
        b's' | b'S' => Some(2),
        b'i' | b'I' | b'f' => Some(4),
        b'Z' | b'H' => Some(bytes[pos..].iter().position(|&b| b == 0)? + 1),
        b'B' => {
            let sub = *bytes.get(pos)?;
            let n = u32::from_le_bytes(bytes.get(pos + 1..pos + 5)?.try_into().ok()?) as usize;
            let w = match sub {
                b'c' | b'C' => 1,
                b's' | b'S' => 2,
                b'i' | b'I' | b'f' => 4,
                _ => return None,
            };
            Some(1 + 4 + n * w)
        }
        _ => None,
    }
}

/// Maximum number of aux tags supported. 16 covers all practical cases.
const MAX_TAGS: usize = 16;

/// Build the aux CRC for one record.
///
/// Concatenates selected tags in the requested order (missing tags are skipped),
/// then chains the CRC from `crc_seq`. Also extracts the RG:Z: value if present.
fn hash_aux_tags(
    rec: RecordRef<'_>,
    tags: &[[u8; 2]],
    crc_seq: u32,
    rg_out: &mut Option<Vec<u8>>,
    aux_buf: &mut Vec<u8>,
) -> u32 {
    let ntags = tags.len().min(MAX_TAGS);

    // Per-slot storage: (type_code, value_start, value_end) into aux_raw, or absent.
    // We store into aux_buf directly to avoid a second allocation.
    // Instead: collect (tag_idx, start, end) offsets into a small stack array.
    #[derive(Copy, Clone)]
    struct Slot {
        code: u8,
        start: u16,
        end: u16,
    }
    let mut slots = [None::<Slot>; MAX_TAGS];

    // aux_buf will accumulate canonicalized tag bytes for found slots.
    // We write into it in two passes: first scan (building slots array), then
    // reconstruct in tag order. To avoid a second heap buffer, we write each
    // found tag's canonicalized bytes into aux_buf immediately and record ranges.
    aux_buf.clear();

    for (field_tag, type_code, value) in AuxIter::new(rec) {
        if field_tag == [b'R', b'G'] && type_code == b'Z' {
            let s = value.strip_suffix(&[0]).unwrap_or(value);
            *rg_out = Some(s.to_vec());
        }
        if let Some(idx) = tags[..ntags].iter().position(|&t| t == field_tag) {
            let start = aux_buf.len() as u16;
            match type_code {
                b'C' | b'c' | b'S' | b's' | b'I' | b'i' => {
                    let (code, buf) = canonical_int_tag(type_code, value);
                    let len = match code {
                        b'C' | b'c' => 1,
                        b'S' | b's' => 2,
                        _ => 4,
                    };
                    aux_buf.extend_from_slice(&buf[..len]);
                    let end = aux_buf.len() as u16;
                    slots[idx] = Some(Slot { code, start, end });
                }
                _ => {
                    aux_buf.extend_from_slice(value);
                    let end = aux_buf.len() as u16;
                    slots[idx] = Some(Slot {
                        code: type_code,
                        start,
                        end,
                    });
                }
            }
        }
    }

    // Build final tag sequence: tag bytes in requested order.
    // We use a second pass through slots to emit: tag[0] tag[1] code value...
    // To avoid an extra allocation we reuse the hasher directly.
    let mut h = crc32fast::Hasher::new_with_initial(crc_seq);
    for (i, slot) in slots[..ntags].iter().enumerate() {
        if let Some(s) = slot {
            h.update(&[tags[i][0], tags[i][1], s.code]);
            h.update(&aux_buf[s.start as usize..s.end as usize]);
        }
    }
    h.finalize()
}

/// Compute per-record CRCs.
///
/// Returns `None` for filtered records.
fn record_crcs(
    rec: RecordRef<'_>,
    opts: &ChecksumOpts,
    seq_buf: &mut Vec<u8>,
    qual_buf: &mut Vec<u8>,
    aux_buf: &mut Vec<u8>,
) -> Option<(Crcs, Option<Vec<u8>>)> {
    let flags = rec.flags();

    if flags & opts.excl_flags != 0 {
        return None;
    }
    if flags & opts.req_flags != opts.req_flags {
        return None;
    }

    let masked = (flags & opts.flag_mask) as u8;

    fill_seq_qual(rec, flags, opts.rev_comp, seq_buf, qual_buf);

    // CRC: flag + seq (feed flag byte first, then seq in one continuation).
    let crc_seq = {
        let mut h = crc32fast::Hasher::new_with_initial(0);
        h.update(&[masked]);
        h.update(seq_buf);
        h.finalize()
    };

    // CRC: name + flag + seq.
    // Name includes one NUL terminator (matches C: l_qname - l_extranul).
    let name = rec.name();
    let crc_name = {
        let mut h = crc32fast::Hasher::new_with_initial(0);
        h.update(name);
        h.update(&[0u8]); // one NUL after name
        h.update(&[masked]);
        h.update(seq_buf);
        h.finalize()
    };

    // CRC: flag + seq + qual
    let crc_qual = {
        let mut h = crc32fast::Hasher::new_with_initial(crc_seq);
        h.update(qual_buf);
        h.finalize()
    };

    // CRC: flag + seq + selected aux tags
    let mut rg: Option<Vec<u8>> = None;
    let crc_aux = hash_aux_tags(rec, &opts.tags, crc_seq, &mut rg, aux_buf);

    Some((
        Crcs {
            seq: crc_seq,
            name: crc_name,
            qual: crc_qual,
            aux: crc_aux,
        },
        rg,
    ))
}

/// Run the checksum over a BAM file.
pub fn run_checksum(path: &Path, opts: &ChecksumOpts) -> Result<ChecksumResult> {
    let filename = path.display().to_string();

    let tags_str = opts
        .tags
        .iter()
        .map(|t| std::str::from_utf8(t).unwrap_or("??"))
        .collect::<Vec<_>>()
        .join(",");

    let flag_mask_str = flag_mask_to_str(opts.flag_mask);

    let mut reader = rsomics_bamio::open_with_workers(path, opts.workers)?;
    reader.read_header().map_err(|e| {
        RsomicsError::InvalidInput(format!("reading header from {}: {e}", path.display()))
    })?;

    let mut all = Sums::new();
    let mut no_rg = Sums::new();
    let mut rg_map: HashMap<Vec<u8>, Sums> = HashMap::new();

    // The CRC fold is order-independent (multiplication in GF(PRIME)) and pure
    // read+hash, so it runs serially regardless of worker count: BGZF inflate is
    // the only parallel axis worth spending here, and a second pool for the CRC
    // would just contend for cores. `RecordReader` borrows each record straight
    // out of the inflated block buffer (no per-record alloc or copy), which is
    // what the hash-bound single-thread path was losing to before.
    let mut seq_buf = Vec::new();
    let mut qual_buf = Vec::new();
    let mut aux_buf = Vec::new();
    let mut scanner = RecordReader::new(reader.get_mut());
    while let Some(rec) = scanner.next().map_err(|e| {
        RsomicsError::InvalidInput(format!("reading record from {}: {e}", path.display()))
    })? {
        if let Some((crcs, rg_key)) =
            record_crcs(rec, opts, &mut seq_buf, &mut qual_buf, &mut aux_buf)
        {
            all.update(&crcs);
            match rg_key {
                Some(key) => rg_map.entry(key).or_insert_with(Sums::new).update(&crcs),
                None => no_rg.update(&crcs),
            }
        }
    }

    let mut rg: Vec<(String, Sums)> = rg_map
        .into_iter()
        .map(|(k, v)| (String::from_utf8_lossy(&k).into_owned(), v))
        .collect();
    rg.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(ChecksumResult {
        all,
        no_rg,
        rg,
        filename,
        flag_mask_str,
        tags_str,
    })
}

/// Convert BAM flag mask bits to the samtools flag-name string.
fn flag_mask_to_str(mask: u16) -> String {
    const FLAG_NAMES: &[(u16, &str)] = &[
        (0x001, "PAIRED"),
        (0x002, "PROPER_PAIR"),
        (0x004, "UNMAP"),
        (0x008, "MUNMAP"),
        (0x010, "REVERSE"),
        (0x020, "MREVERSE"),
        (0x040, "READ1"),
        (0x080, "READ2"),
        (0x100, "SECONDARY"),
        (0x200, "QCFAIL"),
        (0x400, "DUP"),
        (0x800, "SUPPLEMENTARY"),
    ];
    let parts: Vec<&str> = FLAG_NAMES
        .iter()
        .filter(|&&(bit, _)| mask & bit != 0)
        .map(|&(_, name)| name)
        .collect();
    if parts.is_empty() {
        "0".to_owned()
    } else {
        parts.join(",")
    }
}
