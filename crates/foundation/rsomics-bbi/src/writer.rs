//! Pure-Rust bigWig writer.
//!
//! Emits a valid little-endian bigWig (bedGraph section type 1) from a
//! sequence of `(chrom, start, end, value)` intervals.  The format follows
//! Jim Kent's BBI spec and the bigtools 0.5.6 write path (MIT, Jack Huey).
//!
//! ## File layout (written in this order)
//!
//! 1. 64-byte main header (magic, version=4, nLevels, offsets — back-patched).
//! 2. Zoom-level headers — `nLevels × 24` bytes; offsets back-patched.
//! 3. Total-summary block — 40 bytes (u64 basesCovered + 4×f64).
//! 4. Chromosome B-tree — magic + 32-byte header + leaf node.
//! 5. Full-resolution data blocks — one zlib-compressed bedGraph section
//!    per chromosome, back-to-back.
//! 6. Full-resolution CIR-tree index — 48-byte header + leaf/internal nodes.
//! 7. Zoom data blocks — one set per level (ascending reduction factor).
//! 8. Zoom CIR-tree indexes — one per level, back-patched into zoom headers.
//!
//! ## Zoom levels
//!
//! Uses the standard Kent reduction schedule: start at 128 × `bin_size` (minimum
//! 128), then multiply by 4 per level.  Up to [`MAX_ZOOM_LEVELS`] levels are
//! written; a level is skipped if its reduction factor exceeds the longest
//! chromosome length.  Each zoom record is the standard 32-byte layout
//! (chromId/start/end/nBases/min/max/sum/sumSquares, all little-endian).

use std::io::{Seek, Write};

use flate2::Compression;
use flate2::write::ZlibEncoder;

use rsomics_common::{Result, RsomicsError};

// ── constants ────────────────────────────────────────────────────────────────

const BIGWIG_MAGIC: u32 = 0x888F_FC26;
const CHROM_TREE_MAGIC: u32 = 0x78CA_8C91;
const CIR_TREE_MAGIC: u32 = 0x2468_ACE0;

const BBI_VERSION: u16 = 4;

/// Maximum number of zoom levels.
const MAX_ZOOM_LEVELS: usize = 10;

/// Minimum initial reduction factor for the first zoom level.
const MIN_INITIAL_REDUCTION: u32 = 128;

/// Items per leaf node in the chromosome B-tree.
const CHROM_BLOCK_SIZE: u32 = 256;

/// Items per leaf node in the CIR-tree (data blocks index).
const CIR_BLOCK_SIZE: u32 = 256;

const COMPRESS_LEVEL: Compression = Compression::best();

/// Maximum intervals per compressed data block (bedGraph section).
const ITEMS_PER_BLOCK: usize = 512;

// ── public API ────────────────────────────────────────────────────────────────

/// One interval supplied to the writer.
#[derive(Clone, Copy)]
pub struct Interval {
    pub chrom_id: u32,
    pub start: u32,
    pub end: u32,
    pub value: f32,
}

/// One chromosome's metadata — id (index in the B-tree) and declared length.
#[derive(Clone)]
pub struct ChromInfo {
    pub name: String,
    pub id: u32,
    pub length: u32,
}

/// Write a bigWig file to `out` from pre-sorted intervals.
///
/// `chroms` must be sorted by `id` ascending.  `intervals` must be sorted by
/// `(chrom_id, start)` ascending.  All intervals must reference valid chrom ids.
///
/// `bin_size` is used only to set the initial zoom-level reduction factor
/// (standard deeptools bamCoverage default is 50).
pub fn write_bigwig<W: Write + Seek>(
    out: &mut W,
    chroms: &[ChromInfo],
    intervals: &[Interval],
    bin_size: u32,
) -> Result<()> {
    let n_levels = compute_zoom_levels(chroms, bin_size);
    let chrom_key_size = max_chrom_name_len(chroms)?;

    let summary_offset = 64u64 + n_levels as u64 * 24;
    let chrom_tree_offset = summary_offset + 40;

    // 1. Placeholder main header.
    write_zeros(out, 64)?;

    // 2. Zoom-header placeholders (back-patched).
    let zoom_header_start = 64u64;
    let mut zoom_reductions = Vec::with_capacity(n_levels);
    {
        let mut red = initial_reduction(bin_size);
        for _ in 0..n_levels {
            zoom_reductions.push(red);
            write_zoom_header_placeholder(out, red)?;
            red = red.saturating_mul(4);
        }
    }

    // 3. Total-summary placeholder.
    write_zeros(out, 40)?;

    // 4. Chromosome B-tree.
    write_chrom_tree(out, chroms, chrom_key_size, CHROM_BLOCK_SIZE)?;

    // 5. Full-resolution data blocks.
    let full_data_offset = stream_pos(out)?;
    let data_blocks = write_full_data(out, intervals)?;

    // 6. Full-resolution CIR-tree.
    let full_index_offset = stream_pos(out)?;
    write_cir_tree(out, &data_blocks, CIR_BLOCK_SIZE)?;

    // 7. Total summary (computed, written back below).
    let summary = compute_total_summary(intervals);

    // 8. Zoom levels.
    let mut zoom_level_info: Vec<(u64, u64)> = Vec::with_capacity(n_levels);
    for &red in &zoom_reductions {
        let zoom_items = compute_zoom_items(chroms, intervals, red);
        let zoom_data_off = stream_pos(out)?;
        let zoom_data_blocks = write_zoom_data(out, &zoom_items)?;
        let zoom_index_off = stream_pos(out)?;
        write_cir_tree(out, &zoom_data_blocks, CIR_BLOCK_SIZE)?;
        zoom_level_info.push((zoom_data_off, zoom_index_off));
    }

    // Back-patch main header.
    out.seek(std::io::SeekFrom::Start(0))
        .map_err(RsomicsError::Io)?;
    write_main_header(
        out,
        u16::try_from(n_levels).unwrap_or(u16::MAX),
        chrom_tree_offset,
        full_data_offset,
        full_index_offset,
        summary_offset,
    )?;

    // Back-patch zoom headers.
    for (i, &(data_off, idx_off)) in zoom_level_info.iter().enumerate() {
        let zoom_hdr_off = zoom_header_start + i as u64 * 24;
        out.seek(std::io::SeekFrom::Start(zoom_hdr_off))
            .map_err(RsomicsError::Io)?;
        write_zoom_header(out, zoom_reductions[i], data_off, idx_off)?;
    }

    // Back-patch total summary.
    out.seek(std::io::SeekFrom::Start(summary_offset))
        .map_err(RsomicsError::Io)?;
    write_total_summary(out, &summary)?;

    // Seek to end so caller can keep writing / flushing.
    out.seek(std::io::SeekFrom::End(0))
        .map_err(RsomicsError::Io)?;
    Ok(())
}

// ── zoom planning ─────────────────────────────────────────────────────────────

fn initial_reduction(bin_size: u32) -> u32 {
    (bin_size.saturating_mul(128)).max(MIN_INITIAL_REDUCTION)
}

fn compute_zoom_levels(chroms: &[ChromInfo], bin_size: u32) -> usize {
    let max_len = chroms.iter().map(|c| c.length).max().unwrap_or(0);
    let mut n = 0;
    let mut red = initial_reduction(bin_size);
    for _ in 0..MAX_ZOOM_LEVELS {
        if red > max_len || red == 0 {
            break;
        }
        n += 1;
        red = red.saturating_mul(4);
    }
    n
}

fn max_chrom_name_len(chroms: &[ChromInfo]) -> Result<u32> {
    let max = chroms.iter().map(|c| c.name.len()).max().unwrap_or(0);
    // Round up to a multiple of 4, minimum 4.
    let padded = max.div_ceil(4) * 4;
    u32::try_from(padded.max(4))
        .map_err(|_| RsomicsError::InvalidInput("chromosome name too long".into()))
}

// ── header writers ───────────────────────────────────────────────────────────

fn write_main_header<W: Write>(
    out: &mut W,
    n_levels: u16,
    chrom_tree_offset: u64,
    full_data_offset: u64,
    full_index_offset: u64,
    total_summary_offset: u64,
) -> Result<()> {
    // uncompressedBufSize: upper bound; readers use it only to pre-allocate.
    let uncompress_buf_size: u32 = 65536;
    let mut buf = [0u8; 64];
    buf[0..4].copy_from_slice(&BIGWIG_MAGIC.to_le_bytes());
    buf[4..6].copy_from_slice(&BBI_VERSION.to_le_bytes());
    buf[6..8].copy_from_slice(&n_levels.to_le_bytes());
    buf[8..16].copy_from_slice(&chrom_tree_offset.to_le_bytes());
    buf[16..24].copy_from_slice(&full_data_offset.to_le_bytes());
    buf[24..32].copy_from_slice(&full_index_offset.to_le_bytes());
    // fieldCount, definedFieldCount: 0 for bigWig.
    buf[32..36].copy_from_slice(&0u32.to_le_bytes());
    // autoSqlOffset: 0.
    buf[36..44].copy_from_slice(&0u64.to_le_bytes());
    buf[44..52].copy_from_slice(&total_summary_offset.to_le_bytes());
    buf[52..56].copy_from_slice(&uncompress_buf_size.to_le_bytes());
    // extensionOffset: 0.
    buf[56..64].copy_from_slice(&0u64.to_le_bytes());
    out.write_all(&buf).map_err(RsomicsError::Io)
}

fn write_zoom_header_placeholder<W: Write>(out: &mut W, reduction: u32) -> Result<()> {
    let mut buf = [0u8; 24];
    buf[0..4].copy_from_slice(&reduction.to_le_bytes());
    out.write_all(&buf).map_err(RsomicsError::Io)
}

fn write_zoom_header<W: Write>(
    out: &mut W,
    reduction: u32,
    data_offset: u64,
    index_offset: u64,
) -> Result<()> {
    let mut buf = [0u8; 24];
    buf[0..4].copy_from_slice(&reduction.to_le_bytes());
    // padding: 4 bytes of zeros.
    buf[8..16].copy_from_slice(&data_offset.to_le_bytes());
    buf[16..24].copy_from_slice(&index_offset.to_le_bytes());
    out.write_all(&buf).map_err(RsomicsError::Io)
}

// ── total summary ─────────────────────────────────────────────────────────────

struct TotalSummary {
    bases_covered: u64,
    min_val: f64,
    max_val: f64,
    sum_data: f64,
    sum_squares: f64,
}

#[allow(clippy::cast_precision_loss)]
fn compute_total_summary(intervals: &[Interval]) -> TotalSummary {
    let mut bases_covered = 0u64;
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    let mut sum_data = 0.0f64;
    let mut sum_squares = 0.0f64;

    for iv in intervals {
        let n = u64::from(iv.end - iv.start);
        let v = f64::from(iv.value);
        bases_covered += n;
        if v < min_val {
            min_val = v;
        }
        if v > max_val {
            max_val = v;
        }
        sum_data += v * n as f64;
        sum_squares += v * v * n as f64;
    }

    if bases_covered == 0 {
        min_val = 0.0;
        max_val = 0.0;
    }

    TotalSummary {
        bases_covered,
        min_val,
        max_val,
        sum_data,
        sum_squares,
    }
}

fn write_total_summary<W: Write>(out: &mut W, s: &TotalSummary) -> Result<()> {
    let mut buf = [0u8; 40];
    buf[0..8].copy_from_slice(&s.bases_covered.to_le_bytes());
    buf[8..16].copy_from_slice(&s.min_val.to_le_bytes());
    buf[16..24].copy_from_slice(&s.max_val.to_le_bytes());
    buf[24..32].copy_from_slice(&s.sum_data.to_le_bytes());
    buf[32..40].copy_from_slice(&s.sum_squares.to_le_bytes());
    out.write_all(&buf).map_err(RsomicsError::Io)
}

// ── chromosome B-tree ─────────────────────────────────────────────────────────

fn write_chrom_tree<W: Write>(
    out: &mut W,
    chroms: &[ChromInfo],
    key_size: u32,
    block_size: u32,
) -> Result<()> {
    let val_size: u32 = 8; // chromId(4) + chromSize(4)
    let item_count = chroms.len() as u64;

    // 32-byte B-tree header.
    let mut hdr = [0u8; 32];
    hdr[0..4].copy_from_slice(&CHROM_TREE_MAGIC.to_le_bytes());
    hdr[4..8].copy_from_slice(&block_size.to_le_bytes());
    hdr[8..12].copy_from_slice(&key_size.to_le_bytes());
    hdr[12..16].copy_from_slice(&val_size.to_le_bytes());
    hdr[16..24].copy_from_slice(&item_count.to_le_bytes());
    // reserved: 8 bytes of zeros.
    out.write_all(&hdr).map_err(RsomicsError::Io)?;

    // Single leaf node (fits up to CHROM_BLOCK_SIZE=256 chromosomes).
    let count = u16::try_from(chroms.len())
        .map_err(|_| RsomicsError::InvalidInput("too many chroms".into()))?;
    // node header: isLeaf(1) + reserved(1) + count(2)
    let node_hdr = [1u8, 0, count.to_le_bytes()[0], count.to_le_bytes()[1]];
    out.write_all(&node_hdr).map_err(RsomicsError::Io)?;

    let item_size = key_size as usize + val_size as usize;
    for chrom in chroms {
        let mut item = vec![0u8; item_size];
        let name_bytes = chrom.name.as_bytes();
        let copy_len = name_bytes.len().min(key_size as usize);
        item[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        item[key_size as usize..key_size as usize + 4].copy_from_slice(&chrom.id.to_le_bytes());
        item[key_size as usize + 4..key_size as usize + 8]
            .copy_from_slice(&chrom.length.to_le_bytes());
        out.write_all(&item).map_err(RsomicsError::Io)?;
    }
    Ok(())
}

// ── full-resolution data blocks ───────────────────────────────────────────────

/// Metadata of one compressed data block, used to build the CIR-tree index.
#[derive(Clone, Copy)]
pub(crate) struct DataBlock {
    pub start_chrom: u32,
    pub start_base: u32,
    pub end_chrom: u32,
    pub end_base: u32,
    pub offset: u64,
    pub size: u64,
}

fn write_full_data<W: Write + Seek>(out: &mut W, intervals: &[Interval]) -> Result<Vec<DataBlock>> {
    let mut blocks = Vec::new();

    let mut i = 0;
    while i < intervals.len() {
        let current_chrom = intervals[i].chrom_id;
        // A section must not cross chromosome boundaries.
        let max_end = (i + ITEMS_PER_BLOCK).min(intervals.len());
        let mut j = i + 1;
        while j < max_end && intervals[j].chrom_id == current_chrom {
            j += 1;
        }
        let chunk = &intervals[i..j];
        let first = chunk[0];
        let last = chunk[chunk.len() - 1];

        let item_count = u16::try_from(chunk.len())
            .map_err(|_| RsomicsError::InvalidInput("block too large".into()))?;
        let uncompressed = encode_bedgraph_section(first.chrom_id, last.end, item_count, chunk);
        let compressed = zlib_compress(&uncompressed)?;

        let offset = stream_pos(out)?;
        let size = compressed.len() as u64;
        out.write_all(&compressed).map_err(RsomicsError::Io)?;

        blocks.push(DataBlock {
            start_chrom: first.chrom_id,
            start_base: first.start,
            end_chrom: last.chrom_id,
            end_base: last.end,
            offset,
            size,
        });

        i = j;
    }

    Ok(blocks)
}

fn encode_bedgraph_section(
    chrom_id: u32,
    chrom_end: u32,
    item_count: u16,
    items: &[Interval],
) -> Vec<u8> {
    let chrom_start = items[0].start;
    // Section header (24 bytes):
    // chromId(4) + chromStart(4) + chromEnd(4) + itemStep(4) + itemSpan(4)
    // + type(1) + reserved(1) + itemCount(2)
    let mut buf = Vec::with_capacity(24 + items.len() * 12);
    buf.extend_from_slice(&chrom_id.to_le_bytes());
    buf.extend_from_slice(&chrom_start.to_le_bytes());
    buf.extend_from_slice(&chrom_end.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // itemStep (unused for type 1)
    buf.extend_from_slice(&0u32.to_le_bytes()); // itemSpan (unused for type 1)
    buf.push(1u8); // type 1 = bedGraph
    buf.push(0u8); // reserved
    buf.extend_from_slice(&item_count.to_le_bytes());
    // Items: start(4) + end(4) + value(4) each.
    for iv in items {
        buf.extend_from_slice(&iv.start.to_le_bytes());
        buf.extend_from_slice(&iv.end.to_le_bytes());
        buf.extend_from_slice(&iv.value.to_le_bytes());
    }
    buf
}

// ── CIR-tree (R-tree index) ───────────────────────────────────────────────────

/// Write a CIR-tree index over `blocks`.
///
/// For up to `block_size` blocks: a single leaf node.
/// For more: a root internal node pointing to leaf nodes of at most
/// `block_size` entries each.
fn write_cir_tree<W: Write + Seek>(
    out: &mut W,
    blocks: &[DataBlock],
    block_size: u32,
) -> Result<()> {
    let (start_chrom, start_base, end_chrom, end_base) = if blocks.is_empty() {
        (0, 0, 0, 0)
    } else {
        (
            blocks[0].start_chrom,
            blocks[0].start_base,
            blocks[blocks.len() - 1].end_chrom,
            blocks[blocks.len() - 1].end_base,
        )
    };

    write_cir_tree_header(
        out,
        block_size,
        blocks.len() as u64,
        start_chrom,
        start_base,
        end_chrom,
        end_base,
    )?;

    if blocks.is_empty() {
        // Single empty leaf node.
        out.write_all(&[1u8, 0, 0, 0]).map_err(RsomicsError::Io)?;
        return Ok(());
    }

    let bs = block_size as usize;
    if blocks.len() <= bs {
        write_cir_leaf_node(out, blocks)?;
    } else {
        // Two-level tree: root internal node + child leaf nodes.
        let n_leaves = blocks.len().div_ceil(bs);
        let root_off = stream_pos(out)?;

        // Write placeholder root (isLeaf=0; will be back-patched).
        // Each internal item: startChromIx(4)+startBase(4)+endChromIx(4)+endBase(4)+childOffset(8) = 24 bytes
        let root_size = 4 + n_leaves * 24;
        write_zeros(out, root_size)?;

        // Write leaf nodes, record their offsets.
        let mut child_offsets = Vec::with_capacity(n_leaves);
        let mut chunk_bounds: Vec<(u32, u32, u32, u32)> = Vec::with_capacity(n_leaves);
        for chunk in blocks.chunks(bs) {
            child_offsets.push(stream_pos(out)?);
            chunk_bounds.push((
                chunk[0].start_chrom,
                chunk[0].start_base,
                chunk[chunk.len() - 1].end_chrom,
                chunk[chunk.len() - 1].end_base,
            ));
            write_cir_leaf_node(out, chunk)?;
        }

        // Back-patch root internal node.
        out.seek(std::io::SeekFrom::Start(root_off))
            .map_err(RsomicsError::Io)?;
        let internal_count = u16::try_from(n_leaves)
            .map_err(|_| RsomicsError::InvalidInput("too many CIR-tree leaves".into()))?;
        out.write_all(&[0u8, 0]).map_err(RsomicsError::Io)?;
        out.write_all(&internal_count.to_le_bytes())
            .map_err(RsomicsError::Io)?;
        for (i, &coff) in child_offsets.iter().enumerate() {
            let (sc, sb, ec, eb) = chunk_bounds[i];
            out.write_all(&sc.to_le_bytes()).map_err(RsomicsError::Io)?;
            out.write_all(&sb.to_le_bytes()).map_err(RsomicsError::Io)?;
            out.write_all(&ec.to_le_bytes()).map_err(RsomicsError::Io)?;
            out.write_all(&eb.to_le_bytes()).map_err(RsomicsError::Io)?;
            out.write_all(&coff.to_le_bytes())
                .map_err(RsomicsError::Io)?;
        }
        // Restore position to end of the tree.
        out.seek(std::io::SeekFrom::End(0))
            .map_err(RsomicsError::Io)?;
    }
    Ok(())
}

fn write_cir_tree_header<W: Write>(
    out: &mut W,
    block_size: u32,
    item_count: u64,
    start_chrom: u32,
    start_base: u32,
    end_chrom: u32,
    end_base: u32,
) -> Result<()> {
    let mut buf = [0u8; 48];
    buf[0..4].copy_from_slice(&CIR_TREE_MAGIC.to_le_bytes());
    buf[4..8].copy_from_slice(&block_size.to_le_bytes());
    buf[8..16].copy_from_slice(&item_count.to_le_bytes());
    buf[16..20].copy_from_slice(&start_chrom.to_le_bytes());
    buf[20..24].copy_from_slice(&start_base.to_le_bytes());
    buf[24..28].copy_from_slice(&end_chrom.to_le_bytes());
    buf[28..32].copy_from_slice(&end_base.to_le_bytes());
    // endFileOffset(8): optional, set to 0.
    // itemsPerSlot(4): 1; reserved(4): 0.
    buf[40..44].copy_from_slice(&1u32.to_le_bytes());
    out.write_all(&buf).map_err(RsomicsError::Io)
}

fn write_cir_leaf_node<W: Write>(out: &mut W, blocks: &[DataBlock]) -> Result<()> {
    let count = u16::try_from(blocks.len())
        .map_err(|_| RsomicsError::InvalidInput("CIR-tree leaf too large".into()))?;
    // node header: isLeaf(1) + reserved(1) + count(2)
    out.write_all(&[1u8, 0]).map_err(RsomicsError::Io)?;
    out.write_all(&count.to_le_bytes())
        .map_err(RsomicsError::Io)?;
    // Each leaf item: startChromIx(4)+startBase(4)+endChromIx(4)+endBase(4)
    //                 +dataOffset(8)+dataSize(8) = 32 bytes
    for b in blocks {
        let mut item = [0u8; 32];
        item[0..4].copy_from_slice(&b.start_chrom.to_le_bytes());
        item[4..8].copy_from_slice(&b.start_base.to_le_bytes());
        item[8..12].copy_from_slice(&b.end_chrom.to_le_bytes());
        item[12..16].copy_from_slice(&b.end_base.to_le_bytes());
        item[16..24].copy_from_slice(&b.offset.to_le_bytes());
        item[24..32].copy_from_slice(&b.size.to_le_bytes());
        out.write_all(&item).map_err(RsomicsError::Io)?;
    }
    Ok(())
}

// ── zoom computation ──────────────────────────────────────────────────────────

struct ZoomRecord {
    chrom_id: u32,
    start: u32,
    end: u32,
    n_bases: u32,
    min_val: f32,
    max_val: f32,
    sum: f32,
    sum_squares: f32,
}

/// Compute zoom records for one reduction factor from full-resolution intervals.
///
/// Each zoom record covers exactly `reduction` bases (or less at chromosome end).
/// Computes nBases/min/max/sum/sumSquares for all intervals overlapping the window.
#[allow(clippy::cast_precision_loss)]
fn compute_zoom_items(
    chroms: &[ChromInfo],
    intervals: &[Interval],
    reduction: u32,
) -> Vec<ZoomRecord> {
    let mut records = Vec::new();
    if intervals.is_empty() || reduction == 0 {
        return records;
    }

    let mut iv_idx = 0;
    for chrom in chroms {
        let chrom_len = chrom.length;
        let chrom_id = chrom.id;

        while iv_idx < intervals.len() && intervals[iv_idx].chrom_id < chrom_id {
            iv_idx += 1;
        }
        if iv_idx >= intervals.len() || intervals[iv_idx].chrom_id != chrom_id {
            continue;
        }

        let chrom_iv_start = iv_idx;
        let chrom_iv_end = {
            let mut e = iv_idx;
            while e < intervals.len() && intervals[e].chrom_id == chrom_id {
                e += 1;
            }
            e
        };

        let mut win_start: u32 = 0;
        while win_start < chrom_len {
            let win_end = (win_start + reduction).min(chrom_len);
            let mut n_bases: u32 = 0;
            let mut min_val = f32::INFINITY;
            let mut max_val = f32::NEG_INFINITY;
            let mut sum = 0.0f32;
            let mut sum_sq = 0.0f32;

            for iv in &intervals[chrom_iv_start..chrom_iv_end] {
                if iv.end <= win_start {
                    continue;
                }
                if iv.start >= win_end {
                    break;
                }
                let ov_start = iv.start.max(win_start);
                let ov_end = iv.end.min(win_end);
                let n = ov_end - ov_start;
                n_bases += n;
                if iv.value < min_val {
                    min_val = iv.value;
                }
                if iv.value > max_val {
                    max_val = iv.value;
                }
                sum += iv.value * n as f32;
                sum_sq += iv.value * iv.value * n as f32;
            }

            if n_bases > 0 {
                records.push(ZoomRecord {
                    chrom_id,
                    start: win_start,
                    end: win_end,
                    n_bases,
                    min_val: if min_val.is_infinite() { 0.0 } else { min_val },
                    max_val: if max_val.is_infinite() { 0.0 } else { max_val },
                    sum,
                    sum_squares: sum_sq,
                });
            }

            win_start = win_end;
        }

        iv_idx = chrom_iv_end;
    }

    records
}

fn encode_zoom_record(r: &ZoomRecord) -> [u8; 32] {
    let mut b = [0u8; 32];
    b[0..4].copy_from_slice(&r.chrom_id.to_le_bytes());
    b[4..8].copy_from_slice(&r.start.to_le_bytes());
    b[8..12].copy_from_slice(&r.end.to_le_bytes());
    b[12..16].copy_from_slice(&r.n_bases.to_le_bytes());
    b[16..20].copy_from_slice(&r.min_val.to_le_bytes());
    b[20..24].copy_from_slice(&r.max_val.to_le_bytes());
    b[24..28].copy_from_slice(&r.sum.to_le_bytes());
    b[28..32].copy_from_slice(&r.sum_squares.to_le_bytes());
    b
}

fn write_zoom_data<W: Write + Seek>(out: &mut W, records: &[ZoomRecord]) -> Result<Vec<DataBlock>> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < records.len() {
        let block_end = (i + ITEMS_PER_BLOCK).min(records.len());
        let chunk = &records[i..block_end];
        let first = &chunk[0];
        let last = &chunk[chunk.len() - 1];

        let mut raw = Vec::with_capacity(chunk.len() * 32);
        for r in chunk {
            raw.extend_from_slice(&encode_zoom_record(r));
        }
        let compressed = zlib_compress(&raw)?;

        let offset = stream_pos(out)?;
        let size = compressed.len() as u64;
        out.write_all(&compressed).map_err(RsomicsError::Io)?;

        blocks.push(DataBlock {
            start_chrom: first.chrom_id,
            start_base: first.start,
            end_chrom: last.chrom_id,
            end_base: last.end,
            offset,
            size,
        });

        i = block_end;
    }
    Ok(blocks)
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn write_zeros<W: Write>(out: &mut W, n: usize) -> Result<()> {
    let zeros = vec![0u8; n];
    out.write_all(&zeros).map_err(RsomicsError::Io)
}

fn stream_pos<W: Seek>(out: &mut W) -> Result<u64> {
    out.stream_position().map_err(RsomicsError::Io)
}

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut enc = ZlibEncoder::new(Vec::new(), COMPRESS_LEVEL);
    enc.write_all(data).map_err(RsomicsError::Io)?;
    enc.finish().map_err(RsomicsError::Io)
}
