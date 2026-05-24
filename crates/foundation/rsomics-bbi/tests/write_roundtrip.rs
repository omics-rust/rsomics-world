use rsomics_bbi::{BigWig, ChromInfo, Interval, write_bigwig};
use std::io::Cursor;

fn make_cursor() -> Cursor<Vec<u8>> {
    Cursor::new(Vec::new())
}

/// Write a small synthetic bigWig and read it back, verifying values match.
#[test]
fn roundtrip_single_chrom() {
    let chroms = vec![ChromInfo {
        name: "chr1".into(),
        id: 0,
        length: 1000,
    }];

    let intervals = vec![
        Interval {
            chrom_id: 0,
            start: 0,
            end: 100,
            value: 1.0,
        },
        Interval {
            chrom_id: 0,
            start: 100,
            end: 300,
            value: 3.0,
        },
        Interval {
            chrom_id: 0,
            start: 300,
            end: 500,
            value: 2.0,
        },
        Interval {
            chrom_id: 0,
            start: 500,
            end: 700,
            value: 4.0,
        },
        Interval {
            chrom_id: 0,
            start: 700,
            end: 1000,
            value: 0.5,
        },
    ];

    let mut buf = make_cursor();
    write_bigwig(&mut buf, &chroms, &intervals, 50).expect("write_bigwig failed");

    // Write to a temp file so BigWig::open can read it.
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), buf.into_inner()).unwrap();

    let mut bw = BigWig::open(tmp.path()).expect("BigWig::open failed");

    // Verify chr1 is present with correct length.
    assert_eq!(bw.chrom_len("chr1"), Some(1000));

    // Check exact mean over [0, 100): should be 1.0
    let mean = bw.mean_stat("chr1", 0, 100).unwrap().unwrap();
    assert!((mean - 1.0).abs() < 1e-4, "mean [0,100) = {mean}");

    // Check exact mean over [100, 300): should be 3.0
    let mean = bw.mean_stat("chr1", 100, 300).unwrap().unwrap();
    assert!((mean - 3.0).abs() < 1e-4, "mean [100,300) = {mean}");

    // Check mean over full range: weighted average.
    // sum = 1*100 + 3*200 + 2*200 + 4*200 + 0.5*300 = 100+600+400+800+150 = 2050
    // covered = 1000
    let expected_mean = 2050.0 / 1000.0;
    let mean = bw.mean_stat("chr1", 0, 1000).unwrap().unwrap();
    assert!(
        (mean - expected_mean).abs() < 1e-3,
        "mean [0,1000) = {mean}, expected {expected_mean}"
    );
}

/// Verify zoom-approximate mean is close to exact mean for large windows.
#[test]
fn zoom_mean_close_to_exact() {
    let chroms = vec![ChromInfo {
        name: "chr1".into(),
        id: 0,
        length: 100_000,
    }];

    // Generate uniform-ish intervals at 50 bp resolution.
    let mut intervals = Vec::new();
    let mut pos = 0u32;
    let mut val = 1.0f32;
    while pos < 100_000 {
        let end = (pos + 50).min(100_000);
        intervals.push(Interval {
            chrom_id: 0,
            start: pos,
            end,
            value: val,
        });
        pos = end;
        val = if val < 5.0 { val + 0.5 } else { 1.0 };
    }

    let mut buf = make_cursor();
    write_bigwig(&mut buf, &chroms, &intervals, 50).expect("write_bigwig failed");
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), buf.into_inner()).unwrap();

    let mut bw = BigWig::open(tmp.path()).expect("BigWig::open failed");

    // Check that zoom-level mean over a large window is within 5% of exact.
    let exact = bw.mean_stat("chr1", 0, 100_000).unwrap().unwrap();
    let zoom = bw.mean_stat_zoom("chr1", 0, 100_000).unwrap().unwrap();
    assert!(
        (exact - zoom).abs() / exact.abs() < 0.05,
        "zoom mean {zoom:.4} vs exact {exact:.4} deviate > 5%"
    );
}
