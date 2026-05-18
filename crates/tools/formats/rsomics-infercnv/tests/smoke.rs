use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-infercnv"))
}

fn golden(name: &str) -> String {
    format!("{}/tests/golden/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn help_exits_zero() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("rsomics-infercnv"));
}

#[test]
fn basic_cnv_run() {
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("cnv.tsv");

    let status = bin()
        .args([
            "--matrix",
            &golden("counts.tsv"),
            "--gtf",
            &golden("genes.gtf"),
            "--ref-cells",
            &golden("normals.txt"),
            "--window",
            "3",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let contents = std::fs::read_to_string(&out_path).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 6, "header + 5 genes");

    let header = lines[0];
    assert!(header.starts_with("gene\t"));
    assert!(header.contains("CELL1"));
    assert!(header.contains("NORMAL1"));

    for line in &lines[1..] {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 6, "gene + 5 cells");
        for val in &fields[1..] {
            val.parse::<f64>()
                .unwrap_or_else(|_| panic!("not a float: {val}"));
        }
    }
}

#[test]
fn no_ref_cells_still_works() {
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("cnv.tsv");

    let status = bin()
        .args([
            "--matrix",
            &golden("counts.tsv"),
            "--gtf",
            &golden("genes.gtf"),
            "-o",
            out_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let contents = std::fs::read_to_string(&out_path).unwrap();
    assert!(contents.lines().count() > 1);
}
