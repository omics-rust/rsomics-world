use criterion::{Criterion, criterion_group, criterion_main};
use rsomics_genebody_coverage::{compute_coverage, load_transcripts};
use std::path::Path;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn bench_coverage(c: &mut Criterion) {
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let bam = Path::new(GOLDEN).join("reads.bam");
    if !bam.exists() || !bed.exists() {
        return;
    }
    let transcripts = load_transcripts(&bed, 100).unwrap();
    c.bench_function("genebody_coverage_golden", |b| {
        b.iter(|| compute_coverage(&bam, &transcripts).unwrap());
    });
}

criterion_group!(benches, bench_coverage);
criterion_main!(benches);
