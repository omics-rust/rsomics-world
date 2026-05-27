use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

use rsomics_common::{Result, RsomicsError};

use crate::cnv::{EmissionParams, make_peaks, site_emission};
use crate::hmm::{self, N_STATES, phred_score};

/// Per-chromosome per-sample record collected before HMM decoding.
struct ChromBuf {
    /// 0-based genomic positions.
    positions: Vec<u32>,
    emissions: Vec<[f64; N_STATES]>,
}

impl ChromBuf {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            emissions: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.positions.clear();
        self.emissions.clear();
    }

    fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

pub struct CnvArgs {
    /// Query sample name (auto-detected from VCF if None and only one sample).
    pub sample: Option<String>,
    /// Output directory for per-sample files.
    pub output_dir: PathBuf,
    /// Allele frequency file (CHR TAB POS TAB REF,ALT TAB AF). None = use default genotype freqs.
    pub af_file: Option<PathBuf>,
    pub emission: EmissionParams,
    /// Off-diagonal transition probability for the 4-state HMM.
    pub xy_prob: f64,
    /// LRR moving-average smoothing window (0 = disabled).
    pub lrr_smooth_win: usize,
    /// Suppress per-site CN output (write only summary).
    pub summary_only: bool,
}

/// Open a VCF/BCF file for reading, transparently decompressing gzip.
fn open_vcf(path: &Path) -> Result<Box<dyn BufRead>> {
    use std::fs::File;
    use std::io::Read;
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut peek = [0u8; 2];
    let mut buf = std::io::BufReader::new(file);
    let n = buf.read(&mut peek).map_err(RsomicsError::Io)?;
    let is_gz = n == 2 && peek[0] == 0x1f && peek[1] == 0x8b;
    let chain: Box<dyn Read> = {
        let chain = std::io::Cursor::new(peek[..n].to_vec()).chain(buf);
        if is_gz {
            Box::new(flate2::read::MultiGzDecoder::new(chain))
        } else {
            Box::new(chain)
        }
    };
    Ok(Box::new(std::io::BufReader::new(chain)))
}

/// Apply a moving-average smoothing to `lrr` in-place, window half-size `win`.
fn smooth_lrr(lrr: &mut [f32], win: usize) {
    if win == 0 || lrr.len() < 2 {
        return;
    }
    let n = lrr.len();
    let mut out = vec![0f32; n];
    for (i, val) in out.iter_mut().enumerate() {
        let lo = i.saturating_sub(win / 2);
        let hi = (i + win / 2 + 1).min(n);
        let count = hi - lo;
        let sum: f32 = lrr[lo..hi].iter().sum();
        *val = sum / count as f32;
    }
    lrr.copy_from_slice(&out);
}

/// Run the CNV caller on the input VCF and write per-sample output files.
pub fn run_cnv(input: &Path, args: &CnvArgs) -> Result<()> {
    std::fs::create_dir_all(&args.output_dir)
        .map_err(|e| RsomicsError::InvalidInput(format!("cannot create output dir: {e}")))?;

    let reader = open_vcf(input)?;

    // Load AF file into a lookup table keyed by (chrom, pos).
    let af_lookup = args
        .af_file
        .as_deref()
        .map(load_af_file)
        .transpose()?
        .unwrap_or_default();

    // Default genotype frequencies when no AF file is provided (bcftools defaults).
    const DEFAULT_FRR: f64 = 0.76;
    const DEFAULT_FRA: f64 = 0.14;
    const DEFAULT_FAA: f64 = 0.098;

    let peaks = make_peaks(&args.emission);

    let mut sample_names: Vec<String> = Vec::new();
    let mut target_idx: Option<usize> = None; // column index in sample list (0-based)

    let mut current_chrom = String::new();
    let mut buf = ChromBuf::new();
    // Separate LRR buffer so we can smooth before computing emissions.
    let mut lrr_buf: Vec<f32> = Vec::new();
    // BAF buffer parallel to lrr_buf (one float per site).
    let mut baf_buf: Vec<f32> = Vec::new();
    // genotype-frequency buffer per site (for AF-file mode)
    let mut gt_freq_buf: Vec<(f64, f64, f64)> = Vec::new();

    // Output file handles, opened lazily once sample name is known.
    let mut cn_fh: Option<Box<dyn Write>> = None;
    let mut summary_fh: Option<Box<dyn Write>> = None;
    let mut sample_name = String::new();

    for line_res in reader.lines() {
        let line = line_res.map_err(RsomicsError::Io)?;

        if line.starts_with("##") {
            continue;
        }

        if line.starts_with('#') {
            // #CHROM header line
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 9 {
                return Err(RsomicsError::InvalidInput(
                    "VCF header has fewer than 9 columns".into(),
                ));
            }
            for s in &cols[9..] {
                sample_names.push(s.to_string());
            }

            // Resolve query sample
            let sname = match &args.sample {
                Some(s) => s.clone(),
                None => {
                    if sample_names.len() == 1 {
                        sample_names[0].clone()
                    } else {
                        return Err(RsomicsError::InvalidInput(
                            "multi-sample VCF requires --sample".into(),
                        ));
                    }
                }
            };
            target_idx = sample_names.iter().position(|n| n == &sname);
            if target_idx.is_none() {
                return Err(RsomicsError::InvalidInput(format!(
                    "sample '{sname}' not found in VCF"
                )));
            }
            sample_name = sname.clone();

            // Open output files
            let cn_path = args.output_dir.join(format!("cn.{sname}.tab"));
            let summary_path = args.output_dir.join(format!("summary.{sname}.tab"));

            let mut cn_w =
                BufWriter::new(std::fs::File::create(&cn_path).map_err(|e| {
                    RsomicsError::InvalidInput(format!("{}: {e}", cn_path.display()))
                })?);
            let mut sum_w = BufWriter::new(std::fs::File::create(&summary_path).map_err(|e| {
                RsomicsError::InvalidInput(format!("{}: {e}", summary_path.display()))
            })?);

            writeln!(
                cn_w,
                "# [1]Chromosome\t[2]Position\t[3]CN\t[4]P(CN0)\t[5]P(CN1)\t[6]P(CN2)\t[7]P(CN3)"
            )
            .map_err(RsomicsError::Io)?;
            writeln!(
                sum_w,
                "# RG, Regions\t[2]Chromosome\t[3]Start\t[4]End\t[5]Copy number:{sname}\t[6]Quality\t[7]nSites\t[8]nHETs"
            )
            .map_err(RsomicsError::Io)?;

            cn_fh = Some(Box::new(cn_w));
            summary_fh = Some(Box::new(sum_w));
            continue;
        }

        let tidx = match target_idx {
            Some(i) => i,
            None => {
                return Err(RsomicsError::InvalidInput(
                    "VCF data line seen before #CHROM header".into(),
                ));
            }
        };

        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 9 {
            continue;
        }

        let chrom = cols[0];
        let pos_1based: u32 = cols[1]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad POS: {}", cols[1])))?;
        let pos = pos_1based.saturating_sub(1); // 0-based

        let format_str = cols[8];
        let sample_col = 9 + tidx;
        if sample_col >= cols.len() {
            continue;
        }
        let sample_field = cols[sample_col];

        let format_tags: Vec<&str> = format_str.split(':').collect();
        let sample_fields: Vec<&str> = sample_field.split(':').collect();

        let baf_idx = format_tags.iter().position(|&t| t == "BAF");
        let lrr_idx = format_tags.iter().position(|&t| t == "LRR");
        let gt_idx = format_tags.iter().position(|&t| t == "GT");

        // Extract BAF and LRR float values (missing = -1)
        let baf = baf_idx
            .and_then(|i| sample_fields.get(i).copied())
            .and_then(|v| {
                if v == "." {
                    None
                } else {
                    v.parse::<f32>().ok()
                }
            })
            .unwrap_or(-1.0);
        let lrr = lrr_idx
            .and_then(|i| sample_fields.get(i).copied())
            .and_then(|v| {
                if v == "." {
                    None
                } else {
                    v.parse::<f32>().ok()
                }
            })
            .unwrap_or(0.0);

        // Skip sites where LRR is missing if LRR bias > 0 (bcftools skips these)
        if args.emission.lrr_bias > 0.0 && lrr_idx.is_none() && baf < 0.0 {
            continue;
        }

        // Genotype frequencies for this site
        let (f_rr, f_ra, f_aa) = if let Some(af) = af_lookup.get(&(chrom.to_string(), pos_1based)) {
            let af = *af;
            let q = 1.0 - af;
            (q * q, 2.0 * af * q, af * af)
        } else {
            (DEFAULT_FRR, DEFAULT_FRA, DEFAULT_FAA)
        };

        // Flush previous chromosome buffer when chromosome changes
        if chrom != current_chrom {
            if !current_chrom.is_empty() && !buf.is_empty() {
                flush_chrom(
                    &current_chrom,
                    &mut buf,
                    &mut lrr_buf,
                    &mut baf_buf,
                    &mut gt_freq_buf,
                    &peaks,
                    &args.emission,
                    args.xy_prob,
                    args.lrr_smooth_win,
                    args.summary_only,
                    &sample_name,
                    cn_fh.as_deref_mut().unwrap(),
                    summary_fh.as_deref_mut().unwrap(),
                    &format_tags,
                    gt_idx,
                )?;
            }
            current_chrom = chrom.to_string();
        }

        buf.positions.push(pos);
        baf_buf.push(baf);
        lrr_buf.push(lrr);
        gt_freq_buf.push((f_rr, f_ra, f_aa));

        // Track sample_fields/gt_idx — we pass format_tags to flush; stash placeholder
        // emission (will be recomputed in flush after LRR smoothing).
        buf.emissions.push([0.0; N_STATES]);
    }

    // Flush final chromosome
    if !current_chrom.is_empty() && !buf.is_empty() {
        flush_chrom(
            &current_chrom,
            &mut buf,
            &mut lrr_buf,
            &mut baf_buf,
            &mut gt_freq_buf,
            &peaks,
            &args.emission,
            args.xy_prob,
            args.lrr_smooth_win,
            args.summary_only,
            &sample_name,
            cn_fh.as_deref_mut().unwrap(),
            summary_fh.as_deref_mut().unwrap(),
            &[],
            None,
        )?;
    }

    if let Some(w) = cn_fh.as_mut() {
        w.flush().map_err(RsomicsError::Io)?;
    }
    if let Some(w) = summary_fh.as_mut() {
        w.flush().map_err(RsomicsError::Io)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn flush_chrom(
    chrom: &str,
    buf: &mut ChromBuf,
    lrr_buf: &mut Vec<f32>,
    baf_buf: &mut Vec<f32>,
    gt_freq_buf: &mut Vec<(f64, f64, f64)>,
    peaks: &crate::cnv::GaussPeaks,
    params: &EmissionParams,
    xy_prob: f64,
    lrr_smooth_win: usize,
    summary_only: bool,
    _sample_name: &str,
    cn_fh: &mut dyn Write,
    summary_fh: &mut dyn Write,
    _format_tags: &[&str],
    _gt_idx: Option<usize>,
) -> Result<()> {
    // Apply LRR smoothing
    smooth_lrr(lrr_buf, lrr_smooth_win);

    // Compute emissions now that LRR is smoothed
    let n = buf.positions.len();
    for i in 0..n {
        let (f_rr, f_ra, f_aa) = gt_freq_buf[i];
        buf.emissions[i] = site_emission(baf_buf[i], lrr_buf[i], f_rr, f_ra, f_aa, params, peaks);
    }

    let result = hmm::run(&buf.emissions, &buf.positions, xy_prob);

    // Count HETs per region: site is heterozygous if BAF is between 0.2 and 0.8
    // (bcftools smpl_nhet: baf in (0.1, 0.9) suggests a heterozygous SNP)
    let is_het = |i: usize| -> bool {
        let baf = baf_buf[i];
        (0.1_f32..=0.9_f32).contains(&baf)
    };

    // --- Write per-site CN lines ---
    if !summary_only {
        for i in 0..n {
            let pos = buf.positions[i] + 1; // 1-based
            let cn_char = b"0123"[result.vpath[i] as usize] as char;
            let e = &buf.emissions[i];
            writeln!(
                cn_fh,
                "{chrom}\t{pos}\t{cn_char}\t{:.6}\t{:.6}\t{:.6}\t{:.6}",
                e[0], e[1], e[2], e[3]
            )
            .map_err(RsomicsError::Io)?;
        }
    }

    // --- Write summary RG lines ---
    // Collect (chrom, start_1based, end_1based, cn, qual_acc, n_sites, n_hets) per contiguous
    // run of identical CN state, then flush each run as one RG line.
    let mut region_start_pos: u32 = buf.positions[0] + 1; // 1-based
    let mut region_cn: u8 = result.vpath[0];
    let mut qual_acc: f64 = result.posterior[0];
    let mut n_sites: usize = 1;
    let mut n_hets: usize = if is_het(0) { 1 } else { 0 };

    let flush_region =
        |start: u32, end: u32, cn: u8, q: f64, ns: usize, nh: usize, out: &mut dyn Write| {
            let mean_qual = phred_score(1.0 - q / ns as f64);
            let cn_char = b"0123"[cn as usize] as char;
            writeln!(
                out,
                "RG\t{chrom}\t{start}\t{end}\t{cn_char}\t{mean_qual:.1}\t{ns}\t{nh}",
            )
            .map_err(RsomicsError::Io)
        };

    for i in 1..n {
        let state = result.vpath[i];
        let pos = buf.positions[i] + 1; // 1-based
        if state == region_cn {
            // Extend current region
            qual_acc += result.posterior[i];
            n_sites += 1;
            if is_het(i) {
                n_hets += 1;
            }
        } else {
            // end = 0-based position of the first site of the new region.
            // This equals 1-based_POS(new_site) - 1, matching bcftools sites[isite] output.
            let end_pos = buf.positions[i]; // 0-based = 1-based_POS - 1
            flush_region(
                region_start_pos,
                end_pos,
                region_cn,
                qual_acc,
                n_sites,
                n_hets,
                summary_fh,
            )?;
            // Start new
            region_start_pos = pos;
            region_cn = state;
            qual_acc = result.posterior[i];
            n_sites = 1;
            n_hets = if is_het(i) { 1 } else { 0 };
        }
    }
    // Flush final region
    let end_pos = buf.positions[n - 1] + 1;
    flush_region(
        region_start_pos,
        end_pos,
        region_cn,
        qual_acc,
        n_sites,
        n_hets,
        summary_fh,
    )?;

    buf.clear();
    lrr_buf.clear();
    baf_buf.clear();
    gt_freq_buf.clear();
    Ok(())
}

/// AF lookup table keyed by (chrom, 1-based pos).
type AfLookup = std::collections::HashMap<(String, u32), f64>;

/// Load an allele frequency file (CHR TAB POS TAB REF,ALT TAB AF format).
fn load_af_file(path: &Path) -> Result<AfLookup> {
    use std::io::BufRead;
    let file = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut map = AfLookup::new();
    for line in std::io::BufReader::new(file).lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            continue;
        }
        let chrom = cols[0].to_string();
        let pos: u32 = cols[1]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad pos in AF file: {}", cols[1])))?;
        let af: f64 = cols[3]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad AF in AF file: {}", cols[3])))?;
        map.insert((chrom, pos), af);
    }
    Ok(map)
}
