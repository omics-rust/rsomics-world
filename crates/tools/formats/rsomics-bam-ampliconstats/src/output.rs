//! Output formatting, ported from samtools `dump_stats` in `amplicon_stats.c`.
//!
//! Emits FSS/FREADS/FVDEPTH/FRPERC/FDEPTH/FPCOV/FDP_ALL/FDP_VALID/FTCOORD/FAMP
//! per-file sections, and CSS/CREADS/CVDEPTH/CRPERC/CDEPTH/CPCOV/CDP_ALL/CDP_VALID/CTCOORD/CAMP
//! combined sections.

use std::io::Write;

use rsomics_common::Result;

use crate::stats::{AmpStatsArgs, MAX_DEPTH_LEVELS, RefData};

/// Emit all output sections. `type_char` is `'F'` (per-file) or `'C'` (combined).
/// `local` selects `lstats` vs `gstats`. `multi_ref` mirrors samtools' flag
/// (default true since 1.13): when true, each row includes a reference-name column.
#[allow(clippy::too_many_arguments)]
pub fn dump_stats(
    type_char: char,
    name: &str,
    nfile: usize,
    refs: &[Option<RefData>],
    args: &AmpStatsArgs,
    local: bool,
    multi_ref: bool,
    out: &mut impl Write,
) -> Result<()> {
    let t = type_char;

    // ── SS: Summary statistics ───────────────────────────────────────────────
    writeln!(out, "# Summary stats.")?;
    writeln!(out, "# Use 'grep ^{t}SS | cut -f 2-' to extract this part.")?;

    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        let nmatch = stats.nseq - stats.nfiltered - stats.nfailprimer;

        let name_ref = name_ref_str(name, &slot.ref_name, multi_ref);

        writeln!(
            out,
            "{t}SS\t{name_ref}\traw total sequences:\t{}",
            stats.nseq
        )?;
        writeln!(
            out,
            "{t}SS\t{name_ref}\tfiltered sequences:\t{}",
            stats.nfiltered
        )?;
        writeln!(
            out,
            "{t}SS\t{name_ref}\tfailed primer match:\t{}",
            stats.nfailprimer
        )?;
        writeln!(out, "{t}SS\t{name_ref}\tmatching sequences:\t{}", nmatch)?;

        // Consensus depth counts.
        let amps = &slot.amps;
        let mut d = 0usize;
        loop {
            let mut start = 0i64;
            let mut covered = 0i64;
            let mut total = 0i64;
            for (i, amp) in amps.iter().enumerate() {
                let offset = amp.min_left - 1;
                let j_lo = start.max(amp.max_left - 1);
                let j_hi = start.max(amp.min_right);
                for j in j_lo..j_hi {
                    let apos = i * stats.max_amp_len + (j - offset) as usize;
                    if apos < stats.coverage.len()
                        && stats.coverage[apos] >= args.min_depth[d] as i64
                    {
                        covered += 1;
                    }
                    total += 1;
                }
                start = start.max(amp.min_right);
            }
            writeln!(
                out,
                "{t}SS\t{name_ref}\tconsensus depth count < {} and >= {}:\t{}\t{}",
                args.min_depth[d],
                args.min_depth[d],
                total - covered,
                covered
            )?;
            d += 1;
            if d >= MAX_DEPTH_LEVELS || args.min_depth[d] == 0 {
                break;
            }
        }
    }

    // ── READS ────────────────────────────────────────────────────────────────
    writeln!(out, "# Absolute matching read counts per amplicon.")?;
    writeln!(
        out,
        "# Use 'grep ^{t}READS | cut -f 2-' to extract this part."
    )?;
    write!(out, "{t}READS\t{name}")?;
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        for v in &stats.nreads[..slot.amps.len()] {
            write!(out, "\t{v}")?;
        }
    }
    writeln!(out)?;

    // ── VDEPTH ───────────────────────────────────────────────────────────────
    write!(out, "{t}VDEPTH\t{name}")?;
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        for v in &stats.nfull_reads[..slot.amps.len()] {
            write!(out, "\t{}", *v as i64)?;
        }
    }
    writeln!(out)?;

    if type_char == 'C' {
        let nf = nfile as f64;
        // CREADS MEAN
        write!(out, "CREADS\tMEAN")?;
        for slot in refs.iter().flatten() {
            let stats = &slot.global;
            for v in &stats.nreads[..slot.amps.len()] {
                write!(out, "\t{:.1}", *v as f64 / nf)?;
            }
        }
        writeln!(out)?;
        // CREADS STDDEV
        write!(out, "CREADS\tSTDDEV")?;
        for slot in refs.iter().flatten() {
            let stats = &slot.global;
            for (i, v) in stats.nreads[..slot.amps.len()].iter().enumerate() {
                let n1 = *v as f64;
                let sd = if nfile > 1 && stats.nreads2[i] > 0 {
                    let var = stats.nreads2[i] as f64 / nf - (n1 / nf) * (n1 / nf);
                    if var > 0.0 { var.sqrt() } else { 0.0 }
                } else {
                    0.0
                };
                write!(out, "\t{sd:.1}")?;
            }
        }
        writeln!(out)?;
    }

    // ── RPERC ────────────────────────────────────────────────────────────────
    writeln!(out, "# Read percentage of distribution between amplicons.")?;
    writeln!(
        out,
        "# Use 'grep ^{t}RPERC | cut -f 2-' to extract this part."
    )?;
    write!(out, "{t}RPERC\t{name}")?;
    let all_nseq: i64 = refs
        .iter()
        .flatten()
        .map(|s| {
            let st = if local { &s.local } else { &s.global };
            st.nseq - st.nfiltered - st.nfailprimer
        })
        .sum();
    let nf = nfile as f64;
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        for (i, v) in stats.nreads[..slot.amps.len()].iter().enumerate() {
            if type_char == 'C' {
                write!(out, "\t{:.3}", stats.nrperc[i] / nf)?;
            } else {
                let pct = if all_nseq > 0 {
                    100.0 * *v as f64 / all_nseq as f64
                } else {
                    0.0
                };
                write!(out, "\t{pct:.3}")?;
            }
        }
    }
    writeln!(out)?;

    if type_char == 'C' {
        write!(out, "CRPERC\tMEAN")?;
        for slot in refs.iter().flatten() {
            let stats = &slot.global;
            for v in &stats.nrperc[..slot.amps.len()] {
                write!(out, "\t{:.3}", v / nf)?;
            }
        }
        writeln!(out)?;
        write!(out, "CRPERC\tSTDDEV")?;
        for slot in refs.iter().flatten() {
            let stats = &slot.global;
            for (i, v) in stats.nrperc[..slot.amps.len()].iter().enumerate() {
                let n1 = *v;
                let var = stats.nrperc2[i] / nf - (n1 / nf) * (n1 / nf);
                write!(out, "\t{:.3}", if var > 0.0 { var.sqrt() } else { 0.0 })?;
            }
        }
        writeln!(out)?;
    }

    // ── DEPTH ────────────────────────────────────────────────────────────────
    writeln!(out, "# Read depth per amplicon.")?;
    writeln!(
        out,
        "# Use 'grep ^{t}DEPTH | cut -f 2-' to extract this part."
    )?;
    write!(out, "{t}DEPTH\t{name}")?;
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        let nseq_local = stats.nseq - stats.nfiltered - stats.nfailprimer;
        for (i, amp) in slot.amps.iter().enumerate() {
            let alen = (amp.min_right - amp.max_left + 1).max(1) as f64;
            let depth = if nseq_local > 0 {
                stats.nbases[i] as f64 / alen
            } else {
                0.0
            };
            write!(out, "\t{depth:.1}")?;
        }
    }
    writeln!(out)?;

    if type_char == 'C' {
        write!(out, "CDEPTH\tMEAN")?;
        for slot in refs.iter().flatten() {
            let stats = &slot.global;
            let nseq_g = stats.nseq - stats.nfiltered - stats.nfailprimer;
            for (i, amp) in slot.amps.iter().enumerate() {
                let alen = (amp.min_right - amp.max_left + 1).max(1) as f64;
                let depth = if nseq_g > 0 {
                    stats.nbases[i] as f64 / alen / nf
                } else {
                    0.0
                };
                write!(out, "\t{depth:.1}")?;
            }
        }
        writeln!(out)?;
        write!(out, "CDEPTH\tSTDDEV")?;
        for slot in refs.iter().flatten() {
            let stats = &slot.global;
            for (i, amp) in slot.amps.iter().enumerate() {
                let alen = (amp.min_right - amp.max_left + 1).max(1) as f64;
                let n1 = stats.nbases[i] as f64 / alen;
                let var = stats.nbases2[i] as f64 / (alen * alen) / nf - (n1 / nf) * (n1 / nf);
                write!(out, "\t{:.1}", if var > 0.0 { var.sqrt() } else { 0.0 })?;
            }
        }
        writeln!(out)?;
    }

    // ── PCOV (F only for per-file; C for combined) ───────────────────────────
    if type_char == 'F' {
        writeln!(out, "# Percentage coverage per amplicon")?;
        writeln!(
            out,
            "# Use 'grep ^{t}PCOV | cut -f 2-' to extract this part."
        )?;
        let mut d = 0usize;
        loop {
            write!(out, "{t}PCOV-{}\t{name}", args.min_depth[d])?;
            for slot in refs.iter().flatten() {
                let stats = if local { &slot.local } else { &slot.global };
                for i in 0..slot.amps.len() {
                    write!(out, "\t{:.2}", stats.covered_perc[i][d])?;
                }
            }
            writeln!(out)?;
            d += 1;
            if d >= MAX_DEPTH_LEVELS || args.min_depth[d] == 0 {
                break;
            }
        }
    } else {
        let mut d = 0usize;
        loop {
            write!(out, "CPCOV-{}\tMEAN", args.min_depth[d])?;
            for slot in refs.iter().flatten() {
                let stats = &slot.global;
                for i in 0..slot.amps.len() {
                    write!(out, "\t{:.1}", stats.covered_perc[i][d] / nf)?;
                }
            }
            writeln!(out)?;
            write!(out, "CPCOV-{}\tSTDDEV", args.min_depth[d])?;
            for slot in refs.iter().flatten() {
                let stats = &slot.global;
                for i in 0..slot.amps.len() {
                    let n1 = stats.covered_perc[i][d] / nf;
                    let var = stats.covered_perc2[i][d] / nf - n1 * n1;
                    write!(out, "\t{:.1}", if var > 0.0 { var.sqrt() } else { 0.0 })?;
                }
            }
            writeln!(out)?;
            d += 1;
            if d >= MAX_DEPTH_LEVELS || args.min_depth[d] == 0 {
                break;
            }
        }
    }

    // ── DP_ALL ───────────────────────────────────────────────────────────────
    writeln!(out, "# Depth per reference base for ALL data.")?;
    writeln!(
        out,
        "# Use 'grep ^{t}DP_ALL | cut -f 2-' to extract this part."
    )?;
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        if multi_ref {
            write!(out, "{t}DP_ALL\t{name}\t{}", slot.ref_name)?;
        } else {
            write!(out, "{t}DP_ALL\t{name}")?;
        }
        emit_depth_rle(
            &stats.depth_all[..slot.ref_len as usize],
            args.depth_bin,
            out,
        )?;
        writeln!(out)?;
    }

    // ── DP_VALID ─────────────────────────────────────────────────────────────
    writeln!(
        out,
        "# Depth per reference base for full-length valid amplicon data."
    )?;
    writeln!(
        out,
        "# Use 'grep ^{t}DP_VALID | cut -f 2-' to extract this part."
    )?;
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        if multi_ref {
            write!(out, "{t}DP_VALID\t{name}\t{}", slot.ref_name)?;
        } else {
            write!(out, "{t}DP_VALID\t{name}")?;
        }
        emit_depth_rle(
            &stats.depth_valid[..slot.ref_len as usize],
            args.depth_bin,
            out,
        )?;
        writeln!(out)?;
    }

    // ── TCOORD ───────────────────────────────────────────────────────────────
    writeln!(out, "# Distribution of aligned template coordinates.")?;
    writeln!(
        out,
        "# Use 'grep ^{t}TCOORD | cut -f 2-' to extract this part."
    )?;
    // samtools emits the "all amplicons" aggregate TCOORD bucket (i=-1) when nref==1,
    // regardless of multi_ref flag. nref here = number of references with primer data.
    let nref_with_primers = refs.iter().flatten().count();
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        let namp = slot.amps.len();

        // i=-1 (all-amps bucket) only when exactly one reference has primers.
        let start_i: i64 = if nref_with_primers == 1 { -1 } else { 0 };
        for i in start_i..namp as i64 {
            let bucket = (i + 1) as usize;
            if bucket >= stats.tcoord.len() {
                continue;
            }
            let amp_num = i + 1 + slot.first_amp_idx as i64;

            // Collect, sort by descending freq.
            let mut tpos: Vec<TCoordEntry> = stats.tcoord[bucket]
                .iter()
                .filter_map(|(&key, &val)| {
                    let count = (val & 0xFFFF_FFFF) as u32;
                    if count == 0 {
                        return None;
                    }
                    let status = (val >> 32) as u32;
                    let start32 = (key & 0xFFFF_FFFF) as i32;
                    let end32 = (key >> 32) as i32;
                    Some(TCoordEntry {
                        start: start32,
                        end: end32,
                        freq: count,
                        status,
                    })
                })
                .collect();

            if args.tcoord_bin > 1 {
                aggregate_tcoord(args.tcoord_bin, &mut tpos);
            }

            tpos.sort_unstable_by(|a, b| {
                b.freq
                    .cmp(&a.freq)
                    .then(a.start.cmp(&b.start))
                    .then(a.end.cmp(&b.end))
            });

            write!(out, "{t}TCOORD\t{name}\t{amp_num}")?;
            for e in &tpos {
                if e.freq < args.tcoord_min_count {
                    continue;
                }
                write!(out, "\t{},{},{},{}", e.start, e.end, e.freq, e.status)?;
            }
            writeln!(out)?;
        }
    }

    // ── AMP ──────────────────────────────────────────────────────────────────
    writeln!(out, "# Classification of amplicon status.  Columns are")?;
    writeln!(
        out,
        "# number with both primers from this amplicon, number with"
    )?;
    writeln!(
        out,
        "# primers from different amplicon, and number with a position"
    )?;
    writeln!(out, "# not matching any valid amplicon primer site")?;
    writeln!(
        out,
        "# Use 'grep ^{t}AMP | cut -f 2-' to extract this part."
    )?;

    // All-amplicons merged row (index 0).
    let mut total_dist = [0i64; 3];
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        for i in 0..slot.amps.len() {
            for (d, cell) in total_dist.iter_mut().enumerate() {
                *cell += stats.amp_dist[i][d];
            }
        }
    }
    writeln!(
        out,
        "{t}AMP\t{name}\t0\t{}\t{}\t{}",
        total_dist[0], total_dist[1], total_dist[2]
    )?;

    // Per-amplicon rows.
    for slot in refs.iter().flatten() {
        let stats = if local { &slot.local } else { &slot.global };
        for (i, _amp) in slot.amps.iter().enumerate() {
            let amp_num = i + 1 + slot.first_amp_idx;
            writeln!(
                out,
                "{t}AMP\t{name}\t{amp_num}\t{}\t{}\t{}",
                stats.amp_dist[i][0], stats.amp_dist[i][1], stats.amp_dist[i][2]
            )?;
        }
    }

    Ok(())
}

struct TCoordEntry {
    start: i32,
    end: i32,
    freq: u32,
    status: u32,
}

/// Run-length encode a depth array. Matches samtools' `depth_bin` RLE encoding.
fn emit_depth_rle(depth: &[i64], depth_bin: f64, out: &mut impl Write) -> Result<()> {
    let n = depth.len();
    let mut i = 0;
    while i < n {
        let mut dmin = depth[i];
        let mut dmax = depth[i];
        let mut dmid = (dmin + dmax) as f64 / 2.0;
        let mut low = dmid * (1.0 - depth_bin);
        let mut high = dmid * (1.0 + depth_bin);
        let mut j = i + 1;
        while j < n {
            let d = depth[j];
            if (d as f64) < low || (d as f64) > high {
                break;
            }
            if d < dmin {
                dmin = d;
                dmid = (dmin + dmax) as f64 / 2.0;
                low = dmid * (1.0 - depth_bin);
                high = dmid * (1.0 + depth_bin);
            } else if d > dmax {
                dmax = d;
                dmid = (dmin + dmax) as f64 / 2.0;
                low = dmid * (1.0 - depth_bin);
                high = dmid * (1.0 + depth_bin);
            }
            j += 1;
        }
        write!(out, "\t{},{}", dmid as i64, j - i)?;
        i = j;
    }
    Ok(())
}

/// Aggregate nearby tcoord entries. Mirrors `aggregate_tcoord` (O(N²) but N small).
fn aggregate_tcoord(tcoord_bin: i64, tpos: &mut Vec<TCoordEntry>) {
    let half = tcoord_bin / 2;

    // Sort by descending freq, then start, then end.
    tpos.sort_unstable_by(|a, b| {
        b.freq
            .cmp(&a.freq)
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
    });

    let n = tpos.len();
    let mut k = 0usize;
    for j in 0..n {
        if tpos[j].freq == 0 {
            continue;
        }
        if k < j {
            tpos.swap(k, j);
        }
        for j2 in (j + 1)..n {
            if tpos[j2].freq == 0 {
                continue;
            }
            let ds = (tpos[k].start as i64 - tpos[j2].start as i64).abs();
            let de = (tpos[k].end as i64 - tpos[j2].end as i64).abs();
            if ds < half && de < half && tpos[k].status == tpos[j2].status {
                tpos[k].freq += tpos[j2].freq;
                tpos[j2].freq = 0;
            }
        }
        k += 1;
    }
    tpos.truncate(k);
}

fn name_ref_str(name: &str, ref_name: &str, multi_ref: bool) -> String {
    if multi_ref {
        format!("{name}\t{ref_name}")
    } else {
        name.to_string()
    }
}
