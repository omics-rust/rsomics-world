/// Per-chromosome BAF histogram with 150 bins over [0, 1].
pub struct BafHistogram {
    pub chrom: String,
    /// Raw bin counts, length == NBINS.
    pub counts: Vec<f64>,
}

/// Default bin count from bcftools polysomy -n flag.
pub const NBINS: usize = 150;

impl BafHistogram {
    pub fn new(chrom: impl Into<String>) -> Self {
        Self {
            chrom: chrom.into(),
            counts: vec![0.0; NBINS],
        }
    }

    /// Add one BAF value to the histogram.
    pub fn push(&mut self, baf: f32) {
        let bin = (baf as f64 * (NBINS - 1) as f64) as usize;
        let bin = bin.min(NBINS - 1);
        self.counts[bin] += 1.0;
    }

    /// x-coordinate for bin i.
    pub fn xval(i: usize) -> f64 {
        i as f64 / (NBINS - 1) as f64
    }

    /// Apply a centred moving-average smoother with the given half-width.
    ///
    /// `smooth` mirrors bcftools' `-S` flag (default -3 → window 7).
    /// The returned vec has the same length as `counts`.
    pub fn smooth(&self, smooth: i32) -> Vec<f64> {
        let win = (smooth.unsigned_abs() as usize) * 2 + 1;
        let hwin = win / 2;
        let n = self.counts.len();
        let mut tmp = vec![0.0f64; n];

        // Left edge: partial windows.  Both tmp[i] and self.counts[2*i-1] need i.
        let mut avg = self.counts[0];
        tmp[0] = avg;
        #[allow(clippy::needless_range_loop)]
        for i in 1..hwin.min(n) {
            if 2 * i - 1 < n {
                avg += self.counts[2 * i - 1];
            }
            tmp[i] = avg / (2 * i + 1) as f64;
        }

        // Centre: full windows.
        let mut running = 0.0f64;
        for i in 0..n {
            running += self.counts[i];
            if i + 1 >= win {
                if i >= hwin {
                    tmp[i - hwin] = running / win as f64;
                }
                running -= self.counts[i + 1 - win];
            }
        }

        tmp
    }

    /// Total count across all bins.
    pub fn total(&self) -> f64 {
        self.counts.iter().sum()
    }
}
