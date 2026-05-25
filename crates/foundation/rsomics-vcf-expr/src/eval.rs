//! Evaluator: runs a parsed Expr against a VCF record line (text representation).
//!
//! The evaluator operates on raw VCF tab-split text — no noodles dependency —
//! keeping the crate in Quadrant ① (pure Rust, no FFI).
//!
//! ## Evaluation model
//!
//! A VCF data line has CHROM/POS/ID/REF/ALT/QUAL/FILTER/INFO/FORMAT/sample...
//! columns.  Fields that are sample-level (FMT/*) are evaluated per-sample,
//! yielding one boolean per sample.  Fields that are site-level (QUAL, INFO/*)
//! produce one boolean that propagates identically to all samples.
//!
//! `SampleResult` reports per-sample pass/fail.  For site-level expressions
//! every sample gets the same value.  The caller (setgt -t q) iterates
//! `SampleResult` to decide which samples to rewrite.

use std::fmt;

use crate::parse::{CmpOp, Expr, FieldRef, LogOp, Value};

// LogOp semantics (bcftools-faithful):
// - `&`  (And)    : per-sample TRUE AND — sample passes if BOTH sub-exprs are true for it
// - `&&` (AndVec) : per-sample OR      — sample passes if EITHER sub-expr is true for it
//                   (bcftools "sample-wise AND" allows different samples to satisfy each arm)
// - `|`  (Or)     : site-level OR — same per-sample semantics as AndVec for setGT
// - `||` (OrVec)  : same as Or

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    MalformedLine(String),
    FieldNotFound(String),
    TypeMismatch {
        field: String,
        expected: &'static str,
        got: String,
    },
    Internal(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::MalformedLine(s) => write!(f, "malformed VCF line: {s}"),
            EvalError::FieldNotFound(s) => write!(f, "field not found: {s}"),
            EvalError::TypeMismatch {
                field,
                expected,
                got,
            } => {
                write!(
                    f,
                    "type mismatch for {field}: expected {expected}, got '{got}'"
                )
            }
            EvalError::Internal(s) => write!(f, "internal evaluator error: {s}"),
        }
    }
}

impl std::error::Error for EvalError {}

/// Return the `n`-th colon-delimited field (0-indexed) of a sample column
/// without allocating a Vec.
#[inline]
fn nth_colon_field(s: &str, n: usize) -> Option<&str> {
    if n == 0 {
        let end = s.find(':').unwrap_or(s.len());
        Some(&s[..end])
    } else {
        let mut skipped = 0usize;
        let mut start = 0usize;
        for (i, &b) in s.as_bytes().iter().enumerate() {
            if b == b':' {
                skipped += 1;
                if skipped == n {
                    start = i + 1;
                    break;
                }
            }
        }
        if skipped < n {
            return None;
        }
        let end = s[start..].find(':').map_or(s.len(), |p| start + p);
        Some(&s[start..end])
    }
}

/// Per-sample evaluation result from `eval_expr`.
#[derive(Debug, Clone)]
pub struct SampleResult {
    /// `true` if this sample passes the filter expression.
    pub pass: Vec<bool>,
}

/// Parsed VCF line split into its fixed columns + format/sample columns.
struct VcfLine<'a> {
    qual: &'a str,
    filter: &'a str,
    info: &'a str,
    /// FORMAT column split by ':' into individual tag names, pre-computed once per line.
    fmt_keys: Vec<&'a str>,
    /// Start byte offset of each sample column inside `line`.  `sample_ends[i]`
    /// is the exclusive end of sample `i` (or `line.len()` for the last sample).
    sample_starts: Vec<usize>,
    sample_ends: Vec<usize>,
    line: &'a str,
}

impl<'a> VcfLine<'a> {
    fn parse(line: &'a str) -> Result<Self, EvalError> {
        // Scan tabs to locate the 9 fixed column boundaries and all sample columns.
        let bytes = line.as_bytes();
        let mut tab_pos = [0usize; 9];
        let mut ntabs = 0usize;
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\t' {
                if ntabs < 9 {
                    tab_pos[ntabs] = i;
                }
                ntabs += 1;
            }
        }
        if ntabs < 7 {
            return Err(EvalError::MalformedLine(format!(
                "expected ≥8 tab-separated columns, got {}",
                ntabs + 1
            )));
        }

        let qual = &line[tab_pos[4] + 1..tab_pos[5]];
        let filter = &line[tab_pos[5] + 1..tab_pos[6]];
        let info = &line[tab_pos[6] + 1..tab_pos[7]];

        let fmt_keys: Vec<&'a str> = if ntabs >= 8 {
            let fmt_start = tab_pos[7] + 1;
            let fmt_end = tab_pos[8];
            line[fmt_start..fmt_end].split(':').collect()
        } else {
            vec![]
        };

        // Pre-compute sample column start/end byte offsets — O(line_len) once,
        // then per-sample field lookup is O(field_width) rather than O(sample_idx × line_len).
        let (sample_starts, sample_ends) = if ntabs >= 8 && tab_pos[8] < line.len() {
            let first_sample_start = tab_pos[8] + 1;
            let mut starts = Vec::with_capacity(ntabs.saturating_sub(8));
            let mut ends = Vec::with_capacity(ntabs.saturating_sub(8));
            let mut cur = first_sample_start;
            for (i, &b) in bytes[first_sample_start..].iter().enumerate() {
                if b == b'\t' {
                    starts.push(cur);
                    ends.push(first_sample_start + i);
                    cur = first_sample_start + i + 1;
                }
            }
            // Last sample: no trailing tab
            starts.push(cur);
            ends.push(line.len());
            (starts, ends)
        } else {
            (vec![], vec![])
        };

        Ok(VcfLine {
            qual,
            filter,
            info,
            fmt_keys,
            sample_starts,
            sample_ends,
            line,
        })
    }

    /// Number of samples in this line.
    fn n_samples(&self) -> usize {
        self.sample_starts.len()
    }

    /// Return the raw text of sample column `sample_idx` (0-based). O(1).
    fn sample_col(&self, sample_idx: usize) -> Option<&'a str> {
        let start = *self.sample_starts.get(sample_idx)?;
        let end = *self.sample_ends.get(sample_idx)?;
        Some(&self.line[start..end])
    }

    /// Retrieve the value of a FORMAT tag for a specific sample (0-indexed).
    /// Returns `None` if the tag is absent from FORMAT or the sample value is `.`.
    fn fmt_value(&self, tag: &str, sample_idx: usize) -> Option<&'a str> {
        let field_pos = self.fmt_keys.iter().position(|&k| k == tag)?;
        let sample_col = self.sample_col(sample_idx)?;
        nth_colon_field(sample_col, field_pos).filter(|&v| v != ".")
    }

    /// Retrieve a numeric FORMAT value for one sample.
    fn fmt_num(&self, tag: &str, sample_idx: usize) -> Result<Option<f64>, EvalError> {
        match self.fmt_value(tag, sample_idx) {
            None => Ok(None),
            Some(v) => v
                .parse::<f64>()
                .map(Some)
                .map_err(|_| EvalError::TypeMismatch {
                    field: format!("FMT/{tag}"),
                    expected: "numeric",
                    got: v.to_owned(),
                }),
        }
    }

    /// Retrieve the string GT value for one sample (the raw field before any ':').
    fn gt_str(&self, sample_idx: usize) -> Option<&'a str> {
        let gt_pos = self.fmt_keys.iter().position(|&k| k == "GT")?;
        let sample_col = self.sample_col(sample_idx)?;
        nth_colon_field(sample_col, gt_pos)
    }

    /// Retrieve a numeric INFO field value. Only the first value is returned
    /// for multi-value fields.
    fn info_num(&self, tag: &str) -> Result<Option<f64>, EvalError> {
        for entry in self.info.split(';') {
            if entry == "." {
                continue;
            }
            if let Some((k, v)) = entry.split_once('=') {
                if k.eq_ignore_ascii_case(tag) {
                    let first = v.split(',').next().unwrap_or(v);
                    return first
                        .parse::<f64>()
                        .map(Some)
                        .map_err(|_| EvalError::TypeMismatch {
                            field: format!("INFO/{tag}"),
                            expected: "numeric",
                            got: first.to_owned(),
                        });
                }
            } else if entry.eq_ignore_ascii_case(tag) {
                // Flag INFO entry (no value) — treat as 1.0 for numeric comparison.
                return Ok(Some(1.0));
            }
        }
        Ok(None)
    }

    /// Retrieve QUAL as f64.  Returns `None` for missing (`.`).
    fn qual_num(&self) -> Result<Option<f64>, EvalError> {
        if self.qual == "." {
            return Ok(None);
        }
        self.qual
            .parse::<f64>()
            .map(Some)
            .map_err(|_| EvalError::TypeMismatch {
                field: "QUAL".into(),
                expected: "numeric",
                got: self.qual.to_owned(),
            })
    }
}

// ── GT matching semantics ─────────────────────────────────────────────────────

/// Classify a raw GT string (e.g. `0/1`, `./.`, `1|1`) into categories.
fn gt_classify(gt: &str) -> GtClass {
    let alleles: Vec<&str> = gt.split(['/', '|']).collect();
    let n = alleles.len();
    let n_miss = alleles.iter().filter(|&&a| a == ".").count();
    if n_miss == n {
        return GtClass::Missing;
    }
    if n_miss > 0 {
        return GtClass::PartialMiss;
    }
    let all_ref = alleles.iter().all(|&a| a == "0");
    let any_ref = alleles.contains(&"0");
    let any_alt = alleles.iter().any(|&a| a != "0");
    let all_same = alleles.windows(2).all(|w| w[0] == w[1]);

    if all_ref {
        GtClass::HomRef
    } else if !any_ref && all_same {
        GtClass::HomAlt
    } else if n == 1 {
        GtClass::Haploid
    } else if any_ref && any_alt {
        GtClass::Het
    } else {
        // multi-alt het or other
        GtClass::Het
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GtClass {
    Missing,
    PartialMiss,
    HomRef,
    HomAlt,
    Het,
    Haploid,
}

/// Evaluate `GT == <str_val>` or `GT != <str_val>` for a single sample.
///
/// Supported string values match bcftools semantics:
/// - `"."` / `"miss"` / `"missing"` → any missing (full or partial)
/// - `"hom"` → homozygous (ref or alt)
/// - `"het"` → heterozygous
/// - `"ref"` → homozygous ref (0/0 etc.)
/// - `"alt"` → homozygous alt (non-ref)
/// - `"hap"` → haploid
/// - Bare allele string (e.g. `"0/1"`) → literal match
fn eval_gt_str(gt: &str, op: &CmpOp, pattern: &str) -> bool {
    let class = gt_classify(gt);
    let matches_pattern = match pattern.to_ascii_lowercase().as_str() {
        "." | "miss" | "missing" => {
            matches!(class, GtClass::Missing | GtClass::PartialMiss)
        }
        "hom" => matches!(class, GtClass::HomRef | GtClass::HomAlt),
        "het" => class == GtClass::Het,
        "ref" => class == GtClass::HomRef,
        "alt" => class == GtClass::HomAlt,
        "hap" => class == GtClass::Haploid,
        other => {
            // Literal genotype comparison — normalize separators for comparison.
            let norm_gt: String = gt.chars().map(|c| if c == '|' { '/' } else { c }).collect();
            let norm_pat: String = other
                .chars()
                .map(|c| if c == '|' { '/' } else { c })
                .collect();
            norm_gt == norm_pat
        }
    };
    match op {
        CmpOp::Eq => matches_pattern,
        CmpOp::Ne => !matches_pattern,
        _ => false, // GT only supports == and != in bcftools
    }
}

// ── Numeric comparison helper ─────────────────────────────────────────────────

fn cmp_num(lhs: f64, op: &CmpOp, rhs: f64) -> bool {
    match op {
        CmpOp::Lt => lhs < rhs,
        CmpOp::Le => lhs <= rhs,
        CmpOp::Gt => lhs > rhs,
        CmpOp::Ge => lhs >= rhs,
        CmpOp::Eq => (lhs - rhs).abs() < f64::EPSILON,
        CmpOp::Ne => (lhs - rhs).abs() >= f64::EPSILON,
    }
}

fn cmp_str(lhs: &str, op: &CmpOp, rhs: &str) -> bool {
    match op {
        CmpOp::Eq => lhs == rhs,
        CmpOp::Ne => lhs != rhs,
        _ => false,
    }
}

// ── Core evaluator ────────────────────────────────────────────────────────────

/// Evaluate a single Cmp node for one sample.
///
/// Missing field values: bcftools treats missing as "not matching" for `<` / `>` etc.,
/// and matching for `==` against `"."`.  We implement: missing numeric → eval returns `false`
/// for all comparisons (conservative / bcftools default).
fn eval_cmp_sample(
    vcf: &VcfLine<'_>,
    field: &FieldRef,
    op: &CmpOp,
    val: &Value,
    sample_idx: usize,
) -> Result<bool, EvalError> {
    match field {
        FieldRef::Qual => {
            let q = vcf.qual_num()?;
            match val {
                Value::Num(threshold) => Ok(q.is_some_and(|v| cmp_num(v, op, *threshold))),
                Value::Str(s) => Err(EvalError::TypeMismatch {
                    field: "QUAL".into(),
                    expected: "numeric",
                    got: s.clone(),
                }),
            }
        }

        FieldRef::Filter => {
            let filter_val = vcf.filter;
            match val {
                Value::Str(s) => Ok(cmp_str(filter_val, op, s)),
                Value::Num(n) => Err(EvalError::TypeMismatch {
                    field: "FILTER".into(),
                    expected: "string",
                    got: n.to_string(),
                }),
            }
        }

        FieldRef::Gt => {
            let Some(gt) = vcf.gt_str(sample_idx) else {
                return Ok(false);
            };
            match val {
                Value::Str(pattern) => Ok(eval_gt_str(gt, op, pattern)),
                Value::Num(_) => Ok(false),
            }
        }

        FieldRef::Fmt(tag) => {
            match val {
                Value::Num(threshold) => {
                    let v = vcf.fmt_num(tag, sample_idx)?;
                    Ok(v.is_some_and(|n| cmp_num(n, op, *threshold)))
                }
                Value::Str(s) => {
                    // String comparison for FORMAT field (e.g. GT handled above,
                    // but allow literal string match for other fields).
                    let raw = vcf.fmt_value(tag, sample_idx);
                    Ok(raw.is_some_and(|v| cmp_str(v, op, s)))
                }
            }
        }

        FieldRef::Info(tag) => {
            // Try FORMAT first (bare tag fallback per bcftools behaviour), then INFO.
            match val {
                Value::Num(threshold) => {
                    // Check FORMAT first.
                    if let Ok(Some(v)) = vcf.fmt_num(tag, sample_idx) {
                        return Ok(cmp_num(v, op, *threshold));
                    }
                    // Fall back to INFO.
                    let v = vcf.info_num(tag)?;
                    Ok(v.is_some_and(|n| cmp_num(n, op, *threshold)))
                }
                Value::Str(s) => {
                    let raw = vcf.fmt_value(tag, sample_idx);
                    if let Some(v) = raw {
                        return Ok(cmp_str(v, op, s));
                    }
                    // INFO string match not commonly needed but fall through.
                    Ok(false)
                }
            }
        }
    }
}

/// Recursively evaluate an Expr for one sample.
fn eval_one(expr: &Expr, vcf: &VcfLine<'_>, sample_idx: usize) -> Result<bool, EvalError> {
    match expr {
        Expr::Cmp { field, op, val } => eval_cmp_sample(vcf, field, op, val, sample_idx),
        Expr::Paren(inner) => eval_one(inner.as_ref(), vcf, sample_idx),
        Expr::Logic { op, lhs, rhs } => {
            let l = eval_one(lhs, vcf, sample_idx)?;
            match op {
                // `&`: per-sample true AND — both arms must be true for THIS sample.
                LogOp::And => {
                    if l {
                        eval_one(rhs, vcf, sample_idx)
                    } else {
                        Ok(false)
                    }
                }
                // `&&` / `|` / `||`: per-sample OR — sample passes if EITHER arm is true.
                // `&&` is bcftools "vec-and" which confusingly acts as per-sample OR.
                LogOp::AndVec | LogOp::Or | LogOp::OrVec => {
                    if l {
                        Ok(true)
                    } else {
                        eval_one(rhs, vcf, sample_idx)
                    }
                }
            }
        }
    }
}

/// Evaluate `expr` against a raw VCF data line, returning per-sample pass/fail.
///
/// `n_samples` must equal the number of sample columns in the line.  Pass 0
/// for site-level-only expressions (QUAL/INFO) — the result will be a single-
/// element vec that all samples can read.
///
/// Missing field values → `false` (conservative, matches bcftools default).
pub fn eval_expr(expr: &Expr, line: &str, n_samples: usize) -> Result<SampleResult, EvalError> {
    let vcf = VcfLine::parse(line)?;
    let actual_n = vcf.n_samples();
    // If the line has samples, use that count; otherwise fall back to caller hint.
    let count = if actual_n > 0 {
        actual_n
    } else {
        n_samples.max(1)
    };
    let mut pass = Vec::with_capacity(count);
    for i in 0..count {
        pass.push(eval_one(expr, &vcf, i)?);
    }
    Ok(SampleResult { pass })
}

/// Convenience wrapper: evaluate `expr` against a raw VCF line and return
/// per-sample booleans.  Equivalent to `eval_expr` but returns `EvalError`
/// wrapped in a `Box<dyn Error>`.
pub struct EvalContext {
    pub expr: Expr,
    pub negate: bool,
}

impl EvalContext {
    /// `negate = true` for `-e` (exclude) mode: a sample PASSES if the expr is FALSE.
    #[must_use]
    pub fn new(expr: Expr, negate: bool) -> Self {
        Self { expr, negate }
    }

    /// Evaluate against one VCF data line; returns per-sample boolean (true → rewrite).
    pub fn eval_line(&self, line: &str, n_samples: usize) -> Result<SampleResult, EvalError> {
        let mut result = eval_expr(&self.expr, line, n_samples)?;
        if self.negate {
            for p in &mut result.pass {
                *p = !*p;
            }
        }
        Ok(result)
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_expr;

    fn make_line(format: &str, samples: &[&str]) -> String {
        let sample_part = samples.join("\t");
        format!("chr1\t100\t.\tA\tT\t50\tPASS\t.\t{format}\t{sample_part}")
    }

    #[test]
    fn fmt_dp_lt_selects_low_dp() {
        // S1: DP=3 < 5 → true; S2: DP=10 not < 5 → false; S3: DP=0 < 5 → true
        let line = make_line("GT:DP", &["0/1:3", "0/0:10", "./.:0"]);
        let expr = parse_expr("FMT/DP<5").unwrap();
        let res = eval_expr(&expr, &line, 3).unwrap();
        assert_eq!(res.pass, vec![true, false, true]);
    }

    #[test]
    fn fmt_dp_missing_returns_false() {
        // DP="." is treated as missing → false for any numeric comparison
        let line = make_line("GT:DP", &["0/1:.", "0/0:10"]);
        let expr = parse_expr("FMT/DP<5").unwrap();
        let res = eval_expr(&expr, &line, 2).unwrap();
        assert_eq!(res.pass, vec![false, false]);
    }

    #[test]
    fn qual_ge_site_level() {
        let line = make_line("GT", &["0/1", "0/0"]);
        let expr = parse_expr("QUAL>=30").unwrap();
        let res = eval_expr(&expr, &line, 2).unwrap();
        // QUAL=50, both samples get same site-level result
        assert_eq!(res.pass, vec![true, true]);
    }

    #[test]
    fn gt_eq_missing() {
        let line = make_line("GT:DP", &["0/1:10", "./.:5", "0/0:20"]);
        let expr = parse_expr(r#"GT=".""#).unwrap();
        let res = eval_expr(&expr, &line, 3).unwrap();
        assert_eq!(res.pass, vec![false, true, false]);
    }

    #[test]
    fn gt_eq_hom() {
        let line = make_line("GT", &["0/0", "0/1", "1/1"]);
        let expr = parse_expr(r#"GT="hom""#).unwrap();
        let res = eval_expr(&expr, &line, 3).unwrap();
        assert_eq!(res.pass, vec![true, false, true]);
    }

    #[test]
    fn gt_eq_het() {
        let line = make_line("GT", &["0/0", "0/1", "1/1"]);
        let expr = parse_expr(r#"GT="het""#).unwrap();
        let res = eval_expr(&expr, &line, 3).unwrap();
        assert_eq!(res.pass, vec![false, true, false]);
    }

    #[test]
    fn andvec_combination_is_per_sample_or() {
        // `&&` in bcftools is per-sample OR (vec-and): sample passes if EITHER condition is true.
        let line = make_line("GT:DP:GQ", &["0/1:3:25", "0/1:10:15", "0/0:8:30"]);
        let expr = parse_expr("FMT/DP<5 && FMT/GQ>=20").unwrap();
        let res = eval_expr(&expr, &line, 3).unwrap();
        // S1: DP=3<5 TRUE  OR GQ=25>=20 TRUE  → true
        // S2: DP=10<5 FALSE OR GQ=15>=20 FALSE → false
        // S3: DP=8<5 FALSE  OR GQ=30>=20 TRUE  → true (passes via GQ)
        assert_eq!(res.pass, vec![true, false, true]);
    }

    #[test]
    fn and_single_is_per_sample_and() {
        // `&` is per-sample true AND: sample passes only if BOTH conditions are true for it.
        let line = make_line("GT:DP:GQ", &["0/1:3:25", "0/1:10:25", "0/0:8:30"]);
        let expr = parse_expr("FMT/DP<5 & FMT/GQ>=20").unwrap();
        let res = eval_expr(&expr, &line, 3).unwrap();
        // S1: DP=3<5 TRUE  AND GQ=25>=20 TRUE  → true
        // S2: DP=10<5 FALSE → false (short-circuit)
        // S3: DP=8<5 FALSE  → false
        assert_eq!(res.pass, vec![true, false, false]);
    }

    #[test]
    fn negate_mode() {
        let line = make_line("GT:DP", &["0/1:3", "0/0:10"]);
        let expr = parse_expr("FMT/DP<5").unwrap();
        let ctx = EvalContext::new(expr, true); // -e mode
        let res = ctx.eval_line(&line, 2).unwrap();
        // expr: [true, false] → negated: [false, true]
        assert_eq!(res.pass, vec![false, true]);
    }

    #[test]
    fn filter_string_eq() {
        let line = "chr1\t100\t.\tA\tT\t50\tPASS\t.\tGT\t0/1";
        let expr = parse_expr(r#"FILTER="PASS""#).unwrap();
        let res = eval_expr(&expr, line, 1).unwrap();
        assert_eq!(res.pass, vec![true]);
    }

    #[test]
    fn missing_dp_returns_false() {
        // DP is "." → treated as missing → expr returns false
        let line = make_line("GT:DP", &["0/1:."]);
        let expr = parse_expr("FMT/DP<5").unwrap();
        let res = eval_expr(&expr, &line, 1).unwrap();
        assert_eq!(res.pass, vec![false]);
    }

    #[test]
    fn info_field_numeric() {
        let line = "chr1\t100\t.\tA\tT\t50\tPASS\tDP=3\tGT\t0/1";
        let expr = parse_expr("INFO/DP<5").unwrap();
        let res = eval_expr(&expr, line, 1).unwrap();
        assert_eq!(res.pass, vec![true]);
    }
}
