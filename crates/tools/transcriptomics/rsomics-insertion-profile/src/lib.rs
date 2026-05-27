//! Per-position CIGAR-insertion rate along the read.
//!
//! For each read cycle position (0-based, 0..read_length-1), counts how many
//! reads carry a CIGAR `I` (insertion) operation at that query position, and
//! reports the inserted and non-inserted counts per position.
//!
//! Unlike soft-clipping, insertion positions are **not mirrored** for
//! reverse-strand reads — query position `i` maps directly to table row `i`
//! for both strands, matching the upstream oracle behaviour.
//!
//! `read_length` is taken from the first passing read's sequence length.
//!
//! ## Filters applied
//!
//! - Skip unmapped reads (FLAG 0x0004).
//! - Skip QC-fail reads (FLAG 0x0200).
//! - Skip reads with MAPQ < `mapq_cut`.
//! - Secondary and supplementary reads are **not** filtered — matching
//!   upstream oracle behaviour.
//!
//! ## Output files
//!
//! SE layout:
//! - `<prefix>.insertion_profile.xls`: `Position`, `Insert_nt`, `Non_insert_nt`.
//! - `<prefix>.insertion_profile.r`: R script for plotting.
//!
//! PE layout:
//! - `<prefix>.insertion_profile.xls`: same header; `Read-1:` and `Read-2:`
//!   section labels separate the two mate tables.
//! - `<prefix>.insertion_profile.r`: R script with `r1_insert_count` and
//!   `r2_insert_count` vectors, two `pdf()` blocks (`.R1.pdf` / `.R2.pdf`).
//!
//! Number formatting matches Python `%s` on a float accumulator: `0` when
//! the count is zero, `N.0` when N > 0.
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation of `RSeQC`
//! `insertion_profile.py` based on:
//! - The published method: Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
//! - The public SAM/BAM format specification (CIGAR `I` operation)
//! - Black-box behaviour testing against `RSeQC` 5.0.4
//!   (`insertion_profile.py` — GPL-v2+; source not read; clean-room implementation)
//!
//! No source code from the GPL upstream was used as reference during
//! implementation. Test fixtures are independently generated.
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (GPL-v2+).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

const FLAG_UNMAPPED: u16 = 0x0004;
const FLAG_QCFAIL: u16 = 0x0200;

const OP_INSERTION: u8 = 1;

/// Per-position insertion profile.
pub struct InsertionProfile {
    /// Insertion count per read cycle position (length = `read_length`).
    pub insert_count: Vec<u64>,
    /// Total reads that passed quality filters.
    pub total_reads: u64,
    /// Read cycle length (from first passing read's sequence length).
    pub read_length: usize,
}

/// Sequencing layout.
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

/// Collect query positions covered by CIGAR `I` operations.
fn insertion_positions(rec: &RawRecord) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut query_pos: usize = 0;

    for (op, len) in rec.cigar_ops() {
        let len = len as usize;
        if op == OP_INSERTION {
            for i in query_pos..query_pos + len {
                positions.push(i);
            }
        }
        // Ops that consume query bases: M(0), I(1), S(4), =(7), X(8).
        match op {
            0 | 1 | 4 | 7 | 8 => query_pos += len,
            _ => {}
        }
    }

    positions
}

fn passes_filters(flags: u16, mapq: u8, mapq_cut: u8) -> bool {
    if flags & (FLAG_UNMAPPED | FLAG_QCFAIL) != 0 {
        return false;
    }
    mapq >= mapq_cut
}

/// Accumulate insertion profile for SE layout.
pub fn compute_se(
    bam_path: &Path,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<InsertionProfile> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;

    let inner = reader.get_mut();
    let mut rec = RawRecord::default();

    let mut read_length: Option<usize> = None;
    let mut insert_count: Vec<u64> = Vec::new();
    let mut total_reads: u64 = 0;

    loop {
        let n = raw::read_record(inner, &mut rec)?;
        if n == 0 {
            break;
        }

        let flags = rec.flags();
        if !passes_filters(flags, rec.mapping_quality(), mapq_cut) {
            continue;
        }

        let seq_len = rec.sequence_len();
        let rl = *read_length.get_or_insert_with(|| {
            insert_count = vec![0u64; seq_len];
            seq_len
        });

        total_reads += 1;

        for pos in insertion_positions(&rec) {
            if pos < rl {
                insert_count[pos] += 1;
            }
        }
    }

    Ok(InsertionProfile {
        insert_count,
        total_reads,
        read_length: read_length.unwrap_or(0),
    })
}

/// Accumulate insertion profiles for PE layout (read-1 and read-2 separately).
pub fn compute_pe(
    bam_path: &Path,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<(InsertionProfile, InsertionProfile)> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;

    let inner = reader.get_mut();
    let mut rec = RawRecord::default();

    let mut r1_length: Option<usize> = None;
    let mut r2_length: Option<usize> = None;
    let mut r1_insert: Vec<u64> = Vec::new();
    let mut r2_insert: Vec<u64> = Vec::new();
    let mut r1_total: u64 = 0;
    let mut r2_total: u64 = 0;

    loop {
        let n = raw::read_record(inner, &mut rec)?;
        if n == 0 {
            break;
        }

        let flags = rec.flags();
        if !passes_filters(flags, rec.mapping_quality(), mapq_cut) {
            continue;
        }

        let seq_len = rec.sequence_len();
        let is_read2 = flags & 0x0080 != 0;

        let (rl_opt, insert_vec, total) = if is_read2 {
            (&mut r2_length, &mut r2_insert, &mut r2_total)
        } else {
            (&mut r1_length, &mut r1_insert, &mut r1_total)
        };

        let rl = *rl_opt.get_or_insert_with(|| {
            *insert_vec = vec![0u64; seq_len];
            seq_len
        });

        *total += 1;

        for pos in insertion_positions(&rec) {
            if pos < rl {
                insert_vec[pos] += 1;
            }
        }
    }

    Ok((
        InsertionProfile {
            insert_count: r1_insert,
            total_reads: r1_total,
            read_length: r1_length.unwrap_or(0),
        },
        InsertionProfile {
            insert_count: r2_insert,
            total_reads: r2_total,
            read_length: r2_length.unwrap_or(0),
        },
    ))
}

/// Format a count value matching Python `%s` on float accumulators:
/// `0` when count is zero, `N.0` when N > 0.
fn fmt_count(n: u64) -> String {
    if n == 0 {
        "0".to_string()
    } else {
        format!("{n}.0")
    }
}

fn write_profile_rows(w: &mut impl Write, profile: &InsertionProfile) -> Result<()> {
    for (pos, &ins) in profile.insert_count.iter().enumerate() {
        let non_ins = profile.total_reads.saturating_sub(ins);
        writeln!(w, "{pos}\t{}\t{}", fmt_count(ins), fmt_count(non_ins))
            .map_err(RsomicsError::Io)?;
    }
    Ok(())
}

/// Write the `.insertion_profile.xls` file.
pub fn write_xls(
    out_prefix: &Path,
    layout: Layout,
    se: Option<&InsertionProfile>,
    pe: Option<(&InsertionProfile, &InsertionProfile)>,
) -> Result<()> {
    let prefix_str = out_prefix
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let dir = out_prefix.parent().unwrap_or(Path::new("."));
    let xls_path = dir.join(format!("{prefix_str}.insertion_profile.xls"));

    let f = File::create(&xls_path).map_err(RsomicsError::Io)?;
    let mut w = BufWriter::new(f);
    writeln!(w, "Position\tInsert_nt\tNon_insert_nt").map_err(RsomicsError::Io)?;

    match layout {
        Layout::Se => {
            if let Some(profile) = se {
                write_profile_rows(&mut w, profile)?;
            }
        }
        Layout::Pe => {
            if let Some((r1, r2)) = pe {
                writeln!(w, "Read-1:").map_err(RsomicsError::Io)?;
                write_profile_rows(&mut w, r1)?;
                writeln!(w, "Read-2:").map_err(RsomicsError::Io)?;
                write_profile_rows(&mut w, r2)?;
            }
        }
    }
    Ok(())
}

/// Write the `.insertion_profile.r` R script.
///
/// SE: single `pdf()` block.
/// PE: two `pdf()` blocks, `r1_insert_count` / `r2_insert_count` vectors,
/// `.R1.pdf` and `.R2.pdf` output files.
pub fn write_r_script(
    out_prefix: &Path,
    layout: Layout,
    se: Option<&InsertionProfile>,
    pe: Option<(&InsertionProfile, &InsertionProfile)>,
) -> Result<()> {
    let prefix_str = out_prefix
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let dir = out_prefix.parent().unwrap_or(Path::new("."));
    let r_path = dir.join(format!("{prefix_str}.insertion_profile.r"));

    let f = File::create(&r_path).map_err(RsomicsError::Io)?;
    let mut w = BufWriter::new(f);

    match layout {
        Layout::Se => {
            let profile = se.expect("SE profile required for R script");
            let pdf_path = dir.join(format!("{prefix_str}.insertion_profile.pdf"));
            let positions: Vec<String> = (0..profile.read_length).map(|i| i.to_string()).collect();
            let inserts: Vec<String> = profile.insert_count.iter().map(|&c| fmt_count(c)).collect();

            writeln!(w, "pdf(\"{}\")", pdf_path.display()).map_err(RsomicsError::Io)?;
            writeln!(w, "read_pos=c({})", positions.join(",")).map_err(RsomicsError::Io)?;
            writeln!(w, "insert_count=c({})", inserts.join(",")).map_err(RsomicsError::Io)?;
            writeln!(w, "noninsert_count= {} - insert_count", profile.total_reads)
                .map_err(RsomicsError::Io)?;
            writeln!(
                w,
                "plot(read_pos, insert_count*100/(insert_count+noninsert_count),col=\"blue\",main=\"Insertion profile\",xlab=\"Position of read\",ylab=\"Insertion %\",type=\"b\")"
            ).map_err(RsomicsError::Io)?;
            writeln!(w, "dev.off()").map_err(RsomicsError::Io)?;
        }
        Layout::Pe => {
            let (r1, r2) = pe.expect("PE profiles required for R script");
            let r1_pdf = dir.join(format!("{prefix_str}.insertion_profile.R1.pdf"));
            let r2_pdf = dir.join(format!("{prefix_str}.insertion_profile.R2.pdf"));

            let positions_r1: Vec<String> = (0..r1.read_length).map(|i| i.to_string()).collect();
            let r1_inserts: Vec<String> = r1.insert_count.iter().map(|&c| fmt_count(c)).collect();
            let positions_r2: Vec<String> = (0..r2.read_length).map(|i| i.to_string()).collect();
            let r2_inserts: Vec<String> = r2.insert_count.iter().map(|&c| fmt_count(c)).collect();

            writeln!(w, "pdf(\"{}\")", r1_pdf.display()).map_err(RsomicsError::Io)?;
            writeln!(w, "read_pos=c({})", positions_r1.join(",")).map_err(RsomicsError::Io)?;
            writeln!(w, "r1_insert_count=c({})", r1_inserts.join(",")).map_err(RsomicsError::Io)?;
            writeln!(
                w,
                "r1_noninsert_count = {} - r1_insert_count",
                r1.total_reads
            )
            .map_err(RsomicsError::Io)?;
            writeln!(
                w,
                "plot(read_pos, r1_insert_count*100/(r1_insert_count + r1_noninsert_count),col=\"blue\",main=\"Insertion profile\",xlab=\"Position of read (read-1)\",ylab=\"Insertion %\",type=\"b\")"
            ).map_err(RsomicsError::Io)?;
            writeln!(w, "dev.off()").map_err(RsomicsError::Io)?;

            writeln!(w, "pdf(\"{}\")", r2_pdf.display()).map_err(RsomicsError::Io)?;
            writeln!(w, "read_pos=c({})", positions_r2.join(",")).map_err(RsomicsError::Io)?;
            writeln!(w, "r2_insert_count=c({})", r2_inserts.join(",")).map_err(RsomicsError::Io)?;
            writeln!(
                w,
                "r2_noninsert_count = {} - r2_insert_count",
                r2.total_reads
            )
            .map_err(RsomicsError::Io)?;
            writeln!(
                w,
                "plot(read_pos, r2_insert_count*100/(r2_insert_count + r2_noninsert_count),col=\"blue\",main=\"Insertion profile\",xlab=\"Position of read (read-2)\",ylab=\"Insertion %\",type=\"b\")"
            ).map_err(RsomicsError::Io)?;
            writeln!(w, "dev.off()").map_err(RsomicsError::Io)?;
        }
    }
    Ok(())
}

/// Entry point: run analysis and write all output files.
pub fn run_insertion_profile(
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
            write_r_script(out_prefix, layout, Some(&profile), None)?;
        }
        Layout::Pe => {
            let (r1, r2) = compute_pe(bam_path, mapq_cut, workers)?;
            eprintln!("  Done");
            eprintln!("Totoal read-1 used: {}", r1.total_reads);
            eprintln!("Totoal read-2 used: {}", r2.total_reads);
            write_xls(out_prefix, layout, None, Some((&r1, &r2)))?;
            write_r_script(out_prefix, layout, None, Some((&r1, &r2)))?;
        }
    }
    Ok(())
}
