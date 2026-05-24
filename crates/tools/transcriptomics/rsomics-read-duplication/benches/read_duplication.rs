use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_read_duplication::compute_duplication;
use std::num::NonZero;
use std::path::Path;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn bench_compute(c: &mut Criterion) {
    let bam = Path::new(GOLDEN).join("dup.bam");
    if !bam.exists() {
        return;
    }
    c.bench_function("compute_duplication_golden", |b| {
        b.iter(|| {
            compute_duplication(&bam, 30, NonZero::new(1).unwrap()).unwrap();
        });
    });
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);
