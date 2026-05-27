use criterion::{Criterion, criterion_group, criterion_main};
use std::num::NonZero;

fn bench_insertion_profile(c: &mut Criterion) {
    let bam = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/ins.bam");
    if !bam.exists() {
        return;
    }
    c.bench_function("insertion_profile_se", |b| {
        b.iter(|| {
            rsomics_insertion_profile::compute_se(&bam, 30, NonZero::new(1).unwrap()).unwrap();
        });
    });
}

criterion_group!(benches, bench_insertion_profile);
criterion_main!(benches);
