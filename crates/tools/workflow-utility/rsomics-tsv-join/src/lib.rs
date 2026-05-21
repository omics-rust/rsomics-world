use rsomics_common::{Result, RsomicsError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn inner_join(left: &Path, right: &Path, key_col: &str, output: &mut dyn Write) -> Result<u64> {
    let (rh, rdata) = load_tsv(right, key_col)?;
    let src_file = File::open(left)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", left.display())))?;
    let buf_reader = BufReader::new(src_file);
    let mut out = BufWriter::new(output);
    let mut lines = buf_reader.lines();

    let hdr_line = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty left".into()))?
        .map_err(RsomicsError::Io)?;
    let lcols: Vec<&str> = hdr_line.split('\t').collect();
    let lki = lcols
        .iter()
        .position(|c| *c == key_col)
        .ok_or_else(|| RsomicsError::InvalidInput(format!("'{key_col}' not in left")))?;

    write!(out, "{hdr_line}").map_err(RsomicsError::Io)?;
    for h in &rh {
        if *h != key_col {
            write!(out, "\t{h}").map_err(RsomicsError::Io)?;
        }
    }
    writeln!(out).map_err(RsomicsError::Io)?;

    let mut count = 0u64;
    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let fields: Vec<&str> = line.split('\t').collect();
        let key = fields.get(lki).copied().unwrap_or("");
        if let Some(rrow) = rdata.get(key) {
            write!(out, "{line}").map_err(RsomicsError::Io)?;
            for (i, val) in rrow.iter().enumerate() {
                if rh[i] != key_col {
                    write!(out, "\t{val}").map_err(RsomicsError::Io)?;
                }
            }
            writeln!(out).map_err(RsomicsError::Io)?;
            count += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

type RowMap = HashMap<String, Vec<String>>;

fn load_tsv(path: &Path, key_col_name: &str) -> Result<(Vec<String>, RowMap)> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty".into()))?
        .map_err(RsomicsError::Io)?;
    let cols: Vec<String> = header.split('\t').map(String::from).collect();
    let ki = cols
        .iter()
        .position(|c| c == key_col_name)
        .ok_or_else(|| RsomicsError::InvalidInput(format!("'{key_col_name}' not in right")))?;

    let mut data = HashMap::new();
    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let fields: Vec<String> = line.split('\t').map(String::from).collect();
        let key = fields.get(ki).cloned().unwrap_or_default();
        data.insert(key, fields);
    }
    Ok((cols, data))
}
