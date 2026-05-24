use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

fn bench_inner_distance(c: &mut Criterion) {
    let bam = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/pairs.bam");
    let bed = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/genes.bed12");

    if !bam.exists() {
        eprintln!("SKIP: golden fixture not found (run tests/make_golden.py first)");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("bench_out").to_string_lossy().into_owned();

    c.bench_function("inner_distance_golden", |b| {
        b.iter(|| {
            rsomics_inner_distance::compute_inner_distance(&bam, &bed, 1_000_000, 30, -250, 250, 5)
                .unwrap();
        });
    });

    // Compare against inner_distance.py if available.
    let rseqc = which_inner_distance();
    if let Some(rseqc_bin) = rseqc {
        let our_bin = env!("CARGO_BIN_EXE_rsomics-inner-distance");

        let our_start = Instant::now();
        for _ in 0..10 {
            Command::new(our_bin)
                .args([
                    "-i",
                    bam.to_str().unwrap(),
                    "-r",
                    bed.to_str().unwrap(),
                    "-o",
                    &prefix,
                ])
                .output()
                .unwrap();
        }
        let our_mean = our_start.elapsed() / 10;

        let rseqc_start = Instant::now();
        for _ in 0..10 {
            Command::new(&rseqc_bin)
                .args([
                    "-i",
                    bam.to_str().unwrap(),
                    "-r",
                    bed.to_str().unwrap(),
                    "-o",
                    &prefix,
                ])
                .output()
                .unwrap();
        }
        let rseqc_mean = rseqc_start.elapsed() / 10;

        let ratio = rseqc_mean.as_secs_f64() / our_mean.as_secs_f64();
        println!(
            "\nPerf: ours={:.1}ms rseqc={:.1}ms ratio={:.2}x",
            our_mean.as_millis(),
            rseqc_mean.as_millis(),
            ratio
        );
    }
}

fn which_inner_distance() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")?;
    let base = Path::new(&home).join("Library").join("Python");
    if let Ok(rd) = std::fs::read_dir(&base) {
        let mut versions: Vec<String> = rd
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        versions.sort_unstable_by(|a, b| b.cmp(a));
        for v in versions {
            let p = base.join(&v).join("bin").join("inner_distance.py");
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(5));
    targets = bench_inner_distance
}
criterion_main!(benches);
