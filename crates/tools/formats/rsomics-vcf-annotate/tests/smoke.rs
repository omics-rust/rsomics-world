use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-vcf-annotate"))
}

fn golden(name: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn annotate_with_bed() {
    let out = bin()
        .arg(golden("small.vcf"))
        .args(["-a", &golden("regions.bed")])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ANN=GENE_A"));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("5 variants"));
}
