//! Deterministic FASTA sorting by abundance or by length.
//!
//! Ports `vsearch --sortbysize` and `vsearch --sortbylength` (v2.31.0, BSD-2).
//!
//! ## Sorting rules
//!
//! **`--sortbysize`** (abundance sort):
//! - Primary: abundance descending.
//! - Tie-break: byte-wise `strcmp` on the **raw input header** (the original
//!   header string as it appears in the file, including any `;size=N`
//!   annotation) ascending.
//!
//! **`--sortbylength`** (length sort):
//! - Primary: sequence length descending.
//! - Secondary: abundance descending (parsed or default 1).
//! - Tie-break: byte-wise `strcmp` on the raw input header ascending.
//!
//! ## Length filtering
//!
//! Default `--minseqlength 1` and `--maxseqlength 50000` for these operations.
//! (vsearch uses 1, not 32, for sort commands — the 32 default applies only
//! to clust/derep/search.)
//!
//! ## Abundance annotation
//!
//! vsearch always reads `;size=N` from input headers regardless of `--sizein`.
//! Sequences without `;size=N` get a default abundance of 1.
//! `--sizein` is accepted for compatibility but has no additional effect for
//! these operations.
//!
//! When `--sizeout` is given the `;size=N` token is stripped from the stored
//! header and reappended at the end of the output label.  Without `--sizeout`
//! the original header bytes are written unchanged.
//!
//! ## Case and U preservation
//!
//! Sequence bytes are written exactly as they appeared in the input (case and U
//! preserved); no normalisation is applied to the output.

pub mod fasta;
pub mod header;

pub use fasta::{FastaWidth, write_fasta_raw, write_fasta_with_size};
pub use header::parse_size_annotation;

use std::io::BufRead;

use crate::fasta::parse_fasta;
use crate::header::parse_size_annotation as parse_size;

/// A single FASTA record ready for sorting.
#[derive(Debug)]
pub struct SortRecord {
    /// Raw input header (label after `>`), used for tie-breaking.
    ///
    /// Stored verbatim from the input file, including any `;size=N` annotation.
    /// Matches what vsearch's `db_getheader` returns for sort comparisons.
    pub raw_header: String,
    /// Header with any `;size=N` token stripped, used for `--sizeout` output.
    pub stripped_header: String,
    /// Parsed abundance (from `;size=N` if present, otherwise 1).
    pub abundance: u64,
    /// Sequence length in bases.
    pub seq_len: usize,
    /// Original sequence bytes (case and U preserved).
    pub seq: Vec<u8>,
}

/// Read all FASTA records, applying length filters, and return them.
///
/// `minseqlength` and `maxseqlength` match the vsearch defaults for sort
/// operations (1 and 50000 respectively; the 32 default applies only to
/// clust/derep/search).
#[allow(clippy::missing_errors_doc)]
pub fn read_records(
    reader: &mut dyn BufRead,
    minseqlength: usize,
    maxseqlength: usize,
) -> anyhow::Result<(Vec<SortRecord>, usize)> {
    let mut records: Vec<SortRecord> = Vec::new();
    let mut discarded: usize = 0;

    for item in parse_fasta(reader)? {
        let (raw_header, seq) = item?;

        if seq.len() < minseqlength || seq.len() > maxseqlength {
            discarded += 1;
            continue;
        }

        // vsearch always parses ;size= regardless of --sizein flag.
        let (stripped_header, parsed_size) = parse_size(&raw_header);
        let abundance = parsed_size.unwrap_or(1);
        let seq_len = seq.len();

        records.push(SortRecord {
            raw_header,
            stripped_header,
            abundance,
            seq_len,
            seq,
        });
    }

    Ok((records, discarded))
}

/// Sort records by abundance descending (vsearch `--sortbysize`).
///
/// Tie-break: byte-wise `strcmp` on the raw input header ascending, matching
/// vsearch's `compare_sequences` lambda in `sortbysize.cc`.
pub fn sort_by_size(records: &mut [SortRecord]) {
    records.sort_unstable_by(|a, b| {
        b.abundance
            .cmp(&a.abundance)
            .then_with(|| a.raw_header.as_bytes().cmp(b.raw_header.as_bytes()))
    });
}

/// Sort records by length descending (vsearch `--sortbylength`).
///
/// Tie-break order (matching `sortbylength.cc`):
/// 1. Length descending.
/// 2. Abundance descending.
/// 3. Raw header ascending.
pub fn sort_by_length(records: &mut [SortRecord]) {
    records.sort_unstable_by(|a, b| {
        b.seq_len
            .cmp(&a.seq_len)
            .then_with(|| b.abundance.cmp(&a.abundance))
            .then_with(|| a.raw_header.as_bytes().cmp(b.raw_header.as_bytes()))
    });
}
