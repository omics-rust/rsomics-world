use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_tin::compute_tin_score;

fn bench_tin_score(c: &mut Criterion) {
    // Simulated non-uniform coverage vector (1000 bp mRNA, ramping edges)
    // Non-uniform coverage: ramp up, plateau, ramp down (mirrors realistic RNA-seq)
    let cov: Vec<u32> = (0u32..1000)
        .map(|i| {
            if i < 100 {
                i / 5 + 1
            } else if i >= 900 {
                (1000 - i) / 5 + 1
            } else {
                20
            }
        })
        .collect();

    c.bench_function("compute_tin_score_1kb", |b| {
        b.iter(|| std::hint::black_box(compute_tin_score(std::hint::black_box(&cov), 100)));
    });
}

criterion_group!(benches, bench_tin_score);
criterion_main!(benches);
