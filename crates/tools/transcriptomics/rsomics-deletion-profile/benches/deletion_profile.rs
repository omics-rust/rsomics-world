use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

fn bench_ours(c: &mut Criterion) {
    let bam = std::env::var("BCMR_BENCH_BAM")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/rnaseq_perf_100k.bam".into());

    if !Path::new(&bam).exists() {
        eprintln!("SKIP bench: fixture not found at {bam}");
        return;
    }

    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let binary = format!("{target_dir}/release/rsomics-deletion-profile");

    if !Path::new(&binary).exists() {
        eprintln!("SKIP bench: release binary not found at {binary}");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();

    c.bench_function("rsomics-deletion-profile", |b| {
        b.iter(|| {
            let prefix = tmp.path().join("out");
            let status = Command::new(&binary)
                .args(["-i", &bam, "-l", "100", "-o", prefix.to_str().unwrap()])
                .status()
                .expect("failed to run rsomics-deletion-profile");
            assert!(status.success());
        });
    });
}

fn bench_rseqc(c: &mut Criterion) {
    let bam = std::env::var("BCMR_BENCH_BAM")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/rnaseq_perf_100k.bam".into());

    if !Path::new(&bam).exists() {
        eprintln!("SKIP bench: fixture not found");
        return;
    }

    let oracle = std::env::var("RSEQC_DELETION_PROFILE").unwrap_or_else(|_| {
        "/opt/homebrew/Caskroom/miniforge/base/envs/rs-up/bin/deletion_profile.py".into()
    });
    if !Path::new(&oracle).exists() {
        eprintln!("SKIP bench: deletion_profile.py not found at {oracle}");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();

    c.bench_function("rseqc-deletion_profile.py", |b| {
        b.iter(|| {
            let prefix = tmp.path().join("ref");
            let output = Command::new(&oracle)
                .args(["-i", &bam, "-l", "100", "-o", prefix.to_str().unwrap()])
                .output()
                .expect("failed to run deletion_profile.py");
            assert!(output.status.success());
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(30));
    targets = bench_ours, bench_rseqc
);
criterion_main!(benches);
