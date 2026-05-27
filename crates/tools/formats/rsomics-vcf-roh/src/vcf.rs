use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use rsomics_common::{Result, RsomicsError};

use crate::hmm::{self, Site, TransParams, phred_score};
use crate::open_vcf_reader;

pub struct OutputMode {
    pub sites: bool,
    pub regions: bool,
}

pub struct RohArgs {
    /// P(AZ|HW): probability of transitioning from HW to autozygous state.
    pub hw_to_az: f64,
    /// P(HW|AZ): probability of transitioning from AZ to HW state.
    pub az_to_hw: f64,
    pub af_dflt: Option<f64>,
    pub af_tag: Option<String>,
    /// When Some, use GT-only mode with this value as error prob for non-called genotypes.
    pub fake_pl_error: Option<f64>,
    pub skip_indels: bool,
    pub ignore_homref: bool,
    pub samples: Option<String>,
    pub output_mode: OutputMode,
    pub output: Option<PathBuf>,
}

/// Per-sample accumulator of HMM emission sites for the current chromosome.
struct SampleBuf {
    name: String,
    sites: Vec<Site>,
}

/// Flush one sample's site buffer: run Viterbi + FwdBwd, then emit ST/RG lines.
fn flush_sample(
    buf: &mut SampleBuf,
    chrom: &str,
    params: TransParams,
    mode: &OutputMode,
    out: &mut dyn Write,
) -> Result<()> {
    if buf.sites.is_empty() {
        return Ok(());
    }

    let result = hmm::run(&buf.sites, params);

    if mode.sites {
        for (i, site) in buf.sites.iter().enumerate() {
            let state = result.vpath[i];
            let qual = phred_score(result.posterior[i]);
            writeln!(
                out,
                "ST\t{}\t{}\t{}\t{}\t{:.1}",
                buf.name,
                chrom,
                site.pos + 1, // 1-based
                state,
                qual
            )
            .map_err(RsomicsError::Io)?;
        }
    }

    if mode.regions {
        // Accumulate contiguous AZ runs and emit RG lines.
        let mut rg_state: Option<RegionAcc> = None;

        for (i, site) in buf.sites.iter().enumerate() {
            let state = result.vpath[i];
            let qual = phred_score(result.posterior[i]);

            if state == 1 {
                match &mut rg_state {
                    None => {
                        rg_state = Some(RegionAcc {
                            beg: site.pos,
                            end: site.pos,
                            qual_sum: qual,
                            nmarkers: 1,
                        });
                    }
                    Some(acc) => {
                        acc.end = site.pos;
                        acc.qual_sum += qual;
                        acc.nmarkers += 1;
                    }
                }
            } else if let Some(acc) = rg_state.take() {
                writeln!(
                    out,
                    "RG\t{}\t{}\t{}\t{}\t{}\t{}\t{:.1}",
                    buf.name,
                    chrom,
                    acc.beg + 1,
                    acc.end + 1,
                    acc.end - acc.beg + 1,
                    acc.nmarkers,
                    acc.qual_sum / acc.nmarkers as f64,
                )
                .map_err(RsomicsError::Io)?;
            }
        }

        // Flush any open region at end of chromosome.
        if let Some(acc) = rg_state {
            writeln!(
                out,
                "RG\t{}\t{}\t{}\t{}\t{}\t{}\t{:.1}",
                buf.name,
                chrom,
                acc.beg + 1,
                acc.end + 1,
                acc.end - acc.beg + 1,
                acc.nmarkers,
                acc.qual_sum / acc.nmarkers as f64,
            )
            .map_err(RsomicsError::Io)?;
        }
    }

    buf.sites.clear();
    Ok(())
}

struct RegionAcc {
    beg: u32,
    end: u32,
    qual_sum: f64,
    nmarkers: u32,
}

/// Parse the VCF `##FORMAT=<ID=PL,...>` or `##FORMAT=<ID=GT,...>` header to find indices.
/// Returns (pl_col_offset_in_format, gt_col_offset_in_format) for runtime line parsing.
struct FormatIds {
    pl_idx: Option<usize>,
    gt_idx: Option<usize>,
}

fn find_format_ids(format_str: &str) -> FormatIds {
    let tags: Vec<&str> = format_str.split(':').collect();
    FormatIds {
        pl_idx: tags.iter().position(|&t| t == "PL"),
        gt_idx: tags.iter().position(|&t| t == "GT"),
    }
}

/// Convert a PL string (comma-sep Phred-encoded int) to P(D|G) for diploid biallelic.
/// pl2p[x] = 10^(-x/10). Indices: 0=RR, 1=RA, 2=AA.
/// Returns None if the field is missing ('.') or malformed.
fn parse_pl(pl_str: &str, pl2p: &[f64; 256]) -> Option<[f64; 3]> {
    if pl_str == "." || pl_str.is_empty() {
        return None;
    }
    let mut parts = pl_str.splitn(3, ',');
    let rr: usize = parts.next()?.parse().ok()?;
    let ra: usize = parts.next()?.parse().ok()?;
    let aa: usize = parts.next()?.parse().ok()?;
    // All-same → uninformative, skip
    if rr == ra && rr == aa {
        return None;
    }
    Some([pl2p[rr.min(255)], pl2p[ra.min(255)], pl2p[aa.min(255)]])
}

/// Fake PL from GT: assign high probability to called genotype, `unseen_err` to others.
/// unseen_err is the linear-scale error probability (= 10^(-phred/10)).
fn fake_pl_from_gt(gt_str: &str, unseen_err: f64) -> Option<[f64; 3]> {
    // GT field: "0/0", "0/1", "1/1", "0|1", etc. Missing: "./."
    let sep = if gt_str.contains('|') { '|' } else { '/' };
    let (a_str, b_str) = gt_str.split_once(sep)?;
    if a_str == "." || b_str == "." {
        return None;
    }
    let a: u8 = a_str.parse().ok()?;
    let b: u8 = b_str.parse().ok()?;
    let e = unseen_err;
    let e2 = e * e;
    // pdg[0]=P(D|RR), pdg[1]=P(D|RA), pdg[2]=P(D|AA) — matching bcftools vcfroh.c fake_PLs block
    let pdg = if a != b {
        // het
        [e, 1.0 - 2.0 * e, e]
    } else if a == 0 {
        // hom-ref
        [1.0 - e - e2, e, e2]
    } else {
        // hom-alt
        [e2, e, 1.0 - e - e2]
    };
    Some(pdg)
}

/// Extract allele frequency from an INFO field string.
fn get_af_from_info(info: &str, tag: &str) -> Option<f64> {
    for field in info.split(';') {
        if let Some(rest) = field.strip_prefix(tag)
            && let Some(val_str) = rest.strip_prefix('=')
        {
            // May be comma-sep (multi-allelic); take first value.
            let first = val_str.split(',').next()?;
            return first.parse().ok();
        }
    }
    None
}

/// Extract AC/AN from INFO and compute AF.
fn get_af_from_ac_an(info: &str) -> Option<f64> {
    let mut ac: Option<i64> = None;
    let mut an: Option<i64> = None;
    for field in info.split(';') {
        if let Some(rest) = field.strip_prefix("AC=") {
            ac = rest.split(',').next().and_then(|v| v.parse().ok());
        } else if let Some(rest) = field.strip_prefix("AN=") {
            an = rest.parse().ok();
        }
    }
    let ac = ac?;
    let an = an?;
    if an <= 0 || ac < 0 {
        return None;
    }
    Some(ac as f64 / an as f64)
}

/// Compute emission probabilities P(D|HW) and P(D|AZ) from genotype likelihoods.
///
/// oAZ = (1-f)*P(D|RR) + f*P(D|AA)
/// oHW = (1-f)^2*P(D|RR) + 2*f*(1-f)*P(D|RA) + f^2*P(D|AA)
fn emission_probs(pdg: &[f64; 3], af: f64) -> [f64; 2] {
    let f = af;
    let q = 1.0 - f;
    let e_az = q * pdg[0] + f * pdg[2];
    let e_hw = q * q * pdg[0] + 2.0 * f * q * pdg[1] + f * f * pdg[2];
    [e_hw, e_az]
}

pub fn run_roh(input: &Path, args: &RohArgs, out: &mut dyn Write) -> Result<()> {
    let reader = open_vcf_reader(input)?;

    let params = TransParams {
        t2az: args.hw_to_az,
        t2hw: args.az_to_hw,
    };

    // pl2p lookup table: index = Phred score (0..=255), value = 10^(-score/10)
    let mut pl2p = [0f64; 256];
    for (i, v) in pl2p.iter_mut().enumerate() {
        *v = 10f64.powf(-(i as f64) / 10.0);
    }

    // State machine: collect header, then parse data lines.
    let mut sample_names: Vec<String> = Vec::new();
    // Indices into sample_names that we're analysing.
    let mut target_indices: Vec<usize> = Vec::new();
    // Per-target accumulator; parallel to target_indices.
    let mut buffers: Vec<SampleBuf> = Vec::new();

    let mut current_chrom = String::new();

    // Write output header matching bcftools.
    if args.output_mode.regions {
        writeln!(out, "# RG\t[2]Sample\t[3]Chromosome\t[4]Start\t[5]End\t[6]Length (bp)\t[7]Number of markers\t[8]Quality (average fwd-bwd phred score)")
            .map_err(RsomicsError::Io)?;
    }
    if args.output_mode.sites {
        writeln!(out, "# ST\t[2]Sample\t[3]Chromosome\t[4]Position\t[5]State (0:HW, 1:AZ)\t[6]Quality (fwd-bwd phred score)")
            .map_err(RsomicsError::Io)?;
    }

    let sample_filter: Option<Vec<&str>> = args.samples.as_deref().map(|s| s.split(',').collect());

    // Collect all VCF lines
    for line_res in reader.lines() {
        let line = line_res.map_err(RsomicsError::Io)?;

        if line.starts_with("##") {
            continue;
        }

        if line.starts_with('#') {
            // Header line: #CHROM POS ID REF ALT QUAL FILTER INFO FORMAT sample1 sample2 ...
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 9 {
                return Err(RsomicsError::InvalidInput(
                    "VCF header line has fewer than 9 columns".into(),
                ));
            }
            for name in &cols[9..] {
                sample_names.push(name.to_string());
            }

            // Build target set
            for (idx, name) in sample_names.iter().enumerate() {
                let include = match &sample_filter {
                    None => true,
                    Some(filter) => filter.contains(&name.as_str()),
                };
                if include {
                    target_indices.push(idx);
                    buffers.push(SampleBuf {
                        name: name.clone(),
                        sites: Vec::new(),
                    });
                }
            }

            if buffers.is_empty() {
                return Err(RsomicsError::InvalidInput(
                    "No target samples found in VCF".into(),
                ));
            }
            continue;
        }

        if sample_names.is_empty() {
            return Err(RsomicsError::InvalidInput(
                "VCF data line seen before header line".into(),
            ));
        }

        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 9 {
            continue;
        }

        let chrom = cols[0];
        let pos_str = cols[1];
        let ref_allele = cols[3];
        let alt_field = cols[4];
        let info = cols[7];
        let format_str = cols[8];

        // 0-based position
        let pos: u32 = pos_str
            .parse::<u32>()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad POS: {pos_str}")))?
            .saturating_sub(1);

        // Skip multiallelic: count non-<*>/<NON_REF> alt alleles.
        let real_alts: Vec<&str> = alt_field
            .split(',')
            .filter(|a| *a != "<*>" && *a != "<NON_REF>" && *a != ".")
            .collect();

        if real_alts.len() > 1 {
            continue;
        }

        // Skip indels if requested
        if args.skip_indels {
            // An indel has any allele (ref or any real alt) with length != 1.
            let is_snp = ref_allele.len() == 1 && real_alts.iter().all(|a| a.len() == 1);
            if !is_snp {
                continue;
            }
        }

        // Flush previous chromosome's data when chromosome changes.
        if chrom != current_chrom {
            if !current_chrom.is_empty() {
                for buf in &mut buffers {
                    flush_sample(buf, &current_chrom, params, &args.output_mode, out)?;
                }
            }
            current_chrom = chrom.to_string();
        }

        // Determine allele frequency.
        let af_opt: Option<f64> = if let Some(tag) = &args.af_tag {
            get_af_from_info(info, tag)
        } else if real_alts.is_empty() {
            // No alt allele; use dflt_AF if set.
            args.af_dflt
        } else {
            get_af_from_ac_an(info).or(args.af_dflt)
        };

        let af = match af_opt {
            None => continue, // skip site — no AF available
            Some(v) => {
                if v == 0.0 {
                    // AF=0 means monomorphic; skip unless dflt_AF overrides.
                    match args.af_dflt {
                        Some(d) if d > 0.0 => d,
                        _ => continue,
                    }
                } else {
                    v
                }
            }
        };

        // Parse FORMAT column to find PL and GT field indices.
        let fids = find_format_ids(format_str);

        // Accumulate emissions per target sample.
        for (buf_i, &smpl_col_idx) in target_indices.iter().enumerate() {
            let sample_field_idx = 9 + smpl_col_idx;
            if sample_field_idx >= cols.len() {
                continue;
            }
            let sample_str = cols[sample_field_idx];
            let sample_fields: Vec<&str> = sample_str.split(':').collect();

            let pdg: Option<[f64; 3]> = if let Some(err) = args.fake_pl_error {
                // GT-only mode
                fids.gt_idx
                    .and_then(|gi| sample_fields.get(gi).copied())
                    .and_then(|gt| fake_pl_from_gt(gt, err))
            } else {
                // PL mode
                fids.pl_idx
                    .and_then(|pi| sample_fields.get(pi).copied())
                    .and_then(|pl| parse_pl(pl, &pl2p))
            };

            let pdg = match pdg {
                None => continue,
                Some(v) => v,
            };

            let sum = pdg[0] + pdg[1] + pdg[2];
            if sum == 0.0 {
                continue;
            }
            let pdg = [pdg[0] / sum, pdg[1] / sum, pdg[2] / sum];

            // Skip hom-ref if requested (pdg[0] > 0.99 after normalisation).
            if args.ignore_homref && pdg[0] > 0.99 {
                continue;
            }

            let eprob = emission_probs(&pdg, af);
            buffers[buf_i].sites.push(Site { pos, eprob });
        }
    }

    // Flush final chromosome.
    if !current_chrom.is_empty() {
        for buf in &mut buffers {
            flush_sample(buf, &current_chrom, params, &args.output_mode, out)?;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}
