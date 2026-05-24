//! bigWig signal → per-region score matrix, matching deeptools `computeMatrix`
//! `reference-point` and `scale-regions` output.
//!
//! ## Output format (deeptools `heatmapper.save_matrix`)
//!
//! A gzipped file whose first line is `@` followed by a JSON dict of the
//! parameters (no spaces; keys in deeptools' fixed order; the per-sample
//! "special" params — upstream, downstream, body, bin size, ref point,
//! unscaled 5/3 prime — are emitted as one-element lists). Every subsequent
//! line is one region: `chrom`, comma-joined exon starts, comma-joined exon
//! ends, name, score, strand, then the per-bin signal values formatted with
//! Python `%f` (six decimals; missing → `nan`).
//!
//! ## Per-region binning (deeptools `coverage_from_big_wig` + `coverage_from_array`)
//!
//! For each region a reference point is chosen by mode and strand, two flank
//! spans are laid out around it, the bigWig is read per-base (NaN where the
//! file carries no data or the span runs off the chromosome), each flank is
//! partitioned into bins by `numpy.linspace(start, end, nbins, endpoint=False)`
//! truncated to int, and each bin's value is the NaN-masked mean of its bases.
//! Minus-strand regions read the flanks swapped and reverse the final row.
//! With `missing data as zero`, NaN bases become 0 before averaging.
//!
//! ### reference-point spans (`b` = upstream, `a` = downstream, refpoint `rp`)
//!
//! - plus strand: left flank `[rp-b, rp]` → `b/binSize` bins; right flank
//!   `[rp, rp+a]` → `a/binSize` bins.
//! - minus strand: left flank `[rp-a, rp]` → `a/binSize` bins; right flank
//!   `[rp, rp+b]` → `b/binSize` bins; the row is then reversed.
//!
//! `rp` is `start` (TSS), `end` (TES) or `(start+end)/2` (center) for the plus
//! strand; `end` (TSS), `start` (TES) or `(start+end)/2` (center) for minus.
//!
//! ### scale-regions spans
//!
//! upstream flank `[start-b, start]` (`b/binSize` bins), the region body
//! `[start, end]` scaled to `body/binSize` bins, downstream flank `[end, end+a]`
//! (`a/binSize` bins). Minus strand swaps the up/down flanks and reverses.

// Genomic coordinates and bin indices move freely between i64 / u32 / usize /
// f64 throughout the bin math; every value is bounded by a bigWig chromosome
// length (< u32::MAX), so these casts cannot truncate or lose sign in practice.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use std::io::Write;
use std::path::Path;

use flate2::Compression;
use flate2::write::GzEncoder;
use rsomics_common::{Result, RsomicsError};

mod bigwig;
use bigwig::BigWig;

/// Which point of each region anchors the flanks (reference-point mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefPoint {
    Tss,
    Tes,
    Center,
}

impl std::str::FromStr for RefPoint {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "TSS" => Ok(Self::Tss),
            "TES" => Ok(Self::Tes),
            "center" => Ok(Self::Center),
            _ => Err(format!("invalid --reference-point '{s}' (TSS|TES|center)")),
        }
    }
}

impl RefPoint {
    fn json(self) -> &'static str {
        match self {
            Self::Tss => "TSS",
            Self::Tes => "TES",
            Self::Center => "center",
        }
    }
}

/// The averaging statistic applied within each bin (deeptools `--averageTypeBins`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinAvg {
    Mean,
    Median,
    Min,
    Max,
    Std,
    Sum,
}

impl std::str::FromStr for BinAvg {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "mean" => Ok(Self::Mean),
            "median" => Ok(Self::Median),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "std" => Ok(Self::Std),
            "sum" => Ok(Self::Sum),
            _ => Err(format!(
                "invalid --average-type-bins '{s}' (mean|median|min|max|std|sum)"
            )),
        }
    }
}

impl BinAvg {
    fn json(self) -> &'static str {
        match self {
            Self::Mean => "mean",
            Self::Median => "median",
            Self::Min => "min",
            Self::Max => "max",
            Self::Std => "std",
            Self::Sum => "sum",
        }
    }
}

/// Which subcommand layout to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    ReferencePoint(RefPoint),
    ScaleRegions,
}

/// Knobs that drive matrix layout and value computation, mirroring the
/// deeptools parameter dict that ends up in the gzipped header.
#[derive(Debug, Clone)]
pub struct MatrixParams {
    pub mode: Mode,
    pub upstream: u32,
    pub downstream: u32,
    pub body: u32,
    pub bin_size: u32,
    pub bin_avg: BinAvg,
    pub missing_data_as_zero: bool,
    pub min_threshold: Option<f64>,
    pub max_threshold: Option<f64>,
    pub scale: f64,
    pub skip_zeros: bool,
    pub nan_after_end: bool,
    pub proc_number: usize,
    pub sample_label: String,
    pub group_label: String,
}

impl MatrixParams {
    fn ref_point(&self) -> Option<RefPoint> {
        match self.mode {
            Mode::ReferencePoint(rp) => Some(rp),
            Mode::ScaleRegions => None,
        }
    }

    /// Total per-bin columns per region, per sample.
    fn matrix_cols(&self) -> usize {
        ((self.downstream + self.upstream + self.body) / self.bin_size) as usize
    }

    /// Validate deeptools' multiple-of-binSize constraints (it `exit()`s on these).
    fn validate(&self) -> Result<()> {
        let bs = self.bin_size;
        if bs == 0 {
            return Err(RsomicsError::InvalidInput("--bin-size must be > 0".into()));
        }
        if !self.body.is_multiple_of(bs) {
            return Err(RsomicsError::InvalidInput(format!(
                "--region-body-length ({}) must be a multiple of --bin-size ({bs})",
                self.body
            )));
        }
        if !self.downstream.is_multiple_of(bs) {
            return Err(RsomicsError::InvalidInput(format!(
                "downstream length ({}) must be a multiple of --bin-size ({bs})",
                self.downstream
            )));
        }
        if !self.upstream.is_multiple_of(bs) {
            return Err(RsomicsError::InvalidInput(format!(
                "upstream length ({}) must be a multiple of --bin-size ({bs})",
                self.upstream
            )));
        }
        if matches!(self.mode, Mode::ReferencePoint(_))
            && self.upstream == 0
            && self.downstream == 0
        {
            return Err(RsomicsError::InvalidInput(
                "reference-point: upstream and downstream are both 0 — nothing to output".into(),
            ));
        }
        if matches!(self.mode, Mode::ScaleRegions) && self.body == 0 {
            return Err(RsomicsError::InvalidInput(
                "scale-regions: --region-body-length must be > 0".into(),
            ));
        }
        Ok(())
    }
}

/// One BED6 region. `score` is kept as the literal BED field so a `.` stays `.`
/// while a numeric value is re-emitted as deeptools' float (`0` → `0.0`).
#[derive(Debug, Clone)]
pub struct Region {
    pub chrom: String,
    pub start: u32,
    pub end: u32,
    pub name: String,
    pub score: String,
    pub strand: char,
}

/// Parse a BED file into BED6 regions. `#`-delimited multi-group BEDs are not
/// supported (single "genes" group only); a `#` line is a hard error so we
/// never silently mis-group.
pub fn read_bed(path: &Path) -> Result<Vec<Region>> {
    let text = std::fs::read_to_string(path).map_err(RsomicsError::Io)?;
    let mut regions = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') || line.starts_with("track") || line.starts_with("browser") {
            if line.starts_with('#') {
                return Err(RsomicsError::InvalidInput(
                    "'#'-delimited multi-group BED files are not supported".into(),
                ));
            }
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            return Err(RsomicsError::InvalidInput(format!(
                "BED line {} has fewer than 3 columns",
                lineno + 1
            )));
        }
        let chrom = f[0].to_string();
        let start: u32 = f[1].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("BED line {}: bad start", lineno + 1))
        })?;
        let end: u32 = f[2]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("BED line {}: bad end", lineno + 1)))?;
        let name = f.get(3).map_or_else(String::new, |s| (*s).to_string());
        let score = f
            .get(4)
            .map_or_else(|| "0".to_string(), |s| (*s).to_string());
        let strand = f.get(5).and_then(|s| s.chars().next()).unwrap_or('+');
        regions.push(Region {
            chrom,
            start,
            end,
            name,
            score,
            strand,
        });
    }
    Ok(regions)
}

/// A contiguous flank span and the number of bins it is partitioned into.
struct Flank {
    start: i64,
    end: i64,
    nbins: usize,
}

/// Lay out the (up to two) flanks for reference-point mode in genomic
/// coordinates, in the order deeptools reads them (before any strand reversal).
fn reference_point_flanks(r: &Region, p: &MatrixParams, rp: RefPoint) -> Vec<Flank> {
    let bs = i64::from(p.bin_size);
    let b = i64::from(p.upstream);
    let a = i64::from(p.downstream);
    let start = i64::from(r.start);
    let end = i64::from(r.end);
    let minus = r.strand == '-';

    let refpoint = match (rp, minus) {
        (RefPoint::Tss, false) | (RefPoint::Tes, true) => start,
        (RefPoint::Tes, false) | (RefPoint::Tss, true) => end,
        (RefPoint::Center, _) => i64::midpoint(start, end),
    };

    // Plus strand reads [rp-b, rp] then [rp, rp+a]; minus reads the flanks
    // swapped ([rp-a, rp] then [rp, rp+b]) and reverses the row afterwards.
    let (left_len, right_len) = if minus { (a, b) } else { (b, a) };
    let mut flanks = Vec::with_capacity(2);
    if left_len > 0 {
        flanks.push(Flank {
            start: refpoint - left_len,
            end: refpoint,
            nbins: (left_len / bs) as usize,
        });
    }
    if right_len > 0 {
        flanks.push(Flank {
            start: refpoint,
            end: refpoint + right_len,
            nbins: (right_len / bs) as usize,
        });
    }
    flanks
}

/// Lay out the flanks + scaled body for scale-regions mode (before reversal).
fn scale_regions_flanks(r: &Region, p: &MatrixParams) -> Vec<Flank> {
    let bs = i64::from(p.bin_size);
    let b = i64::from(p.upstream);
    let a = i64::from(p.downstream);
    let start = i64::from(r.start);
    let end = i64::from(r.end);
    let minus = r.strand == '-';

    // The body is scaled to body/binSize bins regardless of its bp width.
    let body_bins = (i64::from(p.body) / bs) as usize;
    // On the minus strand the upstream flank sits past the region end.
    let (up_len, down_len) = if minus { (a, b) } else { (b, a) };

    let mut flanks = Vec::with_capacity(3);
    if up_len > 0 {
        flanks.push(Flank {
            start: start - up_len,
            end: start,
            nbins: (up_len / bs) as usize,
        });
    }
    flanks.push(Flank {
        start,
        end,
        nbins: body_bins,
    });
    if down_len > 0 {
        flanks.push(Flank {
            start: end,
            end: end + down_len,
            nbins: (down_len / bs) as usize,
        });
    }
    flanks
}

/// NaN-masked statistic over a base slice, matching deeptools `my_average`.
/// An all-NaN (or empty) slice yields NaN.
fn bin_stat(vals: &[f64], avg: BinAvg, missing_as_zero: bool) -> f64 {
    if missing_as_zero {
        // Bases were already zero-filled upstream; operate on the raw slice.
        return reduce_stat(vals, avg);
    }
    let mut buf: Vec<f64> = Vec::with_capacity(vals.len());
    for &v in vals {
        if !v.is_nan() {
            buf.push(v);
        }
    }
    if buf.is_empty() {
        return f64::NAN;
    }
    reduce_stat(&buf, avg)
}

fn reduce_stat(vals: &[f64], avg: BinAvg) -> f64 {
    if vals.is_empty() {
        return f64::NAN;
    }
    match avg {
        BinAvg::Mean => vals.iter().sum::<f64>() / vals.len() as f64,
        BinAvg::Sum => vals.iter().sum(),
        BinAvg::Min => vals.iter().copied().fold(f64::INFINITY, f64::min),
        BinAvg::Max => vals.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        BinAvg::Std => {
            // numpy.ma.std defaults to population (ddof=0).
            let mean = vals.iter().sum::<f64>() / vals.len() as f64;
            let var = vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / vals.len() as f64;
            var.sqrt()
        }
        BinAvg::Median => {
            let mut s = vals.to_vec();
            s.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let n = s.len();
            if n % 2 == 1 {
                s[n / 2]
            } else {
                f64::midpoint(s[n / 2 - 1], s[n / 2])
            }
        }
    }
}

/// Partition a per-base value array into `nbins` bins via the deeptools
/// `linspace(valStart, valEnd, nbins, endpoint=False)` + final `valEnd` scheme,
/// reducing each bin with the chosen statistic. `val_start`/`val_end` index into
/// `values` (the concatenated per-base array for one flank).
fn bin_flank(
    values: &[f64],
    val_start: usize,
    val_end: usize,
    nbins: usize,
    avg: BinAvg,
    missing_as_zero: bool,
    out: &mut Vec<f64>,
) {
    if nbins == 0 {
        return;
    }
    let total = (val_end - val_start) as f64;
    // numpy.linspace(start, end, n, endpoint=False) computes a single
    // `step = total / n` then `start + idx * step`, truncating to int. The
    // `step`-multiply (not `idx/n*total`) is load-bearing: it reproduces
    // numpy's exact float rounding at bin boundaries.
    let step = total / nbins as f64;
    let pos = |idx: usize| -> usize {
        if idx >= nbins {
            val_end
        } else {
            val_start + (idx as f64 * step) as usize
        }
    };
    for idx in 0..nbins {
        let s = pos(idx);
        let e = pos(idx + 1).max(s + 1);
        out.push(bin_stat(&values[s..e], avg, missing_as_zero));
    }
}

/// Format one f64 the way deeptools' `np.char.mod('%f', ...)` does: six
/// decimals, or the literal `nan`.
fn fmt_value(v: f64, out: &mut String) {
    use std::fmt::Write;
    if v.is_nan() {
        out.push_str("nan");
    } else {
        let _ = write!(out, "{v:.6}");
    }
}

/// Normalise the BED score field to deeptools' emitted form: a parseable number
/// becomes its float repr (`0` → `0.0`, `3.5` → `3.5`), anything else (e.g. `.`)
/// is kept verbatim.
fn fmt_score(score: &str) -> String {
    match score.parse::<f64>() {
        Ok(f) => {
            // Exact integral test: matches Python's `str(float)` collapsing
            // `0.0`/`7.0` while keeping `3.5`.
            #[allow(clippy::float_cmp)]
            if f == f.trunc() && f.is_finite() {
                format!("{f:.1}")
            } else {
                // Python str(float) — Rust's default f64 Display matches for the
                // common decimal cases produced by BED scores.
                format!("{f}")
            }
        }
        Err(_) => score.to_string(),
    }
}

/// Build the `@`-prefixed JSON header line in deeptools' exact key order.
fn header_json(p: &MatrixParams, n_regions: usize) -> String {
    // scale-regions emits `"ref point":[null]`; reference-point emits `["TSS"]`.
    let rp = p
        .ref_point()
        .map_or("[null]".to_string(), |r| format!("[\"{}\"]", r.json()));
    let q = |b: bool| if b { "true" } else { "false" };
    let opt = |o: Option<f64>| o.map_or("null".to_string(), fmt_json_num);
    let body = i64::from(p.body);
    let sample = serde_json::to_string(&p.sample_label).unwrap();
    let group = serde_json::to_string(&p.group_label).unwrap();
    format!(
        "@{{\"upstream\":[{up}],\"downstream\":[{down}],\"body\":[{body}],\"bin size\":[{bs}],\
\"ref point\":{rp},\"verbose\":false,\"bin avg type\":\"{avg}\",\
\"missing data as zero\":{mdz},\"min threshold\":{mint},\"max threshold\":{maxt},\
\"scale\":{scale},\"skip zeros\":{skip},\"nan after end\":{nae},\
\"proc number\":{proc},\"sort regions\":\"keep\",\"sort using\":\"mean\",\
\"unscaled 5 prime\":[0],\"unscaled 3 prime\":[0],\
\"group_labels\":[{group}],\"group_boundaries\":[0,{nreg}],\
\"sample_labels\":[{sample}],\"sample_boundaries\":[0,{cols}]}}",
        up = p.upstream,
        down = p.downstream,
        bs = p.bin_size,
        avg = p.bin_avg.json(),
        mdz = q(p.missing_data_as_zero),
        mint = opt(p.min_threshold),
        maxt = opt(p.max_threshold),
        scale = fmt_json_num(p.scale),
        skip = q(p.skip_zeros),
        nae = q(p.nan_after_end),
        proc = p.proc_number,
        nreg = n_regions,
        cols = p.matrix_cols(),
    )
}

/// JSON-serialise a float the way Python's `json.dumps` would: integral values
/// drop the fraction (`1.0` → `1`), others use the shortest round-trip repr.
fn fmt_json_num(v: f64) -> String {
    // Exact integral test mirrors Python's json int/float emission.
    #[allow(clippy::float_cmp)]
    if v == v.trunc() && v.is_finite() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// One computed region: its source plus the binned row (rows that survive
/// thresholding/skip-zeros).
struct ComputedRegion {
    region: Region,
    row: Vec<f64>,
}

/// Compute the matrix and write the gzipped deeptools-format file.
///
/// Returns `(n_regions_written, n_regions_no_score)`.
pub fn compute_matrix(
    bigwig: &Path,
    regions: &[Region],
    params: &MatrixParams,
    out: &Path,
) -> Result<(usize, usize)> {
    params.validate()?;

    let mut bw = BigWig::open(bigwig)?;

    let mut computed: Vec<ComputedRegion> = Vec::with_capacity(regions.len());
    let mut no_score = 0usize;

    for r in regions {
        let minus = r.strand == '-';
        let flanks = match params.mode {
            Mode::ReferencePoint(rp) => reference_point_flanks(r, params, rp),
            Mode::ScaleRegions => scale_regions_flanks(r, params),
        };

        let chrom_len = bw.chrom_len(&r.chrom);
        let present = chrom_len.is_some();

        // scale-regions with a body shorter than one bin can't be binned;
        // deeptools emits an all-NaN row (all-zero with missing-data-as-zero).
        let body_too_short = matches!(params.mode, Mode::ScaleRegions)
            && params.body > 0
            && (r.end - r.start) < params.bin_size;

        let mut row = if !present {
            no_score += 1;
            vec![f64::NAN; params.matrix_cols()]
        } else if body_too_short {
            let fill = if params.missing_data_as_zero {
                0.0
            } else {
                f64::NAN
            };
            vec![fill; params.matrix_cols()]
        } else {
            compute_row(&mut bw, &r.chrom, chrom_len, &flanks, params, minus)?
        };

        // deeptools applies scale, then min/max thresholds on the (nan→0) view.
        if (params.scale - 1.0).abs() > f64::EPSILON {
            for v in &mut row {
                *v *= params.scale;
            }
        }
        let mut as_zero = row.clone();
        for v in &mut as_zero {
            if v.is_nan() {
                *v = 0.0;
            }
        }
        if let Some(mt) = params.min_threshold
            && as_zero.iter().copied().fold(f64::INFINITY, f64::min) <= mt
        {
            continue;
        }
        if let Some(mt) = params.max_threshold
            && as_zero.iter().copied().fold(f64::NEG_INFINITY, f64::max) >= mt
        {
            continue;
        }

        computed.push(ComputedRegion {
            region: r.clone(),
            row,
        });
    }

    if params.skip_zeros {
        computed.retain(|c| {
            let valid: Vec<f64> = c.row.iter().copied().filter(|v| !v.is_nan()).collect();
            if valid.is_empty() {
                return false;
            }
            let mean = valid.iter().sum::<f64>() / valid.len() as f64;
            #[allow(clippy::float_cmp)]
            {
                mean != 0.0
            }
        });
    }

    write_matrix(out, params, &computed)?;
    Ok((computed.len(), no_score))
}

/// Per-region row computation, reading the bigWig per flank.
fn compute_row(
    bw: &mut BigWig,
    chrom: &str,
    chrom_len: Option<u32>,
    flanks: &[Flank],
    p: &MatrixParams,
    minus: bool,
) -> Result<Vec<f64>> {
    let total_bases: usize = flanks
        .iter()
        .map(|f| (f.end - f.start).max(0) as usize)
        .sum();
    let mut values = vec![f64::NAN; total_bases];
    let clen = chrom_len.map_or(i64::MAX, i64::from);

    let mut cursor = 0usize;
    let mut bounds: Vec<(usize, usize, usize)> = Vec::with_capacity(flanks.len());
    for f in flanks {
        let width = (f.end - f.start).max(0) as usize;
        let flank_start = cursor;
        let qs = f.start.max(0);
        let qe = f.end.min(clen);
        if qs < qe
            && let Some(vals) = bw.values(chrom, qs as u32, qe as u32)?
        {
            let offset = (qs - f.start) as usize;
            for (i, v) in vals.into_iter().enumerate() {
                values[flank_start + offset + i] = f64::from(v);
            }
        }
        cursor += width;
        bounds.push((flank_start, cursor, f.nbins));
    }

    if p.missing_data_as_zero {
        for v in &mut values {
            if v.is_nan() {
                *v = 0.0;
            }
        }
    }

    let mut row = Vec::with_capacity(p.matrix_cols());
    for (s, e, nbins) in bounds {
        bin_flank(
            &values,
            s,
            e,
            nbins,
            p.bin_avg,
            p.missing_data_as_zero,
            &mut row,
        );
    }
    if minus {
        row.reverse();
    }
    Ok(row)
}

fn write_matrix(out: &Path, p: &MatrixParams, computed: &[ComputedRegion]) -> Result<()> {
    let file = std::fs::File::create(out).map_err(RsomicsError::Io)?;
    let mut gz = GzEncoder::new(std::io::BufWriter::new(file), Compression::default());

    let header = header_json(p, computed.len());
    gz.write_all(header.as_bytes()).map_err(RsomicsError::Io)?;
    gz.write_all(b"\n").map_err(RsomicsError::Io)?;

    let mut line = String::with_capacity(p.matrix_cols() * 12 + 64);
    for c in computed {
        line.clear();
        line.push_str(&c.region.chrom);
        line.push('\t');
        // Single-exon BED6: starts/ends are single values.
        line.push_str(&c.region.start.to_string());
        line.push('\t');
        line.push_str(&c.region.end.to_string());
        line.push('\t');
        line.push_str(&c.region.name);
        line.push('\t');
        line.push_str(&fmt_score(&c.region.score));
        line.push('\t');
        line.push(c.region.strand);
        for &v in &c.row {
            line.push('\t');
            fmt_value(v, &mut line);
        }
        line.push('\n');
        gz.write_all(line.as_bytes()).map_err(RsomicsError::Io)?;
    }
    gz.finish().map_err(RsomicsError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linspace_partition_even() {
        // width 1000, 20 bins → 50-bp bins.
        let mut out = Vec::new();
        let vals: Vec<f64> = (0..1000).map(f64::from).collect();
        bin_flank(&vals, 0, 1000, 20, BinAvg::Mean, false, &mut out);
        assert_eq!(out.len(), 20);
        // First bin mean of 0..50 = 24.5.
        assert!((out[0] - 24.5).abs() < 1e-9);
    }

    #[test]
    fn linspace_partition_uneven() {
        // width 1000, 7 bins: first bin spans [0,142).
        let mut out = Vec::new();
        let vals: Vec<f64> = (0..1000).map(|_| 1.0).collect();
        bin_flank(&vals, 0, 1000, 7, BinAvg::Mean, false, &mut out);
        assert_eq!(out.len(), 7);
        assert!(out.iter().all(|v| (v - 1.0).abs() < 1e-9));
    }

    #[test]
    fn nan_masked_mean() {
        let vals = [1.0, f64::NAN, 3.0];
        assert!((bin_stat(&vals, BinAvg::Mean, false) - 2.0).abs() < 1e-9);
        assert!(bin_stat(&[f64::NAN, f64::NAN], BinAvg::Mean, false).is_nan());
    }

    #[test]
    fn missing_as_zero_mean() {
        let vals = [1.0, 0.0, 3.0];
        assert!((bin_stat(&vals, BinAvg::Mean, true) - (4.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn score_formatting() {
        assert_eq!(fmt_score("0"), "0.0");
        assert_eq!(fmt_score("7"), "7.0");
        assert_eq!(fmt_score("3.5"), "3.5");
        assert_eq!(fmt_score("."), ".");
    }

    #[test]
    fn value_formatting() {
        let mut s = String::new();
        fmt_value(3.0, &mut s);
        assert_eq!(s, "3.000000");
        s.clear();
        fmt_value(f64::NAN, &mut s);
        assert_eq!(s, "nan");
    }

    #[test]
    fn json_num() {
        assert_eq!(fmt_json_num(1.0), "1");
        assert_eq!(fmt_json_num(0.5), "0.5");
    }

    #[test]
    fn refpoint_tss_plus_spans() {
        let r = Region {
            chrom: "c".into(),
            start: 5000,
            end: 5200,
            name: "x".into(),
            score: "0".into(),
            strand: '+',
        };
        let p = base_params(Mode::ReferencePoint(RefPoint::Tss), 1000, 1000, 0, 50);
        let f = reference_point_flanks(&r, &p, RefPoint::Tss);
        assert_eq!(f.len(), 2);
        assert_eq!((f[0].start, f[0].end, f[0].nbins), (4000, 5000, 20));
        assert_eq!((f[1].start, f[1].end, f[1].nbins), (5000, 6000, 20));
    }

    #[test]
    fn refpoint_tss_minus_spans() {
        let r = Region {
            chrom: "c".into(),
            start: 6000,
            end: 6200,
            name: "x".into(),
            score: "0".into(),
            strand: '-',
        };
        let p = base_params(Mode::ReferencePoint(RefPoint::Tss), 200, 600, 0, 50);
        let f = reference_point_flanks(&r, &p, RefPoint::Tss);
        // minus TSS refpoint = end = 6200; left=[rp-a,rp]=[5600,6200], right=[rp,rp+b]=[6200,6400].
        assert_eq!((f[0].start, f[0].end, f[0].nbins), (5600, 6200, 12));
        assert_eq!((f[1].start, f[1].end, f[1].nbins), (6200, 6400, 4));
    }

    fn base_params(mode: Mode, up: u32, down: u32, body: u32, bs: u32) -> MatrixParams {
        MatrixParams {
            mode,
            upstream: up,
            downstream: down,
            body,
            bin_size: bs,
            bin_avg: BinAvg::Mean,
            missing_data_as_zero: false,
            min_threshold: None,
            max_threshold: None,
            scale: 1.0,
            skip_zeros: false,
            nan_after_end: false,
            proc_number: 1,
            sample_label: "s".into(),
            group_label: "genes".into(),
        }
    }
}
