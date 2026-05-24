//! Full-length FASTA dereplication.
//!
//! Collapses identical sequences (after uppercasing and U→T normalisation)
//! into a single representative record, summing abundances.  Output is sorted
//! by **descending abundance**; ties are broken lexicographically by header
//! label, then by input order (first-occurrence seqno).
//!
//! Behaviour matches `vsearch --derep_fulllength` v2.31.0 (BSD-2):
//!
//! - Default `fasta_width` = 80 (wrap sequence lines at 80 characters).
//! - Default `minseqlength` = 32 (shorter sequences are discarded).
//! - Default `maxseqlength` = 50000 (longer sequences are discarded).
//! - `--sizeout`: appended `;size=N` on every output record.
//! - `--sizein`: parse existing `;size=N` in input headers; absent ⇒ 1.
//! - Header handling: the `;size=N` field is stripped from the input label
//!   and the newly computed abundance is appended at the end.
//! - Case preserved: the representative's original byte case is written to
//!   output; normalisation (uppercase + U→T) is used only for matching.
//! - Comparison: sequence bytes compared after uppercasing + U→T; plus
//!   strand only (no reverse-complement by default).

pub mod fasta;
pub mod header;

pub use fasta::{FastaWidth, write_fasta};
pub use header::{strip_size, write_header_with_size};

use std::collections::HashMap;
use std::io::BufRead;

use ahash::AHashMap;

use crate::fasta::parse_fasta;
use crate::header::parse_size_annotation;

/// A dereplicated sequence cluster.
#[derive(Debug)]
pub struct DereplicatedRecord {
    /// Label with the `;size=N` field stripped (used for output).
    pub label: String,
    /// Original input label, before any `;size=N` stripping.
    ///
    /// vsearch's tie-break `strcmp` runs on the raw input header (truncated at
    /// first whitespace but otherwise unmodified — `;size=N` annotation left
    /// in place).  We store this separately so the sort comparator can match
    /// that byte-for-byte ordering.
    pub sort_key: String,
    /// Abundance sum across all identical input sequences.
    pub abundance: u64,
    /// Representative sequence in its original bytes (case + U preserved),
    /// exactly as it appeared in the input for the first-occurrence record.
    pub seq: Vec<u8>,
    /// 0-based index of the first occurrence in the input file (for
    /// stable tie-breaking).
    pub seqno_first: usize,
}

/// Core dereplication logic.
///
/// Reads FASTA from `reader`, deduplicates identical sequences, and returns
/// clusters sorted by descending abundance (ties: lexicographic label, then
/// input order).
///
/// `sizein` enables parsing of `;size=N` from input headers.  When absent or
/// `false`, each record contributes abundance 1 regardless of any annotation.
/// `minseqlength` and `maxseqlength` discard sequences outside the length
/// range before deduplication; these match vsearch defaults (32 and 50000).
#[allow(clippy::missing_errors_doc)]
pub fn derep_fulllength(
    reader: &mut dyn BufRead,
    sizein: bool,
    minseqlength: usize,
    maxseqlength: usize,
) -> anyhow::Result<(Vec<DereplicatedRecord>, usize)> {
    // hash-map key: normalised sequence bytes → index into `clusters`
    let mut seq_index: AHashMap<Vec<u8>, usize> = AHashMap::new();
    let mut clusters: Vec<DereplicatedRecord> = Vec::new();
    let mut discarded: usize = 0;
    let mut seqno: usize = 0;

    for record in parse_fasta(reader)? {
        let (raw_label, raw_seq) = record?;

        if raw_seq.len() < minseqlength || raw_seq.len() > maxseqlength {
            discarded += 1;
            continue;
        }

        let norm_seq: Vec<u8> = raw_seq
            .iter()
            .map(|&b| {
                if b == b'u' || b == b'U' {
                    b'T'
                } else {
                    b.to_ascii_uppercase()
                }
            })
            .collect();

        let (label, input_abund) = if sizein {
            let (stripped, abund) = parse_size_annotation(&raw_label);
            (stripped, abund.unwrap_or(1))
        } else {
            let (stripped, _) = parse_size_annotation(&raw_label);
            (stripped, 1u64)
        };

        if let Some(idx) = seq_index.get(&norm_seq).copied() {
            clusters[idx].abundance += input_abund;
        } else {
            let idx = clusters.len();
            seq_index.insert(norm_seq, idx);
            clusters.push(DereplicatedRecord {
                sort_key: raw_label,
                label,
                abundance: input_abund,
                seq: raw_seq,
                seqno_first: seqno,
            });
        }
        seqno += 1;
    }

    // Sort: descending abundance; ties: byte-wise strcmp on the original input
    // header (matching vsearch's derep_compare_full which compares bp->header,
    // the raw label truncated at first whitespace but with ;size=N left in);
    // ties: input order (seqno_first ascending).
    clusters.sort_unstable_by(|a, b| {
        b.abundance
            .cmp(&a.abundance)
            .then_with(|| a.sort_key.as_bytes().cmp(b.sort_key.as_bytes()))
            .then_with(|| a.seqno_first.cmp(&b.seqno_first))
    });

    Ok((clusters, discarded))
}

/// Parallel dereplication using rayon for the normalisation pass.
///
/// Normalisation is parallelised; the hash-map aggregation phase is
/// sequential (inherently ordered by first-occurrence).
#[allow(clippy::missing_errors_doc)]
pub fn derep_fulllength_parallel(
    reader: &mut dyn BufRead,
    sizein: bool,
    minseqlength: usize,
    maxseqlength: usize,
    _threads: usize,
) -> anyhow::Result<(Vec<DereplicatedRecord>, usize)> {
    use rayon::prelude::*;

    // Collect all records first, then normalise in parallel.
    let raw_records: anyhow::Result<Vec<_>> = parse_fasta(reader)?.collect();
    let raw_records = raw_records?;

    // Parallel normalisation pass: filter by length and normalise for key.
    // Returns (sort_key, label, orig_seq, norm_seq, abundance) or None if filtered.
    let normalised: Vec<_> = raw_records
        .into_par_iter()
        .map(|(raw_label, raw_seq)| {
            if raw_seq.len() < minseqlength || raw_seq.len() > maxseqlength {
                return None;
            }
            let norm_seq: Vec<u8> = raw_seq
                .iter()
                .map(|&b| {
                    if b == b'u' || b == b'U' {
                        b'T'
                    } else {
                        b.to_ascii_uppercase()
                    }
                })
                .collect();
            let (label, input_abund) = if sizein {
                let (stripped, abund) = parse_size_annotation(&raw_label);
                (stripped, abund.unwrap_or(1))
            } else {
                let (stripped, _) = parse_size_annotation(&raw_label);
                (stripped, 1u64)
            };
            Some((raw_label, label, raw_seq, norm_seq, input_abund))
        })
        .collect();

    // Sequential aggregation.
    let mut seq_index: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut clusters: Vec<DereplicatedRecord> = Vec::new();
    let mut discarded: usize = 0;
    let mut seqno: usize = 0;

    for item in normalised {
        match item {
            None => discarded += 1,
            Some((sort_key, label, orig_seq, norm_seq, input_abund)) => {
                if let Some(idx) = seq_index.get(&norm_seq).copied() {
                    clusters[idx].abundance += input_abund;
                } else {
                    let idx = clusters.len();
                    seq_index.insert(norm_seq, idx);
                    clusters.push(DereplicatedRecord {
                        sort_key,
                        label,
                        abundance: input_abund,
                        seq: orig_seq,
                        seqno_first: seqno,
                    });
                }
                seqno += 1;
            }
        }
    }

    clusters.sort_unstable_by(|a, b| {
        b.abundance
            .cmp(&a.abundance)
            .then_with(|| a.sort_key.as_bytes().cmp(b.sort_key.as_bytes()))
            .then_with(|| a.seqno_first.cmp(&b.seqno_first))
    });

    Ok((clusters, discarded))
}
