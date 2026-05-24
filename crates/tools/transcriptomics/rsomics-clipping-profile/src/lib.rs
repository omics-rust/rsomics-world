//! Per-position soft-clipping profile from a BAM file.
//!
//! For each read cycle position (0-based, 0..read_length-1), counts how many
//! reads are soft-clipped (CIGAR 'S') at that position, and reports the
//! clipped and non-clipped counts per position.
//!
//! Read cycle positions are in **sequencing order**: for reverse-strand reads
//! (FLAG 0x10), CIGAR soft-clip positions are mirrored — position `i` in
//! read-coordinate space maps to `read_length - 1 - i` in the output table.
//!
//! `read_length` is taken from the first passing read's sequence length.
//! Reads shorter than `read_length` are counted as non-clipped at positions
//! beyond their length.
//!
//! ## Filters applied
//!
//! - Skip unmapped reads (FLAG 0x0004).
//! - Skip QC-fail reads (FLAG 0x0200).
//! - Skip reads with MAPQ < `mapq_cut`.
//! - Secondary (FLAG 0x0100) and supplementary (FLAG 0x0800) reads are
//!   **not** filtered — matching `clipping_profile.py` behaviour.
//!
//! ## Output files
//!
//! - `<prefix>.clipping_profile.xls`: tab-separated, columns
//!   `Position`, `Clipped_nt`, `Non_clipped_nt`. Zero counts are formatted as
//!   `"0"` (integer); non-zero counts as `"N.0"` (one decimal), matching
//!   Python `%s` formatting of float vs int accumulator values.
//! - `<prefix>.clipping_profile.r`: R script that reproduces the upstream
//!   plot from the same data vectors.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation of `RSeQC`
//! `clipping_profile.py` based on:
//! - The published method: Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
//! - The public SAM/BAM format specification
//! - Black-box behaviour testing against `RSeQC` 5.0.4
//!   (`clipping_profile.py` — GPL-v3; source not read; clean-room implementation)
//!
//! No source code from the GPL upstream was used as reference during
//! implementation. Test fixtures are independently generated.
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (GPL-v3).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

// SAM flag bits consulted during filtering.
const FLAG_UNMAPPED: u16 = 0x0004;
const FLAG_REVERSE: u16 = 0x0010;
const FLAG_QCFAIL: u16 = 0x0200;

// BAM CIGAR operation codes.
const OP_SOFT_CLIP: u8 = 4;

/// Clipping profile: per-position clipped count, total passing-read count, read length.
pub struct ClippingProfile {
    /// Clipped read count per read cycle position (length = `read_length`).
    pub clip_count: Vec<u64>,
    /// Total reads that passed the quality filters (used to derive non-clipped count).
    pub total_reads: u64,
    /// Read cycle length (from first passing read's sequence length).
    pub read_length: usize,
}

/// Sequencing layout for paired-end mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Se,
    Pe,
}

impl Layout {
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "SE" => Some(Self::Se),
            "PE" => Some(Self::Pe),
            _ => None,
        }
    }
}

/// Per-read soft-clip positions in read-cycle coordinates (0-based).
///
/// For a forward read, position `i` is clipped when the CIGAR places an 'S'
/// operation spanning query position `i`.
///
/// For a reverse-strand read, the CIGAR is still in read order (5'→3' of the
/// original sequencing molecule), but `clipping_profile.py` reports clips in
/// genomic/anti-sense order — effectively mirroring: position `i` maps to
/// `read_length - 1 - i` in the output table. This reversal is applied by the
/// caller after this function returns.
fn soft_clip_positions(rec: &RawRecord) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut query_pos: usize = 0;

    for (op, len) in rec.cigar_ops() {
        let len = len as usize;
        if op == OP_SOFT_CLIP {
            for i in query_pos..query_pos + len {
                positions.push(i);
            }
        }
        // Only M, I, S, =, X consume query bases (ops 0,1,4,7,8).
        // D, N, H, P do not consume query bases.
        match op {
            0 | 1 | 4 | 7 | 8 => query_pos += len,
            _ => {}
        }
    }

    positions
}

/// Scan BAM and accumulate clipping profile for SE layout.
pub fn compute_se_pub(
    bam_path: &Path,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<ClippingProfile> {
    compute_se(bam_path, mapq_cut, workers)
}

fn compute_se(bam_path: &Path, mapq_cut: u8, workers: NonZero<usize>) -> Result<ClippingProfile> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;

    let inner = reader.get_mut();
    let mut rec = RawRecord::default();

    let mut read_length: Option<usize> = None;
    let mut clip_count: Vec<u64> = Vec::new();
    let mut total_reads: u64 = 0;

    loop {
        let n = raw::read_record(inner, &mut rec)?;
        if n == 0 {
            break;
        }

        let flags = rec.flags();
        if flags & (FLAG_UNMAPPED | FLAG_QCFAIL) != 0 {
            continue;
        }
        if rec.mapping_quality() < mapq_cut {
            continue;
        }

        let seq_len = rec.sequence_len();
        let rl = *read_length.get_or_insert_with(|| {
            let len = seq_len;
            clip_count = vec![0u64; len];
            len
        });

        total_reads += 1;
        let is_reverse = flags & FLAG_REVERSE != 0;

        for pos in soft_clip_positions(&rec) {
            if pos >= rl {
                // Position beyond read_length: not counted (read is shorter than
                // the first read; positions past rl are already excluded from the table).
                continue;
            }
            let table_pos = if is_reverse { rl - 1 - pos } else { pos };
            clip_count[table_pos] += 1;
        }
    }

    let read_length = read_length.unwrap_or(0);
    if read_length == 0 {
        clip_count = Vec::new();
    }

    Ok(ClippingProfile {
        clip_count,
        total_reads,
        read_length,
    })
}

/// Scan BAM and accumulate clipping profiles for PE layout (read-1 and read-2 separately).
///
/// Returns `(read1_profile, read2_profile)`.
pub fn compute_pe(
    bam_path: &Path,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<(ClippingProfile, ClippingProfile)> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;

    let inner = reader.get_mut();
    let mut rec = RawRecord::default();

    let mut r1_length: Option<usize> = None;
    let mut r2_length: Option<usize> = None;
    let mut r1_clip: Vec<u64> = Vec::new();
    let mut r2_clip: Vec<u64> = Vec::new();
    let mut r1_total: u64 = 0;
    let mut r2_total: u64 = 0;

    loop {
        let n = raw::read_record(inner, &mut rec)?;
        if n == 0 {
            break;
        }

        let flags = rec.flags();
        if flags & (FLAG_UNMAPPED | FLAG_QCFAIL) != 0 {
            continue;
        }
        if rec.mapping_quality() < mapq_cut {
            continue;
        }

        let seq_len = rec.sequence_len();
        let is_read2 = flags & 0x0080 != 0;
        let is_reverse = flags & FLAG_REVERSE != 0;

        let (rl_opt, clip_vec, total) = if is_read2 {
            (&mut r2_length, &mut r2_clip, &mut r2_total)
        } else {
            (&mut r1_length, &mut r1_clip, &mut r1_total)
        };

        let rl = *rl_opt.get_or_insert_with(|| {
            *clip_vec = vec![0u64; seq_len];
            seq_len
        });

        *total += 1;

        for pos in soft_clip_positions(&rec) {
            if pos >= rl {
                continue;
            }
            let table_pos = if is_reverse { rl - 1 - pos } else { pos };
            clip_vec[table_pos] += 1;
        }
    }

    let r1_len = r1_length.unwrap_or(0);
    let r2_len = r2_length.unwrap_or(0);

    Ok((
        ClippingProfile {
            clip_count: r1_clip,
            total_reads: r1_total,
            read_length: r1_len,
        },
        ClippingProfile {
            clip_count: r2_clip,
            total_reads: r2_total,
            read_length: r2_len,
        },
    ))
}

/// Format a count for the `.xls` table.
///
/// Matches Python `%s` behaviour: `0` (int, no decimal) when the accumulator
/// was never incremented; `"N.0"` (float representation) when N > 0.
fn fmt_count(n: u64) -> String {
    if n == 0 {
        "0".to_string()
    } else {
        format!("{n}.0")
    }
}

/// Write a single clipping profile table to `<prefix>.<suffix>.clipping_profile.xls`.
fn write_xls_section(
    w: &mut impl Write,
    profile: &ClippingProfile,
    section_header: Option<&str>,
) -> Result<()> {
    if let Some(hdr) = section_header {
        writeln!(w, "{hdr}:").map_err(RsomicsError::Io)?;
    }
    for (pos, &clipped) in profile.clip_count.iter().enumerate() {
        let non_clipped = profile.total_reads.saturating_sub(clipped);
        writeln!(
            w,
            "{pos}\t{}\t{}",
            fmt_count(clipped),
            fmt_count(non_clipped)
        )
        .map_err(RsomicsError::Io)?;
    }
    Ok(())
}

/// Write the `.clipping_profile.xls` file.
pub fn write_xls(
    out_prefix: &Path,
    layout: Layout,
    se: Option<&ClippingProfile>,
    pe: Option<(&ClippingProfile, &ClippingProfile)>,
) -> Result<()> {
    let prefix_str = out_prefix
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let dir = out_prefix.parent().unwrap_or(Path::new("."));
    let xls_path = dir.join(format!("{prefix_str}.clipping_profile.xls"));

    let f = File::create(&xls_path).map_err(RsomicsError::Io)?;
    let mut w = BufWriter::new(f);
    writeln!(w, "Position\tClipped_nt\tNon_clipped_nt").map_err(RsomicsError::Io)?;

    match layout {
        Layout::Se => {
            if let Some(profile) = se {
                write_xls_section(&mut w, profile, None)?;
            }
        }
        Layout::Pe => {
            if let Some((r1, r2)) = pe {
                write_xls_section(&mut w, r1, Some("Read-1"))?;
                write_xls_section(&mut w, r2, Some("Read-2"))?;
            }
        }
    }
    Ok(())
}

/// Write the `.clipping_profile.r` R script.
pub fn write_r_script(out_prefix: &Path, profile: &ClippingProfile) -> Result<()> {
    let prefix_str = out_prefix
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let dir = out_prefix.parent().unwrap_or(Path::new("."));
    let r_path = dir.join(format!("{prefix_str}.clipping_profile.r"));
    let pdf_path = dir.join(format!("{prefix_str}.clipping_profile.pdf"));

    let f = File::create(&r_path).map_err(RsomicsError::Io)?;
    let mut w = BufWriter::new(f);

    let positions: Vec<String> = (0..profile.read_length).map(|i| i.to_string()).collect();
    let clips: Vec<String> = profile.clip_count.iter().map(|&c| fmt_count(c)).collect();

    writeln!(w, "pdf(\"{}\")", pdf_path.display()).map_err(RsomicsError::Io)?;
    writeln!(w, "read_pos=c({})", positions.join(",")).map_err(RsomicsError::Io)?;
    writeln!(w, "clip_count=c({})", clips.join(",")).map_err(RsomicsError::Io)?;
    writeln!(w, "nonclip_count= {} - clip_count", profile.total_reads).map_err(RsomicsError::Io)?;
    writeln!(
        w,
        "plot(read_pos, nonclip_count*100/(clip_count+nonclip_count),col=\"blue\",main=\"clipping profile\",xlab=\"Position of read\",ylab=\"Non-clipped %\",type=\"b\")"
    ).map_err(RsomicsError::Io)?;
    writeln!(w, "dev.off()").map_err(RsomicsError::Io)?;
    Ok(())
}

/// Run the full clipping-profile analysis and write output files.
pub fn run_clipping_profile(
    bam_path: &Path,
    out_prefix: &Path,
    sequencing: &str,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<()> {
    let layout = Layout::parse(sequencing).ok_or_else(|| {
        RsomicsError::InvalidInput(format!(
            "unknown sequencing layout: {sequencing:?}; expected SE or PE"
        ))
    })?;

    eprintln!("Load BAM file ...");

    match layout {
        Layout::Se => {
            let profile = compute_se(bam_path, mapq_cut, workers)?;
            eprintln!("  Done");
            eprintln!("Totoal reads used: {}", profile.total_reads);
            write_xls(out_prefix, layout, Some(&profile), None)?;
            write_r_script(out_prefix, &profile)?;
        }
        Layout::Pe => {
            let (r1, r2) = compute_pe(bam_path, mapq_cut, workers)?;
            eprintln!("  Done");
            eprintln!("Totoal read-1 used: {}", r1.total_reads);
            eprintln!("Totoal read-2 used: {}", r2.total_reads);
            // R script for PE: use read-1 profile (matching oracle behavior).
            write_xls(out_prefix, layout, None, Some((&r1, &r2)))?;
            write_r_script(out_prefix, &r1)?;
        }
    }
    Ok(())
}
