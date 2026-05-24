//! Prefix FASTA dereplication.
//!
//! A shorter sequence that is an exact prefix of a longer one is collapsed
//! into the longer sequence (the representative), summing abundances. Output
//! is sorted by **descending abundance**; ties are broken by `strcmp` on the
//! raw input header of the representative's first occurrence, then by input
//! order.
//!
//! Behaviour matches `vsearch --derep_prefix` v2.31.0 (BSD-2).
//!
//! Algorithm (vsearch `src/derep_prefix.cc`):
//! 1. Read all sequences; normalise (uppercase + U→T) for matching only.
//! 2. Sort input sequences **shortest-first** (stable on input order).
//! 3. For each sequence in that order, compute FNV-1A prefix hashes for all
//!    prefix lengths.
//!    a. Exact match in table → accumulate abundance.
//!    b. No exact match: scan from `seqlen-1` down to `len_shortest`, looking
//!    for a matching prefix entry that is *not* deleted. Found → mark the
//!    shorter entry deleted, promote the current (longer) seq as the new
//!    representative.
//!    c. No match at all → new cluster.
//! 4. Sort clusters: descending abundance; ties → `strcmp` on the raw input
//!    header of `seqno_first`; ties → `seqno_first` ascending.
//!
//! Key difference from `--derep_fulllength`: the representative is always the
//! **longest** sequence. Both commands share the same minseqlength default of 32.

pub mod fasta;
pub mod header;

pub use fasta::{FastaWidth, write_fasta};
pub use header::{parse_size_annotation, strip_size};

use std::io::BufRead;

use crate::fasta::parse_fasta;

/// A prefix-dereplicated cluster.
#[derive(Debug)]
pub struct DereplicatedRecord {
    /// Label with `;size=N` stripped (used for output).
    pub label: String,
    /// Raw input label of the representative (the longest seq in the cluster),
    /// used for the `strcmp` tie-break in output sorting.
    pub sort_key: String,
    /// Abundance sum across all sequences in this cluster.
    pub abundance: u64,
    /// Representative sequence bytes, case-preserved.
    pub seq: Vec<u8>,
    /// 0-based input index of the representative's first occurrence.
    pub seqno_first: usize,
}

// FNV-1A constants matching vsearch's `compute_hashes_of_all_prefixes`.
const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

/// Compute FNV-1A prefix hashes: `prefix_hashes[k]` = hash of the first k bytes.
///
/// `prefix_hashes[0]` = `FNV_OFFSET` (empty-string hash, matching vsearch).
fn compute_prefix_hashes(seq: &[u8]) -> Vec<u64> {
    let mut hashes = Vec::with_capacity(seq.len() + 1);
    let mut h = FNV_OFFSET;
    hashes.push(h);
    for &b in seq {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
        hashes.push(h);
    }
    hashes
}

/// Entry in the open-addressing hash table.
#[derive(Clone, Default)]
struct Bucket {
    hash: u64,
    /// Representative's position in the sorted-by-length input slice.
    seqno_first: usize,
    /// Abundance sum.
    size: u64,
    /// True when this entry has been superseded by a longer representative.
    deleted: bool,
}

fn bucket_is_empty(b: &Bucket) -> bool {
    b.size == 0
}

/// Core prefix dereplication.
///
/// Reads FASTA from `reader`, deduplicates by the prefix rule, and returns
/// clusters sorted by descending abundance.
///
/// `sizein` enables parsing of `;size=N` from input headers. When absent,
/// each record contributes abundance 1.
/// `minseqlength` / `maxseqlength` filter sequences before deduplication
/// (vsearch defaults: 32 and 50000 for `--derep_prefix`).
#[allow(clippy::missing_errors_doc, clippy::too_many_lines)]
pub fn derep_prefix(
    reader: &mut dyn BufRead,
    sizein: bool,
    minseqlength: usize,
    maxseqlength: usize,
) -> anyhow::Result<(Vec<DereplicatedRecord>, usize)> {
    struct RawRecord {
        raw_label: String,
        label: String,
        abundance: u64,
        orig_seq: Vec<u8>,
        norm_seq: Vec<u8>,
        /// 0-based index in the original input order (before length sort).
        input_idx: usize,
    }

    let mut raw: Vec<RawRecord> = Vec::new();
    let mut discarded: usize = 0;

    for (input_idx, record) in parse_fasta(reader)?.enumerate() {
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

        raw.push(RawRecord {
            raw_label,
            label,
            abundance: input_abund,
            orig_seq: raw_seq,
            norm_seq,
            input_idx,
        });
    }

    if raw.is_empty() {
        return Ok((Vec::new(), discarded));
    }

    // Sort shortest-first; ties keep input order (stable sort).
    raw.sort_by(|a, b| {
        a.norm_seq
            .len()
            .cmp(&b.norm_seq.len())
            .then(a.input_idx.cmp(&b.input_idx))
    });

    let len_shortest = raw.first().map_or(0, |r| r.norm_seq.len());

    // Open-addressing hash table, 2/3 fill-rate.
    let n = raw.len();
    let mut hashtable_size: usize = 1;
    while 3 * n > 2 * hashtable_size {
        hashtable_size <<= 1;
    }
    let hash_mask = hashtable_size - 1;
    let mut table: Vec<Bucket> = vec![Bucket::default(); hashtable_size];

    for (sorted_idx, rec) in raw.iter().enumerate() {
        let seq = &rec.norm_seq;
        let seqlen = seq.len();
        let prefix_hashes = compute_prefix_hashes(seq);

        // Look for exact match.
        let exact_hash = prefix_hashes[seqlen];
        #[allow(clippy::cast_possible_truncation)]
        let mut j = exact_hash as usize & hash_mask;
        loop {
            let b = &table[j];
            if bucket_is_empty(b) {
                break;
            }
            if !b.deleted
                && b.hash == exact_hash
                && raw[b.seqno_first].norm_seq.len() == seqlen
                && raw[b.seqno_first].norm_seq == *seq
            {
                table[j].size += rec.abundance;
                break;
            }
            j = (j + 1) & hash_mask;
        }
        let orig_j = j;

        if !bucket_is_empty(&table[j]) {
            // Exact match found and accumulated; move on.
            continue;
        }

        // No exact match: look for a shorter prefix entry.
        let mut found_prefix = false;
        let mut prefix_len = seqlen;

        while !found_prefix && prefix_len > len_shortest {
            prefix_len -= 1;
            let ph = prefix_hashes[prefix_len];
            #[allow(clippy::cast_possible_truncation)]
            let mut k = ph as usize & hash_mask;
            loop {
                let b = &table[k];
                if bucket_is_empty(b) {
                    break;
                }
                if !b.deleted
                    && b.hash == ph
                    && raw[b.seqno_first].norm_seq.len() == prefix_len
                    && raw[b.seqno_first].norm_seq[..] == seq[..prefix_len]
                {
                    // Shorter seq is a prefix of current seq; absorb it.
                    let old_size = table[k].size;
                    table[k].deleted = true;
                    table[orig_j] = Bucket {
                        hash: exact_hash,
                        seqno_first: sorted_idx,
                        size: old_size + rec.abundance,
                        deleted: false,
                    };
                    found_prefix = true;
                    break;
                }
                k = (k + 1) & hash_mask;
            }
        }

        if !found_prefix {
            // New cluster.
            table[orig_j] = Bucket {
                hash: exact_hash,
                seqno_first: sorted_idx,
                size: rec.abundance,
                deleted: false,
            };
        }
    }

    // Collect non-deleted clusters and sort.
    let mut clusters: Vec<DereplicatedRecord> = table
        .into_iter()
        .filter(|b| !bucket_is_empty(b) && !b.deleted)
        .map(|b| {
            let rep = &raw[b.seqno_first];
            DereplicatedRecord {
                sort_key: rep.raw_label.clone(),
                label: rep.label.clone(),
                abundance: b.size,
                seq: rep.orig_seq.clone(),
                seqno_first: rep.input_idx,
            }
        })
        .collect();

    // Sort: descending abundance; ties: strcmp on raw header; ties: input order.
    clusters.sort_unstable_by(|a, b| {
        b.abundance
            .cmp(&a.abundance)
            .then_with(|| a.sort_key.as_bytes().cmp(b.sort_key.as_bytes()))
            .then_with(|| a.seqno_first.cmp(&b.seqno_first))
    });

    Ok((clusters, discarded))
}
