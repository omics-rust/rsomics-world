//! Shared VCF record parser used by every subcommand.
//!
//! Parses GT fields only (genotype-level stats). FORMAT-order-aware so GT need
//! not be the first FORMAT field, matching vcftools behaviour.

use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

use anyhow::{Context, Result};

/// Parsed genotype: 0=ref, 1..=alt index. `None` = missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Allele {
    Ref,
    Alt(u8), // alt index (1-based, matching VCF ALT field)
    Missing,
}

impl Allele {
    pub fn is_missing(self) -> bool {
        self == Self::Missing
    }
    pub fn is_ref(self) -> bool {
        self == Self::Ref
    }
}

/// A single sample's diploid genotype.
#[derive(Debug, Clone, Copy)]
pub struct Gt {
    pub a1: Allele,
    pub a2: Allele,
}

impl Gt {
    pub fn is_missing(self) -> bool {
        self.a1.is_missing() || self.a2.is_missing()
    }

    pub fn is_het(self) -> bool {
        !self.is_missing() && self.a1 != self.a2
    }

    pub fn is_hom_ref(self) -> bool {
        !self.is_missing() && self.a1.is_ref() && self.a2.is_ref()
    }

    pub fn is_hom_alt(self) -> bool {
        !self.is_missing() && !self.a1.is_ref() && !self.a2.is_ref() && self.a1 == self.a2
    }
}

/// A VCF data record with parsed GT fields.
pub struct VcfRecord {
    pub chrom: String,
    pub pos: u64, // 1-based
    pub id: String,
    pub ref_allele: String,
    pub alt_alleles: Vec<String>, // split by comma, no "."
    pub gts: Vec<Gt>,             // one per sample, in sample order
}

impl VcfRecord {
    /// Number of allele copies observed (non-missing).
    pub fn allele_count(&self) -> (u64, u64) {
        // returns (n_obs_alleles, n_alt_alleles)
        let mut n_obs = 0u64;
        let mut n_alt = 0u64;
        for gt in &self.gts {
            if !gt.a1.is_missing() {
                n_obs += 1;
                if !gt.a1.is_ref() {
                    n_alt += 1;
                }
            }
            if !gt.a2.is_missing() {
                n_obs += 1;
                if !gt.a2.is_ref() {
                    n_alt += 1;
                }
            }
        }
        (n_obs, n_alt)
    }
}

pub struct VcfReader {
    lines: Lines<BufReader<File>>,
    pub samples: Vec<String>,
}

impl VcfReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
        let mut reader = BufReader::new(file);
        let mut samples = Vec::new();
        // Find #CHROM header line to extract sample names.
        let mut buf = String::new();
        loop {
            buf.clear();
            let n = reader
                .read_line(&mut buf)
                .with_context(|| "reading VCF header")?;
            if n == 0 {
                break; // header-only VCF
            }
            if buf.starts_with("#CHROM") {
                let fields: Vec<&str> = buf.trim_end().split('\t').collect();
                if fields.len() > 9 {
                    samples = fields[9..].iter().map(|s| s.to_string()).collect();
                }
                break;
            }
        }
        Ok(Self {
            lines: reader.lines(),
            samples,
        })
    }

    pub fn n_samples(&self) -> usize {
        self.samples.len()
    }
}

impl Iterator for VcfReader {
    type Item = Result<VcfRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = match self.lines.next()? {
                Ok(l) => l,
                Err(e) => return Some(Err(e.into())),
            };
            if line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.splitn(10, '\t').collect();
            if fields.len() < 9 {
                continue; // malformed
            }
            let chrom = fields[0].to_string();
            let pos: u64 = match fields[1].parse() {
                Ok(p) => p,
                Err(e) => return Some(Err(anyhow::anyhow!("bad POS '{}': {e}", fields[1]))),
            };
            let id = fields[2].to_string();
            let ref_allele = fields[3].to_string();
            let alt_field = fields[4];
            let alt_alleles: Vec<String> = alt_field
                .split(',')
                .filter(|a| *a != "." && *a != "*")
                .map(|a| a.to_string())
                .collect();

            let format_field = fields[8];
            // Find GT index in FORMAT.
            let gt_idx = format_field.split(':').position(|f| f == "GT");

            let sample_fields = if fields.len() > 9 { fields[9] } else { "" };
            let gts = if let Some(gt_i) = gt_idx {
                // Split at first tab; fields[9] has the rest of the samples.
                // Re-split the full line for samples.
                let all_fields: Vec<&str> = line.splitn(10 + self.samples.len(), '\t').collect();
                self.samples
                    .iter()
                    .enumerate()
                    .map(|(si, _)| {
                        let sf = all_fields.get(9 + si).copied().unwrap_or(".");
                        let gt_str = sf.split(':').nth(gt_i).unwrap_or(".");
                        parse_gt(gt_str)
                    })
                    .collect()
            } else {
                // No GT in FORMAT — treat all as missing.
                let _ = sample_fields;
                vec![
                    Gt {
                        a1: Allele::Missing,
                        a2: Allele::Missing
                    };
                    self.n_samples()
                ]
            };

            return Some(Ok(VcfRecord {
                chrom,
                pos,
                id,
                ref_allele,
                alt_alleles,
                gts,
            }));
        }
    }
}

fn parse_allele(s: &str) -> Allele {
    if s == "." {
        return Allele::Missing;
    }
    match s.parse::<u8>() {
        Ok(0) => Allele::Ref,
        Ok(n) => Allele::Alt(n),
        Err(_) => Allele::Missing,
    }
}

fn parse_gt(s: &str) -> Gt {
    if s == "." || s == "./." || s == ".|." {
        return Gt {
            a1: Allele::Missing,
            a2: Allele::Missing,
        };
    }
    // Phased (|) or unphased (/).
    let sep = if s.contains('|') { '|' } else { '/' };
    let mut parts = s.splitn(2, sep);
    let a1 = parse_allele(parts.next().unwrap_or("."));
    let a2 = parts.next().map_or(Allele::Missing, parse_allele);
    Gt { a1, a2 }
}
