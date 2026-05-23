//! Byte-level CIGAR/SEQ/QUAL surgery for primer clipping, a direct port of
//! `bam_ampliconclip.c`'s `bam_trim_left` / `bam_trim_right`.
//!
//! These operate on the BAM record payload (everything after the 4-byte
//! `block_size`) exactly as the htslib `bam1_t` does: the same memcpy of the
//! 4-bit-packed SEQ halves, the same CIGAR regeneration, the same POS update.
//! Working on the raw bytes (rather than decoding to a noodles `RecordBuf` and
//! re-encoding) keeps the hot path a tight copy and matches samtools' output
//! byte-for-byte, including the `seq_nt16` packing of an odd clip offset.
//!
//! BAM payload layout (SAMv1 §4.2), offsets from the start of the payload:
//! ```text
//! refID@0 pos@4 l_read_name@8 mapq@9 bin@10 n_cigar@12 flag@14 l_seq@16
//! next_refID@20 next_pos@24 tlen@28
//! read_name(l_read_name) cigar(4*n_cigar) seq((l_seq+1)/2) qual(l_seq) aux
//! ```

const POS: usize = 4;
const L_READ_NAME: usize = 8;
const MAPQ: usize = 9;
const N_CIGAR: usize = 12;
const FLAG: usize = 14;
const L_SEQ: usize = 16;
const FIXED_HEAD: usize = 32;

/// The 0x4 FUNMAP flag bit, set when `--unmap-len` unmaps a too-short read.
const FLAG_UNMAPPED: u16 = 0x4;

// BAM CIGAR op codes (low nibble of the packed u32): M=0 I=1 D=2 N=3 S=4 H=5
// P=6 ==7 X=8.
const CIGAR_SOFT_CLIP: u32 = 4;
const CIGAR_HARD_CLIP: u32 = 5;

/// htslib `bam_cigar_type(op)`: bit 1 (`& 1`) set iff the op consumes query, bit
/// 2 (`& 2`) set iff it consumes reference. M/=/X = 3, I/S = 1, D/N = 2, H/P = 0.
fn cigar_type(op: u32) -> u32 {
    // Same packed lookup htslib uses: (0x3C1A7 >> (op << 1)) & 3.
    (0x0003_C1A7u32 >> (op << 1)) & 3
}

/// Clipping mode — `--soft-clip` (default) keeps SEQ/QUAL and records the cut as
/// leading/trailing S; `--hard-clip` removes the bases and records them as H.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clipping {
    Soft,
    Hard,
}

/// A read held as its raw BAM payload bytes, with cached field offsets for the
/// variable-length regions. Construct from a `block_size`-stripped payload.
pub struct RawBam {
    pub bytes: Vec<u8>,
}

impl RawBam {
    pub fn flags(&self) -> u16 {
        u16::from_le_bytes([self.bytes[FLAG], self.bytes[FLAG + 1]])
    }

    pub fn set_flag_bits(&mut self, bits: u16) {
        let new = self.flags() | bits;
        self.bytes[FLAG..FLAG + 2].copy_from_slice(&new.to_le_bytes());
    }

    pub fn reference_sequence_id(&self) -> i32 {
        i32::from_le_bytes(self.bytes[0..4].try_into().unwrap())
    }

    pub fn pos(&self) -> i64 {
        i64::from(i32::from_le_bytes(
            self.bytes[POS..POS + 4].try_into().unwrap(),
        ))
    }

    fn name_len(&self) -> usize {
        usize::from(self.bytes[L_READ_NAME])
    }

    fn n_cigar(&self) -> usize {
        usize::from(u16::from_le_bytes([
            self.bytes[N_CIGAR],
            self.bytes[N_CIGAR + 1],
        ]))
    }

    pub fn l_qseq(&self) -> usize {
        u32::from_le_bytes(self.bytes[L_SEQ..L_SEQ + 4].try_into().unwrap()) as usize
    }

    fn cigar_start(&self) -> usize {
        FIXED_HEAD + self.name_len()
    }

    fn seq_start(&self) -> usize {
        self.cigar_start() + self.n_cigar() * 4
    }

    fn qual_start(&self) -> usize {
        self.seq_start() + self.l_qseq().div_ceil(2)
    }

    fn aux_start(&self) -> usize {
        self.qual_start() + self.l_qseq()
    }

    /// Packed CIGAR ops as `(op, oplen)`.
    fn cigar(&self) -> Vec<(u32, u32)> {
        let start = self.cigar_start();
        (0..self.n_cigar())
            .map(|i| {
                let off = start + i * 4;
                let raw = u32::from_le_bytes(self.bytes[off..off + 4].try_into().unwrap());
                (raw & 0xf, raw >> 4)
            })
            .collect()
    }

    /// htslib `bam_endpos`: 0-based exclusive reference end. A clip-only/empty
    /// CIGAR yields `pos + 1`, matching htslib's `endpos == pos ? pos + 1`.
    pub fn endpos(&self) -> i64 {
        let span: i64 = self
            .cigar()
            .iter()
            .filter(|(op, _)| cigar_type(*op) & 2 != 0)
            .map(|(_, len)| i64::from(*len))
            .sum();
        self.pos() + if span > 0 { span } else { 1 }
    }

    /// htslib `active_query_len`: query bases consumed by non-soft-clip ops —
    /// the read's mapped length after clipping, used by the length filters.
    pub fn active_query_len(&self) -> i64 {
        self.cigar()
            .iter()
            .filter(|(op, _)| cigar_type(*op) & 1 != 0 && *op != CIGAR_SOFT_CLIP)
            .map(|(_, len)| i64::from(*len))
            .sum()
    }

    /// `--unmap-len` rebuild (samtools' `bam_set1` re-creation): clear the CIGAR
    /// (`n_cigar = 0`), zero MAPQ, set FUNMAP, and keep SEQ/QUAL/POS/tid/mate
    /// fields and the aux tail. The read stays at its mapped POS so a coordinate
    /// sort keeps it in place, matching samtools.
    pub fn unmap(&self) -> RawBam {
        let name_len = self.name_len();
        let l_qseq = self.l_qseq();
        let seq_start = self.seq_start();
        let aux_start = self.aux_start();

        let mut out = Vec::with_capacity(
            FIXED_HEAD + name_len + l_qseq.div_ceil(2) + l_qseq + self.bytes.len() - aux_start,
        );
        // Fixed core + name.
        out.extend_from_slice(&self.bytes[..FIXED_HEAD + name_len]);
        // No CIGAR. SEQ + QUAL kept verbatim.
        out.extend_from_slice(&self.bytes[seq_start..aux_start]);
        // AUX.
        out.extend_from_slice(&self.bytes[aux_start..]);

        let mut rec = RawBam { bytes: out };
        rec.bytes[N_CIGAR..N_CIGAR + 2].copy_from_slice(&0u16.to_le_bytes());
        rec.bytes[MAPQ] = 0;
        rec.set_flag_bits(FLAG_UNMAPPED);
        rec
    }

    /// Remove the aux field with `tag` (e.g. NM/MD on a clipped read, samtools'
    /// default `del_tag`). No-op if absent.
    pub fn remove_aux(&mut self, tag: [u8; 2]) {
        let start = self.aux_start();
        let mut pos = start;
        let end = self.bytes.len();
        while pos + 3 <= end {
            let field_tag = [self.bytes[pos], self.bytes[pos + 1]];
            let type_code = self.bytes[pos + 2];
            let Some(value_len) = aux_value_len(&self.bytes, pos + 3, type_code) else {
                return;
            };
            let field_end = pos + 3 + value_len;
            if field_tag == tag {
                self.bytes.drain(pos..field_end);
                return;
            }
            pos = field_end;
        }
    }
}

/// Length in bytes of an aux value (excluding the 1-byte type code) starting at
/// `pos`. `None` on a malformed/truncated field.
fn aux_value_len(bytes: &[u8], pos: usize, type_code: u8) -> Option<usize> {
    match type_code {
        b'A' | b'c' | b'C' => Some(1),
        b's' | b'S' => Some(2),
        b'i' | b'I' | b'f' => Some(4),
        b'Z' | b'H' => bytes[pos..].iter().position(|&b| b == 0).map(|n| n + 1),
        b'B' => {
            let sub = *bytes.get(pos)?;
            let count = u32::from_le_bytes(bytes.get(pos + 1..pos + 5)?.try_into().ok()?) as usize;
            let elem = match sub {
                b'c' | b'C' => 1,
                b's' | b'S' => 2,
                b'i' | b'I' | b'f' => 4,
                _ => return None,
            };
            Some(1 + 4 + count * elem)
        }
        _ => None,
    }
}

/// `bam_cigar_gen(len, op)`: pack an op/len into the 4-byte CIGAR encoding.
fn cigar_gen(len: u32, op: u32) -> u32 {
    (len << 4) | op
}

/// Build a new payload from `src`, replacing CIGAR/SEQ/QUAL/POS but keeping name
/// and the aux tail. `new_cigar` is the rebuilt op list; `qry_removed` is the
/// number of query bases physically removed (0 for soft clip); `new_pos` is the
/// new 0-based POS; `from_left` true drops the head of SEQ/QUAL (a 5'/left cut),
/// false keeps the head and drops the tail (a 3'/right cut). This mirrors the
/// SEQ/QUAL memcpy and `seq_nt16` re-pack in `bam_trim_left`/`bam_trim_right`.
fn rebuild(
    src: &RawBam,
    new_cigar: &[u32],
    qry_removed: usize,
    new_pos: i64,
    from_left: bool,
) -> RawBam {
    let name_len = src.name_len();
    let l_qseq = src.l_qseq();
    let new_l_qseq = l_qseq - qry_removed;
    let aux_start = src.aux_start();
    let aux = &src.bytes[aux_start..];

    let mut out =
        Vec::with_capacity(FIXED_HEAD + name_len + new_cigar.len() * 4 + l_qseq + aux.len());

    // Fixed core + name, copied verbatim from the source.
    out.extend_from_slice(&src.bytes[..FIXED_HEAD + name_len]);

    // New CIGAR.
    for &op in new_cigar {
        out.extend_from_slice(&op.to_le_bytes());
    }

    // SEQ: 4-bit nibbles, two bases per byte. The C copies the half-packed
    // sequence with a half-byte shift when the removed-base count is odd.
    let src_seq_start = src.seq_start();
    let src_seq = &src.bytes[src_seq_start..src_seq_start + l_qseq.div_ceil(2)];
    let mut seq_out = vec![0u8; new_l_qseq.div_ceil(2)];
    if from_left {
        copy_seq_drop_head(src_seq, &mut seq_out, l_qseq, qry_removed);
    } else {
        // Right trim keeps the head of SEQ unchanged; just copy the first
        // ceil(new_l_qseq/2) bytes (the last byte's low nibble is naturally 0
        // if new_l_qseq is odd, matching the C which leaves it as the source
        // had it — the source's odd-tail nibble is overwritten only if it held
        // a removed base, which here it does not because removed bases are at
        // the tail beyond new_l_qseq).
        seq_out.copy_from_slice(&src_seq[..new_l_qseq.div_ceil(2)]);
        // Mask the final low nibble to 0 when the new length is odd so the
        // trailing half-byte is deterministic (htslib leaves the pad nibble 0).
        if !new_l_qseq.is_multiple_of(2) {
            let last = seq_out.len() - 1;
            seq_out[last] &= 0xf0;
        }
    }
    out.extend_from_slice(&seq_out);

    // QUAL: byte-per-base. Left trim drops the head, right trim keeps the head.
    let src_qual_start = src.qual_start();
    let src_qual = &src.bytes[src_qual_start..src_qual_start + l_qseq];
    if from_left {
        out.extend_from_slice(&src_qual[qry_removed..]);
    } else {
        out.extend_from_slice(&src_qual[..new_l_qseq]);
    }

    // AUX, copied verbatim.
    out.extend_from_slice(aux);

    let mut rec = RawBam { bytes: out };
    // Patch n_cigar, l_qseq, pos.
    let n = u16::try_from(new_cigar.len()).expect("cigar op count fits u16");
    rec.bytes[N_CIGAR..N_CIGAR + 2].copy_from_slice(&n.to_le_bytes());
    let lq = u32::try_from(new_l_qseq).expect("l_qseq fits u32");
    rec.bytes[L_SEQ..L_SEQ + 4].copy_from_slice(&lq.to_le_bytes());
    let p = i32::try_from(new_pos).expect("pos fits i32");
    rec.bytes[POS..POS + 4].copy_from_slice(&p.to_le_bytes());
    rec
}

/// Copy the SEQ nibbles dropping the first `drop` bases — the odd-offset branch
/// of `bam_trim_left` (`seq_nt16` half-byte shift). `l_qseq` is the original
/// base count.
fn copy_seq_drop_head(src_seq: &[u8], out: &mut [u8], l_qseq: usize, drop: usize) {
    let new_len = l_qseq - drop;
    if drop.is_multiple_of(2) {
        // Aligned: copy whole bytes from byte `drop/2`.
        out.copy_from_slice(&src_seq[drop / 2..drop / 2 + new_len.div_ceil(2)]);
    } else {
        // Odd offset: each out byte takes the low nibble of one src byte and the
        // high nibble of the next, matching the C `((in[0] & 0x0f) << 4) |
        // ((in[1] & 0xf0) >> 4)` loop, then the tail half-byte.
        let mut in_idx = drop / 2;
        let mut out_idx = 0;
        let mut i = drop;
        while i < l_qseq - 1 {
            out[out_idx] = ((src_seq[in_idx] & 0x0f) << 4) | ((src_seq[in_idx + 1] & 0xf0) >> 4);
            in_idx += 1;
            out_idx += 1;
            i += 2;
        }
        if i < l_qseq {
            out[out_idx] = (src_seq[in_idx] & 0x0f) << 4;
        }
    }
}

/// `bam_trim_left`: remove `bases` reference bases from the 5' (left) end,
/// advancing POS over the consumed reference and rewriting the CIGAR. Returns
/// the clipped record. For hard clip with the whole read consumed, returns a
/// zero-length read (CIGAR/SEQ/QUAL emptied), matching the C special case.
pub fn trim_left(src: &RawBam, bases: u32, clipping: Clipping) -> RawBam {
    let cigar = src.cigar();
    let mut ref_remove = bases;
    let mut qry_removed: u32 = 0;
    let mut hardclip: u32 = 0;
    let mut new_pos = src.pos();
    let n = cigar.len();

    // Walk ops left→right, consuming reference until `ref_remove` is spent.
    let mut i = 0;
    while i < n {
        let (op, oplen) = cigar[i];
        let ctype = cigar_type(op);
        if op == CIGAR_HARD_CLIP {
            hardclip += oplen;
        } else {
            if ctype & 2 != 0 {
                if oplen <= ref_remove {
                    ref_remove -= oplen;
                } else {
                    break;
                }
                new_pos += i64::from(oplen);
            }
            if ctype & 1 != 0 {
                qry_removed += oplen;
            }
        }
        i += 1;
    }

    if i < n {
        // Partial consumption of op `i`: account the remaining ref_remove.
        let (op, _) = cigar[i];
        let ctype = cigar_type(op);
        if ctype & 2 != 0 {
            new_pos += i64::from(ref_remove);
        }
        if ctype & 1 != 0 {
            qry_removed += ref_remove;
        }
    } else {
        if clipping == Clipping::Hard {
            return empty_read(src);
        }
        qry_removed = src.l_qseq() as u32;
    }

    // Emit the leading clip op(s), then the (partially consumed) op `i` and the
    // rest of the CIGAR.
    let mut new_cigar: Vec<u32> = Vec::with_capacity(n + 2);
    match clipping {
        Clipping::Hard => {
            if hardclip + qry_removed > 0 {
                new_cigar.push(cigar_gen(hardclip + qry_removed, CIGAR_HARD_CLIP));
            }
        }
        Clipping::Soft => {
            if hardclip > 0 {
                new_cigar.push(cigar_gen(hardclip, CIGAR_HARD_CLIP));
            }
            if qry_removed > 0 {
                new_cigar.push(cigar_gen(qry_removed, CIGAR_SOFT_CLIP));
            }
        }
    }

    if i < n {
        let (op, oplen) = cigar[i];
        if oplen > ref_remove {
            new_cigar.push(cigar_gen(oplen - ref_remove, op));
            for &(o, l) in &cigar[i + 1..] {
                new_cigar.push(cigar_gen(l, o));
            }
        }
    }

    // Soft clip retains all SEQ/QUAL; only hard clip physically removes them.
    let phys_removed = if clipping == Clipping::Soft {
        0
    } else {
        qry_removed as usize
    };

    rebuild(src, &new_cigar, phys_removed, new_pos, true)
}

/// `bam_trim_right`: remove `bases` reference bases from the 3' (right) end,
/// rewriting the CIGAR. POS is unchanged. Returns the clipped record (zero-length
/// read for the whole-read hard-clip special case).
///
/// The CIGAR is rebuilt with the exact reverse slot-fill the C uses: a slot
/// count `j` is computed, the clip op(s) are written at the top slot(s), the
/// partially-consumed op at `--j`, and the prefix ops fill the remaining slots
/// down to 0. Reproducing the slot arithmetic (rather than building forward)
/// keeps a degenerate-CIGAR edge — where `j` hits 0 before the partial op and
/// drops it — byte-identical to samtools.
pub fn trim_right(src: &RawBam, bases: u32, clipping: Clipping) -> RawBam {
    let cigar = src.cigar();
    let mut ref_remove = bases;
    let mut qry_removed: u32 = 0;
    let mut hardclip: u32 = 0;
    let n = cigar.len() as i64;

    // Walk ops right→left, consuming reference until `ref_remove` is spent.
    let mut i: i64 = n - 1;
    while i >= 0 {
        let (op, oplen) = cigar[i as usize];
        let ctype = cigar_type(op);
        if op == CIGAR_HARD_CLIP {
            hardclip += oplen;
        } else {
            if ctype & 2 != 0 {
                if oplen <= ref_remove {
                    ref_remove -= oplen;
                } else {
                    break;
                }
            }
            if ctype & 1 != 0 {
                qry_removed += oplen;
            }
        }
        i -= 1;
    }

    // `slots` is a fixed-size scratch matching the C `new_cigar` slot writes; we
    // fill it by descending slot index `j`. `new_n_cigar` counts the writes
    // exactly as the C does (incremented at each slot write), so the emitted
    // CIGAR is `slots[..new_n_cigar]`.
    let cap = cigar.len() + 2;
    let mut slots = vec![0u32; cap];
    let mut new_n_cigar: usize = 0;
    let mut j: i64;

    if i >= 0 {
        let ctype = cigar_type(cigar[i as usize].0);
        if ctype & 1 != 0 {
            qry_removed += ref_remove;
        }
        j = i;
        if qry_removed > 0 {
            j += 1;
        }
        if hardclip > 0 && (clipping == Clipping::Soft || qry_removed == 0) {
            j += 1;
        }
    } else {
        if clipping == Clipping::Hard {
            return empty_read(src);
        }
        qry_removed = src.l_qseq() as u32;
        j = 0;
        if hardclip > 0 && clipping == Clipping::Soft {
            j += 1;
        }
    }

    if clipping == Clipping::Hard && hardclip + qry_removed > 0 {
        slots[j as usize] = cigar_gen(hardclip + qry_removed, CIGAR_HARD_CLIP);
        new_n_cigar += 1;
    }
    if clipping == Clipping::Soft {
        if hardclip > 0 {
            slots[j as usize] = cigar_gen(hardclip, CIGAR_HARD_CLIP);
            new_n_cigar += 1;
            if qry_removed > 0 {
                j -= 1;
            }
        }
        if qry_removed > 0 {
            slots[j as usize] = cigar_gen(qry_removed, CIGAR_SOFT_CLIP);
            new_n_cigar += 1;
        }
    }

    if j > 0 {
        j -= 1;
        let (op, oplen) = cigar[i as usize];
        slots[j as usize] = cigar_gen(oplen - ref_remove, op);
        new_n_cigar += 1;
    }

    // Fill the rest of the CIGAR (prefix ops) downward.
    while j > 0 {
        j -= 1;
        i -= 1;
        let (op, oplen) = cigar[i as usize];
        slots[j as usize] = cigar_gen(oplen, op);
        new_n_cigar += 1;
    }

    let new_cigar = slots[..new_n_cigar].to_vec();

    let phys_removed = if clipping == Clipping::Soft {
        0
    } else {
        qry_removed as usize
    };

    rebuild(src, &new_cigar, phys_removed, src.pos(), false)
}

/// The hard-clip whole-read special case: l_qseq = 0, n_cigar = 0, only the aux
/// tail survives. POS is left unchanged.
fn empty_read(src: &RawBam) -> RawBam {
    let name_len = src.name_len();
    let aux_start = src.aux_start();
    let aux = &src.bytes[aux_start..];
    let mut out = Vec::with_capacity(FIXED_HEAD + name_len + aux.len());
    out.extend_from_slice(&src.bytes[..FIXED_HEAD + name_len]);
    out.extend_from_slice(aux);
    let mut rec = RawBam { bytes: out };
    rec.bytes[N_CIGAR..N_CIGAR + 2].copy_from_slice(&0u16.to_le_bytes());
    rec.bytes[L_SEQ..L_SEQ + 4].copy_from_slice(&0u32.to_le_bytes());
    rec
}
