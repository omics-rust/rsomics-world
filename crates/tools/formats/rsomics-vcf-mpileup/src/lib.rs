//! bcftools mpileup SNP mode — per-position genotype-likelihood VCF from a
//! single coordinate-sorted BAM with a required reference.
//!
//! Implements the revised MAQ error model (`errmod_cal` / `htslib errmod.c`)
//! to compute per-genotype Phred-scaled likelihoods (PL), together with the
//! AD (allele depth) and DP (total depth) FORMAT fields that bcftools mpileup
//! emits by default for single-sample SNP positions.
//!
//! The PL vector ordering follows VCF 4.2 / bcftools convention: for a diploid
//! with `n` alleles the entries are lexicographic (i,j) pairs where i≤j —
//! for biallelic (REF,ALT): PL[RR], PL[RA], PL[AA].
//!
//! Source references (MIT-licensed):
//!   bcftools 1.21 `mpileup.c`, `bam2bcf.c`
//!   htslib 1.21 `errmod.c` (MAQ error model)

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::num::NonZeroUsize;
use std::path::Path;

use noodles::fasta;
use rsomics_bamio::raw::{RawRecord, read_record};
use rsomics_common::{Result, RsomicsError};
use rsomics_pileup::{Column, PileupEngine, PileupOpts};

// ── defaults matching bcftools mpileup ────────────────────────────────────────

/// Default minimum mapping quality (`-q`).
pub const DEFAULT_MIN_MAPQ: u8 = 0;
/// Default minimum base quality (`-Q`).
pub const DEFAULT_MIN_BASEQ: u8 = 1;
/// Default maximum base quality cap (bcftools `max_baseQ = 60`).
const MAX_BASEQ: u8 = 60;
/// Default maximum depth (bcftools `max_depth = 250`).
pub const DEFAULT_MAX_DEPTH: u32 = 250;
/// FLAG bits filtered by default: UNMAP|SECONDARY|QCFAIL|DUP.
const DEFAULT_RFLAG_FILTER: u16 = 0x004 | 0x100 | 0x200 | 0x400;
/// capQ: max quality we honour for the error model (bcftools `bca->capQ = 60`).
const CAP_Q: u8 = 60;
/// MAP quality for reads with mapQ == 255 (unmapped quality sentinel).
const DEF_MAPQ: u8 = 20;

// ── MAQ error model (htslib errmod.c) ────────────────────────────────────────
//
// Precomputed tables are allocated once at startup and reused per-position.
//
//   fk[n]       = pow(1-depcorr, n) * (1-eta) + eta
//   beta[q,n,k] = Phred P(k matches | n total, error-rate 10^(-q/10))
//   lhet[n,k]   = log P(k minor allele out of n in a het genotype)
//
// `errmod_cal` takes an array of packed observations `bases[i] = q<<5 | strand<<4 | base`
// and fills `q[j*m + k]` — the phred-scale likelihood of each diploid genotype (j, k).

struct ErrMod {
    fk: Vec<f64>,
    /// beta[q<<16 | n<<8 | k]: Phred-scale cost term for the error model.
    beta: Vec<f64>,
    /// lhet[n<<8 | k]: log P(k out of n being the minor allele at a het site).
    lhet: Vec<f64>,
}

impl ErrMod {
    /// Construct with the bcftools default depcorr = 0.17 (theta = 0.83), eta = 0.03.
    fn new() -> Self {
        const DEPCORR: f64 = 0.17;
        const ETA: f64 = 0.03;

        let mut fk = vec![0f64; 256];
        fk[0] = 1.0;
        for (n, slot) in fk.iter_mut().enumerate().skip(1) {
            *slot = (1.0 - DEPCORR).powi(n as i32) * (1.0 - ETA) + ETA;
        }

        // logbinomial table lC[n,k] = log C(n,k).
        let n_size: usize = 256;
        let mut lc = vec![0f64; n_size * n_size];
        for n in 1..n_size {
            let lf_n = lgamma(n as f64 + 1.0);
            for k in 1..=n {
                lc[n << 8 | k] = lf_n - lgamma(k as f64 + 1.0) - lgamma((n - k) as f64 + 1.0);
            }
        }

        let mut beta = vec![0f64; 64 * 256 * 256];
        for q in 1usize..64 {
            let e = 10f64.powf(-(q as f64) / 10.0);
            let le = e.ln();
            let le1 = (1.0 - e).ln();
            for n in 1usize..=255 {
                let base_idx = q << 16 | n << 8;
                let mut sum1 = lc[n << 8 | n] + n as f64 * le;
                beta[base_idx | n] = f64::INFINITY;
                let mut k = n as isize - 1;
                while k >= 0 {
                    let uk = k as usize;
                    let prev = sum1;
                    let delta = lc[n << 8 | uk] + uk as f64 * le + (n - uk) as f64 * le1 - prev;
                    sum1 = prev + delta.exp().ln_1p();
                    // -10/ln(10) factor for converting ln-scale to Phred.
                    beta[base_idx | uk] = -4.342_944_819_032_518 * (prev - sum1);
                    k -= 1;
                }
            }
        }

        let mut lhet = vec![0f64; 256 * 256];
        for n in 0..256usize {
            for k in 0..256usize {
                lhet[n << 8 | k] = lc[n << 8 | k] - std::f64::consts::LN_2 * n as f64;
            }
        }

        Self { fk, beta, lhet }
    }

    /// `errmod_cal`: compute phred-scaled genotype likelihoods.
    ///
    /// `bases`: packed `q<<5 | strand<<4 | base` (4-bit base index 0=A 1=C 2=G 3=T 4=N).
    /// `m` = number of allele types considered (5 for SNPs: A C G T N).
    /// Fills `q_out[j*m+k]` with phred-scale likelihood of genotype (j/k).
    fn cal(&self, bases_in: &mut [u16], m: usize, q_out: &mut [f32]) {
        let n_orig = bases_in.len();
        q_out.iter_mut().for_each(|v| *v = 0.0);
        if n_orig == 0 {
            return;
        }
        let n = n_orig.min(255);
        bases_in[..n].sort_unstable();

        let mut bsum = [0f64; 16];
        let mut c = [0usize; 16];
        let mut w = [0usize; 32]; // per (base|strand) run length counter

        for j in (0..n).rev() {
            let b = bases_in[j];
            let q = {
                let q_raw = (b >> 5) as usize;
                q_raw.clamp(4, 63)
            };
            let basestrand = (b & 0x1f) as usize;
            let base = (b & 0x0f) as usize;
            bsum[base] += self.fk[w[basestrand]] * self.beta[q << 16 | n << 8 | c[base]];
            c[base] += 1;
            w[basestrand] += 1;
        }

        for j in 0..m {
            // Homozygous j/j: cost = sum of non-j bsum contributions.
            let (mut tmp1, mut tmp2) = (0f64, 0usize);
            for k in 0..m {
                if k == j {
                    continue;
                }
                tmp1 += bsum[k];
                tmp2 += c[k];
            }
            if tmp2 > 0 {
                q_out[j * m + j] = tmp1 as f32;
            }
            // Heterozygous j/k for k > j.
            for k in (j + 1)..m {
                let cjk = c[j] + c[k];
                let (mut t1, mut t2) = (0f64, 0usize);
                for i in 0..m {
                    if i == j || i == k {
                        continue;
                    }
                    t1 += bsum[i];
                    t2 += c[i];
                }
                let lh_idx = (cjk << 8 | c[k]).min(256 * 256 - 1);
                let v =
                    (-4.342_944_819_032_518 * self.lhet[lh_idx]) + if t2 > 0 { t1 } else { 0.0 };
                q_out[j * m + k] = v as f32;
                q_out[k * m + j] = v as f32;
            }
            // Clamp to ≥ 0.
            for k in 0..m {
                if q_out[j * m + k] < 0.0 {
                    q_out[j * m + k] = 0.0;
                }
            }
        }
    }
}

/// Lanczos lgamma, accurate ~15 digits for x > 0 (Numerical Recipes §6.1).
fn lgamma(x: f64) -> f64 {
    const C: [f64; 6] = [
        76.180_091_729_471_46,
        -86.505_320_329_416_77,
        24.014_098_240_830_91,
        -1.231_739_572_450_155,
        1.208_650_973_866_179e-3,
        -5.395_239_384_953_e-6,
    ];
    let x = x - 1.0;
    let y = x + 5.5;
    let tmp = (x + 0.5) * y.ln() - y;
    let mut ser = 1.000_000_000_190_015;
    let mut xx = x + 1.0;
    for ci in &C {
        ser += ci / xx;
        xx += 1.0;
    }
    tmp + (2.506_628_274_631_001 * ser).ln()
}

// ── 4-bit BAM base encoding helpers ──────────────────────────────────────────

/// BAM `seq_nt16_int`: 4-bit seq_nt16 code → ACGT index (N=4).
const NT16_TO_INT: [u8; 16] = [4, 0, 1, 4, 2, 4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4];
/// ACGTN index → uppercase ASCII.
const INT_TO_BASE: [u8; 5] = *b"ACGTN";

// ── ASCII FASTA base → 4-bit NT16 (htslib seq_nt16_table) ─────────────────

fn nt16_of_ascii(b: u8) -> u8 {
    match b.to_ascii_uppercase() {
        b'A' => 1,
        b'C' => 2,
        b'G' => 4,
        b'T' => 8,
        _ => 15, // N and ambiguous codes
    }
}

// ── VCF header ────────────────────────────────────────────────────────────────

fn write_header(w: &mut impl Write, sq: &[(String, u64)], sample: &str) -> std::io::Result<()> {
    writeln!(w, "##fileformat=VCFv4.2")?;
    writeln!(w, "##FILTER=<ID=PASS,Description=\"All filters passed\">")?;
    for (name, len) in sq {
        writeln!(w, "##contig=<ID={name},length={len}>")?;
    }
    writeln!(
        w,
        "##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Raw read depth\">"
    )?;
    writeln!(
        w,
        "##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled genotype likelihoods\">"
    )?;
    writeln!(
        w,
        "##FORMAT=<ID=DP,Number=1,Type=Integer,Description=\"Number of high-quality bases\">"
    )?;
    writeln!(
        w,
        "##FORMAT=<ID=AD,Number=R,Type=Integer,Description=\"Allelic depths for ref and alt alleles\">"
    )?;
    writeln!(
        w,
        "##source=rsomics-vcf-mpileup {}",
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(
        w,
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{sample}"
    )?;
    Ok(())
}

// ── per-position GL/PL computation ────────────────────────────────────────────

struct GlResult {
    ref_base: u8, // ACGTN index (0-4)
    alt_base: u8, // ACGTN index; only meaningful when has_alt
    has_alt: bool,
    ad: Vec<u32>,
    dp: u32,
    pl: Vec<u32>, // PL[RR], PL[RA], PL[AA] or PL[RR] only
}

fn process_column(
    col: &Column,
    ref_nt16: u8, // 4-bit NT16 code of the reference base
    min_baseq: u8,
    max_depth: u32,
    errmod: &ErrMod,
    bases_buf: &mut Vec<u16>,
    gl_buf: &mut Vec<f32>,
) -> GlResult {
    let ref_int = NT16_TO_INT[(ref_nt16 & 0xf) as usize]; // 0=A 1=C 2=G 3=T 4=N

    let mut counts = [0u32; 5]; // A C G T N
    bases_buf.clear();

    let mut seen = 0u32;
    for (read, rec) in col.reads.iter().zip(col.records.iter()) {
        if read.is_del || read.is_refskip {
            continue;
        }
        if seen >= max_depth {
            break;
        }
        let qual_scores = rec.quality_scores();
        let qpos = read.qpos;

        let bq_raw = if qpos < qual_scores.len() {
            qual_scores[qpos]
        } else {
            0
        };
        if bq_raw < min_baseq {
            continue;
        }

        let nib = rec.seq_nibble(qpos);
        let base_int = NT16_TO_INT[(nib & 0xf) as usize];

        let mapq: u8 = {
            let mq = rec.mapping_quality();
            if mq == 255 { DEF_MAPQ } else { mq }
        };

        // Effective quality: min(bq, max_baseQ, mapQ, capQ, 63), floor 4.
        let q = bq_raw.min(MAX_BASEQ).min(mapq).min(CAP_Q).clamp(4, 63);

        let strand: u8 = u8::from(rec.flags() & 0x10 != 0);
        bases_buf.push((q as u16) << 5 | (strand as u16) << 4 | (base_int as u16 & 0xf));
        counts[base_int as usize] += 1;
        seen += 1;
    }

    let dp = bases_buf.len() as u32;
    let m = 5; // A C G T N

    gl_buf.resize(m * m, 0.0);
    errmod.cal(bases_buf, m, gl_buf);

    // Pick best ALT allele by count (excluding REF and N).
    let mut best_alt = 5usize;
    let mut best_alt_count = 0u32;
    for (i, &cnt) in counts[..4].iter().enumerate() {
        if ref_int < 4 && i == ref_int as usize {
            continue;
        }
        if cnt > best_alt_count {
            best_alt_count = cnt;
            best_alt = i;
        }
    }
    // Require at least one ALT observation.
    let has_alt = best_alt < 5 && best_alt_count > 0;

    let ref_ad = if ref_int < 4 {
        counts[ref_int as usize]
    } else {
        0
    };
    let alt_ad = if has_alt { counts[best_alt] } else { 0 };
    let ad = if has_alt {
        vec![ref_ad, alt_ad]
    } else {
        vec![ref_ad]
    };

    let pl = if dp == 0 || !has_alt {
        vec![0u32; if has_alt { 3 } else { 1 }]
    } else {
        let r = if ref_int < 4 { ref_int as usize } else { 4 };
        let a = best_alt;
        let pl_rr = gl_buf[r * m + r];
        let pl_ra = gl_buf[r * m + a];
        let pl_aa = gl_buf[a * m + a];
        let min_pl = pl_rr.min(pl_ra).min(pl_aa);
        vec![
            (pl_rr - min_pl).round() as u32,
            (pl_ra - min_pl).round() as u32,
            (pl_aa - min_pl).round() as u32,
        ]
    };

    GlResult {
        ref_base: ref_int,
        alt_base: if has_alt { best_alt as u8 } else { 4 },
        has_alt,
        ad,
        dp,
        pl,
    }
}

// ── public options ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VcfMpileupOpts {
    pub min_mapq: u8,
    pub min_baseq: u8,
    pub max_depth: u32,
    pub region: Option<String>,
    pub threads: NonZeroUsize,
}

impl Default for VcfMpileupOpts {
    fn default() -> Self {
        Self {
            min_mapq: DEFAULT_MIN_MAPQ,
            min_baseq: DEFAULT_MIN_BASEQ,
            max_depth: DEFAULT_MAX_DEPTH,
            region: None,
            threads: NonZeroUsize::new(1).unwrap(),
        }
    }
}

// ── main entry point ──────────────────────────────────────────────────────────

/// Run vcf-mpileup, writing VCF (uncompressed) to `out`.
///
/// `bam`: coordinate-sorted BAM (indexed only when `opts.region` is set).
/// `fasta_ref`: faidx-indexed FASTA. Required.
pub fn run(bam: &Path, fasta_ref: &Path, out: impl Write, opts: &VcfMpileupOpts) -> Result<()> {
    let ref_seqs = load_reference(fasta_ref)?;

    let mut reader = rsomics_bamio::open_with_workers(bam, opts.threads)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let sq: Vec<(String, u64)> = header
        .reference_sequences()
        .iter()
        .map(|(name, map)| {
            let len: u64 = usize::from(map.length()) as u64;
            (name.to_string(), len)
        })
        .collect();

    let mut out = BufWriter::with_capacity(256 * 1024, out);
    let sample_name = bam.file_stem().and_then(|s| s.to_str()).unwrap_or("SAMPLE");
    write_header(&mut out, &sq, sample_name).map_err(RsomicsError::Io)?;

    let tid_to_name: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|n| n.to_string())
        .collect();

    // Region filter: pre-parse so we can skip columns outside the requested window.
    let region_filter: Option<(i32, i64, i64)> = opts.region.as_deref().map(|reg| {
        let (chrom, beg, end) = parse_region(reg);
        let tid = tid_to_name
            .iter()
            .position(|n| n == &chrom)
            .map(|i| i as i32)
            .unwrap_or(-1);
        (tid, beg as i64, end as i64)
    });

    let pileup_opts = PileupOpts {
        min_mapq: opts.min_mapq,
        no_orphan: true,
        rflag_filter: DEFAULT_RFLAG_FILTER,
        ..PileupOpts::default()
    };

    let errmod = ErrMod::new();
    let mut bases_buf: Vec<u16> = Vec::with_capacity(512);
    let mut gl_buf: Vec<f32> = vec![0.0; 25];
    let mut engine = PileupEngine::new(pileup_opts);

    // Emit closure — called once per pileup column in coordinate order.
    let emit = |col: &Column| -> Result<()> {
        // Region filter: skip if outside requested window.
        if let Some((req_tid, beg, end)) = region_filter
            && (col.tid != req_tid || col.pos < beg || col.pos >= end)
        {
            return Ok(());
        }

        let tid = col.tid as usize;
        let pos0 = col.pos as usize;

        let ref_byte = tid_to_name
            .get(tid)
            .and_then(|name| ref_seqs.get(name))
            .and_then(|seq| seq.get(pos0).copied())
            .unwrap_or(b'N');

        let ref_nt16 = nt16_of_ascii(ref_byte);

        let gl = process_column(
            col,
            ref_nt16,
            opts.min_baseq,
            opts.max_depth,
            &errmod,
            &mut bases_buf,
            &mut gl_buf,
        );

        if gl.dp == 0 || !gl.has_alt {
            return Ok(());
        }

        // gl.has_alt is true here (checked above).
        let contig_name = tid_to_name.get(tid).map_or(".", String::as_str);
        let ref_char = INT_TO_BASE[gl.ref_base.min(4) as usize];
        let alt_char = INT_TO_BASE[gl.alt_base as usize];
        let pos1 = pos0 + 1;

        // QUAL: PL[AA] for the best alt (proxy for "variant quality").
        let qual_str = gl
            .pl
            .get(2)
            .map_or_else(|| ".".to_owned(), |v| v.to_string());

        write!(out, "{contig_name}\t{pos1}\t.\t")?;
        out.write_all(&[ref_char]).map_err(RsomicsError::Io)?;
        write!(out, "\t")?;
        out.write_all(&[alt_char]).map_err(RsomicsError::Io)?;
        write!(out, "\t{qual_str}\t.\tDP={dp}\tPL:DP:AD\t", dp = gl.dp)?;

        let pl_str = gl
            .pl
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let ad_str = gl
            .ad
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        writeln!(out, "{pl_str}:{dp}:{ad_str}", dp = gl.dp).map_err(RsomicsError::Io)
    };

    // Drive the pileup engine: forward-streaming (no index jump).
    let inner = reader.get_mut();
    let mut rec = RawRecord::default();

    rsomics_pileup::run(
        &mut engine,
        || -> Result<Option<RawRecord>> {
            let n = read_record(inner, &mut rec)?;
            if n == 0 {
                Ok(None)
            } else {
                Ok(Some(rec.clone()))
            }
        },
        emit,
    )?;

    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

/// Load all FASTA sequences into memory.
fn load_reference(path: &Path) -> Result<HashMap<String, Vec<u8>>> {
    let file = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut reader = fasta::io::Reader::new(std::io::BufReader::new(file));
    let mut map = HashMap::new();
    for result in reader.records() {
        let rec =
            result.map_err(|e| RsomicsError::InvalidInput(format!("FASTA read error: {e}")))?;
        let name = String::from_utf8_lossy(rec.name()).into_owned();
        let seq: Vec<u8> = rec.sequence().as_ref().to_vec();
        map.insert(name, seq);
    }
    Ok(map)
}

/// Parse "chrom", "chrom:start-end" (1-based inclusive) → (chrom, 0-based beg, 0-based end excl).
fn parse_region(region: &str) -> (String, u64, u64) {
    if let Some((chrom, range)) = region.split_once(':') {
        if let Some((s, e)) = range.split_once('-') {
            let beg = s
                .replace(',', "")
                .parse::<u64>()
                .unwrap_or(1)
                .saturating_sub(1);
            let end = e.replace(',', "").parse::<u64>().unwrap_or(u64::MAX);
            return (chrom.to_owned(), beg, end);
        }
        let beg = range
            .replace(',', "")
            .parse::<u64>()
            .unwrap_or(1)
            .saturating_sub(1);
        (chrom.to_owned(), beg, u64::MAX)
    } else {
        (region.to_owned(), 0, u64::MAX)
    }
}
