use std::num::NonZero;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_bam_junctions::annotate_junctions;

fn bench_large(c: &mut Criterion) {
    let base = std::env::var("BCMR_BENCH_DATA").unwrap_or_else(|_| "/tmp".to_string());
    let bam = Path::new(&base).join("large_spliced.bam");
    let bed = Path::new(&base).join("large_genes.bed12");
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: bench fixture not found at {base}");
        return;
    }

    c.bench_function("annotate_junctions_t1", |b| {
        b.iter(|| annotate_junctions(&bam, &bed, 50, 30, NonZero::new(1).unwrap()).unwrap());
    });
}

criterion_group!(benches, bench_large);
criterion_main!(benches);
