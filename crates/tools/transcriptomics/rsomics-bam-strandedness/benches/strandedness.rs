use std::num::NonZero;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_bam_strandedness::infer_strandedness;

fn bench_large(c: &mut Criterion) {
    // Tier-2 fixture: set BCMR_BENCH_DATA to a directory containing
    // large_fwd_pe.bam + large_genes.bed12. Falls back to /tmp.
    let base = std::env::var("BCMR_BENCH_DATA")
        .unwrap_or_else(|_| "/tmp".to_string());
    let bam = Path::new(&base).join("large_fwd_pe.bam");
    let bed = Path::new(&base).join("large_genes.bed12");
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: bench fixture not found at {base}");
        return;
    }

    c.bench_function("infer_strandedness_100k_t1", |b| {
        b.iter(|| {
            infer_strandedness(&bam, &bed, 200_000, 30, NonZero::new(1).unwrap()).unwrap()
        });
    });
}

criterion_group!(benches, bench_large);
criterion_main!(benches);
