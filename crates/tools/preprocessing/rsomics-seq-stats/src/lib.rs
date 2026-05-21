use rsomics_common::{Result, RsomicsError};
use std::io::Write;
use std::path::Path;

#[allow(clippy::cast_precision_loss)]
pub fn seq_stats(input: &Path, output: &mut dyn Write) -> Result<()> {
    let mut reader = needletail::parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut lengths: Vec<u64> = Vec::new();
    let mut gc = 0u64;
    let mut total_bp = 0u64;

    while let Some(result) = reader.next() {
        let record = result.map_err(|e| RsomicsError::InvalidInput(format!("read: {e}")))?;
        let seq = record.seq();
        let len = seq.len() as u64;
        lengths.push(len);
        total_bp += len;
        gc += seq
            .iter()
            .filter(|b| matches!(b, b'G' | b'g' | b'C' | b'c'))
            .count() as u64;
    }

    if lengths.is_empty() {
        return Err(RsomicsError::InvalidInput("no sequences".into()));
    }

    lengths.sort_unstable();
    let n = lengths.len();
    let min = lengths[0];
    let max = lengths[n - 1];
    let mean = total_bp as f64 / n as f64;

    let mut cumsum = 0u64;
    let half = total_bp / 2;
    let mut n50 = 0u64;
    for &l in lengths.iter().rev() {
        cumsum += l;
        if cumsum >= half {
            n50 = l;
            break;
        }
    }

    let gc_pct = if total_bp > 0 {
        gc as f64 / total_bp as f64 * 100.0
    } else {
        0.0
    };

    writeln!(output, "sequences\t{n}").map_err(RsomicsError::Io)?;
    writeln!(output, "total_bp\t{total_bp}").map_err(RsomicsError::Io)?;
    writeln!(output, "min_len\t{min}").map_err(RsomicsError::Io)?;
    writeln!(output, "max_len\t{max}").map_err(RsomicsError::Io)?;
    writeln!(output, "mean_len\t{mean:.1}").map_err(RsomicsError::Io)?;
    writeln!(output, "N50\t{n50}").map_err(RsomicsError::Io)?;
    writeln!(output, "GC%\t{gc_pct:.2}").map_err(RsomicsError::Io)?;
    Ok(())
}
