use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

fn bench_ours(c: &mut Criterion) {
    let bam = std::env::var("BCMR_BENCH_BAM")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/mpileup_2_4m_cs.bam".into());
    let fasta = std::env::var("BCMR_BENCH_FASTA")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/calmd_ref.fa".into());

    if !Path::new(&bam).exists() || !Path::new(&fasta).exists() {
        eprintln!("SKIP bench: fixtures not found at {bam} / {fasta}");
        return;
    }

    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let binary = format!("{target_dir}/release/rsomics-vcf-mpileup");

    if !Path::new(&binary).exists() {
        eprintln!("SKIP bench: release binary not found at {binary}");
        return;
    }

    c.bench_function("rsomics-vcf-mpileup", |b| {
        b.iter(|| {
            let status = Command::new(&binary)
                .args(["-f", &fasta, &bam])
                .stdout(std::process::Stdio::null())
                .status()
                .expect("failed to run rsomics-vcf-mpileup");
            assert!(status.success());
        });
    });
}

fn bench_bcftools(c: &mut Criterion) {
    let bam = std::env::var("BCMR_BENCH_BAM")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/mpileup_2_4m_cs.bam".into());
    let fasta = std::env::var("BCMR_BENCH_FASTA")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures/calmd_ref.fa".into());

    if !Path::new(&bam).exists() || !Path::new(&fasta).exists() {
        eprintln!("SKIP bench: fixtures not found");
        return;
    }

    c.bench_function("bcftools-mpileup", |b| {
        b.iter(|| {
            let status = Command::new("bcftools")
                .args([
                    "mpileup", "-Ov", "-a", "AD,DP", "-f", &fasta, "-q", "0", "-Q", "1", &bam,
                ])
                .stdout(std::process::Stdio::null())
                .status()
                .expect("failed to run bcftools mpileup");
            assert!(status.success());
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(5)
        .measurement_time(Duration::from_secs(60));
    targets = bench_ours, bench_bcftools
);
criterion_main!(benches);
