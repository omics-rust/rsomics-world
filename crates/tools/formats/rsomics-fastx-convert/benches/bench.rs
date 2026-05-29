use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_fastx_convert::fq2fa;

fn bench_fq2fa(c: &mut Criterion) {
    let data = b"@read1\nACGTACGT\n+\nIIIIIIII\n@read2\nTTTTGGGG\n+\nIIIIIIII\n";
    c.bench_function("fq2fa_small", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            fq2fa(data.as_ref(), &mut out).unwrap();
        });
    });
}

criterion_group!(benches, bench_fq2fa);
criterion_main!(benches);
