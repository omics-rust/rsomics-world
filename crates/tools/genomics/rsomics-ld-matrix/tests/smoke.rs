use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-ld-matrix"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn compute_ld() {
    let out = bin().arg(golden("geno.tsv")).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("var1\tvar2\tr_squared"));
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 3); // 3 pairs from 3 variants
    // rs1 and rs2 are identical → r²=1.0
    assert!(data[0].contains("1.000000"));
}
