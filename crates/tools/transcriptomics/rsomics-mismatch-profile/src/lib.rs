use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use noodles::sam::alignment::record::Cigar;
use noodles::sam::alignment::record::cigar::op::Kind;
use noodles::sam::alignment::record::data::field::Tag;
use rsomics_bamio::open_with_workers;
use rsomics_common::{Result, RsomicsError};

/// Counts of each mismatch type at each read position.
///
/// Positions are 0-based across the read.  `read_len` is the declared alignment
/// length supplied by the caller (matching the upstream `-l` flag).
pub struct MismatchProfile {
    /// read_len × 12 counts: [A2C, A2G, A2T, C2A, C2G, C2T, G2A, G2C, G2T, T2A, T2C, T2G]
    counts: Vec<[u32; 12]>,
    pub reads_used: u64,
}

impl MismatchProfile {
    fn new(read_len: usize) -> Self {
        Self {
            counts: vec![[0u32; 12]; read_len],
            reads_used: 0,
        }
    }
}

/// Map a ref→read substitution to one of the 12 canonical mismatch indices.
///
/// Returns `None` for non-standard bases or identity.
/// Index layout: A2C=0, A2G=1, A2T=2, C2A=3, C2G=4, C2T=5, G2A=6, G2C=7, G2T=8, T2A=9, T2C=10, T2G=11
#[inline]
fn mismatch_index(ref_base: u8, read_base: u8) -> Option<usize> {
    let r = match ref_base.to_ascii_uppercase() {
        b'A' => 0usize,
        b'C' => 1,
        b'G' => 2,
        b'T' => 3,
        _ => return None,
    };
    let c = match read_base.to_ascii_uppercase() {
        b'A' => 0usize,
        b'C' => 1,
        b'G' => 2,
        b'T' => 3,
        _ => return None,
    };
    if r == c {
        return None;
    }
    let col_in_row = if c < r { c } else { c - 1 };
    Some(r * 3 + col_in_row)
}

/// Parse an MD string and accumulate mismatch counts into `counts[pos][type_idx]`.
///
/// `cigar_soft_lead` is the number of soft-clipped bases before the aligned
/// region in the SEQ field; it offsets the MD walk so profile positions are
/// relative to the aligned portion of the read.
fn accumulate_md(
    md: &[u8],
    seq: &[u8],
    cigar_soft_lead: usize,
    read_len: usize,
    counts: &mut [[u32; 12]],
) {
    let mut read_pos: usize = cigar_soft_lead;
    let mut i = 0;
    while i < md.len() {
        let b = md[i];
        if b.is_ascii_digit() {
            let mut n: usize = (b - b'0') as usize;
            i += 1;
            while i < md.len() && md[i].is_ascii_digit() {
                n = n * 10 + (md[i] - b'0') as usize;
                i += 1;
            }
            read_pos += n;
        } else if b == b'^' {
            i += 1;
            while i < md.len() && md[i].is_ascii_alphabetic() {
                i += 1;
            }
        } else if b.is_ascii_alphabetic() {
            if read_pos < seq.len() {
                let ref_base = b;
                let read_base = seq[read_pos];
                let profile_pos = read_pos.saturating_sub(cigar_soft_lead);
                if profile_pos < read_len
                    && let Some(idx) = mismatch_index(ref_base, read_base)
                {
                    counts[profile_pos][idx] += 1;
                }
            }
            read_pos += 1;
            i += 1;
        } else {
            i += 1;
        }
    }
}

/// Count soft-clipped bases at the start of a CIGAR string.
fn leading_soft_clip(cigar: &dyn Cigar) -> usize {
    let mut clip = 0usize;
    for result in cigar.iter() {
        match result {
            Ok(op) => match op.kind() {
                Kind::SoftClip => {
                    clip += op.len();
                    break;
                }
                Kind::HardClip => {}
                _ => break,
            },
            Err(_) => break,
        }
    }
    clip
}

pub struct ProfileOpts {
    pub read_len: usize,
    pub max_reads: u64,
    pub min_mapq: u8,
    pub threads: usize,
}

pub fn compute_profile(input: &Path, opts: &ProfileOpts) -> Result<MismatchProfile> {
    let workers = NonZero::new(opts.threads).unwrap_or(NonZero::<usize>::MIN);
    let mut reader = open_with_workers(input, workers)?;
    let _header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut profile = MismatchProfile::new(opts.read_len);
    let md_tag = Tag::MISMATCHED_POSITIONS;

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();

        if flags.is_unmapped() || flags.is_secondary() || flags.is_supplementary() {
            continue;
        }

        let mapq = record.mapping_quality().map_or(0, |q| q.get());
        if mapq < opts.min_mapq {
            continue;
        }

        // Count every read that passes mapq/flag filters (matches RSeQC "Total reads used").
        profile.reads_used += 1;
        if profile.reads_used > opts.max_reads {
            break;
        }

        let data = record.data();
        let md_bytes: Vec<u8> = match data.get(&md_tag) {
            Some(Ok(noodles::sam::alignment::record::data::field::Value::String(s))) => {
                (**s).to_vec()
            }
            _ => continue,
        };

        let seq: Vec<u8> = record.sequence().iter().collect();

        let cigar = record.cigar();
        let soft_lead = leading_soft_clip(&cigar);

        accumulate_md(
            &md_bytes,
            &seq,
            soft_lead,
            opts.read_len,
            &mut profile.counts,
        );
    }

    Ok(profile)
}

/// Write the `.mismatch_profile.xls` TSV.
///
/// Format matches RSeQC exactly: a `Total reads used:` preamble line, then a
/// header, then one row per position where the mismatch sum > 0.
/// Column order: `read_pos  sum  A2C  A2G  A2T  C2A  C2G  C2T  G2A  G2C  G2T  T2A  T2C  T2G`
pub fn write_xls(profile: &MismatchProfile, out: &mut dyn Write) -> Result<()> {
    let mut w = BufWriter::with_capacity(256 * 1024, out);
    writeln!(w, "Total reads used: {}", profile.reads_used).map_err(RsomicsError::Io)?;
    writeln!(
        w,
        "read_pos\tsum\tA2C\tA2G\tA2T\tC2A\tC2G\tC2T\tG2A\tG2C\tG2T\tT2A\tT2C\tT2G"
    )
    .map_err(RsomicsError::Io)?;
    for (pos, row) in profile.counts.iter().enumerate() {
        let sum: u32 = row.iter().sum();
        if sum == 0 {
            continue;
        }
        write!(w, "{}\t{}", pos, sum).map_err(RsomicsError::Io)?;
        for &v in row.iter() {
            write!(w, "\t{v}").map_err(RsomicsError::Io)?;
        }
        writeln!(w).map_err(RsomicsError::Io)?;
    }
    w.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

/// Write the `.mismatch_profile.r` R script for visualisation.
pub fn write_r_script(profile: &MismatchProfile, prefix: &str, out: &mut dyn Write) -> Result<()> {
    let names = [
        "A2C", "A2G", "A2T", "C2A", "C2G", "C2T", "G2A", "G2C", "G2T", "T2A", "T2C", "T2G",
    ];
    let colors = [
        "green",
        "powderblue",
        "lightseagreen",
        "red",
        "violetred4",
        "mediumorchid1",
        "blue",
        "royalblue",
        "steelblue1",
        "orange",
        "gold",
        "black",
    ];

    let mut w = BufWriter::with_capacity(256 * 1024, out);
    for (col, name) in names.iter().enumerate() {
        write!(w, "{name}=c(").map_err(RsomicsError::Io)?;
        let vals: Vec<String> = profile.counts.iter().map(|r| r[col].to_string()).collect();
        write!(w, "{}", vals.join(",")).map_err(RsomicsError::Io)?;
        writeln!(w, ")").map_err(RsomicsError::Io)?;
    }

    let color_vec: Vec<String> = colors.iter().map(|c| format!("\"{c}\"")).collect();
    writeln!(w, "color_code = c({})", color_vec.join(",")).map_err(RsomicsError::Io)?;

    let log_exprs: Vec<String> = names.iter().map(|n| format!("log10({n}+1)")).collect();
    let combined = log_exprs.join(",");
    writeln!(w, "y_up_bound = max(c({combined}))").map_err(RsomicsError::Io)?;
    writeln!(w, "y_low_bound = min(c({combined}))").map_err(RsomicsError::Io)?;
    writeln!(w, r#"pdf("{prefix}.mismatch_profile.pdf")"#).map_err(RsomicsError::Io)?;
    writeln!(
        w,
        r#"plot(log10(A2C+1),type="l",col=color_code[1],ylim=c(y_low_bound,y_up_bound),ylab="log10(# of mismatch)",xlab="Read position (5'->3')")"#
    )
    .map_err(RsomicsError::Io)?;
    for (i, name) in names.iter().enumerate().skip(1) {
        writeln!(w, "lines(log10({name}+1), col=color_code[{}])", i + 1)
            .map_err(RsomicsError::Io)?;
    }
    let legend_names: Vec<String> = names.iter().map(|n| format!("\"{n}\"")).collect();
    writeln!(
        w,
        r#"legend(13,y_up_bound,legend=c({}), fill=color_code, border=color_code, ncol=4)"#,
        legend_names.join(",")
    )
    .map_err(RsomicsError::Io)?;
    writeln!(w, "dev.off()").map_err(RsomicsError::Io)?;
    w.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatch_index_coverage() {
        let pairs = [
            (b'A', b'C', 0usize),
            (b'A', b'G', 1),
            (b'A', b'T', 2),
            (b'C', b'A', 3),
            (b'C', b'G', 4),
            (b'C', b'T', 5),
            (b'G', b'A', 6),
            (b'G', b'C', 7),
            (b'G', b'T', 8),
            (b'T', b'A', 9),
            (b'T', b'C', 10),
            (b'T', b'G', 11),
        ];
        for (r, q, expected) in pairs {
            assert_eq!(
                mismatch_index(r, q),
                Some(expected),
                "r={} q={}",
                r as char,
                q as char
            );
        }
    }

    #[test]
    fn identity_returns_none() {
        for b in [b'A', b'C', b'G', b'T'] {
            assert_eq!(mismatch_index(b, b), None);
        }
    }

    #[test]
    fn accumulate_md_single_mismatch() {
        // MD:Z:36C62 — mismatch at aligned read position 36, ref=C, read=A → C2A index 3
        let md = b"36C62";
        let seq = vec![b'A'; 99];
        let mut counts = vec![[0u32; 12]; 100];
        accumulate_md(md, &seq, 0, 100, &mut counts);
        assert_eq!(counts[36][3], 1, "C2A at pos 36");
        let total: u32 = counts.iter().flat_map(|r| r.iter()).sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn accumulate_md_deletion_skipped() {
        // MD:Z:10^ACG5 — deletion then matches, no mismatches
        let md = b"10^ACG5";
        let seq = vec![b'A'; 15];
        let mut counts = vec![[0u32; 12]; 100];
        accumulate_md(md, &seq, 0, 100, &mut counts);
        let total: u32 = counts.iter().flat_map(|r| r.iter()).sum();
        assert_eq!(total, 0);
    }
}
