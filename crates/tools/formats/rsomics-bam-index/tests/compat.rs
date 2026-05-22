use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-index"))
}

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.bam")
}

fn samtools_available() -> bool {
    Command::new("samtools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn idxstats(bam: &Path) -> String {
    let out = Command::new("samtools")
        .arg("idxstats")
        .arg(bam)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "idxstats: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// The .bai ours writes is byte-different from samtools' (different binning) but
// must be FUNCTIONALLY equivalent: `samtools idxstats` (which reads the .bai's
// per-reference record counts) yields identical output with either index.
#[test]
fn bai_functionally_matches_samtools() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-index-compat");
    let _ = std::fs::create_dir_all(&dir);
    let ours_bam = dir.join("ours.bam");
    let sam_bam = dir.join("sam.bam");
    std::fs::copy(golden(), &ours_bam).unwrap();
    std::fs::copy(golden(), &sam_bam).unwrap();

    assert!(ours().arg(&ours_bam).status().unwrap().success());
    assert!(
        Command::new("samtools")
            .arg("index")
            .arg(&sam_bam)
            .status()
            .unwrap()
            .success()
    );

    assert_eq!(idxstats(&ours_bam), idxstats(&sam_bam));
}
