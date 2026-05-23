//! Primer BED loading for ampliconstats.
//!
//! Mirrors samtools `load_bed_file_multi_ref` with `sort_by_pos = 0` (the
//! ampliconstats variant: sorted by left coordinate, not right). Each primer
//! row alternates + then - strand; the transition from - back to + marks the
//! start of the next amplicon.

use std::collections::HashMap;
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

/// One primer row from the BED file.
#[derive(Debug, Clone)]
pub struct PrimerEntry {
    /// BED start (0-based, inclusive).
    pub left: i64,
    /// BED end (0-based, exclusive).
    pub right: i64,
    /// `true` = reverse strand (`-`), `false` = forward (`+`).
    pub rev: bool,
}

/// Primers for one reference, in BED file order (alternating +/-).
#[derive(Debug, Default)]
pub struct PrimerList {
    pub entries: Vec<PrimerEntry>,
}

/// All primers keyed by reference name, with reference order preserved.
#[derive(Debug, Default)]
pub struct PrimerBed {
    pub by_ref: HashMap<String, PrimerList>,
    /// Reference names in BED file first-seen order.
    pub ref_order: Vec<String>,
}

/// Load primer BED. Column 6 (strand: `+`/`-`) is required.
///
/// Entries are left in BED file order (samtools `sort_by_pos = 0`).
pub fn load(path: &Path) -> Result<PrimerBed> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;

    let mut bed = PrimerBed::default();

    for (lineno, line) in text.lines().enumerate() {
        let line_num = lineno + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("track ") || line.starts_with("browser ") {
            continue;
        }

        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 6 {
            // Try whitespace split as fallback.
            let wcols: Vec<&str> = line.split_whitespace().collect();
            if wcols.len() < 6 {
                return Err(RsomicsError::InvalidInput(format!(
                    "{}:{line_num}: need at least 6 columns (chrom start end name score strand)",
                    path.display()
                )));
            }
        }
        let cols: Vec<&str> = line.split_whitespace().collect();

        let ref_name = cols[0];
        let left: i64 = cols[1].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!(
                "{}:{line_num}: bad start coordinate",
                path.display()
            ))
        })?;
        let right: i64 = cols[2].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("{}:{line_num}: bad end coordinate", path.display()))
        })?;
        let rev = match cols[5] {
            "+" => false,
            "-" => true,
            other => {
                return Err(RsomicsError::InvalidInput(format!(
                    "{}:{line_num}: bad strand '{other}', expected '+' or '-'",
                    path.display()
                )));
            }
        };

        let list = bed.by_ref.entry(ref_name.to_string()).or_insert_with(|| {
            bed.ref_order.push(ref_name.to_string());
            PrimerList::default()
        });
        list.entries.push(PrimerEntry { left, right, rev });
    }

    if bed.by_ref.is_empty() {
        return Err(RsomicsError::InvalidInput(format!(
            "no BED entries in {}",
            path.display()
        )));
    }

    Ok(bed)
}

/// Count amplicons in a primer list by counting + → - → + transitions.
///
/// Mirrors samtools `count_amplicon`: each time we see a `+` row after at
/// least one `-` row, a new amplicon starts. Returns the count.
pub fn count_amplicons(entries: &[PrimerEntry]) -> usize {
    let mut namp = 1usize;
    let mut last_rev = false;
    for e in entries {
        if !e.rev && last_rev {
            namp += 1;
        }
        last_rev = e.rev;
    }
    namp
}

/// Per-amplicon coordinate summary derived from the primer BED.
///
/// `left[i]` holds the `right` coordinate of the i-th `+` primer (BED end).
/// `right[j]` holds the `left` coordinate of the j-th `-` primer (BED start).
/// Mirrors samtools' `amplicon_t`.
#[derive(Debug, Clone)]
pub struct Amplicon {
    /// Inner left bound: `max(all left-primer right coords) + 1`, 1-based.
    pub max_left: i64,
    /// Inner right bound: `min(all right-primer left coords) - 1`, 1-based.
    pub min_right: i64,
    /// Outer left: `min(all left-primer right coords) + 1`, 1-based.
    pub min_left: i64,
    /// Outer right: `max(all right-primer left coords) - 1`, 1-based.
    pub max_right: i64,
    /// All left-primer right coords (1-based), for pos2start lookup.
    pub lefts: Vec<i64>,
    /// All right-primer left coords (1-based), for pos2end lookup.
    pub rights: Vec<i64>,
}

impl Amplicon {
    fn new() -> Self {
        Amplicon {
            max_left: 0,
            min_right: i64::MAX,
            min_left: i64::MAX,
            max_right: 0,
            lefts: Vec::new(),
            rights: Vec::new(),
        }
    }
}

/// Build per-amplicon `Amplicon` structs from the primer list. Returns the
/// amplicons and also emits the `AMPLICON` header lines to `out`.
///
/// Mirrors samtools `bed2amplicon`. The BED file must start with a `+` row and
/// end with a `-` row; violations return an error. `do_title` controls whether
/// the column-header comment block is emitted (only once, for the first reference).
pub fn build_amplicons(
    entries: &[PrimerEntry],
    out: &mut impl std::io::Write,
    ref_name: Option<&str>,
    first_amp_idx: usize,
    do_title: bool,
) -> Result<Vec<Amplicon>> {
    if entries.is_empty() {
        return Err(RsomicsError::InvalidInput(
            "BED file has no primer entries".to_string(),
        ));
    }
    if entries[0].rev {
        return Err(RsomicsError::InvalidInput(
            "[ampliconstats] error: BED file should start with the + strand primer".to_string(),
        ));
    }

    let namp = count_amplicons(entries);
    let mut amps: Vec<Amplicon> = (0..namp).map(|_| Amplicon::new()).collect();

    if do_title {
        writeln!(out, "# Amplicon locations from BED file.")?;
        writeln!(
            out,
            "# LEFT/RIGHT are <start>-<end> format and comma-separated for alt-primers."
        )?;
        if ref_name.is_some() {
            writeln!(out, "#\n# AMPLICON\tREF\tNUMBER\tLEFT\tRIGHT")?;
        } else {
            writeln!(out, "#\n# AMPLICON\tNUMBER\tLEFT\tRIGHT")?;
        }
    }

    let mut amp_idx = 0usize;
    let mut last_rev = false;
    let mut left_count = 0usize;
    let mut right_count = 0usize;

    for e in entries {
        if !e.rev && last_rev {
            // Finishing previous amplicon line.
            writeln!(out)?;
            amp_idx += 1;
            left_count = 0;
            right_count = 0;
        }

        if !e.rev {
            // Forward strand: left primer. BED end is the inner boundary.
            let inner = e.right; // 0-based exclusive → 1-based inclusive end of primer
            if left_count == 0 {
                // Start the AMPLICON line.
                if let Some(r) = ref_name {
                    write!(out, "AMPLICON\t{}\t{}", r, amp_idx + 1 + first_amp_idx)?;
                } else {
                    write!(out, "AMPLICON\t{}", amp_idx + 1)?;
                }
            }
            // LEFT coords: BED left+1 to BED right (1-based, inclusive range)
            write!(
                out,
                "{}{}-{}",
                if left_count == 0 { "\t" } else { "," },
                e.left + 1,
                e.right
            )?;
            left_count += 1;

            let a = &mut amps[amp_idx];
            a.lefts.push(inner);
            if a.max_left < inner + 1 {
                a.max_left = inner + 1;
            }
            if a.min_left > inner + 1 {
                a.min_left = inner + 1;
            }
        } else {
            // Reverse strand: right primer. BED start is the inner boundary.
            let inner = e.left; // 0-based start → 1-based exclusive start of amplicon right end
            // RIGHT coords: BED left+1 to BED right
            let sep = if right_count == 0 { "\t" } else { "," };
            write!(out, "{sep}{}-{}", e.left + 1, e.right)?;
            right_count += 1;

            let a = &mut amps[amp_idx];
            a.rights.push(inner);
            if a.min_right > inner - 1 {
                a.min_right = inner - 1;
            }
            if a.max_right < inner - 1 {
                a.max_right = inner - 1;
            }
        }
        last_rev = e.rev;
    }

    // Final amplicon line must end with a newline.
    if last_rev {
        writeln!(out)?;
    } else {
        writeln!(out)?;
        return Err(RsomicsError::InvalidInput(
            "[ampliconstats] error: bed file does not end on a reverse strand primer.".to_string(),
        ));
    }

    Ok(amps)
}
