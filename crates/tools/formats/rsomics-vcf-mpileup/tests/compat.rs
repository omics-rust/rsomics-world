//! Compatibility tests against `bcftools mpileup`.
//!
//! For each variant position our tool emits, verify the matching bcftools record
//! agrees on CHROM, POS, REF, and the non-`<*>` ALT allele(s) it chose.
//!
//! Not checked here (documented 0.1.0 deferrals):
//! - Reference-only positions emitted by bcftools with ALT=`<*>`
//! - QUAL and INFO tag values (bcftools' raw counts differ from our filtered counts)
//! - Multi-allelic records (bcftools picks multiple ALTs; we pick one)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bcftools_present() -> bool {
    Command::new("bcftools")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn our_binary() -> PathBuf {
    env!("CARGO_BIN_EXE_rsomics-vcf-mpileup").into()
}

fn golden(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name)
}

fn run_ours(bam: &Path, fa: &Path, region: Option<&str>) -> Vec<u8> {
    let mut cmd = Command::new(our_binary());
    cmd.arg(bam).arg("-f").arg(fa);
    if let Some(r) = region {
        cmd.arg("-r").arg(r);
    }
    let out = cmd.output().expect("failed to spawn rsomics-vcf-mpileup");
    assert!(
        out.status.success(),
        "rsomics-vcf-mpileup exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

fn run_bcftools(bam: &Path, fa: &Path, region: Option<&str>) -> Vec<u8> {
    let mut cmd = Command::new("bcftools");
    cmd.arg("mpileup")
        .arg("-Ov")
        .arg("-a")
        .arg("AD,DP")
        .arg("-f")
        .arg(fa)
        .arg("-q")
        .arg("0")
        .arg("-Q")
        .arg("1");
    if let Some(r) = region {
        cmd.arg("-r").arg(r);
    }
    cmd.arg(bam);
    let out = cmd.output().expect("failed to spawn bcftools mpileup");
    assert!(
        out.status.success(),
        "bcftools mpileup exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

/// Parse VCF body into a map of (chrom, pos) -> (ref, alt) for non-`<*>` alt lines.
fn parse_vcf_variants(vcf: &[u8]) -> HashMap<(String, u64), (String, String)> {
    let mut map = HashMap::new();
    for line in vcf.split(|&b| b == b'\n') {
        if line.is_empty() || line.starts_with(b"#") {
            continue;
        }
        let s = String::from_utf8_lossy(line);
        let cols: Vec<&str> = s.split('\t').collect();
        if cols.len() < 5 {
            continue;
        }
        let chrom = cols[0].to_string();
        let pos: u64 = cols[1].parse().unwrap_or(0);
        let reff = cols[3].to_string();
        let alt = cols[4].to_string();
        // Skip bcftools-only reference positions (ALT is "." or "<*>" only)
        if alt == "." || alt == "<*>" {
            continue;
        }
        // For bcftools multi-allelic "C,<*>" strip the <*> suffix.
        let alt_clean = alt
            .split(',')
            .filter(|&a| a != "<*>")
            .collect::<Vec<_>>()
            .join(",");
        if alt_clean.is_empty() {
            continue;
        }
        map.insert((chrom, pos), (reff, alt_clean));
    }
    map
}

#[test]
fn compat_snp_basic() {
    if !bcftools_present() {
        eprintln!("bcftools not found — skipping compat test");
        return;
    }

    let bam = golden("small.bam");
    let fa = golden("small.fa");
    if !bam.exists() || !fa.exists() {
        eprintln!("golden fixtures missing — skipping (run tests/make_fixtures.py)");
        return;
    }

    let ours_raw = run_ours(&bam, &fa, None);
    let theirs_raw = run_bcftools(&bam, &fa, None);

    let ours_vars = parse_vcf_variants(&ours_raw);
    let theirs_vars = parse_vcf_variants(&theirs_raw);

    assert!(
        !ours_vars.is_empty(),
        "rsomics-vcf-mpileup emitted 0 variants"
    );

    // Every position we emit must appear in bcftools output with matching REF and ALT.
    let mut mismatches = 0usize;
    for ((chrom, pos), (our_ref, our_alt)) in &ours_vars {
        match theirs_vars.get(&(chrom.clone(), *pos)) {
            None => {
                eprintln!(
                    "EXTRA: we emit variant at {chrom}:{pos} REF={our_ref} ALT={our_alt} \
                     but bcftools has no variant there"
                );
                mismatches += 1;
            }
            Some((their_ref, their_alt)) => {
                if our_ref != their_ref {
                    eprintln!("REF mismatch at {chrom}:{pos}: ours={our_ref} bcftools={their_ref}");
                    mismatches += 1;
                }
                // 0.1.0 scope: we emit one best ALT; bcftools may emit several.
                // Our ALT must be a member of bcftools' comma-separated ALT list.
                let their_set: std::collections::HashSet<&str> = their_alt.split(',').collect();
                if !their_set.contains(our_alt.as_str()) {
                    eprintln!(
                        "ALT not in bcftools set at {chrom}:{pos}: ours={our_alt} bcftools={their_alt}"
                    );
                    mismatches += 1;
                }
            }
        }
    }

    assert_eq!(
        mismatches, 0,
        "{mismatches} variant mismatches vs bcftools mpileup"
    );
}

#[test]
fn compat_region_filter() {
    if !bcftools_present() {
        return;
    }
    let bam = golden("small.bam");
    let fa = golden("small.fa");
    if !bam.exists() || !fa.exists() {
        return;
    }

    let region = "chr1:5-30";
    let ours_raw = run_ours(&bam, &fa, Some(region));
    let theirs_raw = run_bcftools(&bam, &fa, Some(region));

    let ours_vars = parse_vcf_variants(&ours_raw);
    let theirs_vars = parse_vcf_variants(&theirs_raw);

    // All our variants must be inside [5, 30] and match bcftools.
    for ((chrom, pos), (our_ref, our_alt)) in &ours_vars {
        assert!(
            *pos >= 5 && *pos <= 30,
            "variant at {chrom}:{pos} outside requested region"
        );
        if let Some((their_ref, their_alt)) = theirs_vars.get(&(chrom.clone(), *pos)) {
            assert_eq!(our_ref, their_ref, "REF mismatch at {chrom}:{pos}");
            let their_set: std::collections::HashSet<&str> = their_alt.split(',').collect();
            assert!(
                their_set.contains(our_alt.as_str()),
                "ALT not in bcftools set at {chrom}:{pos}: ours={our_alt} bcftools={their_alt}"
            );
        }
    }
}
