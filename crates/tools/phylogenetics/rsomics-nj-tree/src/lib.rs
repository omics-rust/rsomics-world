use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn nj_from_matrix(input: &Path, output: &mut dyn Write) -> Result<()> {
    let file = std::fs::File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty input".into()))?
        .map_err(RsomicsError::Io)?;
    let names: Vec<&str> = header.split('\t').collect();
    let n = names.len();

    let mut dist = vec![vec![0.0f64; n]; n];
    for (i, line) in lines.enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        let vals: Vec<&str> = line.split('\t').collect();
        if vals.len() != n {
            return Err(RsomicsError::InvalidInput(format!(
                "row {i} has {} columns, expected {n}",
                vals.len()
            )));
        }
        for (j, v) in vals.iter().enumerate() {
            dist[i][j] = v
                .parse()
                .map_err(|e| RsomicsError::InvalidInput(format!("bad float: {e}")))?;
        }
    }

    let newick = neighbor_joining(&names, &dist);
    writeln!(output, "{newick}").map_err(RsomicsError::Io)?;
    Ok(())
}

#[allow(clippy::cast_precision_loss, clippy::many_single_char_names)]
fn neighbor_joining(names: &[&str], dist: &[Vec<f64>]) -> String {
    let n = names.len();
    if n <= 1 {
        return names.first().map_or(String::new(), |s| format!("{s};"));
    }

    let mut d: Vec<Vec<f64>> = dist.to_vec();
    let mut labels: Vec<String> = names.iter().map(|s| (*s).to_string()).collect();
    let mut active: Vec<bool> = vec![true; n];

    while labels.iter().filter(|_| true).count() > 0 {
        let alive: Vec<usize> = (0..active.len()).filter(|&i| active[i]).collect();
        let m = alive.len();
        if m <= 2 {
            break;
        }

        let mut r = vec![0.0f64; active.len()];
        for &i in &alive {
            for &j in &alive {
                r[i] += d[i][j];
            }
            r[i] /= (m - 2) as f64;
        }

        let mut min_val = f64::INFINITY;
        let mut min_i = 0;
        let mut min_j = 0;
        for (ai, &i) in alive.iter().enumerate() {
            for &j in &alive[ai + 1..] {
                let q = d[i][j] - r[i] - r[j];
                if q < min_val {
                    min_val = q;
                    min_i = i;
                    min_j = j;
                }
            }
        }

        let dist_ij = d[min_i][min_j];
        let branch_i = 0.5 * dist_ij + 0.5 * (r[min_i] - r[min_j]);
        let branch_j = dist_ij - branch_i;

        let new_label = format!(
            "({}:{branch_i:.6},{}:{branch_j:.6})",
            labels[min_i], labels[min_j]
        );

        let new_idx = d.len();
        let mut new_row = vec![0.0; new_idx + 1];
        for &k in &alive {
            if k == min_i || k == min_j {
                continue;
            }
            let dk = 0.5 * (d[k][min_i] + d[k][min_j] - dist_ij);
            new_row[k] = dk;
        }
        for row in &mut d {
            row.push(0.0);
        }
        d.push(new_row.clone());
        for (k, &val) in new_row.iter().enumerate() {
            if k < d.len() - 1 {
                d[k][new_idx] = val;
            }
        }

        labels.push(new_label);
        active.push(true);
        active[min_i] = false;
        active[min_j] = false;
    }

    let alive: Vec<usize> = (0..active.len()).filter(|&i| active[i]).collect();
    if alive.len() == 2 {
        let i = alive[0];
        let j = alive[1];
        format!(
            "({}:{:.6},{}:{:.6});",
            labels[i],
            d[i][j] / 2.0,
            labels[j],
            d[i][j] / 2.0
        )
    } else if alive.len() == 1 {
        format!("{};", labels[alive[0]])
    } else {
        String::from("();")
    }
}
