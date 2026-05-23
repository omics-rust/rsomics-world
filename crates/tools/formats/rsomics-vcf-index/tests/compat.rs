use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-vcf-index"))
}

fn fixture_gz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.vcf.gz")
}

fn bcftools_path() -> Option<String> {
    let candidates = [
        "bcftools",
        "/opt/homebrew/Caskroom/miniforge/base/envs/imotif-pipeline/bin/bcftools",
        "/usr/bin/bcftools",
        "/usr/local/bin/bcftools",
    ];
    for candidate in &candidates {
        let ok = Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if ok {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Extract non-header lines from bcftools view output for a region.
fn query_records(bcftools: &str, vcf_gz: &Path, region: &str) -> Vec<String> {
    let out = Command::new(bcftools)
        .args(["view", "-r", region])
        .arg(vcf_gz)
        .output()
        .expect("bcftools view failed");
    assert!(
        out.status.success(),
        "bcftools view stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// CSI: build ours' index, query with bcftools using ours' .csi, compare to bcftools' own .csi.
#[test]
fn csi_region_query_matches_bcftools() {
    let Some(bcftools) = bcftools_path() else {
        eprintln!("skipping: bcftools not found");
        return;
    };

    let ver = Command::new(&bcftools)
        .arg("--version")
        .output()
        .unwrap()
        .stdout;
    eprintln!(
        "bcftools: {}",
        String::from_utf8_lossy(&ver).lines().next().unwrap_or("")
    );

    let src = fixture_gz();
    let tmp = TempDir::new().unwrap();

    // Copy the fixture into the temp dir so indexes end up there.
    let our_vcf = tmp.path().join("ours.vcf.gz");
    let ref_vcf = tmp.path().join("ref.vcf.gz");
    std::fs::copy(&src, &our_vcf).unwrap();
    std::fs::copy(&src, &ref_vcf).unwrap();

    // Build ours' CSI index on our_vcf.
    let status = ours()
        .arg(&our_vcf)
        .status()
        .expect("rsomics-vcf-index failed to start");
    assert!(status.success(), "rsomics-vcf-index exited non-zero");

    let our_csi = tmp.path().join("ours.vcf.gz.csi");
    assert!(our_csi.exists(), "our .csi was not created");

    // Build bcftools' CSI index on ref_vcf.
    let status = Command::new(&bcftools)
        .args(["index", "-c"])
        .arg(&ref_vcf)
        .status()
        .expect("bcftools index failed to start");
    assert!(status.success(), "bcftools index exited non-zero");

    // Query with ours' index.
    let regions = [
        "chr1:1-100000",
        "chr1:149999-150010",
        "chr2",
        "chr3:1-30000",
    ];
    for region in &regions {
        let ours_records = query_records(&bcftools, &our_vcf, region);
        let ref_records = query_records(&bcftools, &ref_vcf, region);
        assert_eq!(
            ours_records, ref_records,
            "CSI region {region}: record mismatch between ours and bcftools"
        );
        eprintln!("CSI {region}: {} records, match OK", ours_records.len());
    }

    // Bonus: byte-identical .csi check.
    let our_bytes = std::fs::read(&our_csi).unwrap();
    let ref_csi = tmp.path().join("ref.vcf.gz.csi");
    let ref_bytes = std::fs::read(&ref_csi).unwrap();
    if our_bytes == ref_bytes {
        eprintln!("BONUS: .csi is byte-identical to bcftools index output");
    } else {
        eprintln!(
            "note: .csi differs from bcftools (ours={} bytes, theirs={} bytes) — functional equivalence holds",
            our_bytes.len(),
            ref_bytes.len()
        );
    }
}

/// TBI: build ours' .tbi, query with bcftools using ours' .tbi, compare to reference.
#[test]
fn tbi_region_query_matches_bcftools() {
    let Some(bcftools) = bcftools_path() else {
        eprintln!("skipping: bcftools not found");
        return;
    };

    let src = fixture_gz();
    let tmp = TempDir::new().unwrap();

    let our_vcf = tmp.path().join("ours.vcf.gz");
    let ref_vcf = tmp.path().join("ref.vcf.gz");
    std::fs::copy(&src, &our_vcf).unwrap();
    std::fs::copy(&src, &ref_vcf).unwrap();

    // Build ours' TBI index.
    let status = ours()
        .args(["--tbi"])
        .arg(&our_vcf)
        .status()
        .expect("rsomics-vcf-index --tbi failed to start");
    assert!(status.success(), "rsomics-vcf-index --tbi exited non-zero");

    let our_tbi = tmp.path().join("ours.vcf.gz.tbi");
    assert!(our_tbi.exists(), "our .tbi was not created");

    // Build bcftools' TBI index for comparison.
    let status = Command::new(&bcftools)
        .args(["index", "-t"])
        .arg(&ref_vcf)
        .status()
        .expect("bcftools index -t failed to start");
    assert!(status.success(), "bcftools index -t exited non-zero");

    let regions = [
        "chr1:1-100000",
        "chr1:149999-150010",
        "chr2",
        "chr3:1-30000",
    ];
    for region in &regions {
        let ours_records = query_records(&bcftools, &our_vcf, region);
        let ref_records = query_records(&bcftools, &ref_vcf, region);
        assert_eq!(
            ours_records, ref_records,
            "TBI region {region}: record mismatch between ours and bcftools"
        );
        eprintln!("TBI {region}: {} records, match OK", ours_records.len());
    }
}

/// No-overwrite guard: running without --force on an existing index must fail.
#[test]
fn no_overwrite_without_force() {
    let src = fixture_gz();
    let tmp = TempDir::new().unwrap();
    let vcf = tmp.path().join("a.vcf.gz");
    std::fs::copy(&src, &vcf).unwrap();

    // First run: succeeds.
    let status = ours().arg(&vcf).status().unwrap();
    assert!(status.success());

    // Second run without --force: must fail.
    let status = ours().arg(&vcf).status().unwrap();
    assert!(
        !status.success(),
        "expected failure when index already exists"
    );
}

/// --force allows overwriting.
#[test]
fn force_flag_overwrites() {
    let src = fixture_gz();
    let tmp = TempDir::new().unwrap();
    let vcf = tmp.path().join("b.vcf.gz");
    std::fs::copy(&src, &vcf).unwrap();

    let status = ours().arg(&vcf).status().unwrap();
    assert!(status.success());

    let status = ours().arg("--force").arg(&vcf).status().unwrap();
    assert!(status.success(), "expected success with --force");
}
