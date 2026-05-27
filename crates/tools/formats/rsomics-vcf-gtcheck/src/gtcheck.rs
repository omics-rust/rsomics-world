use std::io::{BufRead, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

use crate::{open_vcf_reader, write_all};

/// Format a float in C `printf("%e")` style: 6 decimal places, sign on exponent, minimum 2-digit exponent.
/// E.g. 4.605170e+01, 0.000000e+00
fn fmt_e(v: f64) -> String {
    // Rust's {:e} produces e.g. "4.60517e1" without sign or zero-padded exponent.
    // We need "4.605170e+01" to match C's %e.
    if v == 0.0 || !v.is_finite() {
        return format!("{v:.6e}")
            .replace("e", "e+0")
            .replace("e+0+", "e+")
            .replace("e+0-", "e-");
    }
    let exp = v.abs().log10().floor() as i32;
    let mantissa = v / 10f64.powi(exp);
    let exp_sign = if exp >= 0 { '+' } else { '-' };
    format!("{mantissa:.6}e{exp_sign}{:02}", exp.unsigned_abs())
}

/// Dosage bitmask encoding matching bcftools vcfgtcheck.c gt_to_dsg():
///   0  = missing
///   1  = hom-ref  (0/0)  → 1<<0
///   2  = het      (0/1)  → 1<<1
///   4  = hom-alt  (1/1)  → 1<<2
///
/// Two dosages are concordant when their bitmasks share at least one bit.
/// This handles PL-uncertainty: a PL=0,0,10 site encodes dosage=3 (hom-ref|het),
/// which matches any dosage.
type Dosage = u8;

const DSG_MISSING: Dosage = 0;
const DSG_HOM_REF: Dosage = 1; // 1<<0
const DSG_HET: Dosage = 2; // 1<<1
const DSG_HOM_ALT: Dosage = 4; // 1<<2

pub struct GtcheckArgs {
    pub error_prob: u32,
    pub no_hwe_prob: bool,
    pub homs_only: bool,
    pub use_gt: bool,
}

pub enum GtcheckMode<'a> {
    CrossCheck(&'a Path),
    QueryVsGt { query: &'a Path, gt: &'a Path },
}

/// Flat-column VCF table. Dosages stored row-major: row i starts at i*ncols.
#[derive(Clone)]
struct VcfTable {
    samples: Vec<String>,
    /// Row-major dosage matrix: record i sample j = data[i * ncols + j]
    data: Vec<Dosage>,
    /// Per-record metadata: (chrom, pos, is_variant)
    rows: Vec<(String, u64, bool)>,
}

impl VcfTable {
    fn nrows(&self) -> usize {
        self.rows.len()
    }
    fn ncols(&self) -> usize {
        self.samples.len()
    }
    fn row(&self, i: usize) -> &[Dosage] {
        let nc = self.ncols();
        &self.data[i * nc..(i + 1) * nc]
    }
}

#[inline(always)]
fn parse_gt_allele(s: &str) -> Option<u8> {
    match s {
        "." => None,
        _ => s.parse::<u8>().ok(),
    }
}

#[inline(always)]
fn gt_field_to_dsg(gt: &str) -> Dosage {
    let bytes = gt.as_bytes();
    // Fast path for common diploid GT format "A/B" or "A|B" (length 3)
    if bytes.len() == 3 {
        let a1 = bytes[0];
        let a2 = bytes[2];
        if a1 == b'.' || a2 == b'.' {
            return DSG_MISSING;
        }
        let alt_count = (a1 != b'0') as usize + (a2 != b'0') as usize;
        return 1 << alt_count;
    }
    let sep = if gt.contains('/') { '/' } else { '|' };
    let mut parts = gt.splitn(2, sep);
    let a1 = parts.next().and_then(parse_gt_allele);
    let a2 = parts.next().and_then(parse_gt_allele);
    match (a1, a2) {
        (Some(x), Some(y)) => 1 << ((x != 0) as usize + (y != 0) as usize),
        _ => DSG_MISSING,
    }
}

#[inline(always)]
fn pl_field_to_dsg(pl: &str) -> Dosage {
    let mut vals = [i32::MAX; 3];
    let mut count = 0;
    for (i, s) in pl.splitn(4, ',').enumerate() {
        if i >= 3 {
            break;
        }
        match s.trim().parse::<i32>() {
            Ok(v) if v >= 0 => vals[i] = v,
            _ => return DSG_MISSING,
        }
        count += 1;
    }
    if count < 3 {
        return DSG_MISSING;
    }
    let min = vals[0].min(vals[1]).min(vals[2]);
    let mut dsg: Dosage = 0;
    if vals[0] == min {
        dsg |= DSG_HOM_REF;
    }
    if vals[1] == min {
        dsg |= DSG_HET;
    }
    if vals[2] == min {
        dsg |= DSG_HOM_ALT;
    }
    dsg
}

fn read_vcf(reader: &mut dyn BufRead, use_gt: bool) -> Result<(VcfTable, usize)> {
    let mut samples: Vec<String> = Vec::new();
    let mut data: Vec<Dosage> = Vec::new();
    let mut rows: Vec<(String, u64, bool)> = Vec::new();
    let mut n_skipped_multiallelic: usize = 0;

    // Cache FORMAT field index to avoid re-parsing every line
    let mut cached_fmt: Option<(String, Option<usize>, bool)> = None; // (fmt_str, idx, is_gt_idx)

    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).map_err(RsomicsError::Io)?;
        if n == 0 {
            break;
        }
        let line = line.trim_end_matches('\n').trim_end_matches('\r');

        if line.starts_with("##") {
            continue;
        }
        if line.starts_with('#') {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() > 9 {
                samples = cols[9..].iter().map(|s| s.to_string()).collect();
            }
            continue;
        }

        let mut col_iter = line.splitn(10, '\t');
        let chrom = col_iter.next().unwrap_or("");
        let pos_str = col_iter.next().unwrap_or("");
        let _ = col_iter.next(); // ID
        let _ = col_iter.next(); // REF
        let alt_allele = col_iter.next().unwrap_or(".");
        let _ = col_iter.next(); // QUAL
        let _ = col_iter.next(); // FILTER
        let _ = col_iter.next(); // INFO
        let fmt_str = col_iter.next().unwrap_or("");
        let sample_data = col_iter.next().unwrap_or("");

        let pos: u64 = pos_str
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad POS: {pos_str}")))?;

        let n_alleles = if alt_allele == "." {
            1usize
        } else {
            alt_allele.split(',').count() + 1
        };
        if n_alleles > 2 {
            n_skipped_multiallelic += 1;
            continue;
        }
        let is_variant = alt_allele != "." && !alt_allele.is_empty();

        let chrom_str = chrom.to_string();

        // Cache FORMAT field index (most VCFs have a single FORMAT)
        let (field_idx, is_gt) = match &cached_fmt {
            Some((cached, idx, is_gt)) if cached == fmt_str => (*idx, *is_gt),
            _ => {
                let fmt_cols: Vec<&str> = fmt_str.split(':').collect();
                let (idx, is_gt) = if use_gt {
                    let i = fmt_cols.iter().position(|&f| f == "GT");
                    (i, true)
                } else {
                    match fmt_cols.iter().position(|&f| f == "PL") {
                        Some(i) => (Some(i), false),
                        None => (fmt_cols.iter().position(|&f| f == "GT"), true),
                    }
                };
                cached_fmt = Some((fmt_str.to_string(), idx, is_gt));
                (idx, is_gt)
            }
        };

        let row_start = data.len();
        data.resize(row_start + samples.len(), DSG_MISSING);
        let row = &mut data[row_start..row_start + samples.len()];

        if let Some(idx) = field_idx {
            for (si, sample_str) in sample_data.split('\t').enumerate() {
                if si >= row.len() {
                    break;
                }
                let field_val = sample_str.split(':').nth(idx).unwrap_or(".");
                row[si] = if is_gt {
                    gt_field_to_dsg(field_val)
                } else {
                    pl_field_to_dsg(field_val)
                };
            }
        }

        rows.push((chrom_str, pos, is_variant));
    }

    Ok((
        VcfTable {
            samples,
            data,
            rows,
        },
        n_skipped_multiallelic,
    ))
}

/// HWE probability for matching sites.
/// Returns the negative log of the HWE probability for a concordant dosage combination.
/// `af` is the alternate allele frequency. `match_mask` is the bitwise AND of the two dosages.
fn hwe_neg_log(af: f64, match_mask: Dosage) -> f64 {
    // Compute -log(P_HWE) for each possible dosage, take min for uncertain calls.
    let hwe = [
        -((1.0 - af) * (1.0 - af)).ln(), // hom-ref
        -(2.0 * af * (1.0 - af)).ln(),   // het
        -(af * af).ln(),                 // hom-alt
    ];
    let mut best = f64::INFINITY;
    for (k, &h) in hwe.iter().enumerate() {
        if match_mask & (1 << k) != 0 && h < best {
            best = h;
        }
    }
    best
}

fn allele_freq(tbl: &VcfTable) -> Vec<f64> {
    (0..tbl.nrows())
        .map(|ri| {
            let row = tbl.row(ri);
            let mut alt_count = 0usize;
            let mut total = 0usize;
            for &dsg in row {
                let (alt, tot) = match dsg {
                    DSG_HOM_REF => (0, 2),
                    DSG_HET => (1, 2),
                    DSG_HOM_ALT => (2, 2),
                    _ => continue,
                };
                alt_count += alt;
                total += tot;
            }
            if total == 0 {
                1e-6
            } else {
                alt_count as f64 / total as f64
            }
        })
        .collect()
}

struct PairStats {
    ndiff: u32,
    pdiff: f64,
    ncnt: u32,
    nmatch: u32,
    hwe_prob: f64,
}

impl PairStats {
    fn new() -> Self {
        Self {
            ndiff: 0,
            pdiff: 0.0,
            ncnt: 0,
            nmatch: 0,
            hwe_prob: 0.0,
        }
    }
}

/// GT error probability model (bcftools -E N): returns -log P(geno|dosage) for each of 3 genotypes.
fn dsg2prob_neg_log(dsg: Dosage, eprob: f64) -> [f64; 3] {
    // eprob = 10^(-0.1*E), the probability of reading one allele wrong.
    // P(geno|dosage):
    //   dsg=1 (hom-ref): [0, -log(e), -2*log(e)]
    //   dsg=2 (het):     [-log(e), 0, -log(e)]
    //   dsg=4 (hom-alt): [-2*log(e), -log(e), 0]
    let neg_log_e = -eprob.ln();
    match dsg {
        DSG_HOM_REF => [0.0, neg_log_e, 2.0 * neg_log_e],
        DSG_HET => [neg_log_e, 0.0, neg_log_e],
        DSG_HOM_ALT => [2.0 * neg_log_e, neg_log_e, 0.0],
        _ => [f64::INFINITY, f64::INFINITY, f64::INFINITY],
    }
}

pub fn run_gtcheck(mode: &GtcheckMode<'_>, args: &GtcheckArgs, out: &mut impl Write) -> Result<()> {
    let eprob = if args.error_prob > 0 {
        10f64.powf(-0.1 * args.error_prob as f64)
    } else {
        0.0
    };
    let use_error_model = args.error_prob > 0;

    let (qry_tbl, skip_multi_qry, gt_tbl, skip_multi_gt, cross_check) = match mode {
        GtcheckMode::CrossCheck(path) => {
            let mut rdr = open_vcf_reader(path)?;
            let (tbl, skip) = read_vcf(&mut *rdr, args.use_gt)?;
            let tbl2 = tbl.clone();
            (tbl, skip, tbl2, 0, true)
        }
        GtcheckMode::QueryVsGt { query, gt } => {
            let mut qrdr = open_vcf_reader(query)?;
            let (qt, qskip) = read_vcf(&mut *qrdr, args.use_gt)?;
            let mut grdr = open_vcf_reader(gt)?;
            let (gt_t, gskip) = read_vcf(&mut *grdr, args.use_gt)?;
            (qt, qskip, gt_t, gskip, false)
        }
    };

    let nqry = qry_tbl.ncols();
    let ngt = gt_tbl.ncols();

    if nqry == 0 {
        return Err(RsomicsError::InvalidInput("no samples in query VCF".into()));
    }

    // npairs: for cross-check it's n*(n-1)/2 (lower triangle), for two-file it's nqry*ngt
    let npairs = if cross_check {
        nqry * (nqry + 1) / 2
    } else {
        nqry * ngt
    };
    let mut pairs_stats: Vec<PairStats> = (0..npairs).map(|_| PairStats::new()).collect();

    let mut ncmp: u32 = 0;
    let nskip_no_match: u32 = 0;
    let nskip_not_ba: u32 = skip_multi_qry as u32 + skip_multi_gt as u32;
    let mut nskip_mono: u32 = 0;
    let nskip_no_data: u32 = 0;
    let nskip_dip_gt: u32 = 0;
    let mut nused_gt_gt: u32 = 0;
    let nused_gt_pl: u32 = 0;
    let nused_pl_gt: u32 = 0;
    let nused_pl_pl: u32 = 0;

    // Precompute allele frequencies if HWE prob is needed
    let afs = if !args.no_hwe_prob {
        allele_freq(&qry_tbl)
    } else {
        Vec::new()
    };

    // In two-file mode build index mapping qry row → matching gt row
    let paired_gt_rows: Vec<Option<usize>> = if cross_check {
        (0..qry_tbl.nrows()).map(Some).collect()
    } else {
        let mut out_idx = vec![None; qry_tbl.nrows()];
        let mut gi = 0usize;
        for (qi, slot) in out_idx.iter_mut().enumerate() {
            let (qchrom, qpos, _) = &qry_tbl.rows[qi];
            while gi < gt_tbl.nrows() {
                let (gchrom, gpos, _) = &gt_tbl.rows[gi];
                if gchrom == qchrom && gpos == qpos {
                    *slot = Some(gi);
                    gi += 1;
                    break;
                }
                if (gchrom, gpos) < (qchrom, qpos) {
                    gi += 1;
                } else {
                    break;
                }
            }
        }
        out_idx
    };

    for (rec_idx, paired_gi) in paired_gt_rows.iter().enumerate() {
        let Some(gi) = paired_gi else {
            continue;
        };
        let qry_is_variant = qry_tbl.rows[rec_idx].2;
        let gt_is_variant = gt_tbl.rows[*gi].2;

        if !qry_is_variant || !gt_is_variant {
            nskip_mono += 1;
            continue;
        }

        ncmp += 1;
        nused_gt_gt += 1;

        let af = if !args.no_hwe_prob && rec_idx < afs.len() {
            afs[rec_idx]
        } else {
            0.5
        };

        let qrow = qry_tbl.row(rec_idx);
        let grow = gt_tbl.row(*gi);

        if cross_check {
            // Lower triangle: i outer, j = 0..i, idx = i*(i-1)/2 + j
            for (i, &dsg_i) in qrow.iter().enumerate() {
                if dsg_i == DSG_MISSING {
                    continue;
                }
                if args.homs_only && dsg_i != DSG_HOM_REF && dsg_i != DSG_HOM_ALT {
                    continue;
                }
                let base_idx = i * (i.wrapping_sub(1)) / 2;
                for (j, &dsg_j) in qrow[..i].iter().enumerate() {
                    if dsg_j == DSG_MISSING {
                        continue;
                    }
                    let ps = &mut pairs_stats[base_idx + j];
                    let match_mask = dsg_i & dsg_j;
                    if use_error_model {
                        let prob_i = dsg2prob_neg_log(dsg_i, eprob);
                        let prob_j = dsg2prob_neg_log(dsg_j, eprob);
                        let min = (0..3)
                            .map(|k| prob_i[k] + prob_j[k])
                            .fold(f64::INFINITY, f64::min);
                        ps.pdiff += min;
                        if !args.no_hwe_prob && match_mask != 0 {
                            ps.hwe_prob += hwe_neg_log(af, match_mask);
                            ps.nmatch += 1;
                        }
                    } else if match_mask == 0 {
                        ps.ndiff += 1;
                    } else if !args.no_hwe_prob {
                        ps.hwe_prob += hwe_neg_log(af, match_mask);
                        ps.nmatch += 1;
                    }
                    ps.ncnt += 1;
                }
            }
        } else {
            // Two-file mode: all qry × gt pairs, idx = i*ngt + j
            for (i, &dsg_i) in qrow.iter().enumerate() {
                if dsg_i == DSG_MISSING {
                    continue;
                }
                let base_idx = i * ngt;
                for (j, &dsg_j) in grow.iter().enumerate() {
                    if dsg_j == DSG_MISSING {
                        continue;
                    }
                    if args.homs_only && dsg_j != DSG_HOM_REF && dsg_j != DSG_HOM_ALT {
                        continue;
                    }
                    let ps = &mut pairs_stats[base_idx + j];
                    let match_mask = dsg_i & dsg_j;
                    if use_error_model {
                        let prob_i = dsg2prob_neg_log(dsg_i, eprob);
                        let prob_j = dsg2prob_neg_log(dsg_j, eprob);
                        let min = (0..3)
                            .map(|k| prob_i[k] + prob_j[k])
                            .fold(f64::INFINITY, f64::min);
                        ps.pdiff += min;
                        if !args.no_hwe_prob && match_mask != 0 {
                            ps.hwe_prob += hwe_neg_log(af, match_mask);
                            ps.nmatch += 1;
                        }
                    } else if match_mask == 0 {
                        ps.ndiff += 1;
                    } else if !args.no_hwe_prob {
                        ps.hwe_prob += hwe_neg_log(af, match_mask);
                        ps.nmatch += 1;
                    }
                    ps.ncnt += 1;
                }
            }
        }
    }

    // Write output matching bcftools gtcheck format exactly (excluding the command header lines).
    write_all(out, format!("INFO\tsites-compared\t{ncmp}\n").as_bytes())?;
    write_all(
        out,
        format!("INFO\tsites-skipped-no-match\t{nskip_no_match}\n").as_bytes(),
    )?;
    write_all(
        out,
        format!("INFO\tsites-skipped-multiallelic\t{nskip_not_ba}\n").as_bytes(),
    )?;
    write_all(
        out,
        format!("INFO\tsites-skipped-monoallelic\t{nskip_mono}\n").as_bytes(),
    )?;
    write_all(
        out,
        format!("INFO\tsites-skipped-no-data\t{nskip_no_data}\n").as_bytes(),
    )?;
    write_all(
        out,
        format!("INFO\tsites-skipped-GT-not-diploid\t{nskip_dip_gt}\n").as_bytes(),
    )?;
    write_all(out, b"INFO\tsites-skipped-PL-not-diploid\t0\n")?;
    write_all(out, b"INFO\tsites-skipped-filtering-expression\t0\n")?;
    write_all(
        out,
        format!("INFO\tsites-used-PL-vs-PL\t{nused_pl_pl}\n").as_bytes(),
    )?;
    write_all(
        out,
        format!("INFO\tsites-used-PL-vs-GT\t{nused_pl_gt}\n").as_bytes(),
    )?;
    write_all(
        out,
        format!("INFO\tsites-used-GT-vs-PL\t{nused_gt_pl}\n").as_bytes(),
    )?;
    write_all(
        out,
        format!("INFO\tsites-used-GT-vs-GT\t{nused_gt_gt}\n").as_bytes(),
    )?;
    write_all(
        out,
        b"# DCv2, discordance version 2:\n\
          #     - Query sample\n\
          #     - Genotyped sample\n\
          #     - Discordance, given either as an abstract score or number of mismatches, see the options -E/-u\n\
          #       in man page for details. Note that samples with high missingness have fewer sites compared,\n\
          #       which results in lower overall discordance. Therefore it is advisable to use the average score\n\
          #       per site rather than the absolute value, i.e. divide the value by the number of sites compared\n\
          #       (smaller value = better match)\n\
          #     - Average negative log of HWE probability at matching sites, attempts to quantify the following\n\
          #       intuition: rare genotype matches are more informative than common genotype matches, hence two\n\
          #       samples with similar discordance can be further stratified by the HWE score (bigger value = better\n\
          #       match, the observed concordance was less likely to occur by chance)\n\
          #     - Number of sites compared for this pair of samples (bigger = more informative)\n\
          #     - Number of matching genotypes\n\
          #DCv2\t[2]Query Sample\t[3]Genotyped Sample\t[4]Discordance\t[5]Average -log P(HWE)\t[6]Number of sites compared\t[7]Number of matching genotypes\n",
    )?;

    let qry_samples = &qry_tbl.samples;
    let gt_samples = &gt_tbl.samples;

    if cross_check {
        // Output lower triangle: outer sample i vs all j < i.
        // idx follows the triangular packing: idx = i*(i-1)/2 + j.
        let mut idx = 0usize;
        for (i, sname_i) in qry_samples.iter().enumerate().skip(1) {
            for sname_j in &qry_samples[..i] {
                let ps = &pairs_stats[idx];
                let hwe_avg = if !args.no_hwe_prob && ps.nmatch > 0 {
                    ps.hwe_prob / ps.nmatch as f64
                } else {
                    0.0
                };
                if use_error_model {
                    write_all(
                        out,
                        format!(
                            "DCv2\t{sname_i}\t{sname_j}\t{}\t{}\t{}\t{}\n",
                            fmt_e(ps.pdiff),
                            fmt_e(hwe_avg),
                            ps.ncnt,
                            if args.no_hwe_prob { 0 } else { ps.nmatch },
                        )
                        .as_bytes(),
                    )?;
                } else {
                    write_all(
                        out,
                        format!(
                            "DCv2\t{sname_i}\t{sname_j}\t{}\t{}\t{}\t{}\n",
                            ps.ndiff,
                            fmt_e(hwe_avg),
                            ps.ncnt,
                            if args.no_hwe_prob { 0 } else { ps.nmatch },
                        )
                        .as_bytes(),
                    )?;
                }
                idx += 1;
            }
        }
    } else {
        for (i, qsname) in qry_samples.iter().enumerate() {
            for (j, gtsname) in gt_samples.iter().enumerate() {
                let idx = i * ngt + j;
                let ps = &pairs_stats[idx];
                let hwe_avg = if !args.no_hwe_prob && ps.nmatch > 0 {
                    ps.hwe_prob / ps.nmatch as f64
                } else {
                    0.0
                };
                if use_error_model {
                    write_all(
                        out,
                        format!(
                            "DCv2\t{qsname}\t{gtsname}\t{}\t{}\t{}\t{}\n",
                            fmt_e(ps.pdiff),
                            fmt_e(hwe_avg),
                            ps.ncnt,
                            if args.no_hwe_prob { 0 } else { ps.nmatch },
                        )
                        .as_bytes(),
                    )?;
                } else {
                    write_all(
                        out,
                        format!(
                            "DCv2\t{qsname}\t{gtsname}\t{}\t{}\t{}\t{}\n",
                            ps.ndiff,
                            fmt_e(hwe_avg),
                            ps.ncnt,
                            if args.no_hwe_prob { 0 } else { ps.nmatch },
                        )
                        .as_bytes(),
                    )?;
                }
            }
        }
    }

    Ok(())
}
