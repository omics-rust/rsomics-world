use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

fn bench_ours(c: &mut Criterion) {
    let bam = std::env::var("BCMR_BENCH_BAM")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/calmd_chr1_150x.bam".into());

    if !Path::new(&bam).exists() {
        eprintln!("SKIP bench: fixture not found at {bam}");
        return;
    }

    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let binary = format!("{target_dir}/release/rsomics-bam-consensus");

    if !Path::new(&binary).exists() {
        eprintln!("SKIP bench: release binary not found at {binary}");
        return;
    }

    c.bench_function("rsomics-bam-consensus", |b| {
        b.iter(|| {
            let status = Command::new(&binary)
                .arg(&bam)
                .stdout(std::process::Stdio::null())
                .status()
                .expect("failed to run rsomics-bam-consensus");
            assert!(status.success());
        });
    });
}

fn bench_samtools(c: &mut Criterion) {
    let bam = std::env::var("BCMR_BENCH_BAM")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/calmd_chr1_150x.bam".into());

    if !Path::new(&bam).exists() {
        eprintln!("SKIP bench: fixture not found");
        return;
    }

    c.bench_function("samtools-consensus-simple", |b| {
        b.iter(|| {
            let status = Command::new("samtools")
                .args(["consensus", "-m", "simple", &bam])
                .stdout(std::process::Stdio::null())
                .status()
                .expect("failed to run samtools consensus");
            assert!(status.success());
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(30));
    targets = bench_ours, bench_samtools
);
criterion_main!(benches);
