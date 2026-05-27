use std::num::NonZero;
use std::path::Path;

use noodles::sam::alignment::record::cigar::op::Kind;
use rsomics_common::{Result, RsomicsError};

pub struct DeletionProfile {
    pub counts: Vec<u64>,
    pub reads_used: u64,
}

/// Walk one BAM file and accumulate per-read-position deletion counts.
///
/// Only reads whose query length equals `read_length` and whose mapping quality
/// is >= `min_mapq` contribute. Each CIGAR D operation increments
/// `counts[query_pos_before_d]` by one.
pub fn compute(
    bam: &Path,
    read_length: usize,
    min_mapq: u8,
    max_reads: u64,
    workers: NonZero<usize>,
) -> Result<DeletionProfile> {
    let mut reader = rsomics_bamio::open_with_workers(bam, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;

    let mut counts = vec![0u64; read_length];
    let mut reads_used: u64 = 0;

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;

        let flags = record.flags();
        if flags.is_unmapped() || flags.is_secondary() || flags.is_supplementary() {
            continue;
        }

        if let Some(mapq) = record.mapping_quality()
            && mapq.get() < min_mapq
        {
            continue;
        }

        let seq_len = record.sequence().len();
        if seq_len != read_length {
            continue;
        }

        let cigar = record.cigar();
        let mut query_pos: usize = 0;
        let mut has_deletion = false;

        for op_result in cigar.iter() {
            let op = op_result
                .map_err(|e| RsomicsError::InvalidInput(format!("cigar parse error: {e}")))?;
            match op.kind() {
                Kind::Match
                | Kind::SequenceMatch
                | Kind::SequenceMismatch
                | Kind::Insertion
                | Kind::SoftClip => {
                    query_pos += op.len();
                }
                Kind::Deletion => {
                    if query_pos < read_length {
                        counts[query_pos] += 1;
                    }
                    has_deletion = true;
                }
                // N (intron skip) is reference-consuming but not a true deletion — skip it.
                Kind::Skip => {}
                Kind::HardClip | Kind::Pad => {}
            }
        }

        if has_deletion {
            reads_used += 1;
            if reads_used >= max_reads {
                break;
            }
        }
    }

    Ok(DeletionProfile { counts, reads_used })
}

/// Write the tab-delimited deletion profile text output.
///
/// Format: header line then one `read_position\tdeletion_count` row per
/// position (0-indexed), matching deletion_profile.py's `.deletion_profile.txt`.
pub fn write_txt(profile: &DeletionProfile, out: &mut dyn std::io::Write) -> Result<()> {
    writeln!(out, "read_position\tdeletion_count").map_err(RsomicsError::Io)?;
    for (pos, &count) in profile.counts.iter().enumerate() {
        writeln!(out, "{pos}\t{count}").map_err(RsomicsError::Io)?;
    }
    Ok(())
}

/// Write the R-script companion output.
///
/// Matches deletion_profile.py's `.deletion_profile.r` format so downstream
/// R workflows expecting that file continue to work.
pub fn write_r(
    profile: &DeletionProfile,
    pdf_path: &str,
    out: &mut dyn std::io::Write,
) -> Result<()> {
    let n = profile.counts.len();
    let pos_vec: Vec<String> = (0..n).map(|i| i.to_string()).collect();
    let val_vec: Vec<String> = profile.counts.iter().map(|v| v.to_string()).collect();

    writeln!(out, "pdf(\"{pdf_path}\")").map_err(RsomicsError::Io)?;
    writeln!(out, "pos=c({})", pos_vec.join(",")).map_err(RsomicsError::Io)?;
    writeln!(out, "value=c({})", val_vec.join(",")).map_err(RsomicsError::Io)?;
    writeln!(
        out,
        "plot(pos,value,type='b', col='blue',xlab=\"Read position (5'->3')\", ylab='Deletion count')"
    )
    .map_err(RsomicsError::Io)?;
    writeln!(out, "dev.off()").map_err(RsomicsError::Io)?;
    Ok(())
}
