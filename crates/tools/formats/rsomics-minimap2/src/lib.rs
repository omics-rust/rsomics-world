use std::path::Path;
use rsomics_common::{Result, RsomicsError};

pub fn align(reference: &Path, query: &Path, preset: &str) -> Result<String> {
    let aligner = minimap2::Aligner::builder()
        .preset(match preset {
            "sr" => minimap2::Preset::Sr,
            "map-ont" => minimap2::Preset::MapOnt,
            "map-hifi" => minimap2::Preset::MapHifi,
            "map-pb" => minimap2::Preset::MapPb,
            _ => minimap2::Preset::MapOnt,
        })
        .with_index(reference.to_str().unwrap_or(""), None)
        .map_err(|e| RsomicsError::InvalidInput(format!("building index: {e}")))?;

    let seqs = std::fs::read_to_string(query)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", query.display())))?;

    let mut output = String::new();
    for record in seqs.split('>').skip(1) {
        let mut lines = record.lines();
        let name = lines.next().unwrap_or("unknown");
        let seq: String = lines.collect();
        
        let mappings = aligner.map(seq.as_bytes(), false, false, None, None, Some(name.as_bytes()))
            .map_err(|e| RsomicsError::InvalidInput(format!("mapping: {e}")))?;
        
        for m in mappings {
            output.push_str(&format!("{}\n", m));
        }
    }

    Ok(output)
}
