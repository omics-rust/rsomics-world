//! Core logic for rsomics-vcf-fixref.
//!
//! Ports the FASTA-based check and fix operations from bcftools +fixref (MIT,
//! Genome Research Ltd). The dbSNP-ID modes (`id`, `stats`) and Illumina TOP
//! conversion (`top`) require a dbSNP VCF or Illumina manifest and are out of
//! scope; they are documented but not implemented.
//!
//! # Algorithm (fixref.c, MIT)
//!
//! For each biallelic SNP record the reference base `ir` is looked up by a
//! random-access fetch from a FASTA+FAI. The VCF REF and ALT single bases are
//! encoded as `A=0 C=1 G=2 T=3`; reverse complement is `revint(x) = 3 - x`
//! (A↔T, C↔G). Four cases:
//!
//! | ref FASTA | action |
//! |-----------|--------|
//! | ir == REF | no change (`FIX_NONE`) |
//! | ir == ALT | swap REF/ALT + swap GT 0↔1 (`FIX_SWAP`) |
//! | ir == rc(REF) | flip both alleles, keep GT (`FIX_FLIP`) |
//! | ir == rc(ALT) | flip + swap REF/ALT + swap GT (`FIX_FLIP_SWAP`) |
//! | none | error (`FIX_ERR`) |
//!
//! In `flip` mode (not `flip-all`) ambiguous pairs — A/T (`pair=0x9`) and C/G
//! (`pair=0x6`) — are skipped as unresolvable without a dbSNP anchor.
//!
//! Non-SNP records (REF or ALT longer than 1 bp, or multi-allelic) and
//! non-ACGT alleles are tallied but passed through unmodified.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::items_after_statements,
    clippy::too_many_lines,    // process_line is a sequential decision tree — splitting adds no clarity
)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;

use rsomics_common::{Result, RsomicsError};

// ── Nucleotide encoding ───────────────────────────────────────────────────────

/// Sentinel for non-ACGT bases.
const NON_ACGT: u8 = u8::MAX;

/// A=0 C=1 G=2 T=3; `NON_ACGT` for anything else.
#[inline]
fn nt2int(b: u8) -> u8 {
    match b.to_ascii_uppercase() {
        b'A' => 0,
        b'C' => 1,
        b'G' => 2,
        b'T' => 3,
        _ => NON_ACGT,
    }
}

/// Integer → nucleotide byte: only valid for 0..=3.
#[inline]
fn int2nt(x: u8) -> u8 {
    b"ACGT"[x as usize]
}

/// Reverse complement in integer space: A↔T (0↔3), C↔G (1↔2).
#[inline]
fn revint(x: u8) -> u8 {
    3 - x
}

// ── FASTA index ───────────────────────────────────────────────────────────────

/// One sequence entry in a .fai file.
struct FaiEntry {
    length: u64,
    /// Byte offset of the first base in the FASTA file.
    offset: u64,
    line_bases: u64,
    line_width: u64,
}

/// Load a .fai index into a name → entry map.
fn load_fai(fai_path: &Path) -> Result<HashMap<String, FaiEntry>> {
    let reader = BufReader::new(
        File::open(fai_path)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", fai_path.display())))?,
    );
    let mut map = HashMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.splitn(5, '\t').collect();
        if cols.len() < 5 {
            return Err(RsomicsError::InvalidInput(format!(
                "malformed .fai line: {line}"
            )));
        }
        let parse_u64 = |s: &str| {
            s.parse::<u64>()
                .map_err(|e| RsomicsError::InvalidInput(format!("bad .fai field '{s}': {e}")))
        };
        map.insert(
            cols[0].to_owned(),
            FaiEntry {
                length: parse_u64(cols[1])?,
                offset: parse_u64(cols[2])?,
                line_bases: parse_u64(cols[3])?,
                line_width: parse_u64(cols[4])?,
            },
        );
    }
    Ok(map)
}

/// Load an entire contig into a contiguous byte buffer, stripping newlines.
///
/// Bases are stored as raw FASTA ASCII (uppercase); callers index directly with
/// the 0-based position. Loading once per contig eliminates per-record seek
/// syscalls — the dominant cost for chromosome-sorted VCF inputs.
fn load_contig(fasta: &mut File, entry: &FaiEntry) -> Option<Arc<Vec<u8>>> {
    // FASTA stores `line_bases` bases per line, then `line_width - line_bases`
    // bytes of newline padding.  To load the whole contig we seek to the first
    // base and read enough raw bytes to cover all lines, then strip newlines.
    let newline_bytes = entry.line_width - entry.line_bases;
    let full_lines = entry.length / entry.line_bases;
    let remainder = entry.length % entry.line_bases;
    // Total raw bytes on disk (bases + newlines), including the partial last line.
    let raw_len = full_lines * entry.line_width
        + if remainder > 0 {
            remainder + newline_bytes
        } else {
            0
        };

    fasta.seek(SeekFrom::Start(entry.offset)).ok()?;
    let mut raw = vec![0u8; raw_len as usize];
    fasta.read_exact(&mut raw).ok()?;

    // Strip newline characters; pre-allocate for exact length.
    let mut bases = Vec::with_capacity(entry.length as usize);
    for b in raw {
        if b != b'\n' && b != b'\r' {
            bases.push(b);
        }
    }
    // Truncate to declared length in case the last line had trailing whitespace.
    bases.truncate(entry.length as usize);
    Some(Arc::new(bases))
}

/// Look up a single base (0-based position) from a cached contig buffer.
///
/// Returns `None` if the position is out of range or the base is not ACGT.
#[inline]
fn base_from_cache(contig: &[u8], pos0: u64) -> Option<u8> {
    let b = *contig.get(pos0 as usize)?;
    let v = nt2int(b);
    if v == NON_ACGT { None } else { Some(v) }
}

// ── Fix mode ─────────────────────────────────────────────────────────────────

/// The operation to apply for this record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixMode {
    /// Check-only: pass through, emit stats to stderr.
    Check,
    /// Swap/flip based on single-base REF/ALT matching; skip A/T and C/G pairs.
    Flip,
    /// Same as flip but also processes ambiguous A/T and C/G pairs.
    FlipAll,
}

impl FixMode {
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "check" | "stats" => Some(Self::Check),
            "flip" => Some(Self::Flip),
            "flip-all" => Some(Self::FlipAll),
            _ => None,
        }
    }
}

// ── Per-record counters ────────────────────────────────────────────────────────

#[derive(Default, Debug)]
struct Stats {
    nsite: u64,
    nok: u64,
    nflip: u64,
    nswap: u64,
    nflip_swap: u64,
    nunresolved: u64,
    nerr: u64,
    nskip: u64,
    non_acgt: u64,
    non_snp: u64,
    non_biallelic: u64,
    /// Substitution type count indexed by `[ref_int][alt_int]`.
    count: [[u64; 4]; 4],
}

// ── VCF record processing ─────────────────────────────────────────────────────

/// Swap 0↔1 allele indices in a GT string, preserving phase.
fn swap_gt(gt: &str) -> String {
    let swap_allele = |tok: &str| -> String {
        match tok {
            "0" => "1".to_owned(),
            "1" => "0".to_owned(),
            other => other.to_owned(),
        }
    };
    if let Some(pos) = gt.find('/') {
        return format!(
            "{}/{}",
            swap_allele(&gt[..pos]),
            swap_allele(&gt[pos + 1..])
        );
    }
    if let Some(pos) = gt.find('|') {
        return format!(
            "{}|{}",
            swap_allele(&gt[..pos]),
            swap_allele(&gt[pos + 1..])
        );
    }
    swap_allele(gt)
}

/// Rewrite a FORMAT/sample block, swapping allele 0↔1 in every sample GT.
fn swap_gt_column(format_and_samples: &str) -> String {
    let mut parts = format_and_samples.splitn(2, '\t');
    let format_col = parts.next().unwrap_or(format_and_samples);
    let Some(samples_str) = parts.next() else {
        return format_and_samples.to_owned();
    };

    let gt_idx = format_col.split(':').position(|f| f == "GT");
    let Some(gt_idx) = gt_idx else {
        return format_and_samples.to_owned();
    };

    let new_samples: Vec<String> = samples_str
        .split('\t')
        .map(|sample| {
            let fields: Vec<&str> = sample.split(':').collect();
            if gt_idx < fields.len() {
                let mut owned: Vec<String> = fields.iter().map(|s| (*s).to_owned()).collect();
                owned[gt_idx] = swap_gt(fields[gt_idx]);
                owned.join(":")
            } else {
                sample.to_owned()
            }
        })
        .collect();

    format!("{format_col}\t{}", new_samples.join("\t"))
}

/// Append a `FIXREF=<action>` annotation to the INFO field of an already-modified line.
///
/// If INFO is `.` it is replaced outright; otherwise the tag is appended with `;`.
/// This matches bcftools +fixref behaviour (fixref.c `process()`).
fn annotate_info(line: &str, action: &str) -> String {
    // Split on tab to locate col[7] (INFO).
    let cols: Vec<&str> = line.splitn(9, '\t').collect();
    if cols.len() < 8 {
        return line.to_owned();
    }
    let tag = format!("FIXREF={action}");
    let new_info = if cols[7] == "." {
        tag
    } else {
        format!("{};{tag}", cols[7])
    };
    let before_info = cols[..7].join("\t");
    if cols.len() >= 9 {
        format!("{before_info}\t{new_info}\t{}", cols[8])
    } else {
        format!("{before_info}\t{new_info}")
    }
}

/// Classify and (in fix modes) rewrite a single VCF data line, updating `stats`.
///
/// Returns `(modified_line, fixref_action)`.  `fixref_action` is `None` only in
/// check mode (stats-only pass-through) or when the line is malformed.
///
/// `contig_cache` holds the sequence bytes for the contig currently loaded into
/// memory.  The caller is responsible for populating it before calling this
/// function whenever the CHROM column changes.
fn process_line(
    line: &str,
    contig_cache: Option<&[u8]>,
    mode: FixMode,
    stats: &mut Stats,
    skipped_chroms: &mut HashMap<String, bool>,
) -> (String, Option<&'static str>) {
    stats.nsite += 1;

    // VCF: CHROM POS ID REF ALT QUAL FILTER INFO [FORMAT samples…]
    let mut cols: Vec<&str> = line.splitn(9, '\t').collect();
    if cols.len() < 8 {
        stats.nerr += 1;
        return (line.to_owned(), None);
    }

    let chrom = cols[0];
    let pos_str = cols[1];
    let ref_col = cols[3];
    let alt_col = cols[4];

    if alt_col.contains(',') {
        stats.non_biallelic += 1;
        stats.nskip += 1;
        return (line.to_owned(), Some("skip"));
    }

    if ref_col.len() != 1 || alt_col.len() != 1 {
        stats.non_snp += 1;
        stats.nskip += 1;
        return (line.to_owned(), Some("skip"));
    }

    let ia = nt2int(ref_col.as_bytes()[0]);
    let ib = nt2int(alt_col.as_bytes()[0]);

    if ia == NON_ACGT || ib == NON_ACGT {
        stats.non_acgt += 1;
        stats.nskip += 1;
        return (line.to_owned(), Some("skip"));
    }

    let pos1: u64 = if let Ok(v) = pos_str.parse() {
        v
    } else {
        stats.nerr += 1;
        return (line.to_owned(), None);
    };
    let pos0 = pos1.saturating_sub(1);

    // contig_cache is None when the contig is absent from the FASTA index.
    let Some(ir) = contig_cache.and_then(|seq| base_from_cache(seq, pos0)) else {
        if !skipped_chroms.contains_key(chrom) {
            eprintln!("Ignoring sequence \"{chrom}\"");
            skipped_chroms.insert(chrom.to_owned(), true);
        }
        stats.nskip += 1;
        // contig absent from FASTA — bcftools emits skip
        return (line.to_owned(), Some("skip"));
    };

    stats.count[ia as usize][ib as usize] += 1;

    // In flip (not flip-all) mode, ambiguous pairs (A/T=0x9, C/G=0x6) are unresolvable
    // without a dbSNP anchor to determine which strand is correct.
    if mode == FixMode::Flip {
        let pair = (1u8 << ia) | (1u8 << ib);
        if pair == 0x9 || pair == 0x6 {
            stats.nunresolved += 1;
            return (line.to_owned(), Some("skip"));
        }
    }

    if mode == FixMode::Check {
        if ir == ia {
            stats.nok += 1;
        } else if ir == ib {
            stats.nswap += 1;
        } else if ir == revint(ia) {
            stats.nflip += 1;
        } else if ir == revint(ib) {
            stats.nflip_swap += 1;
        } else {
            stats.nerr += 1;
        }
        return (line.to_owned(), None);
    }

    if ir == ia {
        stats.nok += 1;
        return (line.to_owned(), Some("none"));
    }

    if ir == ib {
        // bcftools: FIX_SWAP|FIX_GT → "swap,GT"
        stats.nswap += 1;
        cols[3] = alt_col;
        cols[4] = ref_col;
        if cols.len() >= 9 {
            let new_fmt_samples = swap_gt_column(cols[8]);
            return (rebuild_line(&cols, &new_fmt_samples), Some("swap,GT"));
        }
        return (cols.join("\t"), Some("swap,GT"));
    }

    if ir == revint(ia) {
        // bcftools: FIX_FLIP → "flip" (GT unchanged, only alleles complemented)
        stats.nflip += 1;
        let ref_s = String::from(char::from(int2nt(revint(ia))));
        let alt_s = String::from(char::from(int2nt(revint(ib))));
        cols[3] = &ref_s;
        cols[4] = &alt_s;
        return (cols.join("\t"), Some("flip"));
    }

    if ir == revint(ib) {
        // bcftools: FIX_FLIP|FIX_SWAP|FIX_GT → "flip,swap,GT"
        stats.nflip_swap += 1;
        let ref_s = String::from(char::from(int2nt(revint(ib))));
        let alt_s = String::from(char::from(int2nt(revint(ia))));
        cols[3] = &ref_s;
        cols[4] = &alt_s;
        if cols.len() >= 9 {
            let new_fmt_samples = swap_gt_column(cols[8]);
            return (rebuild_line(&cols, &new_fmt_samples), Some("flip,swap,GT"));
        }
        return (cols.join("\t"), Some("flip,swap,GT"));
    }

    stats.nerr += 1;
    (line.to_owned(), Some("err"))
}

/// Rebuild cols[0..8] joined by tabs, then append the new FORMAT+samples string.
fn rebuild_line(cols: &[&str], new_fmt_samples: &str) -> String {
    let mut out = String::with_capacity(512);
    for (i, col) in cols.iter().take(8).enumerate() {
        if i > 0 {
            out.push('\t');
        }
        out.push_str(col);
    }
    out.push('\t');
    out.push_str(new_fmt_samples);
    out
}

// ── Stats output ──────────────────────────────────────────────────────────────

fn print_stats(stats: &Stats) {
    let ncmp = stats.nsite - stats.nskip;
    let nactive = ncmp;
    let tot: u64 = stats.count.iter().flatten().sum();

    eprintln!("# ST, substitution types");
    for i in 0u8..4 {
        for j in 0u8..4 {
            if i == j {
                continue;
            }
            let c = stats.count[i as usize][j as usize];
            let pct = if tot > 0 {
                c as f64 * 100.0 / tot as f64
            } else {
                0.0
            };
            eprintln!(
                "ST\t{}>{}	{c}\t{pct:.1}%",
                char::from(int2nt(i)),
                char::from(int2nt(j)),
            );
        }
    }

    let pct = |n: u64, d: u64| {
        if d > 0 {
            n as f64 * 100.0 / d as f64
        } else {
            0.0
        }
    };

    eprintln!("# NS, Number of sites:");
    eprintln!("NS\ttotal        \t{}", stats.nsite);
    eprintln!(
        "NS\tref match    \t{}\t{:.1}%",
        stats.nok,
        pct(stats.nok, ncmp)
    );
    eprintln!(
        "NS\tref mismatch \t{}\t{:.1}%",
        ncmp.saturating_sub(stats.nok),
        pct(ncmp.saturating_sub(stats.nok), ncmp)
    );
    eprintln!(
        "NS\tflipped      \t{}\t{:.1}%",
        stats.nflip,
        pct(stats.nflip, nactive)
    );
    eprintln!(
        "NS\tswapped      \t{}\t{:.1}%",
        stats.nswap,
        pct(stats.nswap, nactive)
    );
    eprintln!(
        "NS\tflip+swap    \t{}\t{:.1}%",
        stats.nflip_swap,
        pct(stats.nflip_swap, nactive)
    );
    eprintln!(
        "NS\tunresolved   \t{}\t{:.1}%",
        stats.nunresolved,
        pct(stats.nunresolved, nactive)
    );
    eprintln!("NS\terrors       \t{}", stats.nerr);
    eprintln!("NS\tskipped      \t{}", stats.nskip);
    eprintln!("NS\tnon-ACGT     \t{}", stats.non_acgt);
    eprintln!("NS\tnon-SNP      \t{}", stats.non_snp);
    eprintln!("NS\tnon-biallelic\t{}", stats.non_biallelic);
}

// ── Public entry point ────────────────────────────────────────────────────────

pub struct FixrefStats {
    pub total: u64,
    pub fixed: u64,
    pub nok: u64,
    pub nswap: u64,
    pub nflip: u64,
    pub nflip_swap: u64,
    pub nunresolved: u64,
    pub nerr: u64,
}

/// Stream a VCF through the fixref logic.
///
/// `fasta_path` must have an adjacent `.fai` index (`<fasta>.fai` or `<base>.fa.fai`).
/// Writes the (possibly modified) VCF to `output` and emits stats to stderr.
pub fn fixref(
    vcf_path: &Path,
    fasta_path: &Path,
    output: &mut dyn Write,
    mode: FixMode,
) -> Result<FixrefStats> {
    // Try "<base>.fa.fai" first, then "<base>.fai" fallback.
    let fai_path = {
        let ext = fasta_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let with_fai = fasta_path.with_extension(format!("{ext}.fai"));
        if with_fai.exists() {
            with_fai
        } else {
            let mut p = fasta_path.to_path_buf();
            p.set_extension("fai");
            p
        }
    };

    if !fai_path.exists() {
        return Err(RsomicsError::InvalidInput(format!(
            "FASTA index not found: {} — run `samtools faidx {}` first",
            fai_path.display(),
            fasta_path.display()
        )));
    }

    let index = load_fai(&fai_path)?;
    let mut fasta = File::open(fasta_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", fasta_path.display())))?;
    let vcf_file = File::open(vcf_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", vcf_path.display())))?;

    let reader = BufReader::new(vcf_file);
    let mut writer = BufWriter::new(output);
    let mut stats = Stats::default();
    let mut skipped_chroms: HashMap<String, bool> = HashMap::new();

    // Contig cache: load each chromosome's sequence once on first encounter,
    // regardless of record ordering. After the first lookup the HashMap hit is
    // O(1) with no file I/O — eliminates all per-record seek syscalls.
    let mut contig_cache: HashMap<String, Option<Arc<Vec<u8>>>> = HashMap::new();

    // Whether FIXREF annotations are emitted (check/stats mode is pass-through only).
    let annotate = mode != FixMode::Check;
    let fixref_info_header = r#"##INFO=<ID=FIXREF,Number=.,Type=String,Description="The change made by bcftools/fixref">"#;

    for raw in reader.lines() {
        let line = raw.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            // Inject the ##INFO=<ID=FIXREF,...> header immediately before the
            // #CHROM column-header line, matching bcftools output ordering.
            if annotate && line.starts_with("#CHROM") {
                writer
                    .write_all(fixref_info_header.as_bytes())
                    .map_err(RsomicsError::Io)?;
                writer.write_all(b"\n").map_err(RsomicsError::Io)?;
            }
            writer
                .write_all(line.as_bytes())
                .map_err(RsomicsError::Io)?;
            writer.write_all(b"\n").map_err(RsomicsError::Io)?;
            continue;
        }
        if line.is_empty() {
            continue;
        }

        // Extract CHROM (first tab-delimited field) and resolve its cached bytes.
        let chrom = line.split('\t').next().unwrap_or("");
        let contig_seq = contig_cache
            .entry(chrom.to_owned())
            .or_insert_with(|| index.get(chrom).and_then(|e| load_contig(&mut fasta, e)));

        let (out_line, action) = process_line(
            &line,
            contig_seq.as_deref().map(Vec::as_slice),
            mode,
            &mut stats,
            &mut skipped_chroms,
        );
        let final_line = if annotate {
            if let Some(act) = action {
                annotate_info(&out_line, act)
            } else {
                out_line
            }
        } else {
            out_line
        };
        writer
            .write_all(final_line.as_bytes())
            .map_err(RsomicsError::Io)?;
        writer.write_all(b"\n").map_err(RsomicsError::Io)?;
    }

    writer.flush().map_err(RsomicsError::Io)?;
    print_stats(&stats);

    let fixed = stats.nswap + stats.nflip + stats.nflip_swap;
    Ok(FixrefStats {
        total: stats.nsite,
        fixed,
        nok: stats.nok,
        nswap: stats.nswap,
        nflip: stats.nflip,
        nflip_swap: stats.nflip_swap,
        nunresolved: stats.nunresolved,
        nerr: stats.nerr,
    })
}
