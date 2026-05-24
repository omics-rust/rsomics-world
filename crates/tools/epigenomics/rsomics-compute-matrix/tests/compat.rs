//! Byte-for-byte compatibility with deeptools `computeMatrix` 3.5.x.
//!
//! Two layers: a golden check (the deeptools data rows are committed under
//! `tests/golden/`, so this always runs) and a live differential check that
//! runs the deeptools binary when it is on `PATH`.

use std::io::Read;
use std::process::Command;

use flate2::read::GzDecoder;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-compute-matrix"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn read_gz_lines(path: &str) -> Vec<String> {
    let f = std::fs::File::open(path).unwrap();
    let mut s = String::new();
    GzDecoder::new(f).read_to_string(&mut s).unwrap();
    s.lines().map(str::to_owned).collect()
}

fn run_ours(args: &[&str], out: &std::path::Path) {
    let o = bin().args(args).arg("-q").output().unwrap();
    assert!(
        o.status.success(),
        "ours failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let _ = out;
}

/// Our data rows (everything after the `@` header) must equal the committed
/// deeptools golden rows exactly.
#[test]
fn golden_reference_point_tss() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    run_ours(
        &[
            "reference-point",
            "-S",
            &golden("signal.bw"),
            "-R",
            &golden("regions.bed"),
            "-o",
            p,
            "--reference-point",
            "TSS",
            "-b",
            "1000",
            "-a",
            "1000",
            "--bin-size",
            "50",
        ],
        tmp.path(),
    );
    let ours: Vec<String> = read_gz_lines(p).into_iter().skip(1).collect();
    let expected: Vec<String> = std::fs::read_to_string(golden("expected_refpoint_tss.tsv"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(
        ours, expected,
        "reference-point TSS rows diverge from deeptools"
    );
}

#[test]
fn golden_scale_regions() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    run_ours(
        &[
            "scale-regions",
            "-S",
            &golden("signal.bw"),
            "-R",
            &golden("regions.bed"),
            "-o",
            p,
            "-m",
            "1000",
            "-b",
            "500",
            "-a",
            "500",
            "--bin-size",
            "50",
        ],
        tmp.path(),
    );
    let ours: Vec<String> = read_gz_lines(p).into_iter().skip(1).collect();
    let expected: Vec<String> = std::fs::read_to_string(golden("expected_scale.tsv"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(ours, expected, "scale-regions rows diverge from deeptools");
}

/// Our header JSON must match deeptools' exactly, save for `proc number`
/// (a runtime knob) which we normalise on both sides.
#[test]
fn golden_header_params() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let p = tmp.path().to_str().unwrap();
    run_ours(
        &[
            "reference-point",
            "-S",
            &golden("signal.bw"),
            "-R",
            &golden("regions.bed"),
            "-o",
            p,
            "--reference-point",
            "TSS",
            "-b",
            "1000",
            "-a",
            "1000",
            "--bin-size",
            "50",
            "-t",
            "1",
        ],
        tmp.path(),
    );
    let ours = read_gz_lines(p)[0].clone();
    let expected = std::fs::read_to_string(golden("expected_refpoint_header.txt"))
        .unwrap()
        .trim_end()
        .to_owned();
    assert_eq!(ours, expected, "header JSON diverges from deeptools");
}

fn deeptools_available() -> bool {
    Command::new("computeMatrix")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Live differential test against the deeptools binary across several modes.
#[test]
fn live_diff_against_deeptools() {
    if !deeptools_available() {
        eprintln!("computeMatrix not on PATH — skipping live diff");
        return;
    }
    let cases: &[(&str, &[&str])] = &[
        (
            "refpoint-tss",
            &[
                "reference-point",
                "--referencePoint",
                "TSS",
                "-b",
                "1000",
                "-a",
                "1000",
            ],
        ),
        (
            "refpoint-center",
            &[
                "reference-point",
                "--referencePoint",
                "center",
                "-b",
                "800",
                "-a",
                "1200",
            ],
        ),
        (
            "refpoint-tes",
            &[
                "reference-point",
                "--referencePoint",
                "TES",
                "-b",
                "1000",
                "-a",
                "1000",
            ],
        ),
        (
            "scale",
            &["scale-regions", "-m", "1000", "-b", "500", "-a", "500"],
        ),
    ];
    for (name, dt_args) in cases {
        let dt_out = tempfile::NamedTempFile::new().unwrap();
        let our_out = tempfile::NamedTempFile::new().unwrap();

        let mut dt = Command::new("computeMatrix");
        dt.args(*dt_args)
            .args(["-S", &golden("signal.bw")])
            .args(["-R", &golden("regions.bed")])
            .args(["-o", dt_out.path().to_str().unwrap()])
            .args(["--binSize", "50", "-p", "1", "-q"]);
        let s = dt.output().unwrap();
        assert!(s.status.success(), "deeptools {name} failed");

        // Translate the deeptools subcommand/flags into ours.
        let sig = golden("signal.bw");
        let reg = golden("regions.bed");
        let our_path = our_out.path().to_str().unwrap().to_owned();
        let our_args = translate(dt_args);
        let mut ours: Vec<&str> = our_args.iter().map(String::as_str).collect();
        ours.extend([
            "-S",
            sig.as_str(),
            "-R",
            reg.as_str(),
            "-o",
            our_path.as_str(),
            "--bin-size",
            "50",
        ]);
        run_ours(&ours, our_out.path());

        let dt_rows: Vec<String> = read_gz_lines(dt_out.path().to_str().unwrap())
            .into_iter()
            .skip(1)
            .collect();
        let our_rows: Vec<String> = read_gz_lines(our_out.path().to_str().unwrap())
            .into_iter()
            .skip(1)
            .collect();
        assert_eq!(our_rows, dt_rows, "live diff mismatch for {name}");
    }
}

/// Map deeptools CLI flags to our equivalents for the live diff.
fn translate(dt_args: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut it = dt_args.iter().peekable();
    while let Some(a) = it.next() {
        match *a {
            "reference-point" | "scale-regions" => out.push((*a).to_string()),
            "--referencePoint" => {
                out.push("--reference-point".to_string());
                out.push((*it.next().unwrap()).to_string());
            }
            "-b" | "-a" | "-m" => {
                out.push((*a).to_string());
                out.push((*it.next().unwrap()).to_string());
            }
            other => out.push(other.to_string()),
        }
    }
    out
}
