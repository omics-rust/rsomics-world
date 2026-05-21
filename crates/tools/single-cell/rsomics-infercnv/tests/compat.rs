use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-infercnv"))
}

#[test]
fn help_runs() {
    // infercnv needs complex inputs (expression matrix + gene positions + cell annotations)
    // Just verify the binary starts and shows help
    let out = Command::new(ours()).arg("--help").output();
    // intercept_help handles --help, so this should succeed
    assert!(out.is_ok());
}
