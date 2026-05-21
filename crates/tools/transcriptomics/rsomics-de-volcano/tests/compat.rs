use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-de-volcano"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn categories_are_correct() {
    let out = Command::new(ours()).arg(golden("de.tsv")).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines().skip(1) {
        let parts: Vec<&str> = line.split('\t').collect();
        let padj: f64 = parts[1].parse().unwrap();
        let lfc: f64 = parts[2].parse().unwrap();
        let cat = parts[3];
        match cat {
            "UP" => {
                assert!(padj <= 0.05 && lfc >= 1.0, "UP but padj={padj} lfc={lfc}");
            }
            "DOWN" => {
                assert!(
                    padj <= 0.05 && lfc <= -1.0,
                    "DOWN but padj={padj} lfc={lfc}"
                );
            }
            "NS" => {
                assert!(
                    padj > 0.05 || lfc.abs() < 1.0,
                    "NS but padj={padj} lfc={lfc}"
                );
            }
            _ => panic!("unknown category: {cat}"),
        }
    }
}
