//! Per-amplicon statistics accumulation, ported from samtools `amplicon_stats.c`.
//!
//! Origin: samtools amplicon_stats.c (MIT) — James Bonfield, Genome Research Ltd.
//! Implements `accumulate_stats` logic for single-file output (`F` prefix)
//! plus combined multi-file output (`C` prefix).

use std::collections::HashMap;
use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::RecordReader;
use rsomics_common::{Result, RsomicsError};

use crate::bed::{self, Amplicon};
use crate::output;

pub const MAX_DEPTH_LEVELS: usize = 5;

/// Default flag filter lower 16 bits: UNMAP(4)|SECONDARY(0x100)|QCFAIL(0x200)|SUPPLEMENTARY(0x800).
pub const DEFAULT_FLAG_FILTER: u16 = 0x0B04;
pub const DEFAULT_MAX_DELTA: i64 = 30;
pub const DEFAULT_MIN_DEPTH: [u32; MAX_DEPTH_LEVELS] = [1, 0, 0, 0, 0];
pub const DEFAULT_TCOORD_MIN_COUNT: u32 = 10;
pub const DEFAULT_DEPTH_BIN: f64 = 0.01;
pub const MAX_AMP_DEFAULT: usize = 1000;
pub const MAX_AMP_LEN_DEFAULT: usize = 1000;

pub struct AmpStatsArgs {
    pub flag_require: u16,
    pub flag_filter: u16,
    pub max_delta: i64,
    pub min_depth: [u32; MAX_DEPTH_LEVELS],
    pub max_amp: usize,
    pub max_amp_len: usize,
    pub tlen_adj: i64,
    pub depth_bin: f64,
    pub tcoord_min_count: u32,
    pub tcoord_bin: i64,
    pub workers: NonZero<usize>,
    /// Force single-ref (<=1.12) output format. Default false = multi-ref (samtools >=1.13 default).
    pub single_ref: bool,
}

impl Default for AmpStatsArgs {
    fn default() -> Self {
        AmpStatsArgs {
            flag_require: 0,
            flag_filter: DEFAULT_FLAG_FILTER,
            max_delta: DEFAULT_MAX_DELTA,
            min_depth: DEFAULT_MIN_DEPTH,
            max_amp: MAX_AMP_DEFAULT,
            max_amp_len: MAX_AMP_LEN_DEFAULT,
            tlen_adj: 0,
            depth_bin: DEFAULT_DEPTH_BIN,
            tcoord_min_count: DEFAULT_TCOORD_MIN_COUNT,
            tcoord_bin: 1,
            workers: NonZero::new(1).unwrap(),
            single_ref: false,
        }
    }
}

/// Per-amplicon, per-sample statistics. Mirrors `astats_t`.
pub struct AmpStats {
    pub nseq: i64,
    pub nfiltered: i64,
    pub nfailprimer: i64,

    pub max_amp: usize,
    pub max_amp_len: usize,
    pub ref_len: i64,

    pub nreads: Vec<i64>,
    pub nreads2: Vec<i64>,
    pub nfull_reads: Vec<f64>,
    pub nrperc: Vec<f64>,
    pub nrperc2: Vec<f64>,
    pub nbases: Vec<i64>,
    pub nbases2: Vec<i64>,
    /// Flat [max_amp * max_amp_len] coverage array.
    pub coverage: Vec<i64>,
    pub covered_perc: Vec<[f64; MAX_DEPTH_LEVELS]>,
    pub covered_perc2: Vec<[f64; MAX_DEPTH_LEVELS]>,
    /// Template coord histograms; index 0 = "all", 1..=namp = per-amplicon.
    pub tcoord: Vec<HashMap<u64, u64>>,
    pub amp_dist: Vec<[i64; 3]>,
    pub depth_all: Vec<i64>,
    pub depth_valid: Vec<i64>,
}

impl AmpStats {
    pub fn new(ref_len: i64, max_amp: usize, max_amp_len: usize) -> Self {
        AmpStats {
            nseq: 0,
            nfiltered: 0,
            nfailprimer: 0,
            max_amp,
            max_amp_len,
            ref_len,
            nreads: vec![0; max_amp],
            nreads2: vec![0; max_amp],
            nfull_reads: vec![0.0; max_amp],
            nrperc: vec![0.0; max_amp],
            nrperc2: vec![0.0; max_amp],
            nbases: vec![0; max_amp],
            nbases2: vec![0; max_amp],
            coverage: vec![0; max_amp * max_amp_len],
            covered_perc: vec![[0.0; MAX_DEPTH_LEVELS]; max_amp],
            covered_perc2: vec![[0.0; MAX_DEPTH_LEVELS]; max_amp],
            tcoord: (0..=max_amp).map(|_| HashMap::new()).collect(),
            amp_dist: vec![[0; 3]; max_amp],
            depth_all: vec![0; ref_len as usize],
            depth_valid: vec![0; ref_len as usize],
        }
    }

    pub fn reset(&mut self) {
        self.nseq = 0;
        self.nfiltered = 0;
        self.nfailprimer = 0;
        self.nreads.fill(0);
        self.nreads2.fill(0);
        self.nfull_reads.fill(0.0);
        self.nrperc.fill(0.0);
        self.nrperc2.fill(0.0);
        self.nbases.fill(0);
        self.nbases2.fill(0);
        self.coverage.fill(0);
        for x in &mut self.covered_perc {
            *x = [0.0; MAX_DEPTH_LEVELS];
        }
        for x in &mut self.covered_perc2 {
            *x = [0.0; MAX_DEPTH_LEVELS];
        }
        for h in &mut self.tcoord {
            h.clear();
        }
        for x in &mut self.amp_dist {
            *x = [0; 3];
        }
        self.depth_all.fill(0);
        self.depth_valid.fill(0);
    }
}

/// Per-reference state shared by both accumulation and output.
pub struct RefData {
    pub ref_name: String,
    pub ref_len: i64,
    pub amps: Vec<Amplicon>,
    pub lookup: PosLookup,
    pub local: AmpStats,
    pub global: AmpStats,
    pub first_amp_idx: usize,
}

/// Position-to-amplicon lookup tables.
pub struct PosLookup {
    pub pos2start: Vec<i32>,
    pub pos2end: Vec<i32>,
}

impl PosLookup {
    pub fn build(amps: &[Amplicon], ref_len: i64, max_delta: i64) -> Self {
        let len = ref_len as usize;
        let mut pos2start = vec![-1i32; len];
        let mut pos2end = vec![-1i32; len];

        for (i, amp) in amps.iter().enumerate() {
            for &lpos in &amp.lefts {
                // lpos = BED right of + primer = inner left boundary (0-based).
                let lo = ((lpos - max_delta).max(1)) as usize;
                let hi = ((lpos + max_delta) as usize).min(len);
                for p in lo..=hi {
                    if p <= len {
                        pos2start[p - 1] = i as i32;
                    }
                }
            }
            for &rpos in &amp.rights {
                // rpos = BED start of - primer = inner right boundary (0-based).
                let lo = ((rpos - max_delta).max(1)) as usize;
                let hi = ((rpos + max_delta) as usize).min(len);
                for p in lo..=hi {
                    if p <= len {
                        pos2end[p - 1] = i as i32;
                    }
                }
            }
        }
        PosLookup { pos2start, pos2end }
    }

    #[inline]
    pub fn get_start(&self, pos: i64) -> i32 {
        if pos >= 0 && (pos as usize) < self.pos2start.len() {
            self.pos2start[pos as usize]
        } else {
            -1
        }
    }

    #[inline]
    pub fn get_end(&self, pos: i64) -> i32 {
        if pos >= 0 && (pos as usize) < self.pos2end.len() {
            self.pos2end[pos as usize]
        } else {
            -1
        }
    }
}

/// Compute BAM alignment end position from CIGAR. Mirrors htslib `bam_endpos`.
#[inline]
fn bam_endpos(start: i64, cigar_ops: impl Iterator<Item = (u8, u32)>) -> i64 {
    let mut pos = start;
    for (op, len) in cigar_ops {
        if matches!(op, 0 | 2 | 3 | 7 | 8) {
            pos += len as i64;
        }
    }
    pos
}

/// Accumulate one record into local stats. Mirrors `accumulate_stats`.
#[allow(clippy::too_many_arguments)]
fn accumulate_record(
    flags: u16,
    start: i64,
    end: i64,
    tlen: i32,
    qname: &[u8],
    args: &AmpStatsArgs,
    amps: &[Amplicon],
    lookup: &PosLookup,
    stats: &mut AmpStats,
    ref_len: i64,
    qend_map: &mut HashMap<Vec<u8>, (i64, i64)>,
) {
    const BAM_FUNMAP: u16 = 0x4;
    const BAM_FPAIRED: u16 = 0x1;
    const BAM_FREVERSE: u16 = 0x10;
    const BAM_FMUNMAP: u16 = 0x8;
    const BAM_FSUPPLEMENTARY: u16 = 0x800;
    const BAM_FSECONDARY: u16 = 0x100;

    stats.nseq += 1;

    if (u32::from(flags) & u32::from(args.flag_require)) != u32::from(args.flag_require)
        || (u32::from(flags) & u32::from(args.flag_filter)) != 0
    {
        stats.nfiltered += 1;
        return;
    }

    if end == start && (args.flag_filter & BAM_FUNMAP) != 0 {
        stats.nfiltered += 1;
        return;
    }

    let is_paired = (flags & BAM_FPAIRED) != 0;
    let is_reverse = (flags & BAM_FREVERSE) != 0;
    let is_secondary = (flags & BAM_FSECONDARY) != 0;
    let is_supplementary = (flags & BAM_FSUPPLEMENTARY) != 0;
    let mate_unmapped = (flags & BAM_FMUNMAP) != 0;
    let is_unmapped = (flags & BAM_FUNMAP) != 0;

    // Overlap removal for paired primary reads.
    let mut mstart = start;
    let mut prev_start = 0i64;
    let mut prev_end = 0i64;
    if is_paired && !is_supplementary && !is_secondary {
        if let Some(&(ps, pe)) = qend_map.get(qname) {
            prev_start = ps;
            prev_end = pe;
            mstart = mstart.max(prev_end);
            qend_map.remove(qname);
        } else {
            qend_map.insert(qname.to_vec(), (start, end));
        }
    }

    let depth_end = end.min(ref_len);
    for i in mstart..depth_end {
        stats.depth_all[i as usize] += 1;
    }

    let anum = if is_reverse || !is_paired {
        lookup.get_end(end - 1)
    } else {
        lookup.get_start(start)
    };

    if anum == -1 {
        stats.nfailprimer += 1;
    }

    if anum >= 0 {
        let a = anum as usize;
        let amp = &amps[a];
        let c_start = start.max(amp.max_left);
        let c_end = end.min(amp.min_right + 1);
        if c_end > c_start {
            stats.nreads[a] += 1;
            stats.nbases[a] += c_end - c_start;

            let ostart = start.max(amp.min_left - 1);
            let oend = end.min(amp.max_right);
            let offset = amp.min_left - 1;
            for i in ostart..oend {
                let apos = a * stats.max_amp_len + (i - offset) as usize;
                if apos < stats.coverage.len() {
                    stats.coverage[apos] += 1;
                }
            }
        } else {
            stats.nfailprimer += 1;
        }
    }

    // Template-length pair classification.
    let mut oth_anum: i32 = -1;
    let t_end: i64;

    if is_paired {
        let raw_t_end = if is_reverse { end } else { start } + tlen as i64;
        t_end = raw_t_end
            + if tlen > 0 {
                -args.tlen_adj
            } else {
                args.tlen_adj
            };
        if t_end > 0 && t_end < ref_len && tlen != 0 {
            oth_anum = if is_reverse {
                lookup.get_start(t_end)
            } else {
                lookup.get_end(t_end)
            };
        }
    } else {
        oth_anum = lookup.get_start(start);
        t_end = end;
    }

    let astatus: usize;
    if anum != -1 && oth_anum != -1 {
        astatus = if oth_anum == anum { 0 } else { 1 };
        if start <= t_end {
            stats.amp_dist[anum as usize][astatus] += 1;
        }
    } else if anum >= 0 {
        astatus = 2;
        stats.amp_dist[anum as usize][2] += 1;
    } else {
        astatus = 2;
    }

    // depth_valid: only for reads fully spanning their amplicon.
    if astatus == 0 && !is_unmapped && !mate_unmapped {
        if prev_end != 0 && mstart > prev_end {
            for i in prev_start..prev_end {
                if (i as usize) < stats.depth_valid.len() {
                    stats.depth_valid[i as usize] -= 1;
                }
            }
            if anum >= 0 {
                stats.nfull_reads[anum as usize] -= if is_paired { 0.5 } else { 1.0 };
            }
        } else {
            for i in mstart..depth_end {
                stats.depth_valid[i as usize] += 1;
            }
            if anum >= 0 {
                stats.nfull_reads[anum as usize] += if is_paired { 0.5 } else { 1.0 };
            }
        }
    }

    // tcoord: only for left-to-right reads.
    if is_paired && tlen <= 0 {
        return;
    }

    let tc_start = start;
    let tc_end = if is_paired {
        start + tlen as i64 - 1
    } else {
        end
    };
    let tcoord_key =
        (tc_start.min(u32::MAX as i64) as u64) | ((tc_end.min(u32::MAX as i64) as u64) << 32);

    let bucket = if anum >= 0 { anum as usize + 1 } else { 0 };
    if bucket < stats.tcoord.len() {
        let entry = stats.tcoord[bucket].entry(tcoord_key).or_insert(0);
        let count = (*entry & 0xFFFF_FFFF).wrapping_add(1);
        *entry = count | ((astatus as u64) << 32);
    }
}

fn compute_covered_perc(stats: &mut AmpStats, amps: &[Amplicon], args: &AmpStatsArgs) {
    for (a, amp) in amps.iter().enumerate() {
        let alen = (amp.min_right - amp.max_left + 1).max(1) as f64;
        let offset = amp.min_left - 1;
        for d in 0..MAX_DEPTH_LEVELS {
            if d > 0 && args.min_depth[d] == 0 {
                break;
            }
            let mut covered = 0i64;
            for j in (amp.max_left - 1)..amp.min_right {
                let apos = a * stats.max_amp_len + (j - offset) as usize;
                if apos < stats.coverage.len() && stats.coverage[apos] >= args.min_depth[d] as i64 {
                    covered += 1;
                }
            }
            stats.covered_perc[a][d] = 100.0 * covered as f64 / alen;
        }
    }
}

fn append_to_global(local: &AmpStats, global: &mut AmpStats, namp: usize, all_nseq: i64) {
    global.nseq += local.nseq;
    global.nfiltered += local.nfiltered;
    global.nfailprimer += local.nfailprimer;

    for a in 0..=namp {
        if a >= local.tcoord.len() || a >= global.tcoord.len() {
            break;
        }
        for (&key, &lval) in &local.tcoord[a] {
            let lcount = lval & 0xFFFF_FFFF;
            if lcount == 0 {
                continue;
            }
            let gentry = global.tcoord[a].entry(key).or_insert(0);
            *gentry =
                ((*gentry & 0xFFFF_FFFF).wrapping_add(lcount)) | (lval & 0xFFFF_FFFF_0000_0000);
        }
    }

    for a in 0..namp {
        global.nreads[a] += local.nreads[a];
        global.nreads2[a] += local.nreads[a] * local.nreads[a];
        global.nfull_reads[a] += local.nfull_reads[a];

        let rperc = if all_nseq > 0 {
            100.0 * local.nreads[a] as f64 / all_nseq as f64
        } else {
            0.0
        };
        global.nrperc[a] += rperc;
        global.nrperc2[a] += rperc * rperc;

        global.nbases[a] += local.nbases[a];
        global.nbases2[a] += local.nbases[a] * local.nbases[a];

        for d in 0..MAX_DEPTH_LEVELS {
            global.covered_perc[a][d] += local.covered_perc[a][d];
            global.covered_perc2[a][d] += local.covered_perc[a][d] * local.covered_perc[a][d];
        }

        for d in 0..3 {
            global.amp_dist[a][d] += local.amp_dist[a][d];
        }
    }

    for i in 0..local.ref_len as usize {
        if i < global.depth_all.len() {
            global.depth_all[i] += local.depth_all[i];
        }
        if i < global.depth_valid.len() {
            global.depth_valid[i] += local.depth_valid[i];
        }
    }
}

fn sample_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Main entry point.
pub fn run(
    args: &AmpStatsArgs,
    bed_path: &Path,
    bam_paths: &[&Path],
    argv_str: &str,
    out: &mut impl Write,
) -> Result<()> {
    let primer_bed = bed::load(bed_path)?;

    if bam_paths.is_empty() {
        return Err(RsomicsError::InvalidInput("no input BAM files".to_string()));
    }

    // Open first BAM to read header and get reference lengths/names.
    let mut first_reader = rsomics_bamio::open_with_workers(bam_paths[0], args.workers)?;
    let header = first_reader.read_header().map_err(|e| {
        RsomicsError::InvalidInput(format!(
            "reading header from {}: {e}",
            bam_paths[0].display()
        ))
    })?;
    drop(first_reader);

    let nref_bam = header.reference_sequences().len();
    // samtools defaults to multi_ref=1 since 1.13; single_ref is the legacy option.
    let multi_ref = !args.single_ref;

    // Emit SS header.
    writeln!(out, "# Summary statistics, used for scaling the plots.")?;
    writeln!(out, "SS\tSamtools version: 1.23.1")?;
    writeln!(out, "SS\tCommand line: {argv_str}")?;
    writeln!(out, "SS\tNumber of files:\t{}", bam_paths.len())?;

    let mut refs: Vec<Option<RefData>> = Vec::with_capacity(nref_bam);
    let mut amp_offset = 0usize;

    for (name, seq) in header.reference_sequences().iter() {
        let ref_name = name.to_string();
        let ref_len = usize::from(seq.length()) as i64;

        if let Some(primer_list) = primer_bed.by_ref.get(&ref_name) {
            let namp = bed::count_amplicons(&primer_list.entries);

            if multi_ref {
                writeln!(out, "SS\tNumber of amplicons:\t{}\t{}", ref_name, namp)?;
                writeln!(out, "SS\tReference length:\t{}\t{}", ref_name, ref_len)?;
            } else {
                writeln!(out, "SS\tNumber of amplicons:\t{}", namp)?;
                writeln!(out, "SS\tReference length:\t{}", ref_len)?;
            }

            let amps_placeholder: Vec<Amplicon> = Vec::new();
            let lookup = PosLookup {
                pos2start: vec![],
                pos2end: vec![],
            };
            let local = AmpStats::new(ref_len, args.max_amp, args.max_amp_len);
            let global = AmpStats::new(ref_len, args.max_amp, args.max_amp_len);

            refs.push(Some(RefData {
                ref_name,
                ref_len,
                amps: amps_placeholder,
                lookup,
                local,
                global,
                first_amp_idx: amp_offset,
            }));
            amp_offset += namp;
        } else {
            refs.push(None);
        }
    }

    writeln!(out, "SS\tEnd of summary")?;

    // Build amplicons and emit AMPLICON lines (after SS block).
    let mut first_with_primers = true;
    for slot in refs.iter_mut().flatten() {
        let ref_arg = if multi_ref {
            Some(slot.ref_name.as_str())
        } else {
            None
        };
        let primer_list = primer_bed.by_ref.get(&slot.ref_name).unwrap();

        let do_title = first_with_primers;
        first_with_primers = false;

        let amps = bed::build_amplicons(
            &primer_list.entries,
            out,
            ref_arg,
            slot.first_amp_idx,
            do_title,
        )?;

        let lookup = PosLookup::build(&amps, slot.ref_len, args.max_delta);
        slot.amps = amps;
        slot.lookup = lookup;
    }

    // Process each BAM file.
    for bam_path in bam_paths {
        for slot in refs.iter_mut().flatten() {
            slot.local.reset();
        }

        let mut reader = rsomics_bamio::open_with_workers(bam_path, args.workers)?;
        reader.read_header().map_err(|e| {
            RsomicsError::InvalidInput(format!("reading header from {}: {e}", bam_path.display()))
        })?;

        let sample_name = sample_name_from_path(bam_path);

        let mut qend_maps: Vec<HashMap<Vec<u8>, (i64, i64)>> =
            (0..nref_bam).map(|_| HashMap::new()).collect();

        let inner = reader.get_mut();
        let mut scanner = RecordReader::new(inner);

        loop {
            let rec = match scanner.next() {
                Ok(Some(r)) => r,
                Ok(None) => break,
                Err(e) => {
                    return Err(RsomicsError::InvalidInput(format!(
                        "reading record from {}: {e}",
                        bam_path.display()
                    )));
                }
            };

            let ref_id = rec.reference_sequence_id();
            if ref_id < 0 {
                continue;
            }
            let ref_idx = ref_id as usize;
            if ref_idx >= refs.len() || refs[ref_idx].is_none() {
                continue;
            }

            let flags = rec.flags();
            let start = rec.alignment_start() as i64;
            let tlen = rec.template_length();
            let qname = rec.name().to_vec();
            let end = bam_endpos(start, rec.cigar_ops());

            let slot = refs[ref_idx].as_mut().unwrap();
            accumulate_record(
                flags,
                start,
                end,
                tlen,
                &qname,
                args,
                &slot.amps,
                &slot.lookup,
                &mut slot.local,
                slot.ref_len,
                &mut qend_maps[ref_idx],
            );
        }

        drop(scanner);
        drop(reader);

        for slot in refs.iter_mut().flatten() {
            compute_covered_perc(&mut slot.local, &slot.amps, args);
        }

        let all_nseq: i64 = refs
            .iter()
            .flatten()
            .map(|s| s.local.nseq - s.local.nfiltered - s.local.nfailprimer)
            .sum();

        output::dump_stats(
            'F',
            &sample_name,
            bam_paths.len(),
            &refs,
            args,
            true,
            multi_ref,
            out,
        )?;

        for slot in refs.iter_mut().flatten() {
            let namp = slot.amps.len();
            append_to_global(&slot.local, &mut slot.global, namp, all_nseq);
        }
    }

    output::dump_stats(
        'C',
        "COMBINED",
        bam_paths.len(),
        &refs,
        args,
        false,
        multi_ref,
        out,
    )?;

    Ok(())
}
