use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct PairStats {
    pub paired: u64,
    pub singletons: u64,
}

pub fn pair_fastq(
    r1_path: &Path,
    r2_path: &Path,
    out1: &mut dyn Write,
    out2: &mut dyn Write,
    singles: Option<&mut dyn Write>,
) -> Result<PairStats> {
    let r2_index = index_fastq(r2_path)?;

    let file1 = File::open(r1_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", r1_path.display())))?;
    let reader1 = BufReader::new(file1);
    let mut o1 = BufWriter::with_capacity(256 * 1024, out1);
    let mut o2 = BufWriter::with_capacity(256 * 1024, out2);

    let file2 = File::open(r2_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", r2_path.display())))?;
    let reader2 = BufReader::new(file2);
    let r2_records = read_all_records(reader2)?;

    let mut matched: HashMap<String, bool> = HashMap::new();
    let mut stats = PairStats {
        paired: 0,
        singletons: 0,
    };

    let mut lines = reader1.lines();
    while let Some(header) = lines.next() {
        let header = header.map_err(RsomicsError::Io)?;
        let seq = next_line(&mut lines)?;
        let plus = next_line(&mut lines)?;
        let qual = next_line(&mut lines)?;

        let name = read_name(&header);
        if let Some(&idx) = r2_index.get(&name) {
            let r2 = &r2_records[idx];
            writeln!(o1, "{header}").map_err(RsomicsError::Io)?;
            writeln!(o1, "{seq}").map_err(RsomicsError::Io)?;
            writeln!(o1, "{plus}").map_err(RsomicsError::Io)?;
            writeln!(o1, "{qual}").map_err(RsomicsError::Io)?;

            writeln!(o2, "{}", r2[0]).map_err(RsomicsError::Io)?;
            writeln!(o2, "{}", r2[1]).map_err(RsomicsError::Io)?;
            writeln!(o2, "{}", r2[2]).map_err(RsomicsError::Io)?;
            writeln!(o2, "{}", r2[3]).map_err(RsomicsError::Io)?;

            matched.insert(name, true);
            stats.paired += 1;
        } else if let Some(s) = singles.as_deref_mut() {
            writeln!(s, "{header}").map_err(RsomicsError::Io)?;
            writeln!(s, "{seq}").map_err(RsomicsError::Io)?;
            writeln!(s, "{plus}").map_err(RsomicsError::Io)?;
            writeln!(s, "{qual}").map_err(RsomicsError::Io)?;
            stats.singletons += 1;
        }
    }

    o1.flush().map_err(RsomicsError::Io)?;
    o2.flush().map_err(RsomicsError::Io)?;
    Ok(stats)
}

fn read_name(header: &str) -> String {
    header
        .split_once(|c: char| c.is_whitespace() || c == '/')
        .map_or(header, |(name, _)| name)
        .trim_start_matches('@')
        .to_string()
}

fn next_line(lines: &mut std::io::Lines<BufReader<File>>) -> Result<String> {
    lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("truncated FASTQ".into()))?
        .map_err(RsomicsError::Io)
}

fn index_fastq(path: &Path) -> Result<HashMap<String, usize>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut index = HashMap::new();
    let mut record_idx: usize = 0;

    let mut lines = reader.lines();
    while let Some(header) = lines.next() {
        let header = header.map_err(RsomicsError::Io)?;
        let _ = next_line_generic(&mut lines)?;
        let _ = next_line_generic(&mut lines)?;
        let _ = next_line_generic(&mut lines)?;

        let name = read_name(&header);
        index.insert(name, record_idx);
        record_idx += 1;
    }

    Ok(index)
}

fn read_all_records<R: BufRead>(reader: R) -> Result<Vec<[String; 4]>> {
    let mut records = Vec::new();
    let mut lines = reader.lines();
    while let Some(h) = lines.next() {
        let h = h.map_err(RsomicsError::Io)?;
        let s = next_line_generic(&mut lines)?;
        let p = next_line_generic(&mut lines)?;
        let q = next_line_generic(&mut lines)?;
        records.push([h, s, p, q]);
    }
    Ok(records)
}

fn next_line_generic<B: BufRead>(lines: &mut std::io::Lines<B>) -> Result<String> {
    lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("truncated FASTQ".into()))?
        .map_err(RsomicsError::Io)
}
