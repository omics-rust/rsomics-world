use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-vcf-convert"))
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn bcftools_available() -> bool {
    Command::new("bcftools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Round-trip a plain VCF through `-O v` and verify record count matches
/// bcftools view -H output.
#[test]
fn vcf_to_vcf_roundtrip_record_count() {
    if !bcftools_available() {
        eprintln!("skipping: bcftools not found");
        return;
    }

    let vcf = fixture("small.vcf");

    let ours_out = ours()
        .args(["-O", "v"])
        .arg(&vcf)
        .output()
        .expect("failed to run rsomics-vcf-convert");
    assert!(
        ours_out.status.success(),
        "rsomics-vcf-convert exited non-zero: {}",
        String::from_utf8_lossy(&ours_out.stderr)
    );

    let ours_records = String::from_utf8_lossy(&ours_out.stdout)
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .count();

    let bcf_out = Command::new("bcftools")
        .args(["view", "-H"])
        .arg(&vcf)
        .output()
        .expect("failed to run bcftools");
    assert!(bcf_out.status.success(), "bcftools view failed");

    let bcf_records = String::from_utf8_lossy(&bcf_out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .count();

    assert_eq!(
        ours_records, bcf_records,
        "record count mismatch: ours={ours_records} bcftools={bcf_records}"
    );
}

/// Verify that `-O v` is a lossless passthrough: our data lines must equal
/// the source file's data lines byte-for-byte.
///
/// bcftools `view` normalizes INFO float values (e.g. `AF=0.10` → `AF=0.1`);
/// we deliberately do not — preserving source bytes is the contract of a
/// lossless format conversion tool.
#[test]
fn vcf_to_vcf_preserves_source_data_lines() {
    let vcf = fixture("small.vcf");

    let ours_out = ours()
        .args(["-O", "v"])
        .arg(&vcf)
        .output()
        .expect("failed to run rsomics-vcf-convert");
    assert!(
        ours_out.status.success(),
        "rsomics-vcf-convert exited non-zero: {}",
        String::from_utf8_lossy(&ours_out.stderr)
    );

    let source = std::fs::read_to_string(&vcf).expect("could not read fixture");

    // Compare every non-comment data line byte-for-byte with the source.
    let ours_data: Vec<&str> = String::from_utf8_lossy(&ours_out.stdout)
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .leak()
        .lines()
        .collect();

    let src_data: Vec<&str> = source
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect();

    assert_eq!(
        ours_data, src_data,
        "data lines were modified during plain VCF passthrough"
    );
}

/// Compress to VCF.gz then decompress; verify record identity.
#[test]
fn vcf_gz_roundtrip() {
    if !bcftools_available() {
        eprintln!("skipping: bcftools not found");
        return;
    }

    let vcf = fixture("small.vcf");
    let tmp_dir = std::env::temp_dir();
    let gz_out = tmp_dir.join("rsomics_vcf_convert_test.vcf.gz");
    let plain_out = tmp_dir.join("rsomics_vcf_convert_test_rt.vcf");

    // Step 1: compress
    let status = ours()
        .args(["-O", "z"])
        .arg(&vcf)
        .arg("-o")
        .arg(&gz_out)
        .status()
        .expect("failed to run rsomics-vcf-convert (compress)");
    assert!(status.success(), "compress step failed");

    // Step 2: decompress
    let status = ours()
        .args(["-O", "v"])
        .arg(&gz_out)
        .arg("-o")
        .arg(&plain_out)
        .status()
        .expect("failed to run rsomics-vcf-convert (decompress)");
    assert!(status.success(), "decompress step failed");

    // Compare data records: plain source vs round-tripped
    let original = std::fs::read_to_string(&vcf).expect("could not read fixture");
    let roundtripped =
        std::fs::read_to_string(&plain_out).expect("could not read round-trip output");

    let orig_data: Vec<&str> = original
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect();
    let rt_data: Vec<&str> = roundtripped
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect();

    assert_eq!(
        orig_data, rt_data,
        "data lines changed during gz round-trip"
    );

    // Cleanup
    let _ = std::fs::remove_file(&gz_out);
    let _ = std::fs::remove_file(&plain_out);
}

/// bcftools rejects `-O b` (BCF binary) with non-zero exit; ours should too.
#[test]
fn bcf_binary_output_rejected() {
    let vcf = fixture("small.vcf");
    let status = ours()
        .args(["-O", "b"])
        .arg(&vcf)
        .stderr(Stdio::null())
        .status()
        .expect("failed to run rsomics-vcf-convert");
    assert!(
        !status.success(),
        "expected non-zero exit for unsupported -O b"
    );
}

/// HAP/LEGEND/SAMPLE export: verify legend line count equals data record
/// count from `bcftools view -H`.
#[test]
fn haplegendsample_legend_record_count() {
    if !bcftools_available() {
        eprintln!("skipping: bcftools not found");
        return;
    }

    let vcf = fixture("hap_test.vcf");
    let tmp_dir = std::env::temp_dir();
    let prefix = tmp_dir
        .join("rsomics_vcf_convert_hls_test")
        .display()
        .to_string();

    let out = ours()
        .arg("--haplegendsample")
        .arg(&prefix)
        .arg(&vcf)
        .output()
        .expect("failed to run rsomics-vcf-convert -h");
    assert!(
        out.status.success(),
        "haplegendsample export failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let legend_path = format!("{prefix}.legend");
    let legend = std::fs::read_to_string(&legend_path).expect("could not read .legend output");

    // Subtract 1 for the header line "id position a0 a1"
    let legend_records = legend.lines().filter(|l| !l.is_empty()).count() - 1;

    let bcf_out = Command::new("bcftools")
        .args(["view", "-H"])
        .arg(&vcf)
        .output()
        .expect("failed to run bcftools");
    assert!(bcf_out.status.success(), "bcftools view failed");
    let bcf_records = String::from_utf8_lossy(&bcf_out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .count();

    assert_eq!(
        legend_records, bcf_records,
        "legend record count {legend_records} != bcftools record count {bcf_records}"
    );

    // Cleanup
    let _ = std::fs::remove_file(format!("{prefix}.hap"));
    let _ = std::fs::remove_file(&legend_path);
    let _ = std::fs::remove_file(format!("{prefix}.samples"));
}

/// HAP file: verify allele encoding for known phased genotypes.
#[test]
fn haplegendsample_hap_alleles_correct() {
    let vcf = fixture("hap_test.vcf");
    let tmp_dir = std::env::temp_dir();
    let prefix = tmp_dir
        .join("rsomics_vcf_convert_hls_alleles")
        .display()
        .to_string();

    let out = ours()
        .arg("--haplegendsample")
        .arg(&prefix)
        .arg(&vcf)
        .output()
        .expect("failed to run rsomics-vcf-convert -h");
    assert!(
        out.status.success(),
        "haplegendsample export failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let hap_path = format!("{prefix}.hap");
    let hap = std::fs::read_to_string(&hap_path).expect("could not read .hap output");

    // hap_test.vcf first record: SAMPLE1=0|1, SAMPLE2=1|0
    // Expected HAP line: "0 1 1 0"
    let first_line = hap.lines().next().expect("hap file is empty");
    assert_eq!(
        first_line, "0 1 1 0",
        "HAP allele encoding wrong on first record: got '{first_line}'"
    );

    // Third record: SAMPLE1=1|1, SAMPLE2=0|0 → "1 1 0 0"
    let third_line = hap.lines().nth(2).expect("hap file has fewer than 3 lines");
    assert_eq!(
        third_line, "1 1 0 0",
        "HAP allele encoding wrong on third record: got '{third_line}'"
    );

    // Cleanup
    let _ = std::fs::remove_file(&hap_path);
    let _ = std::fs::remove_file(format!("{prefix}.legend"));
    let _ = std::fs::remove_file(format!("{prefix}.samples"));
}
