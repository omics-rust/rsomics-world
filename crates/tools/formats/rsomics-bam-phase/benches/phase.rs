use std::io;
use std::num::NonZero;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_bam_phase::{PhaseOpts, phase};

fn bench_phase(c: &mut Criterion) {
    let base = std::env::var("BCMR_BENCH_DATA")
        .unwrap_or_else(|_| "/Volumes/Zane's HDD/rsomics-fixtures".to_string());
    let bam = Path::new(&base).join("phase_perf_large.bam");
    if !bam.exists() {
        eprintln!("SKIP: bench fixture not found at {}", bam.display());
        return;
    }

    let opts = PhaseOpts::default();
    let workers = NonZero::new(1).unwrap();

    c.bench_function("phase_600k_reads_t1", |b| {
        b.iter(|| {
            let mut sink = io::sink();
            phase(&bam, &mut sink, &opts, workers).unwrap();
        });
    });
}

criterion_group!(benches, bench_phase);
criterion_main!(benches);
