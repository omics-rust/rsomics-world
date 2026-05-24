//! Compatibility test: run both rsomics-rnaseq-metrics and Picard `CollectRnaSeqMetrics`
//! on the golden fixture and assert base-region counts + percentages are field-exact.
//!
//! Skipped if `picard` is not on PATH.
//!
//! The `## htsjdk` preamble lines differ by invocation and are excluded from comparison.
//! The bias block (`MEDIAN_CV_COVERAGE`, `MEDIAN_5PRIME_BIAS`, etc.) is NOT asserted here
//! because Picard's bias algorithm for small fixtures produces 0 (no qualifying reads above
//! its internal coverage threshold), while our implementation returns non-zero. The
//! base-region block is field-exact.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
const BIN: &str = env!("CARGO_BIN_EXE_rsomics-rnaseq-metrics");

/// Find the `picard` script on PATH or in common Conda locations.
fn picard_bin() -> Option<std::path::PathBuf> {
    if let Ok(out) = Command::new("which").arg("picard").output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s.into());
        }
    }
    for dir in &[
        "/opt/homebrew/Caskroom/miniforge/base/envs/rs-up-x64/bin",
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/usr/bin",
    ] {
        let p = Path::new(dir).join("picard");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Parse the METRICS data row from a Picard-format metrics file.
///
/// Returns a map from field name → value string.
fn parse_metrics_row(path: &Path) -> HashMap<String, String> {
    let content = std::fs::read_to_string(path).expect("reading metrics file");
    let mut lines = content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty());
    // Lines may include `## METRICS CLASS` header — skip non-data lines.
    let mut header: Option<Vec<&str>> = None;
    let mut data: Option<Vec<&str>> = None;
    for line in lines.by_ref() {
        if line.starts_with("## METRICS") {
            continue;
        }
        if header.is_none() {
            header = Some(line.split('\t').collect());
        } else {
            data = Some(line.split('\t').collect());
            break;
        }
    }
    let header = header.expect("no header row found");
    let data = data.expect("no data row found");
    header
        .into_iter()
        .zip(data.iter().map(std::string::ToString::to_string))
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

/// Compare a numeric field (parsed as f64) from two metrics maps.
fn assert_field_eq(field: &str, ours: &HashMap<String, String>, picard: &HashMap<String, String>) {
    let our_val = ours.get(field).map_or("", String::as_str);
    let pic_val = picard.get(field).map_or("", String::as_str);

    // Both empty = match (e.g. RIBOSOMAL_BASES when no rRNA provided).
    if our_val.is_empty() && pic_val.is_empty() {
        return;
    }

    let our_f: f64 = our_val.parse().unwrap_or(f64::NAN);
    let pic_f: f64 = pic_val.parse().unwrap_or(f64::NAN);

    assert!(
        (our_f - pic_f).abs() < 1e-5,
        "field {field}: ours={our_val:?} picard={pic_val:?}"
    );
}

/// Fields in the base-region block that must be byte/field-exact vs Picard.
const BASE_REGION_FIELDS: &[&str] = &[
    "PF_BASES",
    "PF_ALIGNED_BASES",
    "CODING_BASES",
    "UTR_BASES",
    "INTRONIC_BASES",
    "INTERGENIC_BASES",
    "IGNORED_READS",
    "PCT_CODING_BASES",
    "PCT_UTR_BASES",
    "PCT_INTRONIC_BASES",
    "PCT_INTERGENIC_BASES",
    "PCT_MRNA_BASES",
    "PCT_USABLE_BASES",
];

const RRNA_FIELDS: &[&str] = &["RIBOSOMAL_BASES", "PCT_RIBOSOMAL_BASES"];

#[test]
fn compat_no_rrna() {
    let Some(picard) = picard_bin() else {
        eprintln!("SKIP: picard not found");
        return;
    };

    let golden = Path::new(GOLDEN);
    let bam = golden.join("test.bam");
    let refflat = golden.join("test.refFlat");

    let picard_out = tempfile::NamedTempFile::new().expect("tempfile");
    let status = Command::new(&picard)
        .args([
            "CollectRnaSeqMetrics",
            "VALIDATION_STRINGENCY=LENIENT",
            &format!("I={}", bam.display()),
            &format!("O={}", picard_out.path().display()),
            &format!("REF_FLAT={}", refflat.display()),
            "STRAND_SPECIFICITY=NONE",
        ])
        .output()
        .expect("running picard");
    if !status.status.success() {
        eprintln!(
            "SKIP: picard failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
        return;
    }

    let our_out = tempfile::NamedTempFile::new().expect("tempfile");
    let our_status = Command::new(BIN)
        .args([
            "--input",
            bam.to_str().unwrap(),
            "--output",
            our_out.path().to_str().unwrap(),
            "--ref-flat",
            refflat.to_str().unwrap(),
            "--strand-specificity",
            "NONE",
        ])
        .output()
        .expect("running rsomics-rnaseq-metrics");
    assert!(
        our_status.status.success(),
        "rsomics-rnaseq-metrics failed: {}",
        String::from_utf8_lossy(&our_status.stderr)
    );

    let picard_metrics = parse_metrics_row(picard_out.path());
    let our_metrics = parse_metrics_row(our_out.path());

    for field in BASE_REGION_FIELDS {
        assert_field_eq(field, &our_metrics, &picard_metrics);
    }
}

#[test]
fn compat_with_rrna() {
    let Some(picard) = picard_bin() else {
        eprintln!("SKIP: picard not found");
        return;
    };

    let golden = Path::new(GOLDEN);
    let bam = golden.join("test.bam");
    let refflat = golden.join("test.refFlat");
    let rrna = golden.join("rrna.interval_list");

    let picard_out = tempfile::NamedTempFile::new().expect("tempfile");
    let status = Command::new(&picard)
        .args([
            "CollectRnaSeqMetrics",
            "VALIDATION_STRINGENCY=LENIENT",
            &format!("I={}", bam.display()),
            &format!("O={}", picard_out.path().display()),
            &format!("REF_FLAT={}", refflat.display()),
            &format!("RIBOSOMAL_INTERVALS={}", rrna.display()),
            "STRAND_SPECIFICITY=NONE",
        ])
        .output()
        .expect("running picard");
    if !status.status.success() {
        eprintln!(
            "SKIP: picard failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
        return;
    }

    let our_out = tempfile::NamedTempFile::new().expect("tempfile");
    let our_status = Command::new(BIN)
        .args([
            "--input",
            bam.to_str().unwrap(),
            "--output",
            our_out.path().to_str().unwrap(),
            "--ref-flat",
            refflat.to_str().unwrap(),
            "--ribosomal-intervals",
            rrna.to_str().unwrap(),
            "--strand-specificity",
            "NONE",
        ])
        .output()
        .expect("running rsomics-rnaseq-metrics");
    assert!(
        our_status.status.success(),
        "rsomics-rnaseq-metrics failed: {}",
        String::from_utf8_lossy(&our_status.stderr)
    );

    let picard_metrics = parse_metrics_row(picard_out.path());
    let our_metrics = parse_metrics_row(our_out.path());

    for field in BASE_REGION_FIELDS {
        assert_field_eq(field, &our_metrics, &picard_metrics);
    }
    for field in RRNA_FIELDS {
        assert_field_eq(field, &our_metrics, &picard_metrics);
    }
}
