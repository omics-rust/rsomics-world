//! Genotype rewriter matching `bcftools +setGT` semantics.
//!
//! Supported `-t` (target) selectors:
//!   - `.`   any missing (partial or fully missing)
//!   - `./x` partially missing (≥1 allele missing, not all)
//!   - `./.` fully missing (all alleles missing)
//!   - `a`   all genotypes
//!   - `q`   filter-expression query (not implemented — would need a full BCF filter engine)
//!
//! Supported `-n` (new-GT) forms:
//!   - `.`   set all alleles to missing
//!   - `0`   set all alleles to REF (unphased)
//!   - `p`   phase existing genotype
//!   - `u`   unphase and sort alleles ascending
//!   - `c:GT` custom literal genotype (e.g. `0/0`, `0|1`)
//!
//! The `q` (filter expression), `b` (binomial), `r` (random), `m`/`M`/`X`/`i`
//! new-GT forms, and partial-missing targeting with `./x` notation require
//! either a full BCF filter engine or FORMAT tag lookups — they are not
//! implemented. Attempting to use them exits non-zero with a clear message.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation of `bcftools +setGT`
//! (plugin in the bcftools suite) based on:
//! - Reading the MIT-licensed upstream source (setGT.c, bcftools develop branch)
//! - The public VCF 4.3 format specification
//! - Black-box behaviour testing against `bcftools +setGT`
//!
//! Source consulted: <https://github.com/samtools/bcftools/blob/develop/plugins/setGT.c>
//! License: MIT OR Apache-2.0
//! Upstream credit: bcftools <https://github.com/samtools/bcftools> (MIT)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

// ── Target selector ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Any missing genotype (partial or full): `-t .`
    AnyMissing,
    /// Partially missing (at least one, not all): `-t ./x`
    PartialMissing,
    /// Fully missing (all alleles missing): `-t ./.`
    FullyMissing,
    /// All genotypes: `-t a`
    All,
}

impl Target {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "." => Ok(Self::AnyMissing),
            "./x" => Ok(Self::PartialMissing),
            "./." => Ok(Self::FullyMissing),
            "a" => Ok(Self::All),
            "q" | "b" | "nb" | "np" | "miss" => Err(RsomicsError::InvalidInput(format!(
                "target '{s}' requires bcftools filter engine — not supported by rsomics-vcf-setgt"
            ))),
            _ => Err(RsomicsError::InvalidInput(format!(
                "unknown target selector '{s}'. Supported: ., ./x, ./., a"
            ))),
        }
    }
}

// ── New-GT form ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewGt {
    /// Set all alleles to missing (`.`).
    Missing,
    /// Set all alleles to REF (0), unphased.
    Ref,
    /// Phase the genotype (add `|` separator, keep alleles).
    Phase,
    /// Unphase and sort alleles ascending.
    Unphase,
    /// Custom literal genotype string (e.g. `0/0`, `0|1`).
    Custom(String),
}

impl NewGt {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "." => Ok(Self::Missing),
            "0" => Ok(Self::Ref),
            "p" => Ok(Self::Phase),
            "u" => Ok(Self::Unphase),
            _ if s.starts_with("c:") => Ok(Self::Custom(s[2..].to_owned())),
            "m" | "M" | "X" | "i" => Err(RsomicsError::InvalidInput(format!(
                "new-GT form '{s}' requires INFO/AC or FORMAT/AD fields — not supported by rsomics-vcf-setgt"
            ))),
            _ => Err(RsomicsError::InvalidInput(format!(
                "unknown new-GT form '{s}'. Supported: ., 0, p, u, c:GT"
            ))),
        }
    }
}

// ── GT field rewriting ────────────────────────────────────────────────────────

/// Count missing and total alleles in a GT field string (e.g. `0/./1`).
/// Returns `(n_missing, n_alleles)`.
#[inline]
fn count_missing(gt: &str) -> (usize, usize) {
    let bytes = gt.as_bytes();
    let mut n_allele = 1usize;
    let mut n_miss = 0usize;

    // Check first allele
    if bytes.first() == Some(&b'.') {
        n_miss += 1;
    }

    for &b in bytes {
        if b == b'/' || b == b'|' {
            n_allele += 1;
            // The byte immediately after the separator is the next allele's first byte.
            // We'll check it in the window-based pass below.
        }
    }

    // Re-scan to check each allele after a separator
    // Walk byte-by-byte tracking position after each separator.
    let mut after_sep = false;
    for &b in bytes.iter().skip(1) {
        if after_sep {
            if b == b'.' {
                n_miss += 1;
            }
            after_sep = false;
        }
        if b == b'/' || b == b'|' {
            after_sep = true;
        }
    }

    (n_miss, n_allele)
}

/// Determine whether a sample GT matches the target selector.
#[inline]
fn matches_target(gt: &str, target: Target) -> bool {
    match target {
        Target::All => true,
        Target::AnyMissing | Target::PartialMissing | Target::FullyMissing => {
            let (n_miss, n_allele) = count_missing(gt);
            match target {
                Target::AnyMissing => n_miss > 0,
                Target::PartialMissing => n_miss > 0 && n_miss < n_allele,
                Target::FullyMissing => n_miss == n_allele,
                Target::All => unreachable!(),
            }
        }
    }
}

/// Parse the ploidy and phasing of a GT string.
/// Returns `(ploidy, is_phased_anywhere)`.
#[inline]
fn gt_ploidy_phased(gt: &str) -> (usize, bool) {
    let bytes = gt.as_bytes();
    let mut ploidy = 1usize;
    let mut phased = false;
    for &b in bytes {
        if b == b'|' {
            phased = true;
            ploidy += 1;
        } else if b == b'/' {
            ploidy += 1;
        }
    }
    (ploidy, phased)
}

/// Rewrite a GT field string according to the new-GT form.
/// Returns the rewritten GT as a String. Allocation is per-sample only for
/// cases that actually change the GT; pass-through paths return slices cheaply
/// via the `rewrite_gt_into` variant below.
fn rewrite_gt_into(gt: &str, new_gt: &NewGt, out: &mut String) {
    match new_gt {
        NewGt::Missing => {
            // bcftools always uses '/' separator for missing GTs regardless of
            // original phasing — bcf_gt_missing encodes an unphased missing allele.
            let (ploidy, _) = gt_ploidy_phased(gt);
            for i in 0..ploidy {
                if i > 0 {
                    out.push('/');
                }
                out.push('.');
            }
        }
        NewGt::Ref => {
            let (ploidy, _) = gt_ploidy_phased(gt);
            for i in 0..ploidy {
                if i > 0 {
                    out.push('/');
                }
                out.push('0');
            }
        }
        NewGt::Phase => {
            // Replace all '/' separators with '|', leaving alleles unchanged.
            for b in gt.bytes() {
                if b == b'/' {
                    out.push('|');
                } else {
                    out.push(b as char);
                }
            }
        }
        NewGt::Unphase => {
            // Replicate bcftools unphase_gt: insertion-sort alleles by their BCF
            // integer encoding. In BCF, missing = 1, allele k = 2*(k+1). So
            // missing (1) sorts before ref (2), before alt1 (4), etc.
            // We map: missing → 0 (sorts first), allele k → k+1 (sorts after missing).
            let sep = if gt.contains('|') { '|' } else { '/' };
            // Sort key: None (missing) → 0, Some(k) → k+1.
            let mut keys: [u32; 8] = [0; 8];
            let mut is_miss: [bool; 8] = [false; 8];
            let mut n = 0usize;
            for tok in gt.split(sep) {
                if n < 8 {
                    if tok == "." {
                        keys[n] = 0;
                        is_miss[n] = true;
                    } else {
                        let v: u32 = tok.parse().unwrap_or(0);
                        keys[n] = v + 1;
                    }
                    n += 1;
                }
            }
            // Insertion sort on keys[0..n] / is_miss[0..n].
            for i in 1..n {
                let mut j = i;
                while j > 0 && keys[j - 1] > keys[j] {
                    keys.swap(j - 1, j);
                    is_miss.swap(j - 1, j);
                    j -= 1;
                }
            }
            for i in 0..n {
                if i > 0 {
                    out.push('/');
                }
                if is_miss[i] {
                    out.push('.');
                } else {
                    // keys[i] = allele_index + 1
                    let allele = keys[i] - 1;
                    let s = allele.to_string();
                    out.push_str(&s);
                }
            }
        }
        NewGt::Custom(custom) => {
            // Custom literal is written as-is, regardless of ploidy.
            out.push_str(custom);
        }
    }
}

// ── Record-level processing ───────────────────────────────────────────────────

/// Rewrite the FORMAT/GT column of one VCF data line in-place into `out`.
///
/// Finds the GT field position from FORMAT, then for each sample rewrites
/// only the GT sub-field if it matches `target`. All other FORMAT fields
/// are preserved byte-for-byte.
pub fn rewrite_record_into(line: &str, target: Target, new_gt: &NewGt, out: &mut String) {
    // VCF: CHROM POS ID REF ALT QUAL FILTER INFO FORMAT sample1 sample2 ...
    // We need columns 8 (FORMAT) and 9+ (samples).
    let bytes = line.as_bytes();

    // Find tab positions for columns 0-8.
    let mut tab_positions = [0usize; 10];
    let mut n_tabs = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\t' {
            tab_positions[n_tabs] = i;
            n_tabs += 1;
            if n_tabs == 9 {
                break;
            }
        }
    }

    // Fewer than 9 columns → no FORMAT/sample, pass through.
    if n_tabs < 8 {
        out.push_str(line);
        return;
    }

    let format_start = tab_positions[7] + 1;
    let format_end = if n_tabs >= 8 {
        tab_positions[8]
    } else {
        bytes.len()
    };
    let format_col = &line[format_start..format_end];

    // Find GT field index in FORMAT (colon-separated keys).
    let gt_idx = format_col
        .split(':')
        .position(|k| k == "GT")
        .unwrap_or(usize::MAX);

    // No GT key → pass through unchanged.
    if gt_idx == usize::MAX {
        out.push_str(line);
        return;
    }

    // Write everything up to and including FORMAT column unchanged.
    out.push_str(&line[..format_end]);

    // Process each sample column. `cursor` points into `line` at the start
    // of the next '\t' separator (or end-of-line).
    let mut cursor = format_end; // position of the '\t' before first sample
    while cursor < line.len() {
        // `cursor` is the tab separator; sample content starts at cursor+1.
        let sample_start = cursor + 1;
        let sample_end = line[sample_start..]
            .find('\t')
            .map_or(line.len(), |p| sample_start + p);
        let sample_col = &line[sample_start..sample_end];
        out.push('\t');

        // Locate the GT sub-field at gt_idx by scanning colon separators.
        let mut field_i = 0usize;
        let mut gt_field_start = 0usize;
        let mut gt_field_end = sample_col.len();
        let sbytes = sample_col.as_bytes();
        let mut prev = 0usize;
        let mut found = false;

        for (j, &b) in sbytes.iter().enumerate() {
            if b == b':' {
                if field_i == gt_idx {
                    gt_field_start = prev;
                    gt_field_end = j;
                    found = true;
                    break;
                }
                field_i += 1;
                prev = j + 1;
            }
        }
        if !found && field_i == gt_idx {
            // GT is the last (or only) sub-field.
            gt_field_start = prev;
            gt_field_end = sbytes
                .iter()
                .skip(prev)
                .position(|&b| b == b':')
                .map_or(sbytes.len(), |p| p + prev);
        }

        let gt_field = &sample_col[gt_field_start..gt_field_end];

        if matches_target(gt_field, target) {
            out.push_str(&sample_col[..gt_field_start]);
            rewrite_gt_into(gt_field, new_gt, out);
            out.push_str(&sample_col[gt_field_end..]);
        } else {
            out.push_str(sample_col);
        }

        cursor = sample_end;
    }
}

// ── I/O driver ────────────────────────────────────────────────────────────────

pub struct SetGtStats {
    pub total: u64,
    pub changed: u64,
}

/// Stream the VCF, rewriting GT fields. Single-threaded, constant RSS.
pub fn set_gt(
    input: &Path,
    output: &mut dyn Write,
    target: Target,
    new_gt: &NewGt,
) -> Result<SetGtStats> {
    let file = std::fs::File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let is_gz = {
        use std::io::Read as _;
        let mut buf = [0u8; 2];
        let mut peek = BufReader::new(&file);
        let n = peek.read(&mut buf).map_err(RsomicsError::Io)?;
        n >= 2 && buf[0] == 0x1f && buf[1] == 0x8b
    };

    let file2 = std::fs::File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut writer = BufWriter::new(output);

    if is_gz {
        let decoder = flate2::read::MultiGzDecoder::new(file2);
        stream_lines(BufReader::new(decoder), &mut writer, target, new_gt)
    } else {
        stream_lines(BufReader::new(file2), &mut writer, target, new_gt)
    }
}

fn stream_lines<R: Read, W: Write>(
    reader: BufReader<R>,
    writer: &mut W,
    target: Target,
    new_gt: &NewGt,
) -> Result<SetGtStats> {
    let mut total = 0u64;
    let mut changed = 0u64;
    let mut out_buf = String::with_capacity(4096);

    for raw in reader.lines() {
        let line = raw.map_err(RsomicsError::Io)?;

        if line.starts_with('#') {
            writer
                .write_all(line.as_bytes())
                .map_err(RsomicsError::Io)?;
            writer.write_all(b"\n").map_err(RsomicsError::Io)?;
            continue;
        }
        if line.is_empty() {
            continue;
        }

        total += 1;
        out_buf.clear();
        rewrite_record_into(&line, target, new_gt, &mut out_buf);

        if out_buf != line {
            changed += 1;
        }

        writer
            .write_all(out_buf.as_bytes())
            .map_err(RsomicsError::Io)?;
        writer.write_all(b"\n").map_err(RsomicsError::Io)?;
    }

    Ok(SetGtStats { total, changed })
}
