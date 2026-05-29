use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_pgen::Pgen;
use rsomics_plink_ld::compute_ld;
use std::path::PathBuf;

fn golden_prefix() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/test")
}

fn bench_ld(c: &mut Criterion) {
    let pgen = Pgen::load(&golden_prefix()).expect("load pgen");
    c.bench_function("compute_ld_window50", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            compute_ld(&pgen, 50, 0.0, &mut out).unwrap();
        })
    });
}

criterion_group!(benches, bench_ld);
criterion_main!(benches);
