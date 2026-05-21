use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_hmm::Hmm;

pub fn decode_observations(
    model_path: &Path,
    obs_path: &Path,
    output: &mut dyn Write,
) -> Result<u64> {
    let model_json = fs::read_to_string(model_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", model_path.display())))?;

    let v: serde_json::Value = serde_json::from_str(&model_json)
        .map_err(|e| RsomicsError::InvalidInput(format!("bad model JSON: {e}")))?;

    let pi: Vec<f64> = v["pi"]
        .as_array()
        .ok_or_else(|| RsomicsError::InvalidInput("missing 'pi' array".into()))?
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .collect();
    let trans: Vec<f64> = v["trans"]
        .as_array()
        .ok_or_else(|| RsomicsError::InvalidInput("missing 'trans' array".into()))?
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .collect();
    let emit: Vec<f64> = v["emit"]
        .as_array()
        .ok_or_else(|| RsomicsError::InvalidInput("missing 'emit' array".into()))?
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .collect();
    let n_symbols = v["n_symbols"]
        .as_u64()
        .ok_or_else(|| RsomicsError::InvalidInput("missing 'n_symbols'".into()))?
        .try_into()
        .map_err(|_| RsomicsError::InvalidInput("n_symbols too large".into()))?;

    let hmm = Hmm::new(pi, trans, emit, n_symbols)
        .map_err(|e| RsomicsError::InvalidInput(format!("invalid HMM: {e}")))?;

    let obs_file = std::fs::File::open(obs_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", obs_path.display())))?;
    let reader = BufReader::new(obs_file);
    let mut out = BufWriter::new(output);
    let mut count = 0u64;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let obs: Vec<usize> = line
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if obs.is_empty() {
            continue;
        }

        let states = hmm
            .viterbi(&obs)
            .map_err(|e| RsomicsError::InvalidInput(format!("viterbi: {e}")))?;
        let state_str: Vec<String> = states.iter().map(ToString::to_string).collect();
        writeln!(out, "{}", state_str.join(" ")).map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
