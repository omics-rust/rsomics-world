/// Compatibility test: compare `rsomics-vcf-fill-tags` output vs `bcftools +fill-tags`.
///
/// Checks the INFO field tags (`AN`, `AC`, `AF`, `MAF`, `NS`, `AC_Hom`, `AC_Het`,
/// `AC_Hemi`, `HWE`, `ExcHet`) field-by-field for each data record.
/// Float tags are compared with a relative tolerance of 1e-3 to account for
/// `f32`/`f64` rounding differences between bcftools and our implementation.
///
/// Skips automatically when bcftools is absent or the fill-tags plugin is unavailable.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bcftools_version() -> Option<String> {
    let out = Command::new("bcftools").arg("--version").output().ok()?;
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()?
            .to_owned(),
    )
}

fn run_bcftools_fill_tags(input: &Path, output: &Path) -> bool {
    Command::new("bcftools")
        .args([
            "+fill-tags",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .is_ok_and(|s| s.success())
}

fn run_ours(input: &Path, output: &Path) {
    let bin = env!("CARGO_BIN_EXE_rsomics-vcf-fill-tags");
    let status = Command::new(bin)
        .args([input.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .status()
        .expect("failed to run rsomics-vcf-fill-tags");
    assert!(status.success(), "rsomics-vcf-fill-tags failed");
}

/// Parse VCF data records into a list of `(chrom, pos) → INFO key→value` maps.
fn parse_vcf_info(vcf: &Path) -> Vec<((String, u64), HashMap<String, String>)> {
    let content = std::fs::read_to_string(vcf).expect("read vcf");
    content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|line| {
            let cols: Vec<&str> = line.splitn(9, '\t').collect();
            let chrom = cols[0].to_owned();
            let pos: u64 = cols[1].parse().unwrap_or(0);
            let info = cols.get(7).copied().unwrap_or(".");
            let mut map = HashMap::new();
            if info != "." {
                for kv in info.split(';') {
                    if let Some((k, v)) = kv.split_once('=') {
                        map.insert(k.to_owned(), v.to_owned());
                    } else {
                        map.insert(kv.to_owned(), String::new());
                    }
                }
            }
            ((chrom, pos), map)
        })
        .collect()
}

fn approx_eq_list(ours: &str, theirs: &str, tag: &str, tol: f64) -> Result<(), String> {
    let ours_vals: Vec<&str> = ours.split(',').collect();
    let theirs_vals: Vec<&str> = theirs.split(',').collect();
    if ours_vals.len() != theirs_vals.len() {
        return Err(format!(
            "{tag}: length mismatch — ours={} bcftools={}",
            ours_vals.len(),
            theirs_vals.len()
        ));
    }
    for (i, (o, t)) in ours_vals.iter().zip(theirs_vals.iter()).enumerate() {
        let ov: f64 = o
            .parse()
            .map_err(|_| format!("{tag}[{i}] bad float: {o}"))?;
        let tv: f64 = t
            .parse()
            .map_err(|_| format!("{tag}[{i}] bad float: {t}"))?;
        let err = (ov - tv).abs();
        let rel_denom = tv.abs().max(ov.abs()).max(1e-30);
        if err / rel_denom > tol && err > 1e-6 {
            return Err(format!(
                "{tag}[{i}]: ours={ov} bcftools={tv} abs_diff={err:.3e}",
            ));
        }
    }
    Ok(())
}

const FLOAT_TAGS: &[&str] = &["AF", "MAF", "HWE", "ExcHet"];
const INT_TAGS: &[&str] = &["AN", "AC", "NS", "AC_Hom", "AC_Het", "AC_Hemi"];
const ALL_TAGS: &[&str] = &[
    "AN", "AC", "AF", "MAF", "NS", "AC_Hom", "AC_Het", "AC_Hemi", "HWE", "ExcHet",
];

fn fixture_path() -> PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest).join("tests/golden/small.vcf")
}

#[test]
fn fill_tags_matches_bcftools() {
    let Some(ver) = bcftools_version() else {
        eprintln!("SKIP fill_tags_matches_bcftools: bcftools not found in PATH");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let ours_out = tmp.path().join("ours.vcf");
    let bcf_out = tmp.path().join("bcftools.vcf");
    let input = fixture_path();

    // Check that the fill-tags plugin is available.
    let plugin_ok = Command::new("bcftools")
        .args(["+fill-tags", "--help"])
        .output()
        .is_ok_and(|o| o.status.success() || !o.stderr.is_empty());
    if !plugin_ok {
        eprintln!("SKIP fill_tags_matches_bcftools: bcftools +fill-tags plugin unavailable");
        return;
    }

    run_ours(&input, &ours_out);
    if !run_bcftools_fill_tags(&input, &bcf_out) {
        eprintln!("SKIP fill_tags_matches_bcftools: bcftools +fill-tags failed");
        return;
    }

    eprintln!("bcftools version: {ver}");

    let ours_records = parse_vcf_info(&ours_out);
    let bcf_records = parse_vcf_info(&bcf_out);

    assert_eq!(
        ours_records.len(),
        bcf_records.len(),
        "record count mismatch"
    );

    for ((ours_key, ours_map), (bcf_key, bcf_map)) in ours_records.iter().zip(bcf_records.iter()) {
        assert_eq!(ours_key, bcf_key, "record key mismatch");
        let loc = format!("{}:{}", ours_key.0, ours_key.1);

        for &tag in ALL_TAGS {
            let ours_val = ours_map.get(tag).map_or(".", String::as_str);
            let bcf_val = bcf_map.get(tag).map_or(".", String::as_str);

            if FLOAT_TAGS.contains(&tag) {
                if let Err(e) = approx_eq_list(ours_val, bcf_val, tag, 1e-3) {
                    panic!("[{loc}] {e}");
                }
            } else if INT_TAGS.contains(&tag) {
                assert_eq!(
                    ours_val, bcf_val,
                    "[{loc}] tag {tag}: ours={ours_val} bcftools={bcf_val}"
                );
            }
        }
    }

    eprintln!("compat OK against {ver}");
}
