use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-tpm"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn tpm_columns_sum_to_one_million() {
    let out = Command::new(ours())
        .arg(golden("counts.tsv"))
        .args(["-l", &golden("lengths.tsv")])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();

    let lines: Vec<&str> = s.trim().lines().collect();
    let n_samples = lines[0].split('\t').count() - 1;
    let mut col_sums = vec![0.0f64; n_samples];

    for line in &lines[1..] {
        let vals: Vec<f64> = line
            .split('\t')
            .skip(1)
            .filter_map(|v| v.parse().ok())
            .collect();
        for (j, &v) in vals.iter().enumerate() {
            col_sums[j] += v;
        }
    }

    for (j, &sum) in col_sums.iter().enumerate() {
        assert!(
            (sum - 1_000_000.0).abs() < 1.0,
            "sample {j} TPM sum = {sum:.1}, expected ~1000000"
        );
    }
}
