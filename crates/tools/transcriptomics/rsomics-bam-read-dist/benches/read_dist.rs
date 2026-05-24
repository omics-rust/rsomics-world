use std::num::NonZero;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_golden(c: &mut Criterion) {
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let bam = golden.join("reads.bam");
    let bed = golden.join("genes.bed12");

    c.bench_function("read_dist_golden", |b| {
        b.iter(|| {
            rsomics_bam_read_dist::run_read_dist(&bam, &bed, NonZero::new(1).unwrap()).unwrap();
        });
    });
}

criterion_group!(benches, bench_golden);
criterion_main!(benches);
