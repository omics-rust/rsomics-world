use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::open_with_workers;
use rsomics_common::{Result, RsomicsError};

pub struct ReadQualityOpts {
    /// Minimum mapping quality; reads with MAPQ < this are skipped.
    pub min_mapq: u8,
    /// Reduce divisor for the boxplot `times` values.
    pub reduce: u64,
    /// Number of BGZF decode threads.
    pub workers: NonZero<usize>,
}

/// Per-position quality score frequency matrix.
///
/// `counts[pos][score]` holds the number of bases at read position `pos`
/// with Phred quality `score` (0–93 range).
pub struct QualMatrix {
    /// `counts[position][phred_score]` = frequency
    pub counts: Vec<[u64; 94]>,
    /// Read length (number of positions observed)
    pub read_len: usize,
}

impl QualMatrix {
    fn new() -> Self {
        QualMatrix {
            counts: Vec::new(),
            read_len: 0,
        }
    }

    fn observe(&mut self, scores: &[u8]) {
        let len = scores.len();
        if len > self.counts.len() {
            self.counts.resize_with(len, || [0u64; 94]);
            self.read_len = self.read_len.max(len);
        }
        for (pos, &q) in scores.iter().enumerate() {
            let q = q as usize;
            if q < 94 {
                self.counts[pos][q] += 1;
            }
        }
    }

    /// Global min and max quality score seen across all positions.
    fn score_range(&self) -> (usize, usize) {
        let mut min_q = 93usize;
        let mut max_q = 0usize;
        for pos_counts in &self.counts {
            for (q, &c) in pos_counts.iter().enumerate() {
                if c > 0 {
                    if q < min_q {
                        min_q = q;
                    }
                    if q > max_q {
                        max_q = q;
                    }
                }
            }
        }
        (min_q, max_q)
    }

    /// Render the R script matching RSeQC's `.qual.r` output format.
    ///
    /// `output_prefix` is used verbatim in the `pdf(...)` calls, matching
    /// what RSeQC writes (absolute path + prefix).
    pub fn render_r_script(&self, output_prefix: &str, reduce: u64) -> String {
        if self.read_len == 0 {
            return String::new();
        }
        let (min_q, max_q) = self.score_range();
        let n_pos = self.read_len;

        let mut out = String::with_capacity(1024 * n_pos);

        // ── boxplot section ──────────────────────────────────────────────
        out.push_str(&format!("pdf('{output_prefix}.qual.boxplot.pdf')\n"));
        for pos in 0..n_pos {
            let pos_counts = &self.counts[pos];
            let scores_str: String = (min_q..=max_q)
                .map(|q| q.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let times_str: String = (min_q..=max_q)
                .map(|q| pos_counts[q].to_string())
                .collect::<Vec<_>>()
                .join(",");
            out.push_str(&format!(
                "p{pos}<-rep(c({scores_str}),times=c({times_str})/{reduce})\n"
            ));
        }

        // boxplot() call with all pN variables
        let args: String = (0..n_pos)
            .map(|i| format!("p{i}"))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!(
            "boxplot({args},xlab=\"Position of Read(5'->3')\",ylab=\"Phred Quality Score\",outline=F)\n"
        ));
        out.push_str("dev.off()\n");
        out.push('\n');
        out.push('\n');

        // ── heatmap section ──────────────────────────────────────────────
        out.push_str(&format!("pdf('{output_prefix}.qual.heatmap.pdf')\n"));

        // Column-major matrix: each column = one position, rows = quality scores.
        // qual[pos * n_levels + level_idx] = count at that position for that score.
        let n_levels = max_q - min_q + 1;
        let mut flat: Vec<u64> = Vec::with_capacity(n_pos * n_levels);
        for pos in 0..n_pos {
            let pos_counts = &self.counts[pos];
            flat.extend_from_slice(&pos_counts[min_q..=max_q]);
        }
        let flat_str: String = flat
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!("qual=c({flat_str})\n"));
        out.push_str(&format!("mat=matrix(qual,ncol={n_pos},byrow=F)\n"));
        out.push_str("Lab.palette <- colorRampPalette(c(\"blue\", \"orange\", \"red3\",\"red2\",\"red1\",\"red\"), space = \"rgb\",interpolate=c('spline'))\n");
        out.push_str(&format!(
            "heatmap(mat,Rowv=NA,Colv=NA,xlab=\"Position of Read\",ylab=\"Phred Quality Score\",labRow=seq(from={min_q},to={max_q}),col = Lab.palette(256),scale=\"none\" )\n"
        ));
        out.push_str("dev.off()\n");

        out
    }
}

pub fn run_read_quality(input: &Path, output_prefix: &str, opts: &ReadQualityOpts) -> Result<()> {
    let mut reader = open_with_workers(input, opts.workers)?;
    let _header = reader.read_header().map_err(RsomicsError::Io)?;

    let mut matrix = QualMatrix::new();

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;

        // MAPQ 255 means unavailable — skip those reads.
        let mapq = match record.mapping_quality() {
            Some(mq) => mq.get(),
            None => continue,
        };
        if mapq < opts.min_mapq {
            continue;
        }

        let scores = record.quality_scores();
        let raw: &[u8] = scores.as_ref();
        if raw.is_empty() {
            continue;
        }
        matrix.observe(raw);
    }

    let r_script = matrix.render_r_script(output_prefix, opts.reduce);

    let out_path = format!("{output_prefix}.qual.r");
    std::fs::write(&out_path, r_script).map_err(RsomicsError::Io)?;

    Ok(())
}
