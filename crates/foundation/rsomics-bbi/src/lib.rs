//! Pure-Rust bigWig/BBI reader: header, chromosome B-tree, R-tree interval
//! search, bedGraph/varStep/fixedStep section decode, and zoom-level statistics.
//!
//! Two mean-signal paths are exposed:
//!
//! - [`BigWig::mean_stat_zoom`] — replicates `pyBigWig.stats(chrom, start,
//!   end)` (default `exact=False`): selects the best zoom level using the same
//!   `determineZoomLevel` heuristic as libBigWig/pyBigWig, reads the zoom
//!   R-tree, and computes `blockMean` with fractional-overlap scalars.  This
//!   is what deeptools `multiBigwigSummary` and `bigwigCompare` call.
//! - [`BigWig::mean_stat`] — exact per-base nanmean from the full-resolution
//!   R-tree (equivalent to `pyBigWig.stats(chrom, start, end, exact=True)`).
//!   Used by consumers that need bit-exact, non-approximated values.
//! - [`BigWig::values`] — per-base value array with NaN for uncovered positions
//!   (`pyBigWig.values` semantics).  Used for chromosome-wide passes.
//!
//! ## Origin
//!
//! The bigWig/BBI on-disk layout (header at offset 0, zoom-level headers,
//! chromosome B-tree, R-tree "cir" index, zlib-compressed data sections in
//! bedGraph/varStep/fixedStep flavours) was read from the bigtools 0.5.6
//! source (`src/bbi/bbiread.rs`, `src/bbi/bigwigread.rs`, MIT, Jack Huey)
//! and Jim Kent's published BBI format spec.
//!
//! The zoom-level selection and `blockMean` algorithm were read from
//! libBigWig (MIT, Devon Ryan): `bwStats.c` — `determineZoomLevel`,
//! `bwStatsFromZoom`, `getVals`, `getScalar`, `blockMean`.  The zoom-record
//! on-disk layout (32 bytes: chromId u32, start u32, end u32, nBases u32,
//! min f32, max f32, sum f32, sumSquares f32) was verified against the BBI
//! format spec and against the pyBigWig output on the perf fixture.
//!
//! We carry our own reader rather than depend on bigtools because bigtools
//! pins `libdeflater = "0.13"` whose `libdeflate-sys` (C FFI,
//! `links = "libdeflate"`) collides with the workspace's `rsomics-fqgz`
//! (`libdeflater = "1"`); two crates may not link the same native library.
//! Block inflation here uses the workspace `flate2` zlib-rs backend (pure
//! Rust), keeping this Quadrant ①.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

const BIGWIG_MAGIC: u32 = 0x888F_FC26;
const CHROM_TREE_MAGIC: u32 = 0x78CA_8C91;
const CIR_TREE_MAGIC: u32 = 0x2468_ACE0;

/// Bytes in a zoom record: chromId(4) + start(4) + end(4) + nBases(4) +
/// min(4) + max(4) + sum(4) + sumSquares(4).
const ZOOM_RECORD_BYTES: usize = 32;

/// One decoded zoom record for a chromosome, used by the batched bins path.
///
/// Zoom records are sorted by `start` within a chromosome (guaranteed by the
/// bigWig format's CIR-tree structure and the sequential data layout).
#[derive(Clone, Copy)]
pub struct ZoomItem {
    pub start: u32,
    pub end: u32,
    pub n_bases: u32,
    pub sum: f32,
}

/// One decoded full-resolution record for a chromosome, used by the batched
/// exact-mean bins path.  Each entry covers `[start, end)` at constant `value`.
#[derive(Clone, Copy)]
pub struct FullItem {
    pub start: u32,
    pub end: u32,
    pub value: f32,
}

/// Exact nanmean of `[q_start, q_end)` against a sorted slice of `FullItem`s,
/// matching `accumulate_mean` semantics: bases absent from the bigWig are
/// excluded from the denominator.
///
/// `items` must be sorted ascending by `start`.  `scan_from` is the index of
/// the first item that could overlap this query; returns the updated scan
/// position for the next (larger) query start.  Forward-sweep O(1) amortised.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn nanmean_from_full_items(
    items: &[FullItem],
    q_start: u32,
    q_end: u32,
    scan_from: usize,
) -> (f64, usize) {
    let mut sum = 0.0f64;
    let mut covered = 0u64;
    let mut i = scan_from;

    while i < items.len() && items[i].end <= q_start {
        i += 1;
    }
    let next_from = i;

    while i < items.len() && items[i].start < q_end {
        let it = &items[i];
        let s = it.start.max(q_start);
        let e = it.end.min(q_end);
        if e > s {
            let n = u64::from(e - s);
            sum += f64::from(it.value) * n as f64;
            covered += n;
        }
        i += 1;
    }

    let mean = if covered == 0 {
        f64::NAN
    } else {
        sum / covered as f64
    };
    (mean, next_from)
}

/// Compute `blockMean` of `[q_start, q_end)` against a sorted slice of
/// `ZoomItem`s, using the same fractional-overlap scalar and FMA ordering as
/// [`BigWig::mean_stat_zoom`].
///
/// `items` must be sorted ascending by `start` (guaranteed by
/// [`BigWig::zoom_items_for_chrom`]).  The caller passes a `scan_from` hint —
/// the index of the first item that could overlap this query — and this
/// function returns the index of the first item whose `start >= q_end` so the
/// next call can pick up where this one left off (forward sweep, O(1) amortised
/// per bin).
///
/// Returns `(mean, next_scan_from)`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn mean_from_zoom_items(
    items: &[ZoomItem],
    q_start: u32,
    q_end: u32,
    scan_from: usize,
) -> (f64, usize) {
    let mut total_sum = 0.0f64;
    let mut total_cov = 0.0f64;
    let mut i = scan_from;

    // Skip items that end at or before q_start (they cannot overlap).
    while i < items.len() && items[i].end <= q_start {
        i += 1;
    }
    let next_from = i;

    while i < items.len() && items[i].start < q_end {
        let it = &items[i];
        let scalar = get_scalar(q_start, q_end, it.start, it.end);
        total_sum = f64::from(it.sum).mul_add(scalar, total_sum);
        total_cov = f64::from(it.n_bases).mul_add(scalar, total_cov);
        i += 1;
    }

    let mean = if total_cov == 0.0 {
        f64::NAN
    } else {
        total_sum / total_cov
    };
    (mean, next_from)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Endian {
    Little,
    Big,
}

/// One chromosome's id (its index in the bigWig).
#[derive(Clone, Copy)]
struct Chrom {
    id: u32,
}

/// A data block to read: file offset and compressed size.
#[derive(Clone, Copy)]
struct Block {
    offset: u64,
    size: u64,
}

/// One zoom level's metadata from the BBI header.
#[derive(Clone, Copy)]
struct ZoomLevel {
    /// Reduction factor (bases per zoom record, approximately).
    reduction: u32,
    /// File offset of this level's CIR-tree index.
    index_offset: u64,
}

/// An open bigWig file, with its header, chromosome table, and index location.
pub struct BigWig {
    reader: BufReader<File>,
    endian: Endian,
    uncompress_buf_size: u32,
    full_index_offset: u64,
    chroms: HashMap<String, Chrom>,
    chrom_lengths: HashMap<String, u32>,
    /// `(name, length)` in the file's B-tree leaf order. pyBigWig's
    /// `chroms()` dict iterates in this order, and deeptools sorts its
    /// output by it, so a faithful port must preserve it.
    chrom_order: Vec<(String, u32)>,
    /// Zoom levels in header order (ascending reduction factor).
    zoom_levels: Vec<ZoomLevel>,
}

/// Byte-reading helpers that respect the file's endianness.
struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
    endian: Endian,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8], endian: Endian) -> Self {
        Self {
            buf,
            pos: 0,
            endian,
        }
    }
    fn u8(&mut self) -> u8 {
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }
    fn u16(&mut self) -> u16 {
        let b: [u8; 2] = self.buf[self.pos..self.pos + 2].try_into().unwrap();
        self.pos += 2;
        match self.endian {
            Endian::Little => u16::from_le_bytes(b),
            Endian::Big => u16::from_be_bytes(b),
        }
    }
    fn u32(&mut self) -> u32 {
        let b: [u8; 4] = self.buf[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        match self.endian {
            Endian::Little => u32::from_le_bytes(b),
            Endian::Big => u32::from_be_bytes(b),
        }
    }
    fn u64(&mut self) -> u64 {
        let b: [u8; 8] = self.buf[self.pos..self.pos + 8].try_into().unwrap();
        self.pos += 8;
        match self.endian {
            Endian::Little => u64::from_le_bytes(b),
            Endian::Big => u64::from_be_bytes(b),
        }
    }
    fn f32(&mut self) -> f32 {
        let b: [u8; 4] = self.buf[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        match self.endian {
            Endian::Little => f32::from_le_bytes(b),
            Endian::Big => f32::from_be_bytes(b),
        }
    }
    fn skip(&mut self, n: usize) {
        self.pos += n;
    }
}

fn io<T, E: std::fmt::Display>(r: std::result::Result<T, E>, ctx: &str) -> Result<T> {
    r.map_err(|e| RsomicsError::InvalidInput(format!("{ctx}: {e}")))
}

/// The chromosome table parsed from the B-tree: id lookup, length lookup, and
/// the `(name, length)` list in on-disk leaf order.
type ChromTable = (
    HashMap<String, Chrom>,
    HashMap<String, u32>,
    Vec<(String, u32)>,
);

impl BigWig {
    pub fn open(path: &Path) -> Result<Self> {
        let file = io(File::open(path), "cannot open bigWig")?;
        let mut reader = BufReader::new(file);

        let mut header = [0u8; 64];
        io(reader.read_exact(&mut header), "reading bigWig header")?;
        let magic_le = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let endian = if magic_le == BIGWIG_MAGIC {
            Endian::Little
        } else if u32::from_be_bytes([header[0], header[1], header[2], header[3]]) == BIGWIG_MAGIC {
            Endian::Big
        } else {
            return Err(RsomicsError::InvalidInput(
                "not a bigWig file (bad magic)".into(),
            ));
        };

        let mut c = Cursor::new(&header, endian);
        c.skip(4); // magic
        let _version = c.u16();
        let n_levels = c.u16();
        let chromosome_tree_offset = c.u64();
        let _full_data_offset = c.u64();
        let full_index_offset = c.u64();
        let _field_count = c.u16();
        let _defined_field_count = c.u16();
        let _auto_sql_offset = c.u64();
        let _total_summary_offset = c.u64();
        let uncompress_buf_size = c.u32();
        // _extension_offset (8 bytes) takes the header to 64 bytes.

        // Zoom-level headers are at bytes [64 .. 64 + n_levels*24).
        // Each entry: reduction(u32), padding(u32), dataOffset(u64), indexOffset(u64).
        let zoom_levels = Self::read_zoom_headers(&mut reader, endian, n_levels)?;

        let (chroms, chrom_lengths, chrom_order) =
            Self::read_chrom_tree(&mut reader, endian, chromosome_tree_offset)?;

        Ok(Self {
            reader,
            endian,
            uncompress_buf_size,
            full_index_offset,
            chroms,
            chrom_lengths,
            chrom_order,
            zoom_levels,
        })
    }

    /// Length of a chromosome, if present in this bigWig.
    #[must_use]
    pub fn chrom_len(&self, name: &str) -> Option<u32> {
        self.chrom_lengths.get(name).copied()
    }

    /// Every `(chromosome name, length)` declared in the file, in B-tree leaf
    /// order (the order pyBigWig's `chroms()` dict yields).
    pub fn chroms(&self) -> impl Iterator<Item = (&str, u32)> {
        self.chrom_order.iter().map(|(k, v)| (k.as_str(), *v))
    }

    // ── zoom helpers ─────────────────────────────────────────────────────────

    fn read_zoom_headers(
        reader: &mut BufReader<File>,
        endian: Endian,
        n_levels: u16,
    ) -> Result<Vec<ZoomLevel>> {
        // Zoom headers start at file offset 64 (immediately after the 64-byte
        // main header).  Each entry is 24 bytes.
        let mut levels = Vec::with_capacity(n_levels as usize);
        let bytes_needed = n_levels as usize * 24;
        let mut buf = vec![0u8; bytes_needed];
        io(reader.read_exact(&mut buf), "reading zoom headers")?;
        let mut c = Cursor::new(&buf, endian);
        for _ in 0..n_levels {
            let reduction = c.u32();
            c.skip(4); // padding
            let _data_offset = c.u64();
            let index_offset = c.u64();
            levels.push(ZoomLevel {
                reduction,
                index_offset,
            });
        }
        Ok(levels)
    }

    /// libBigWig `determineZoomLevel`: divide `bases_per_bin` by 2, then find
    /// the zoom level whose reduction factor is <= that value with the smallest
    /// difference.  Returns `None` when no level qualifies (fall back to full
    /// resolution).
    fn determine_zoom_level(&self, bases_per_bin: u32) -> Option<usize> {
        let target = bases_per_bin / 2;
        let mut best: Option<(u32, usize)> = None; // (diff, index)
        for (i, z) in self.zoom_levels.iter().enumerate() {
            if z.reduction <= target {
                let diff = target - z.reduction;
                if best.is_none_or(|(bd, _)| diff < bd) {
                    best = Some((diff, i));
                }
            }
        }
        best.map(|(_, i)| i)
    }

    // ── public API ────────────────────────────────────────────────────────────

    /// Zoom-approximate mean for `[start, end)` on `chrom`, matching pyBigWig
    /// `bw.stats(chrom, start, end)` with `exact=False` (the default).
    ///
    /// Selects the best zoom level using libBigWig's `determineZoomLevel`
    /// heuristic, reads the zoom CIR-tree, and computes `blockMean` with
    /// fractional-overlap scalars exactly as libBigWig `bwStatsFromZoom`.
    /// Falls back to the full-resolution exact path when no zoom level
    /// qualifies (same as libBigWig `bwStatsFromFull`).
    ///
    /// Returns `Some(NaN)` when the chromosome is present but has no data in
    /// `[start, end)`.  Returns `None` when the chromosome is absent.
    pub fn mean_stat_zoom(&mut self, chrom: &str, start: u32, end: u32) -> Result<Option<f64>> {
        let Some(chrom_info) = self.chroms.get(chrom).copied() else {
            return Ok(None);
        };

        let bases_per_bin = end.saturating_sub(start);
        if let Some(level_idx) = self.determine_zoom_level(bases_per_bin) {
            let idx_off = self.zoom_levels[level_idx].index_offset;
            let blocks = self.search_zoom_index(idx_off, chrom_info.id, start, end)?;
            let mean = self.zoom_block_mean(&blocks, chrom_info.id, start, end)?;
            Ok(Some(mean))
        } else {
            // No applicable zoom level: fall back to full-resolution exact path.
            let blocks = self.search_index(chrom_info.id, start, end)?;
            let mut sum = 0.0f64;
            let mut covered: u64 = 0;
            for block in blocks {
                let data = self.read_block(&block)?;
                self.accumulate_mean(&data, chrom_info.id, start, end, &mut sum, &mut covered)?;
            }
            if covered == 0 {
                Ok(Some(f64::NAN))
            } else {
                #[allow(clippy::cast_precision_loss)]
                Ok(Some(sum / covered as f64))
            }
        }
    }

    /// Batch-load every zoom record for `chrom` at the zoom level selected by
    /// `bases_per_bin`, collecting them into a `Vec<ZoomItem>` sorted by
    /// `start`.  Returns `(zoom_level_index, items)`.
    ///
    /// Returns `None` when:
    /// - the chromosome is absent, OR
    /// - no zoom level qualifies (caller should fall back to `mean_stat_zoom`
    ///   per-bin, which itself falls back to the full-resolution exact path).
    ///
    /// This is the fast path for bins mode: for a fixed `bin_size` every bin
    /// on the same chromosome selects the same zoom level, so the CIR-tree
    /// walk (the dominant I/O cost) happens once per chromosome instead of
    /// once per bin.  The returned `Vec` is then swept forward sequentially
    /// by [`mean_from_zoom_items`] — O(items + bins) total instead of
    /// O(bins × log(items)).
    pub fn zoom_items_for_chrom(
        &mut self,
        chrom: &str,
        bases_per_bin: u32,
    ) -> Result<Option<(usize, Vec<ZoomItem>)>> {
        let Some(chrom_info) = self.chroms.get(chrom).copied() else {
            return Ok(None);
        };
        let Some(level_idx) = self.determine_zoom_level(bases_per_bin) else {
            return Ok(None);
        };
        let chrom_len = self.chrom_lengths.get(chrom).copied().unwrap_or(0);
        let idx_off = self.zoom_levels[level_idx].index_offset;
        // Fetch all blocks covering the entire chromosome [0, chrom_len).
        let blocks = self.search_zoom_index(idx_off, chrom_info.id, 0, chrom_len)?;
        let items = self.decode_zoom_blocks_for_chrom(&blocks, chrom_info.id)?;
        Ok(Some((level_idx, items)))
    }

    /// Decode all zoom records from `blocks` that belong to `chrom_id`,
    /// returning them sorted by `start`.
    fn decode_zoom_blocks_for_chrom(
        &mut self,
        blocks: &[Block],
        chrom_id: u32,
    ) -> Result<Vec<ZoomItem>> {
        let mut items: Vec<ZoomItem> = Vec::new();
        for block in blocks {
            let data = self.read_block(block)?;
            let n_records = data.len() / ZOOM_RECORD_BYTES;
            for rec_idx in 0..n_records {
                let base = rec_idx * ZOOM_RECORD_BYTES;
                let mut c = Cursor::new(&data[base..base + ZOOM_RECORD_BYTES], self.endian);
                let rec_chrom = c.u32();
                let rec_start = c.u32();
                let rec_end = c.u32();
                let n_bases = c.u32();
                c.skip(8); // min(f32) + max(f32)
                let rec_sum = c.f32();
                if rec_chrom != chrom_id {
                    continue;
                }
                items.push(ZoomItem {
                    start: rec_start,
                    end: rec_end,
                    n_bases,
                    sum: rec_sum,
                });
            }
        }
        items.sort_unstable_by_key(|it| it.start);
        Ok(items)
    }

    /// Batch-load every full-resolution data record for `chrom` into a sorted
    /// `Vec<FullItem>`, for use with [`nanmean_from_full_items`].
    ///
    /// This is the full-resolution counterpart of [`BigWig::zoom_items_for_chrom`]:
    /// it walks the full CIR-tree once for the whole chromosome, inflates every
    /// data block, and decodes all bedGraph/varStep/fixedStep records.  The
    /// resulting `Vec` is swept forward by bins in O(items + bins) — the same
    /// asymptotic gain as the zoom batched path, but for the full-resolution case
    /// (invoked when `bin_size` is too small for any zoom level).
    ///
    /// Returns `None` when the chromosome is absent.
    pub fn full_items_for_chrom(&mut self, chrom: &str) -> Result<Option<Vec<FullItem>>> {
        let Some(chrom_info) = self.chroms.get(chrom).copied() else {
            return Ok(None);
        };
        let chrom_len = self.chrom_lengths.get(chrom).copied().unwrap_or(0);
        let blocks = self.search_index(chrom_info.id, 0, chrom_len)?;
        let mut items: Vec<FullItem> = Vec::new();
        for block in blocks {
            let data = self.read_block(&block)?;
            self.decode_full_block_into(&data, chrom_info.id, &mut items)?;
        }
        items.sort_unstable_by_key(|it| it.start);
        Ok(Some(items))
    }

    /// Decode one full-resolution data block and append its records to `out`.
    fn decode_full_block_into(
        &self,
        data: &[u8],
        query_chrom: u32,
        out: &mut Vec<FullItem>,
    ) -> Result<()> {
        let mut c = Cursor::new(data, self.endian);
        let chrom_id = c.u32();
        let chrom_start = c.u32();
        let _chrom_end = c.u32();
        let item_step = c.u32();
        let item_span = c.u32();
        let section_type = c.u8();
        c.skip(1);
        let item_count = c.u16();

        if chrom_id != query_chrom {
            return Ok(());
        }

        match section_type {
            1 => {
                for _ in 0..item_count {
                    let s = c.u32();
                    let e = c.u32();
                    let v = c.f32();
                    out.push(FullItem {
                        start: s,
                        end: e,
                        value: v,
                    });
                }
            }
            2 => {
                for _ in 0..item_count {
                    let s = c.u32();
                    let v = c.f32();
                    out.push(FullItem {
                        start: s,
                        end: s + item_span,
                        value: v,
                    });
                }
            }
            3 => {
                let mut curr = chrom_start;
                for _ in 0..item_count {
                    let v = c.f32();
                    out.push(FullItem {
                        start: curr,
                        end: curr + item_span,
                        value: v,
                    });
                    curr += item_step;
                }
            }
            other => {
                return Err(RsomicsError::InvalidInput(format!(
                    "unknown bigWig section type {other}"
                )));
            }
        }
        Ok(())
    }

    /// Per-base values for `[start, end)` on `chrom`. Positions with no data in
    /// the bigWig are `NAN`. If the chromosome is absent, returns `None`.
    /// Mean signal over `[start, end)` on `chrom`, using nanmean semantics:
    /// positions absent from the bigWig are excluded from the denominator.
    /// Returns `NaN` when the chromosome is absent or no data covers `[start,
    /// end)`. Returns `None` when the chromosome is not in this bigWig.
    ///
    /// This is the equivalent of `pyBigWig.stats(chrom, start, end,
    /// exact=True)`: exact per-base nanmean without zoom approximation.
    // u64 covered bases → f64: precision loss only beyond 2^53 bases (~9 Pbp);
    // no real genome reaches that scale.
    #[allow(clippy::cast_precision_loss)]
    pub fn mean_stat(&mut self, chrom: &str, start: u32, end: u32) -> Result<Option<f64>> {
        let Some(chrom_info) = self.chroms.get(chrom).copied() else {
            return Ok(None);
        };
        let blocks = self.search_index(chrom_info.id, start, end)?;
        let mut sum = 0.0f64;
        let mut covered: u64 = 0;
        for block in blocks {
            let data = self.read_block(&block)?;
            self.accumulate_mean(&data, chrom_info.id, start, end, &mut sum, &mut covered)?;
        }
        if covered == 0 {
            Ok(Some(f64::NAN))
        } else {
            Ok(Some(sum / covered as f64))
        }
    }

    pub fn values(&mut self, chrom: &str, start: u32, end: u32) -> Result<Option<Vec<f32>>> {
        let Some(chrom_info) = self.chroms.get(chrom).copied() else {
            return Ok(None);
        };
        let mut out = vec![f32::NAN; (end - start) as usize];
        let blocks = self.search_index(chrom_info.id, start, end)?;
        for block in blocks {
            let data = self.read_block(&block)?;
            self.apply_block(&data, chrom_info.id, start, end, &mut out)?;
        }
        Ok(Some(out))
    }

    // ── zoom R-tree search ────────────────────────────────────────────────────

    /// Walk the zoom CIR-tree rooted at `index_offset`, collecting data blocks
    /// that overlap `[start, end)` on `chrom_id`.
    fn search_zoom_index(
        &mut self,
        index_offset: u64,
        chrom_id: u32,
        start: u32,
        end: u32,
    ) -> Result<Vec<Block>> {
        io(
            self.reader.seek(SeekFrom::Start(index_offset)),
            "seek zoom index header",
        )?;
        let mut ih = [0u8; 4];
        io(self.reader.read_exact(&mut ih), "reading zoom index magic")?;
        let magic = match self.endian {
            Endian::Little => u32::from_le_bytes(ih),
            Endian::Big => u32::from_be_bytes(ih),
        };
        if magic != CIR_TREE_MAGIC {
            return Err(RsomicsError::InvalidInput("bad zoom cir-tree magic".into()));
        }
        // 48-byte CIR-tree header; root node follows.
        let root = index_offset + 48;
        let mut blocks = Vec::new();
        let mut stack = vec![root];
        while let Some(node_offset) = stack.pop() {
            io(
                self.reader.seek(SeekFrom::Start(node_offset)),
                "seek zoom index node",
            )?;
            let mut node = [0u8; 4];
            io(
                self.reader.read_exact(&mut node),
                "reading zoom index node header",
            )?;
            let isleaf = node[0];
            let count = match self.endian {
                Endian::Little => u16::from_le_bytes([node[2], node[3]]),
                Endian::Big => u16::from_be_bytes([node[2], node[3]]),
            };
            let item_size = if isleaf == 1 { 32 } else { 24 };
            let mut bytes = vec![0u8; item_size * count as usize];
            io(
                self.reader.read_exact(&mut bytes),
                "reading zoom index items",
            )?;
            for i in 0..count as usize {
                let mut c = Cursor::new(&bytes[i * item_size..(i + 1) * item_size], self.endian);
                let start_chrom = c.u32();
                let start_base = c.u32();
                let end_chrom = c.u32();
                let end_base = c.u32();
                if !overlaps(
                    chrom_id,
                    start,
                    end,
                    start_chrom,
                    start_base,
                    end_chrom,
                    end_base,
                ) {
                    continue;
                }
                if isleaf == 1 {
                    let data_offset = c.u64();
                    let data_size = c.u64();
                    blocks.push(Block {
                        offset: data_offset,
                        size: data_size,
                    });
                } else {
                    stack.push(c.u64());
                }
            }
        }
        Ok(blocks)
    }

    // ── zoom blockMean ────────────────────────────────────────────────────────

    /// libBigWig `blockMean` over zoom data blocks.
    ///
    /// Each zoom record is 32 bytes: chromId(u32), start(u32), end(u32),
    /// nBases(u32), min(f32), max(f32), sum(f32), sumSquares(f32).
    ///
    /// The mean is `Σ(sum × scalar) / Σ(nBases × scalar)` where `scalar` is
    /// the fractional overlap of the zoom block with `[start, end)` — i.e.
    /// libBigWig `getScalar(start, end, block_start, block_end)`.
    #[allow(clippy::cast_precision_loss)]
    fn zoom_block_mean(
        &mut self,
        blocks: &[Block],
        chrom_id: u32,
        start: u32,
        end: u32,
    ) -> Result<f64> {
        if blocks.is_empty() {
            return Ok(f64::NAN);
        }
        let mut total_sum = 0.0f64;
        let mut total_coverage = 0.0f64;

        for block in blocks {
            let data = self.read_block(block)?;
            let n_records = data.len() / ZOOM_RECORD_BYTES;
            for rec_idx in 0..n_records {
                let base = rec_idx * ZOOM_RECORD_BYTES;
                let mut c = Cursor::new(&data[base..base + ZOOM_RECORD_BYTES], self.endian);
                let rec_chrom = c.u32();
                let rec_start = c.u32();
                let rec_end = c.u32();
                let n_bases = c.u32();
                c.skip(8); // min(f32) + max(f32)
                let rec_sum = c.f32();
                // sumSquares (f32) not needed for mean.

                if rec_chrom != chrom_id {
                    if rec_chrom > chrom_id {
                        break;
                    }
                    continue;
                }

                // Overlap check: record must intersect [start, end).
                if rec_end <= start || rec_start >= end {
                    if rec_start >= end {
                        break;
                    }
                    continue;
                }

                let scalar = get_scalar(start, end, rec_start, rec_end);
                // FMA (fused multiply-add) matches libBigWig's C compiler output:
                // `output += sum * scalar` is optimised to fma(sum, scalar, output)
                // with -O2 on x86/ARM, giving a single-rounded result.
                total_sum = f64::from(rec_sum).mul_add(scalar, total_sum);
                total_coverage = f64::from(n_bases).mul_add(scalar, total_coverage);
            }
        }

        if total_coverage == 0.0 {
            Ok(f64::NAN)
        } else {
            Ok(total_sum / total_coverage)
        }
    }

    // ── full-resolution R-tree search ─────────────────────────────────────────

    /// Walk the R-tree, collecting data blocks overlapping the query interval.
    fn search_index(&mut self, chrom_id: u32, start: u32, end: u32) -> Result<Vec<Block>> {
        // The 48-byte cir-tree header precedes the root node; verify its magic.
        io(
            self.reader.seek(SeekFrom::Start(self.full_index_offset)),
            "seek index header",
        )?;
        let mut ih = [0u8; 4];
        io(self.reader.read_exact(&mut ih), "reading index magic")?;
        let magic = match self.endian {
            Endian::Little => u32::from_le_bytes(ih),
            Endian::Big => u32::from_be_bytes(ih),
        };
        if magic != CIR_TREE_MAGIC {
            return Err(RsomicsError::InvalidInput("bad cir-tree magic".into()));
        }
        let root = self.full_index_offset + 48;
        let mut blocks = Vec::new();
        let mut stack = vec![root];
        while let Some(node_offset) = stack.pop() {
            io(
                self.reader.seek(SeekFrom::Start(node_offset)),
                "seek index node",
            )?;
            let mut node = [0u8; 4];
            io(
                self.reader.read_exact(&mut node),
                "reading index node header",
            )?;
            let isleaf = node[0];
            let count = match self.endian {
                Endian::Little => u16::from_le_bytes([node[2], node[3]]),
                Endian::Big => u16::from_be_bytes([node[2], node[3]]),
            };
            let item_size = if isleaf == 1 { 32 } else { 24 };
            let mut bytes = vec![0u8; item_size * count as usize];
            io(self.reader.read_exact(&mut bytes), "reading index items")?;
            for i in 0..count as usize {
                let mut c = Cursor::new(&bytes[i * item_size..(i + 1) * item_size], self.endian);
                let start_chrom = c.u32();
                let start_base = c.u32();
                let end_chrom = c.u32();
                let end_base = c.u32();
                if !overlaps(
                    chrom_id,
                    start,
                    end,
                    start_chrom,
                    start_base,
                    end_chrom,
                    end_base,
                ) {
                    continue;
                }
                if isleaf == 1 {
                    let data_offset = c.u64();
                    let data_size = c.u64();
                    blocks.push(Block {
                        offset: data_offset,
                        size: data_size,
                    });
                } else {
                    stack.push(c.u64());
                }
            }
        }
        Ok(blocks)
    }

    fn read_block(&mut self, block: &Block) -> Result<Vec<u8>> {
        io(
            self.reader.seek(SeekFrom::Start(block.offset)),
            "seek block",
        )?;
        let size = usize::try_from(block.size)
            .map_err(|_| RsomicsError::InvalidInput("bigWig block size exceeds usize".into()))?;
        let mut raw = vec![0u8; size];
        io(self.reader.read_exact(&mut raw), "reading block")?;
        if self.uncompress_buf_size > 0 {
            let mut dec = flate2::read::ZlibDecoder::new(&raw[..]);
            let mut out = Vec::with_capacity(self.uncompress_buf_size as usize);
            io(dec.read_to_end(&mut out), "inflating block")?;
            Ok(out)
        } else {
            Ok(raw)
        }
    }

    /// Parse one data section and scatter its values into `out`. Sections come
    /// in bedGraph (1), varStep (2) and fixedStep (3) flavours.
    fn apply_block(
        &self,
        data: &[u8],
        query_chrom: u32,
        start: u32,
        end: u32,
        out: &mut [f32],
    ) -> Result<()> {
        let mut c = Cursor::new(data, self.endian);
        let chrom_id = c.u32();
        let chrom_start = c.u32();
        let _chrom_end = c.u32();
        let item_step = c.u32();
        let item_span = c.u32();
        let section_type = c.u8();
        c.skip(1); // reserved
        let item_count = c.u16();

        if chrom_id != query_chrom {
            return Ok(());
        }

        let mut scatter = |vstart: u32, vend: u32, value: f32| {
            if vend > start && vstart < end {
                let s = vstart.max(start);
                let e = vend.min(end);
                for pos in s..e {
                    out[(pos - start) as usize] = value;
                }
            }
        };

        match section_type {
            1 => {
                for _ in 0..item_count {
                    let s = c.u32();
                    let e = c.u32();
                    let v = c.f32();
                    scatter(s, e, v);
                }
            }
            2 => {
                for _ in 0..item_count {
                    let s = c.u32();
                    let v = c.f32();
                    scatter(s, s + item_span, v);
                }
            }
            3 => {
                let mut curr = chrom_start;
                for _ in 0..item_count {
                    let v = c.f32();
                    scatter(curr, curr + item_span, v);
                    curr += item_step;
                }
            }
            other => {
                return Err(RsomicsError::InvalidInput(format!(
                    "unknown bigWig section type {other}"
                )));
            }
        }
        Ok(())
    }

    /// Accumulate `sum += value * overlap_bases` and `covered += overlap_bases`
    /// for every data entry in one block that overlaps `[start, end)`. This is
    /// the block-level analogue of `apply_block` but avoids per-base scatter,
    /// enabling `mean_stat` to compute a nanmean without allocating a per-base
    /// array.
    // n is u64 (≤ u32::MAX); n as f64 is exact for all u64 values ≤ 2^53.
    #[allow(clippy::cast_precision_loss)]
    fn accumulate_mean(
        &self,
        data: &[u8],
        query_chrom: u32,
        start: u32,
        end: u32,
        sum: &mut f64,
        covered: &mut u64,
    ) -> Result<()> {
        let mut c = Cursor::new(data, self.endian);
        let chrom_id = c.u32();
        let chrom_start = c.u32();
        let _chrom_end = c.u32();
        let item_step = c.u32();
        let item_span = c.u32();
        let section_type = c.u8();
        c.skip(1);
        let item_count = c.u16();

        if chrom_id != query_chrom {
            return Ok(());
        }

        let mut add = |vstart: u32, vend: u32, value: f32| {
            if vend > start && vstart < end {
                let s = vstart.max(start);
                let e = vend.min(end);
                let n = u64::from(e - s);
                *sum += f64::from(value) * n as f64;
                *covered += n;
            }
        };

        match section_type {
            1 => {
                for _ in 0..item_count {
                    let s = c.u32();
                    let e = c.u32();
                    let v = c.f32();
                    add(s, e, v);
                }
            }
            2 => {
                for _ in 0..item_count {
                    let s = c.u32();
                    let v = c.f32();
                    add(s, s + item_span, v);
                }
            }
            3 => {
                let mut curr = chrom_start;
                for _ in 0..item_count {
                    let v = c.f32();
                    add(curr, curr + item_span, v);
                    curr += item_step;
                }
            }
            other => {
                return Err(RsomicsError::InvalidInput(format!(
                    "unknown bigWig section type {other}"
                )));
            }
        }
        Ok(())
    }

    fn read_chrom_tree(
        reader: &mut BufReader<File>,
        endian: Endian,
        offset: u64,
    ) -> Result<ChromTable> {
        io(reader.seek(SeekFrom::Start(offset)), "seek chrom tree")?;
        let mut hdr = [0u8; 32];
        io(reader.read_exact(&mut hdr), "reading chrom tree header")?;
        let mut c = Cursor::new(&hdr, endian);
        if c.u32() != CHROM_TREE_MAGIC {
            return Err(RsomicsError::InvalidInput("bad chrom tree magic".into()));
        }
        c.skip(4); // block size
        let key_size = c.u32();
        let _val_size = c.u32();
        let _item_count = c.u64();

        let mut chroms = HashMap::new();
        let mut lengths = HashMap::new();
        let mut order = Vec::new();
        Self::read_chrom_block(
            reader,
            endian,
            key_size,
            &mut chroms,
            &mut lengths,
            &mut order,
        )?;
        Ok((chroms, lengths, order))
    }

    fn read_chrom_block(
        reader: &mut BufReader<File>,
        endian: Endian,
        key_size: u32,
        chroms: &mut HashMap<String, Chrom>,
        lengths: &mut HashMap<String, u32>,
        order: &mut Vec<(String, u32)>,
    ) -> Result<()> {
        let mut node = [0u8; 4];
        io(reader.read_exact(&mut node), "reading chrom node header")?;
        let isleaf = node[0];
        let count = match endian {
            Endian::Little => u16::from_le_bytes([node[2], node[3]]),
            Endian::Big => u16::from_be_bytes([node[2], node[3]]),
        };
        let item_size = key_size as usize + 8;
        let mut bytes = vec![0u8; item_size * count as usize];
        io(reader.read_exact(&mut bytes), "reading chrom node items")?;

        if isleaf == 1 {
            for i in 0..count as usize {
                let base = i * item_size;
                let key = &bytes[base..base + key_size as usize];
                let name = std::str::from_utf8(key)
                    .map_err(|_| RsomicsError::InvalidInput("bad chrom name utf-8".into()))?
                    .trim_matches('\0')
                    .to_string();
                let mut c = Cursor::new(&bytes[base + key_size as usize..base + item_size], endian);
                let id = c.u32();
                let length = c.u32();
                chroms.insert(name.clone(), Chrom { id });
                lengths.insert(name.clone(), length);
                order.push((name, length));
            }
        } else {
            let mut children = Vec::with_capacity(count as usize);
            for i in 0..count as usize {
                let base = i * item_size;
                let mut c = Cursor::new(&bytes[base + key_size as usize..base + item_size], endian);
                children.push(c.u64());
            }
            for child in children {
                io(reader.seek(SeekFrom::Start(child)), "seek chrom child")?;
                Self::read_chrom_block(reader, endian, key_size, chroms, lengths, order)?;
            }
        }
        Ok(())
    }
}

/// libBigWig `getScalar`: fractional overlap of zoom block `[b_start, b_end)`
/// within query `[i_start, i_end)`.  Handles three cases: block starts before
/// or at query start, block starts within the query.
fn get_scalar(i_start: u32, i_end: u32, b_start: u32, b_end: u32) -> f64 {
    let span = f64::from(b_end - b_start);
    if b_start <= i_start {
        if b_end > i_start {
            f64::from(b_end - i_start) / span
        } else {
            0.0
        }
    } else if b_start < i_end {
        if b_end < i_end {
            // Block fully inside query: scalar = 1.0.
            f64::from(b_end - b_start) / span
        } else {
            f64::from(i_end - b_start) / span
        }
    } else {
        0.0
    }
}

/// R-tree overlap test (Kent's chrom-aware comparison): the query interval
/// `[start, end]` on `chromq` overlaps a node spanning
/// `(chromb1, b1_start) .. (chromb2, b2_end)`.
fn overlaps(
    chromq: u32,
    qstart: u32,
    qend: u32,
    chromb1: u32,
    b1_start: u32,
    chromb2: u32,
    b2_end: u32,
) -> bool {
    use std::cmp::Ordering::Greater;
    cmp_pos(chromq, qstart, chromb2, b2_end) != Greater
        && cmp_pos(chromq, qend, chromb1, b1_start) != std::cmp::Ordering::Less
}

fn cmp_pos(c1: u32, b1: u32, c2: u32, b2: u32) -> std::cmp::Ordering {
    (c1, b1).cmp(&(c2, b2))
}
