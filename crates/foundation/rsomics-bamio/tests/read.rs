use std::path::Path;

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn reads_header_and_all_records() {
    let mut reader = rsomics_bamio::open_parallel(fixture()).unwrap();
    let header = reader.read_header().unwrap();
    assert!(!header.reference_sequences().is_empty());

    let n = reader.records().count();
    assert_eq!(n, 10);
}

#[test]
fn single_worker_reads_same_count() {
    use std::num::NonZero;
    let mut reader = rsomics_bamio::open_with_workers(fixture(), NonZero::new(1).unwrap()).unwrap();
    reader.read_header().unwrap();
    assert_eq!(reader.records().count(), 10);
}
