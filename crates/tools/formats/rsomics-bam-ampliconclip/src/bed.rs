//! Primer BED loading and the per-read clip-site match, ported from
//! `bam_ampliconclip.c` (`load_bed_file_multi_ref` / `matching_clip_site`).
//!
//! Each reference's primers are kept in a list sorted by `right` (the BED end
//! coordinate), exactly as samtools' `bed_entry_sort` orders them, so the match
//! can binary-search the sorted list then linear-scan a bounded window.

use std::collections::HashMap;
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

/// A single primer interval. `left`/`right` are the BED's 0-based half-open
/// coordinates; `rev` is meaningful only when loaded with `--strand`.
#[derive(Debug, Clone)]
pub struct BedEntry {
    pub left: i64,
    pub right: i64,
    pub rev: bool,
    pub num_reads: i64,
}

/// Primers for one reference, sorted by `right` ascending. `longest` is the
/// widest interval span on this reference (`max(right - left)`), used to bound
/// the match scan exactly as samtools' `list->longest`.
#[derive(Debug, Default)]
pub struct BedList {
    pub entries: Vec<BedEntry>,
    pub longest: i64,
}

/// All primers, keyed by reference name. `ref_order` preserves first-seen order
/// (samtools' `ref_list`) so the `--primer-counts` summary emits refs in BED
/// order, not hash order.
#[derive(Debug, Default)]
pub struct PrimerBed {
    pub by_ref: HashMap<String, BedList>,
    pub ref_order: Vec<String>,
}

/// Load primer intervals from `path`. With `get_strand`, column 6 (strand) is
/// required and parsed; a non-`+`/`-` value is a hard error, matching samtools.
/// Without it, only the first three columns are needed. Each reference's list is
/// sorted by `right` (samtools sorts with `sort_by_pos = 1` for clipping).
pub fn load(path: &Path, get_strand: bool) -> Result<PrimerBed> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;

    let mut bed = PrimerBed::default();

    for (lineno, line) in text.lines().enumerate() {
        let line_count = lineno + 1;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("track ") || line.starts_with("browser ") {
            continue;
        }

        let cols: Vec<&str> = line.split_whitespace().collect();
        let need = if get_strand { 6 } else { 3 };
        if cols.len() < need {
            return Err(RsomicsError::InvalidInput(format!(
                "invalid bed file format in line {line_count} of {}. \
                 Parsed {} columns, but need at least {need}",
                path.display(),
                cols.len()
            )));
        }

        let ref_name = cols[0];
        let left: i64 = cols[1].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("line {line_count}: bad start coordinate"))
        })?;
        let right: i64 = cols[2].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("line {line_count}: bad end coordinate"))
        })?;

        let rev = if get_strand {
            match cols[5] {
                "+" => false,
                "-" => true,
                other => {
                    return Err(RsomicsError::InvalidInput(format!(
                        "bad strand value in line {line_count}, expecting '+' or '-', found '{other}'"
                    )));
                }
            }
        } else {
            false
        };

        let list = bed.by_ref.entry(ref_name.to_string()).or_insert_with(|| {
            bed.ref_order.push(ref_name.to_string());
            BedList::default()
        });
        if right - left > list.longest {
            list.longest = right - left;
        }
        list.entries.push(BedEntry {
            left,
            right,
            rev,
            num_reads: 0,
        });
    }

    if bed.by_ref.is_empty() {
        return Err(RsomicsError::InvalidInput(format!(
            "unable to load bed file: no entries in {}",
            path.display()
        )));
    }

    for list in bed.by_ref.values_mut() {
        // samtools bed_entry_sort: ascending by `right`. A stable sort keeps the
        // input order of ties, which matters for which primer wins on an exact
        // size tie (samtools' qsort is unstable, but the size tie-break below
        // keeps the first-seen max, so a stable sort reproduces that choice).
        list.entries.sort_by_key(|e| e.right);
    }

    Ok(bed)
}

/// samtools `matching_clip_site`: find the largest primer overlap for a read at
/// reference coordinate `pos` (forward: read start; reverse: read end), within
/// `tol` bases. Returns the number of reference bases to clip (0 = no match) and,
/// on a match, increments the winning primer's `num_reads`.
///
/// The forward/reverse asymmetry mirrors the C exactly:
/// - forward: window `[left - tol, right]`, clip size `right - pos`;
/// - reverse: window `[left, right + tol]`, clip size `pos - left`.
///
/// The binary search seeds the scan at the first primer whose `right` exceeds a
/// tolerance-shifted position; the scan then breaks once a primer's window
/// starts beyond `pos + longest + tol` (no later primer can overlap).
pub fn matching_clip_site(
    list: &mut BedList,
    pos: i64,
    is_rev: bool,
    use_strand: bool,
    tol: i64,
) -> i64 {
    let longest = list.longest;
    let n = list.entries.len();

    let pos_tol = if is_rev {
        if pos > tol { pos - tol } else { 0 }
    } else {
        pos
    };

    // Binary search for the leftmost index `l` such that all primers before it
    // have `right <= pos_tol` — samtools' exact `r - l > 1` half-open bisection.
    let mut l: usize = 0;
    let mut r: usize = n;
    let mut mid = n / 2;
    while r - l > 1 {
        if list.entries[mid].right <= pos_tol {
            l = mid;
        } else {
            r = mid;
        }
        mid = (l + r) / 2;
    }

    let mut size: i64 = 0;
    let mut used_i: Option<usize> = None;

    for i in l..n {
        let e = &list.entries[i];

        if use_strand && is_rev != e.rev {
            continue;
        }

        let (mod_left, mod_right) = if is_rev {
            (e.left, e.right + tol)
        } else {
            (if e.left > tol { e.left - tol } else { 0 }, e.right)
        };

        if pos + longest + tol < mod_right {
            break;
        }

        if pos >= mod_left && pos <= mod_right {
            if is_rev {
                if size < pos - e.left {
                    size = pos - e.left;
                    used_i = Some(i);
                }
            } else if size < e.right - pos {
                size = e.right - pos;
                used_i = Some(i);
            }
        }
    }

    if let Some(i) = used_i {
        list.entries[i].num_reads += 1;
    }
    size
}
