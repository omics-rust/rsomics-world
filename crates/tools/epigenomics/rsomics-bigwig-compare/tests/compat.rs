//! Byte-for-byte compatibility with deeptools `bigwigCompare` 3.5.x.
//!
//! Two layers: a golden check (the deeptools bedGraph output is committed under
//! `tests/golden/`, so this always runs) and a live differential check that
//! runs the deeptools binary when it is on `PATH`, on both the small golden
//! bigWigs and the larger generated pair under `tests/fixtures/` (gitignored).

use std::process::{Command, Stdio};

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-bigwig-compare"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn fixture(n: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Run ours and capture stdout (we always write to `-`).
fn run_ours(b1: &str, b2: &str, extra: &[&str]) -> String {
    let mut cmd = Command::new(ours());
    cmd.args(["--bigwig1", b1, "--bigwig2", b2, "-o", "-", "-q"])
        .args(extra);
    let out = cmd
        .output()
        .expect("rsomics-bigwig-compare failed to launch");
    assert!(
        out.status.success(),
        "rsomics-bigwig-compare failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn run_deeptools(b1: &str, b2: &str, extra: &[&str]) -> String {
    let dir = std::env::temp_dir().join("rsomics-bigwig-compare-compat");
    let _ = std::fs::create_dir_all(&dir);
    let out = dir.join("dt.bg");
    let status = Command::new("bigwigCompare")
        .args(["-b1", b1, "-b2", b2])
        .args(["-o", out.to_str().unwrap()])
        .args(["--outFileFormat", "bedgraph", "-p", "1"])
        .args(extra)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("bigwigCompare failed to launch");
    assert!(status.success(), "bigwigCompare exited non-zero");
    std::fs::read_to_string(&out).expect("reading bigwigCompare output")
}

/// Golden: ours on the small bigWigs must equal the committed deeptools output.
fn assert_golden(op: &str, expected: &str, extra: &[&str]) {
    let mut args = vec!["--operation", op];
    args.extend_from_slice(extra);
    let got = run_ours(&golden("a.bw"), &golden("b.bw"), &args);
    let want = std::fs::read_to_string(golden(expected)).unwrap();
    assert_eq!(got.trim_end(), want.trim_end(), "golden mismatch for {op}");
}

#[test]
fn golden_all_operations() {
    for op in [
        "log2",
        "ratio",
        "reciprocal_ratio",
        "subtract",
        "add",
        "mean",
        "first",
        "second",
    ] {
        assert_golden(op, &format!("expected_{op}.bg"), &["--bin-size", "50"]);
    }
}

#[test]
fn golden_binsize_and_pseudocount() {
    assert_golden("log2", "expected_log2_bs10.bg", &["--bin-size", "10"]);
    assert_golden(
        "log2",
        "expected_log2_pc23.bg",
        &["--bin-size", "50", "--pseudocount", "2", "3"],
    );
}

/// Live differential test against the deeptools binary on the small bigWigs.
#[test]
fn live_diff_small() {
    if !have("bigwigCompare") {
        eprintln!("bigwigCompare not on PATH — skipping live diff");
        return;
    }
    let a = golden("a.bw");
    let b = golden("b.bw");
    let cases: &[&[&str]] = &[
        &["--operation", "log2", "--bin-size", "50"],
        &["--operation", "ratio", "--bin-size", "10"],
        &["--operation", "reciprocal_ratio", "--bin-size", "50"],
        &["--operation", "subtract", "--bin-size", "100"],
        &[
            "--operation",
            "log2",
            "--skip-zero-over-zero",
            "--bin-size",
            "50",
        ],
        &["--operation", "log2", "--fixed-step", "--bin-size", "50"],
        &[
            "--operation",
            "log2",
            "--skip-non-covered-regions",
            "--bin-size",
            "50",
        ],
        &[
            "--operation",
            "log2",
            "--scale-factors",
            "0.7:1",
            "--bin-size",
            "50",
        ],
    ];
    for our in cases {
        let dt = translate(our);
        let dt_args: Vec<&str> = dt.iter().map(String::as_str).collect();
        let got = run_ours(&a, &b, our);
        let want = run_deeptools(&a, &b, &dt_args);
        assert_eq!(got.trim_end(), want.trim_end(), "live small {our:?}");
    }
}

/// Live differential test on the larger generated pair (catches bugs the tiny
/// golden bigWigs hide). Skips if either deeptools or the fixtures are absent.
#[test]
fn live_diff_large() {
    if !have("bigwigCompare") {
        eprintln!("bigwigCompare not on PATH — skipping large live diff");
        return;
    }
    let a = fixture("big_a.bw");
    let b = fixture("big_b.bw");
    if !std::path::Path::new(&a).exists() {
        eprintln!("large fixtures absent — skipping (regenerate with bamCoverage)");
        return;
    }
    let cases: &[&[&str]] = &[
        &["--operation", "log2", "--bin-size", "50"],
        &["--operation", "ratio", "--bin-size", "10"],
        &["--operation", "subtract", "--bin-size", "50"],
        &[
            "--operation",
            "log2",
            "--skip-zero-over-zero",
            "--bin-size",
            "50",
        ],
        &[
            "--operation",
            "log2",
            "--skip-non-covered-regions",
            "--bin-size",
            "50",
        ],
        &["--operation", "log2", "--fixed-step", "--bin-size", "200"],
    ];
    for our in cases {
        let dt = translate(our);
        let dt_args: Vec<&str> = dt.iter().map(String::as_str).collect();
        let got = run_ours(&a, &b, our);
        let want = run_deeptools(&a, &b, &dt_args);
        assert_eq!(got.trim_end(), want.trim_end(), "live large {our:?}");
    }
}

/// Map our CLI flags to deeptools equivalents for the live diff.
fn translate(our: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut it = our.iter().peekable();
    while let Some(a) = it.next() {
        match *a {
            "--bin-size" => {
                out.push("--binSize".to_string());
                out.push((*it.next().unwrap()).to_string());
            }
            "--skip-zero-over-zero" => out.push("--skipZeroOverZero".to_string()),
            "--skip-non-covered-regions" => out.push("--skipNonCoveredRegions".to_string()),
            "--fixed-step" => out.push("--fixedStep".to_string()),
            "--scale-factors" => {
                out.push("--scaleFactors".to_string());
                out.push((*it.next().unwrap()).to_string());
            }
            other => out.push(other.to_string()),
        }
    }
    out
}
