use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::PathBuf;
use std::process::Command;

fn bench_freq(c: &mut Criterion) {
    let bin = env!("CARGO_BIN_EXE_rsomics-plink-io");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bfile = manifest.join("tests/golden/small");
    c.bench_function("rsomics-plink-io freq golden", |b| {
        b.iter(|| {
            let out = Command::new(black_box(bin))
                .args(["freq", bfile.to_str().unwrap()])
                .output()
                .unwrap();
            assert!(out.status.success());
        });
    });
}

criterion_group!(benches, bench_freq);
criterion_main!(benches);
