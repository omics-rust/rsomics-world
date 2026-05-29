use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_pgen::Pgen;
use rsomics_plink_pca::run_pca;

fn golden_prefix() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/test")
}

fn bench_pca(c: &mut Criterion) {
    let pgen = Pgen::load(&golden_prefix()).expect("load pgen");
    c.bench_function("pca_n10_k5", |b| {
        b.iter(|| {
            let mut evec = Vec::new();
            let mut eval = Vec::new();
            run_pca(&pgen, 5, &mut evec, &mut eval).unwrap();
        });
    });
}

criterion_group!(benches, bench_pca);
criterion_main!(benches);
