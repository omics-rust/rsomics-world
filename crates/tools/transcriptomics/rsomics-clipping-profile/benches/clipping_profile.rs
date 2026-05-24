use criterion::{Criterion, criterion_group, criterion_main};
use std::num::NonZero;

fn bench_clipping_profile(c: &mut Criterion) {
    let bam = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/clip.bam");
    if !bam.exists() {
        return;
    }
    c.bench_function("clipping_profile_se", |b| {
        b.iter(|| {
            rsomics_clipping_profile::compute_se_pub(&bam, 30, NonZero::new(1).unwrap()).unwrap();
        });
    });
}

criterion_group!(benches, bench_clipping_profile);
criterion_main!(benches);
