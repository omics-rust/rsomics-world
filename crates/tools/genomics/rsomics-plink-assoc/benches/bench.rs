use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_pgen::Pgen;
use rsomics_plink_assoc::{assoc_test, linear_test};
use std::hint::black_box;
use std::path::Path;

fn small_bfile() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/small")
}

fn quant_bfile() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/quant")
}

fn bench_assoc(c: &mut Criterion) {
    let pgen = Pgen::load(Path::new(small_bfile())).expect("load small pgen");
    c.bench_function("assoc_test_100v_10s", |b| {
        b.iter(|| {
            let recs = assoc_test(&pgen);
            black_box(recs);
        })
    });
}

fn bench_linear(c: &mut Criterion) {
    let pgen = Pgen::load(Path::new(quant_bfile())).expect("load quant pgen");
    c.bench_function("linear_test_100v_10s", |b| {
        b.iter(|| {
            let recs = linear_test(&pgen);
            black_box(recs);
        })
    });
}

criterion_group!(benches, bench_assoc, bench_linear);
criterion_main!(benches);
