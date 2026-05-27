/// Compat and smoke tests for rsomics-junction-saturation.
///
/// Smoke tests run on golden fixtures committed to tests/golden/.
/// Compat tests compare at 100%-fraction counts (known/novel totals) against
/// junction_saturation.py (RSeQC). Upstream sampling is non-deterministic and
/// uses junction-instance shuffling rather than read subsampling, so per-fraction
/// counts cannot be compared exactly. Only the full-coverage (pct=100) totals
/// and monotonicity are verified.
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn golden(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn run_ours(bam: &Path, bed: &Path, prefix: &str) -> std::path::PathBuf {
    let bin = env!("CARGO_BIN_EXE_rsomics-junction-saturation");
    let out = Command::new(bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            prefix,
            "-l",
            "5",
            "-u",
            "100",
            "-s",
            "5",
            "--mapq",
            "0",
            "--min-intron",
            "50",
            "--seed",
            "42",
        ])
        .output()
        .expect("failed to run rsomics-junction-saturation");
    assert!(
        out.status.success(),
        "rsomics-junction-saturation failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::path::PathBuf::from(format!("{prefix}.junction_saturation.txt"))
}

/// Parse the TSV output into (pct, known, partial_novel, complete_novel) rows.
fn parse_output(path: &Path) -> Vec<(u8, usize, usize, usize)> {
    let content = std::fs::read_to_string(path).expect("read output");
    content
        .lines()
        .skip(1) // header
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            assert_eq!(cols.len(), 4, "expected 4 columns in {l}");
            (
                cols[0].parse().unwrap(),
                cols[1].parse().unwrap(),
                cols[2].parse().unwrap(),
                cols[3].parse().unwrap(),
            )
        })
        .collect()
}

// ── Smoke: output format and 100%-fraction correctness ───────────────────────

/// At 100% coverage the tool must find:
/// - 2 known junctions: (chr1,200,700) and (chr1,1100,1500)
/// - 1 partial_novel: (chr1,200,400) — donor 200 annotated, acceptor 400 not
/// - 1 complete_novel: (chr1,250,350) — neither donor 250 nor acceptor 350 annotated
#[test]
fn smoke_full_coverage_counts() {
    let bam = golden("small.bam");
    let bed = golden("small.bed12");
    let tmp = TempDir::new().expect("tempdir");
    let prefix = tmp.path().join("out").to_str().unwrap().to_string();

    let out_path = run_ours(&bam, &bed, &prefix);
    let rows = parse_output(&out_path);

    assert!(!rows.is_empty(), "output has no rows");

    let row100 = rows
        .iter()
        .find(|(pct, _, _, _)| *pct == 100)
        .expect("no pct=100 row");

    assert_eq!(row100.1, 2, "expected 2 known junctions at 100%");
    assert_eq!(row100.2, 1, "expected 1 partial_novel junction at 100%");
    assert_eq!(row100.3, 1, "expected 1 complete_novel junction at 100%");
}

/// Junction counts must be non-decreasing as sampling fraction increases.
#[test]
fn smoke_monotonicity() {
    let bam = golden("small.bam");
    let bed = golden("small.bed12");
    let tmp = TempDir::new().expect("tempdir");
    let prefix = tmp.path().join("out").to_str().unwrap().to_string();

    let out_path = run_ours(&bam, &bed, &prefix);
    let rows = parse_output(&out_path);

    for window in rows.windows(2) {
        let (p0, k0, pn0, cn0) = window[0];
        let (p1, k1, pn1, cn1) = window[1];
        assert!(
            k1 >= k0,
            "known junctions decreased from pct={p0} ({k0}) to pct={p1} ({k1})"
        );
        assert!(
            (pn1 + cn1) >= (pn0 + cn0),
            "novel junctions decreased from pct={p0} ({}) to pct={p1} ({})",
            pn0 + cn0,
            pn1 + cn1
        );
    }
}

/// Output must contain a header line and 20 data rows (pct=5..100 step 5).
#[test]
fn smoke_row_count() {
    let bam = golden("small.bam");
    let bed = golden("small.bed12");
    let tmp = TempDir::new().expect("tempdir");
    let prefix = tmp.path().join("out").to_str().unwrap().to_string();

    let out_path = run_ours(&bam, &bed, &prefix);
    let content = std::fs::read_to_string(&out_path).expect("read output");

    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines[0], "pct\tknown\tpartial_novel\tcomplete_novel");
    // 5, 10, 15, ..., 100 = 20 fractions
    assert_eq!(lines.len(), 21, "expected header + 20 data rows");
}

// ── Compat: compare 100%-fraction totals against upstream RSeQC ──────────────

/// Compare totals at 100% sampling against junction_saturation.py.
///
/// At 100% both tools see every junction. Upstream reports known + novel (= our
/// partial_novel + complete_novel). Due to different sampling semantics (upstream
/// shuffles junction instances; we sample reads), per-fraction counts differ, but
/// 100%-fraction totals must agree.
///
/// The fixture uses only chr1 (present in the BED annotation) so that upstream's
/// chrom-filter (which skips reads on chromosomes absent from the BED) does not
/// cause a divergence. Upstream also converts chrom to uppercase internally;
/// both tools produce the same 100%-fraction totals regardless.
///
/// Skipped if `junction_saturation.py` is not found at the expected path.
#[test]
fn compat_100pct_totals() {
    const UPSTREAM: &str = "/Users/snaix/Library/Python/3.14/bin/junction_saturation.py";

    if !std::path::Path::new(UPSTREAM).exists() {
        eprintln!("junction_saturation.py not found at {UPSTREAM}; skipping compat test");
        return;
    }

    let bam = golden("small.bam");
    let bed = golden("small.bed12");

    let tmp = TempDir::new().expect("tempdir");
    let ours_prefix = tmp.path().join("ours").to_str().unwrap().to_string();
    let up_prefix = tmp.path().join("upstream").to_str().unwrap().to_string();

    let ours_out = run_ours(&bam, &bed, &ours_prefix);
    let ours_rows = parse_output(&ours_out);
    let ours100 = ours_rows
        .iter()
        .find(|(pct, _, _, _)| *pct == 100)
        .copied()
        .expect("no pct=100 row in ours output");

    let up_status = Command::new(UPSTREAM)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            &up_prefix,
            "-l",
            "5",
            "-u",
            "100",
            "-s",
            "5",
            "-m",
            "50",
            "-q",
            "0",
        ])
        .output()
        .expect("failed to run junction_saturation.py");
    assert!(
        up_status.status.success(),
        "junction_saturation.py failed: {}",
        String::from_utf8_lossy(&up_status.stderr)
    );

    let r_script = std::fs::read_to_string(format!("{up_prefix}.junctionSaturation_plot.r"))
        .expect("upstream R script not found");

    let (up_known100, up_all100) = parse_r_script_100pct(&r_script);
    let up_novel100 = up_all100 - up_known100;

    let (_, ours_known, ours_pn, ours_cn) = ours100;
    let ours_novel = ours_pn + ours_cn;

    assert_eq!(
        ours_known, up_known100,
        "known junction count at 100%: ours={ours_known} upstream={up_known100}"
    );
    assert_eq!(
        ours_novel, up_novel100,
        "novel junction count at 100%: ours={ours_novel} upstream={up_novel100}"
    );
}

/// Parse the R script produced by junction_saturation.py and return
/// (known_at_100pct, all_at_100pct).
fn parse_r_script_100pct(script: &str) -> (usize, usize) {
    let mut known = 0usize;
    let mut all = 0usize;

    for line in script.lines() {
        if let Some(rest) = line.strip_prefix("y=c(") {
            let inner = rest.trim_end_matches(')');
            known = inner
                .split(',')
                .next_back()
                .unwrap()
                .trim()
                .parse()
                .expect("parse known count");
        } else if let Some(rest) = line.strip_prefix("z=c(") {
            let inner = rest.trim_end_matches(')');
            all = inner
                .split(',')
                .next_back()
                .unwrap()
                .trim()
                .parse()
                .expect("parse all count");
        }
    }

    assert!(all > 0, "could not parse all junctions from R script");
    (known, all)
}
