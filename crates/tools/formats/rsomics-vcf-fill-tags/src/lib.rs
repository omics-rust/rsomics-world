#![allow(
    clippy::struct_excessive_bools, // Tags struct is a flat flag set — no invariants to hide
    clippy::too_many_lines,         // compute_info tags sequentially, splitting adds no clarity
    clippy::items_after_statements, // OUR_KEYS const placed near first use for readability
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
)]

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use rayon::prelude::*;
use rsomics_common::{Result, RsomicsError};
use rsomics_stats::hwe_exact;

// ── Tag selector ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tags {
    pub an: bool,
    pub ac: bool,
    pub af: bool,
    pub maf: bool,
    pub ns: bool,
    pub ac_hom: bool,
    pub ac_het: bool,
    pub ac_hemi: bool,
    pub hwe: bool,
    pub exc_het: bool,
}

impl Tags {
    /// Parse a comma-separated tag list such as `"AN,AC,AF"`.
    pub fn from_list(list: &str) -> std::result::Result<Self, String> {
        let mut t = Tags::default();
        for token in list.split(',') {
            match token.trim() {
                "AN" => t.an = true,
                "AC" => t.ac = true,
                "AF" => t.af = true,
                "MAF" => t.maf = true,
                "NS" => t.ns = true,
                "AC_Hom" | "AC_HOM" => t.ac_hom = true,
                "AC_Het" | "AC_HET" => t.ac_het = true,
                "AC_Hemi" | "AC_HEMI" => t.ac_hemi = true,
                "HWE" => t.hwe = true,
                "ExcHet" | "EXC_HET" => t.exc_het = true,
                other => return Err(format!("unknown tag '{other}'")),
            }
        }
        Ok(t)
    }

    /// The default `bcftools +fill-tags` tag set (all 10 standard tags).
    #[must_use]
    pub fn default_set() -> Self {
        Tags {
            an: true,
            ac: true,
            af: true,
            maf: true,
            ns: true,
            ac_hom: true,
            ac_het: true,
            ac_hemi: true,
            hwe: true,
            exc_het: true,
        }
    }
}

// ── Per-record computation ────────────────────────────────────────────────────

/// Per-allele counts accumulated from GT fields.
#[derive(Clone, Default)]
struct AltCounts {
    /// Heterozygous carriers (one allele is this index, the other differs).
    nhet: u32,
    /// Homozygous carriers (every allele is this index).
    nhom: u32,
    /// Hemizygous carriers (single-allele GT carrying this index).
    nhemi: u32,
}

impl AltCounts {
    /// Total allele copies for AC / AN accounting.
    fn allele_count(&self) -> u32 {
        self.nhet + 2 * self.nhom + self.nhemi
    }
}

/// Parse a GT string into a list of allele indices; missing positions become `None`.
fn parse_gt(gt: &str) -> Vec<Option<u32>> {
    let sep = if gt.contains('|') { '|' } else { '/' };
    gt.split(sep)
        .map(|a| {
            if a == "." {
                None
            } else {
                a.parse::<u32>().ok()
            }
        })
        .collect()
}

struct SiteCounts {
    per_allele: Vec<AltCounts>,
    /// Samples with at least one non-missing allele.
    ns: u32,
    /// Total called allele copies (AN).
    an: u32,
    /// Count of homozygous-REF samples (counts[0].nhom in bcftools terms).
    n_hom_ref: u32,
}

fn count_site(n_alleles: usize, sample_gts: &[&str]) -> SiteCounts {
    let mut per_allele = vec![AltCounts::default(); n_alleles];
    let mut ns = 0u32;
    let mut an = 0u32;

    for gt_str in sample_gts {
        let gt_field = gt_str.split(':').next().unwrap_or(".");
        let alleles = parse_gt(gt_field);

        let called: Vec<u32> = alleles.iter().filter_map(|a| *a).collect();
        if called.is_empty() {
            continue;
        }
        ns += 1;

        let ploidy = alleles.len();
        let n_called = called.len();
        // bcftools default (drop_missing=false): half-missing → hemizygous.
        let is_hemi = n_called != ploidy;

        let mut distinct = called.clone();
        distinct.sort_unstable();
        distinct.dedup();
        let is_hom = distinct.len() == 1;

        an += u32::try_from(n_called).unwrap_or(u32::MAX);

        if is_hemi {
            for &idx in &called {
                if (idx as usize) < n_alleles {
                    per_allele[idx as usize].nhemi += 1;
                }
            }
        } else if is_hom {
            let idx = called[0] as usize;
            if idx < n_alleles {
                per_allele[idx].nhom += 1;
            }
        } else {
            // Each distinct allele gets one het credit.
            for &idx in &distinct {
                if (idx as usize) < n_alleles {
                    per_allele[idx as usize].nhet += 1;
                }
            }
        }
    }

    let n_hom_ref = per_allele[0].nhom;
    SiteCounts {
        per_allele,
        ns,
        an,
        n_hom_ref,
    }
}

/// Format a float to match `bcftools` VCF serialisation (`%.6g` on a C `float`).
///
/// bcftools stores AF/HWE/ExcHet internally as IEEE 754 `float` and emits them
/// with `%.6g`. Casting through `f32` and stripping trailing zeros replicates
/// the format closely enough for compat comparison.
fn fmt_g(v: f64) -> String {
    let f = v as f32;
    if f == 0.0 {
        return "0".to_owned();
    }
    // %.6g: up to 6 significant digits, no trailing zeros.
    // Approach: format with .6 fixed, then strip zeros (works for the [0,1] range of
    // probabilities and frequencies we emit).
    let repr = format!("{f:.6}");
    repr.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// Build the INFO field for one record, replacing any existing computed tags.
///
/// Existing key=val pairs whose keys are not in our tag set are preserved.
/// Computed tags are appended in canonical order: `AN`, `AC`, `AF`, `MAF`, `NS`,
/// `AC_Hom`, `AC_Het`, `AC_Hemi`, `HWE`, `ExcHet`.
#[must_use]
pub fn compute_info(
    existing_info: &str,
    n_alleles: usize,
    sample_gts: &[&str],
    tags: Tags,
) -> String {
    if n_alleles == 0 {
        return existing_info.to_owned();
    }

    let counts = count_site(n_alleles, sample_gts);
    let n_alt = n_alleles.saturating_sub(1);

    const OUR_KEYS: &[&str] = &[
        "AN", "AC", "AF", "MAF", "NS", "AC_Hom", "AC_Het", "AC_Hemi", "HWE", "ExcHet",
    ];

    let mut parts: Vec<String> = Vec::new();
    if existing_info != "." {
        for kv in existing_info.split(';') {
            let key = kv.split('=').next().unwrap_or(kv);
            if !OUR_KEYS.contains(&key) {
                parts.push(kv.to_owned());
            }
        }
    }

    if tags.an {
        parts.push(format!("AN={}", counts.an));
    }

    if tags.ac && n_alt > 0 {
        let vals: Vec<String> = (1..n_alleles)
            .map(|i| counts.per_allele[i].allele_count().to_string())
            .collect();
        parts.push(format!("AC={}", vals.join(",")));
    }

    if tags.af && n_alt > 0 {
        let vals: Vec<String> = (1..n_alleles)
            .map(|i| {
                if counts.an > 0 {
                    fmt_g(f64::from(counts.per_allele[i].allele_count()) / f64::from(counts.an))
                } else {
                    "0".to_owned()
                }
            })
            .collect();
        parts.push(format!("AF={}", vals.join(",")));
    }

    if tags.maf {
        let mut afs: Vec<f64> = (0..n_alleles)
            .map(|i| {
                if counts.an > 0 {
                    f64::from(counts.per_allele[i].allele_count()) / f64::from(counts.an)
                } else {
                    0.0
                }
            })
            .collect();
        afs.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let maf = afs.get(1).copied().unwrap_or(0.0);
        parts.push(format!("MAF={}", fmt_g(maf)));
    }

    if tags.ns {
        parts.push(format!("NS={}", counts.ns));
    }

    if tags.ac_hom && n_alt > 0 {
        // Allele copies in hom genotypes: each diploid hom contributes 2 copies.
        let vals: Vec<String> = (1..n_alleles)
            .map(|i| (2 * counts.per_allele[i].nhom).to_string())
            .collect();
        parts.push(format!("AC_Hom={}", vals.join(",")));
    }

    if tags.ac_het && n_alt > 0 {
        // Allele copies in het genotypes: each het contributes 1 copy.
        let vals: Vec<String> = (1..n_alleles)
            .map(|i| counts.per_allele[i].nhet.to_string())
            .collect();
        parts.push(format!("AC_Het={}", vals.join(",")));
    }

    if tags.ac_hemi && n_alt > 0 {
        // Allele copies in hemi genotypes: each hemi contributes 1 copy.
        let vals: Vec<String> = (1..n_alleles)
            .map(|i| counts.per_allele[i].nhemi.to_string())
            .collect();
        parts.push(format!("AC_Hemi={}", vals.join(",")));
    }

    if (tags.hwe || tags.exc_het) && n_alt > 0 {
        // bcftools parameterization (fill-tags.c, process_fmt) uses allele counts:
        //   nhom stores allele copies (2× diploid samples), nhet stores sample count
        //   per allele; total_nhet over all alleles = 2 × het_sample_count (each het
        //   contributes to nhet[ref] and nhet[alt]).
        //   nref_tot = 2*n_hom_ref_samples + total_nhet   (allele copies of REF side)
        //   nref_j   = nref_tot − nhet_j                  (ref allele copies for this alt)
        //   nalt_j   = nhet_j + 2*nhom_j                  (alt allele copies for this alt)
        let total_nhet: u32 = (0..n_alleles).map(|i| counts.per_allele[i].nhet).sum();
        let nref_tot = 2 * counts.n_hom_ref + total_nhet;

        let hwe_vals: Vec<(f64, f64)> = (1..n_alleles)
            .map(|i| {
                let c = &counts.per_allele[i];
                let nhet = c.nhet;
                let nref_j = nref_tot.saturating_sub(nhet);
                // nalt_j = nhet (sample count) + 2*nhom (allele copies from hom samples)
                let nalt_j = nhet + 2 * c.nhom;
                if nref_j == 0 || nalt_j == 0 {
                    (1.0_f64, 1.0_f64)
                } else {
                    hwe_exact(nref_j, nalt_j, nhet)
                }
            })
            .collect();

        if tags.hwe {
            let vals: Vec<String> = hwe_vals.iter().map(|(p, _)| fmt_g(*p)).collect();
            parts.push(format!("HWE={}", vals.join(",")));
        }
        if tags.exc_het {
            let vals: Vec<String> = hwe_vals.iter().map(|(_, q)| fmt_g(*q)).collect();
            parts.push(format!("ExcHet={}", vals.join(",")));
        }
    }

    if parts.is_empty() {
        ".".to_owned()
    } else {
        parts.join(";")
    }
}

// ── I/O driver ───────────────────────────────────────────────────────────────

pub struct FillTagsStats {
    pub total: u64,
    pub processed: u64,
}

/// Recompute INFO tags for every data record of `input`, writing plain VCF to `output`.
///
/// The header is updated with `##INFO` lines for every tag in `tags`.
/// Data records are processed in parallel (rayon) with output order preserved.
pub fn fill_tags(input: &Path, output: &mut dyn Write, tags: Tags) -> Result<FillTagsStats> {
    let raw = std::fs::read(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let data: Vec<u8> = if raw.starts_with(&[0x1f, 0x8b]) {
        let mut d = Vec::new();
        flate2::read::MultiGzDecoder::new(&raw[..])
            .read_to_end(&mut d)
            .map_err(RsomicsError::Io)?;
        d
    } else {
        raw
    };

    let reader = BufReader::new(&data[..]);

    let mut header_lines: Vec<String> = Vec::new();
    let mut data_lines: Vec<String> = Vec::new();

    for raw_line in reader.lines() {
        let line = raw_line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            header_lines.push(line);
        } else if !line.is_empty() {
            data_lines.push(line);
        }
    }

    // Emit header: strip existing ##INFO=<ID=...> lines for tags we recompute,
    // inject fresh ones immediately before the #CHROM line.
    let chrom_line = header_lines.pop();
    let tag_headers = build_tag_headers(tags);

    for h in &header_lines {
        let key = extract_info_id(h);
        let is_ours = key.is_some_and(|k| {
            matches!(
                k,
                "AN" | "AC"
                    | "AF"
                    | "MAF"
                    | "NS"
                    | "AC_Hom"
                    | "AC_Het"
                    | "AC_Hemi"
                    | "HWE"
                    | "ExcHet"
            )
        });
        if !is_ours {
            output.write_all(h.as_bytes()).map_err(RsomicsError::Io)?;
            output.write_all(b"\n").map_err(RsomicsError::Io)?;
        }
    }
    for th in &tag_headers {
        output.write_all(th.as_bytes()).map_err(RsomicsError::Io)?;
        output.write_all(b"\n").map_err(RsomicsError::Io)?;
    }
    if let Some(chrom) = chrom_line {
        output
            .write_all(chrom.as_bytes())
            .map_err(RsomicsError::Io)?;
        output.write_all(b"\n").map_err(RsomicsError::Io)?;
    }

    let total = data_lines.len() as u64;

    let processed_lines: Vec<String> = data_lines
        .par_iter()
        .map(|line| rewrite_record(line, tags))
        .collect();

    let mut processed = 0u64;
    for out_line in &processed_lines {
        output
            .write_all(out_line.as_bytes())
            .map_err(RsomicsError::Io)?;
        output.write_all(b"\n").map_err(RsomicsError::Io)?;
        processed += 1;
    }

    Ok(FillTagsStats { total, processed })
}

fn rewrite_record(line: &str, tags: Tags) -> String {
    // VCF: CHROM POS ID REF ALT QUAL FILTER INFO [FORMAT sample...]
    // splitn(9) keeps everything after the 8th tab as one field.
    let cols: Vec<&str> = line.splitn(9, '\t').collect();
    if cols.len() < 8 {
        return line.to_owned();
    }

    let alts = cols[4];
    let n_alleles = if alts == "." {
        1
    } else {
        alts.split(',').count() + 1
    };

    let sample_gts: Vec<&str> = if cols.len() > 8 {
        cols[8].split('\t').skip(1).collect()
    } else {
        Vec::new()
    };

    let new_info = compute_info(cols[7], n_alleles, &sample_gts, tags);

    let mut out = String::with_capacity(line.len() + 32);
    for (i, col) in cols.iter().enumerate() {
        if i > 0 {
            out.push('\t');
        }
        if i == 7 {
            out.push_str(&new_info);
        } else {
            out.push_str(col);
        }
    }
    out
}

fn build_tag_headers(tags: Tags) -> Vec<String> {
    let mut h = Vec::new();
    if tags.an {
        h.push(r#"##INFO=<ID=AN,Number=1,Type=Integer,Description="Total number of alleles in called genotypes">"#.to_owned());
    }
    if tags.ac {
        h.push(r#"##INFO=<ID=AC,Number=A,Type=Integer,Description="Allele count in genotypes, for each ALT allele, in the same order as listed">"#.to_owned());
    }
    if tags.af {
        h.push(r#"##INFO=<ID=AF,Number=A,Type=Float,Description="Allele frequency from FORMAT/GT or AC,AN">"#.to_owned());
    }
    if tags.maf {
        h.push(r#"##INFO=<ID=MAF,Number=1,Type=Float,Description="Frequency of the second most common allele">"#.to_owned());
    }
    if tags.ns {
        h.push(
            r#"##INFO=<ID=NS,Number=1,Type=Integer,Description="Number of samples with data">"#
                .to_owned(),
        );
    }
    if tags.ac_hom {
        h.push(r#"##INFO=<ID=AC_Hom,Number=A,Type=Integer,Description="Allele counts in homozygous genotypes">"#.to_owned());
    }
    if tags.ac_het {
        h.push(r#"##INFO=<ID=AC_Het,Number=A,Type=Integer,Description="Allele counts in heterozygous genotypes">"#.to_owned());
    }
    if tags.ac_hemi {
        h.push(r#"##INFO=<ID=AC_Hemi,Number=A,Type=Integer,Description="Allele counts in hemizygous genotypes">"#.to_owned());
    }
    if tags.hwe {
        h.push(
            r#"##INFO=<ID=HWE,Number=A,Type=Float,Description="HWE p-value (Wigginton 2005)">"#
                .to_owned(),
        );
    }
    if tags.exc_het {
        h.push(
            r#"##INFO=<ID=ExcHet,Number=A,Type=Float,Description="Excess heterozygosity p-value">"#
                .to_owned(),
        );
    }
    h
}

fn extract_info_id(line: &str) -> Option<&str> {
    if !line.starts_with("##INFO=<") {
        return None;
    }
    let rest = &line["##INFO=<".len()..];
    for part in rest.split(',') {
        if let Some(id) = part.strip_prefix("ID=") {
            return Some(id);
        }
    }
    None
}
