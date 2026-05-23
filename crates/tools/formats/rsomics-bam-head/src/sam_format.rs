//! Direct BAM-record → SAM-text formatter.
//!
//! `samtools head -n N` is dominated by per-record SAM serialisation, not BGZF
//! inflate (the header read is instant). htslib's `sam_format1` writes fields
//! straight into a growable string with hand-tuned integer/seq/aux encoders;
//! noodles' generic alignment-record writer, decoding each field through trait
//! objects, runs ~2× slower on the same records. To beat samtools we format the
//! raw record payload ourselves, reading fixed-offset fields and the variable
//! tail (name/cigar/seq/qual/aux) directly per the BAM spec (SAMv1 §4.2) and
//! appending into a reused output buffer — no per-record allocation.
//!
//! Field offsets from the start of the payload (after the 4-byte `block_size`):
//!
//! ```text
//! refID@0 pos@4 l_read_name@8 mapq@9 bin@10 n_cigar@12 flag@14 l_seq@16
//! next_refID@20 next_pos@24 tlen@28
//! read_name(l_read_name) cigar(4*n_cigar) seq((l_seq+1)/2) qual(l_seq) aux
//! ```

use rsomics_common::{Result, RsomicsError};

const REF_ID: usize = 0;
const POS: usize = 4;
const L_READ_NAME: usize = 8;
const MAPQ: usize = 9;
const N_CIGAR: usize = 12;
const FLAG: usize = 14;
const L_SEQ: usize = 16;
const NEXT_REF_ID: usize = 20;
const NEXT_POS: usize = 24;
const TLEN: usize = 28;
const FIXED_HEAD: usize = 32;

const CIGAR_OPS: &[u8; 9] = b"MIDNSHP=X";
/// BAM 4-bit base codes 0..=15 → SAM seq chars (`=ACMGRSVTWYHKDBN`).
const SEQ_CHARS: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

/// Decoded base pair for each packed seq byte: high nibble then low nibble.
/// Lets the SEQ loop emit two ASCII bases per byte with one table read.
const SEQ_PAIR: [[u8; 2]; 256] = {
    let mut table = [[0u8; 2]; 256];
    let mut b = 0usize;
    while b < 256 {
        table[b] = [SEQ_CHARS[b >> 4], SEQ_CHARS[b & 0xf]];
        b += 1;
    }
    table
};

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(b[o..o + 2].try_into().unwrap())
}
fn i32_at(b: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

/// Append a decimal `i64` to `out` without allocating (itoa-style).
fn push_int(out: &mut Vec<u8>, mut v: i64) {
    if v == 0 {
        out.push(b'0');
        return;
    }
    if v < 0 {
        out.push(b'-');
    }
    let mut tmp = [0u8; 20];
    let mut i = tmp.len();
    let mut n = if v < 0 {
        // Avoid overflow on i64::MIN by stepping through u64.
        (v as i128).unsigned_abs() as u64
    } else {
        v as u64
    };
    let _ = &mut v;
    while n > 0 {
        i -= 1;
        tmp[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    out.extend_from_slice(&tmp[i..]);
}

/// Format one raw BAM record payload as a SAM line (with trailing `\n`) into
/// `out`. `ref_names[i]` is the i-th reference's name; `refID == -1` renders `*`.
pub fn format_record(out: &mut Vec<u8>, payload: &[u8], ref_names: &[Vec<u8>]) -> Result<()> {
    if payload.len() < FIXED_HEAD {
        return Err(RsomicsError::InvalidInput(
            "truncated BAM record".to_string(),
        ));
    }
    let ref_id = i32_at(payload, REF_ID);
    let pos = i32_at(payload, POS);
    let l_read_name = usize::from(payload[L_READ_NAME]);
    let mapq = payload[MAPQ];
    let n_cigar = usize::from(u16_at(payload, N_CIGAR));
    let flag = u16_at(payload, FLAG);
    let l_seq = usize::try_from(u32_at(payload, L_SEQ)).unwrap();
    let next_ref_id = i32_at(payload, NEXT_REF_ID);
    let next_pos = i32_at(payload, NEXT_POS);
    let tlen = i32_at(payload, TLEN);

    let name_start = FIXED_HEAD;
    let name_end = name_start + l_read_name;
    let cigar_start = name_end;
    let seq_start = cigar_start + n_cigar * 4;
    let qual_start = seq_start + l_seq.div_ceil(2);
    let aux_start = qual_start + l_seq;
    if payload.len() < aux_start {
        return Err(RsomicsError::InvalidInput(
            "truncated BAM record".to_string(),
        ));
    }

    // QNAME (name without the trailing NUL).
    out.extend_from_slice(&payload[name_start..name_end.saturating_sub(1)]);
    out.push(b'\t');

    // FLAG.
    push_int(out, i64::from(flag));
    out.push(b'\t');

    // RNAME.
    push_ref_name(out, ref_id, ref_names)?;
    out.push(b'\t');

    // POS (1-based; -1 → 0).
    push_int(out, i64::from(pos) + 1);
    out.push(b'\t');

    // MAPQ.
    push_int(out, i64::from(mapq));
    out.push(b'\t');

    // CIGAR.
    if n_cigar == 0 {
        out.push(b'*');
    } else {
        for i in 0..n_cigar {
            let raw = u32_at(payload, cigar_start + i * 4);
            push_int(out, i64::from(raw >> 4));
            out.push(CIGAR_OPS[(raw & 0xf) as usize]);
        }
    }
    out.push(b'\t');

    // RNEXT: `=` when the mate is on this record's reference, else the name.
    if next_ref_id == -1 {
        out.push(b'*');
    } else if next_ref_id == ref_id {
        out.push(b'=');
    } else {
        push_ref_name(out, next_ref_id, ref_names)?;
    }
    out.push(b'\t');

    // PNEXT.
    push_int(out, i64::from(next_pos) + 1);
    out.push(b'\t');

    // TLEN.
    push_int(out, i64::from(tlen));
    out.push(b'\t');

    // SEQ. Decode two bases per packed byte via a 256-entry lookup so the inner
    // loop is one table read + 2-byte write, not two nibble shifts and pushes.
    if l_seq == 0 {
        out.push(b'*');
    } else {
        let packed = &payload[seq_start..seq_start + l_seq.div_ceil(2)];
        let full = l_seq / 2;
        out.reserve(l_seq);
        for &byte in &packed[..full] {
            out.extend_from_slice(&SEQ_PAIR[usize::from(byte)]);
        }
        if l_seq % 2 == 1 {
            out.push(SEQ_CHARS[usize::from(packed[full] >> 4)]);
        }
    }
    out.push(b'\t');

    // QUAL: `*` when missing (first byte 0xff) or empty, else Phred+33.
    let qual = &payload[qual_start..qual_start + l_seq];
    if l_seq == 0 || qual.first() == Some(&0xff) {
        out.push(b'*');
    } else {
        for &q in qual {
            out.push(q + 33);
        }
    }

    // AUX fields.
    format_aux(out, &payload[aux_start..])?;

    out.push(b'\n');
    Ok(())
}

fn push_ref_name(out: &mut Vec<u8>, id: i32, ref_names: &[Vec<u8>]) -> Result<()> {
    if id == -1 {
        out.push(b'*');
        return Ok(());
    }
    let name = ref_names
        .get(usize::try_from(id).unwrap())
        .ok_or_else(|| RsomicsError::InvalidInput(format!("reference id {id} out of range")))?;
    out.extend_from_slice(name);
    Ok(())
}

/// Append every aux field as `\tTAG:TYPE:VALUE`, matching htslib's `sam_format1`
/// type rendering (single-letter SAM type for the integer subtypes, `B` arrays
/// comma-joined).
fn format_aux(out: &mut Vec<u8>, mut aux: &[u8]) -> Result<()> {
    while aux.len() >= 3 {
        let tag = &aux[..2];
        let type_code = aux[2];
        out.push(b'\t');
        out.extend_from_slice(tag);
        out.push(b':');
        let consumed = format_aux_value(out, type_code, &aux[3..])?;
        aux = &aux[3 + consumed..];
    }
    if !aux.is_empty() {
        return Err(RsomicsError::InvalidInput(
            "truncated BAM aux field".to_string(),
        ));
    }
    Ok(())
}

/// Render one aux value (after the type byte) and return the bytes consumed from
/// `v`. The SAM type letter is emitted first: BAM stores c/C/s/S/i/I but SAM
/// collapses all of them to `i` (htslib `sam_format1`); `A`/`f`/`Z`/`H`/`B`
/// keep their letters.
fn format_aux_value(out: &mut Vec<u8>, type_code: u8, v: &[u8]) -> Result<usize> {
    match type_code {
        b'A' => {
            out.extend_from_slice(b"A:");
            out.push(v[0]);
            Ok(1)
        }
        b'c' => {
            out.extend_from_slice(b"i:");
            push_int(out, i64::from(v[0] as i8));
            Ok(1)
        }
        b'C' => {
            out.extend_from_slice(b"i:");
            push_int(out, i64::from(v[0]));
            Ok(1)
        }
        b's' => {
            out.extend_from_slice(b"i:");
            push_int(
                out,
                i64::from(i16::from_le_bytes(v[..2].try_into().unwrap())),
            );
            Ok(2)
        }
        b'S' => {
            out.extend_from_slice(b"i:");
            push_int(
                out,
                i64::from(u16::from_le_bytes(v[..2].try_into().unwrap())),
            );
            Ok(2)
        }
        b'i' => {
            out.extend_from_slice(b"i:");
            push_int(
                out,
                i64::from(i32::from_le_bytes(v[..4].try_into().unwrap())),
            );
            Ok(4)
        }
        b'I' => {
            out.extend_from_slice(b"i:");
            push_int(
                out,
                i64::from(u32::from_le_bytes(v[..4].try_into().unwrap())),
            );
            Ok(4)
        }
        b'f' => {
            out.extend_from_slice(b"f:");
            push_float(out, f32::from_le_bytes(v[..4].try_into().unwrap()));
            Ok(4)
        }
        b'Z' => {
            out.extend_from_slice(b"Z:");
            let nul = v
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| RsomicsError::InvalidInput("unterminated Z aux".to_string()))?;
            out.extend_from_slice(&v[..nul]);
            Ok(nul + 1)
        }
        b'H' => {
            out.extend_from_slice(b"H:");
            let nul = v
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| RsomicsError::InvalidInput("unterminated H aux".to_string()))?;
            out.extend_from_slice(&v[..nul]);
            Ok(nul + 1)
        }
        b'B' => format_aux_array(out, v),
        _ => Err(RsomicsError::InvalidInput(format!(
            "unknown BAM aux type {}",
            type_code as char
        ))),
    }
}

/// `B` array: subtype byte, u32 count, then `count` elements. SAM renders it
/// `B:<subtype>,<e0>,<e1>,…` (htslib `sam_format1`).
fn format_aux_array(out: &mut Vec<u8>, v: &[u8]) -> Result<usize> {
    let subtype = v[0];
    let count = usize::try_from(u32::from_le_bytes(v[1..5].try_into().unwrap())).unwrap();
    out.push(b'B');
    out.push(b':');
    out.push(subtype);
    let elem = match subtype {
        b'c' | b'C' => 1,
        b's' | b'S' => 2,
        b'i' | b'I' | b'f' => 4,
        _ => {
            return Err(RsomicsError::InvalidInput(format!(
                "unknown B-array subtype {}",
                subtype as char
            )));
        }
    };
    let mut off = 5;
    for _ in 0..count {
        out.push(b',');
        let e = &v[off..off + elem];
        match subtype {
            b'c' => push_int(out, i64::from(e[0] as i8)),
            b'C' => push_int(out, i64::from(e[0])),
            b's' => push_int(out, i64::from(i16::from_le_bytes(e.try_into().unwrap()))),
            b'S' => push_int(out, i64::from(u16::from_le_bytes(e.try_into().unwrap()))),
            b'i' => push_int(out, i64::from(i32::from_le_bytes(e.try_into().unwrap()))),
            b'I' => push_int(out, i64::from(u32::from_le_bytes(e.try_into().unwrap()))),
            b'f' => push_float(out, f32::from_le_bytes(e.try_into().unwrap())),
            _ => unreachable!(),
        }
        off += elem;
    }
    Ok(off)
}

/// Float rendering matching htslib: `%g`. Rust's default `f32` Display differs
/// (`1` vs `1.0`); htslib uses C `printf("%g")`, so format via the `%g` rule.
fn push_float(out: &mut Vec<u8>, f: f32) {
    // htslib formats aux floats with kputd, which is "%g"-equivalent. Reproduce
    // with a small g-style formatter so values round-trip identically.
    let s = format_g(f64::from(f));
    out.extend_from_slice(s.as_bytes());
}

/// C `%g` with the default precision 6, as htslib's `kputd` emits aux floats.
///
/// The C rule (C11 §7.21.6.1): round to 6 significant figures; let X be the
/// resulting decimal exponent. If `-4 <= X < 6` use `%f` with precision
/// `6-1-X`, else use `%e` with precision `5`; then strip trailing zeros and a
/// bare decimal point. The exponent MUST be taken from the rounded value, not
/// `log10(v)` — `0.0001f` is `9.999e-5` whose `log10` floors to -5, but rounded
/// to 6 figures its exponent is -4, so C prints `0.0001`, not `1e-04`.
fn format_g(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    if !v.is_finite() {
        return if v.is_nan() {
            "nan".to_string()
        } else if v > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }

    // `%.5e` rounds to 6 sig figs and exposes the rounded exponent in `eNN`.
    let sci = format!("{v:.5e}");
    let (_, exp_str) = sci.split_once('e').unwrap();
    let exp: i32 = exp_str.parse().unwrap();

    if (-4..6).contains(&exp) {
        let decimals = (5 - exp).max(0) as usize;
        trim_g_fixed(&format!("{v:.decimals$}"))
    } else {
        trim_g_sci(&sci, exp)
    }
}

fn trim_g_fixed(s: &str) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s.to_string()
    }
}

/// Render `%e` form `1.23000e2` as C does: trim mantissa zeros, exponent with
/// sign and at least two digits (`e+06`, `e-04`).
fn trim_g_sci(sci: &str, exp: i32) -> String {
    let (mant, _) = sci.split_once('e').unwrap();
    let mant = if mant.contains('.') {
        mant.trim_end_matches('0').trim_end_matches('.')
    } else {
        mant
    };
    format!("{mant}e{exp:+03}")
}
