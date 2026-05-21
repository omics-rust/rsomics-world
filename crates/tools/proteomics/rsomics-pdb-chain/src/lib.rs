use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn list_chains(input: &Path, output: &mut dyn Write) -> Result<Vec<String>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut chains = BTreeSet::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if (line.starts_with("ATOM") || line.starts_with("HETATM")) && line.len() > 21 {
            let chain = &line[21..22];
            chains.insert(chain.to_string());
        }
    }

    let result: Vec<String> = chains.into_iter().collect();
    for c in &result {
        writeln!(output, "{c}").map_err(RsomicsError::Io)?;
    }
    Ok(result)
}

pub fn extract_chain(input: &Path, chain_id: &str, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::new(output);
    let mut count = 0u64;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with("ATOM") || line.starts_with("HETATM") {
            if line.len() > 21 && &line[21..22] == chain_id {
                writeln!(out, "{line}").map_err(RsomicsError::Io)?;
                count += 1;
            }
        } else if line.starts_with("HEADER")
            || line.starts_with("TITLE")
            || line.starts_with("REMARK")
            || line.starts_with("END")
        {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

pub fn split_chains(input: &Path, prefix: &Path) -> Result<BTreeMap<String, u64>> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut writers: BTreeMap<String, BufWriter<File>> = BTreeMap::new();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if (line.starts_with("ATOM") || line.starts_with("HETATM")) && line.len() > 21 {
            let chain = line[21..22].to_string();
            let writer = writers.entry(chain.clone()).or_insert_with(|| {
                let path = format!("{}_{chain}.pdb", prefix.display());
                BufWriter::new(File::create(path).expect("create chain file"))
            });
            writeln!(writer, "{line}").map_err(RsomicsError::Io)?;
            *counts.entry(chain).or_insert(0) += 1;
        }
    }

    for (_, mut w) in writers {
        w.flush().map_err(RsomicsError::Io)?;
    }
    Ok(counts)
}
