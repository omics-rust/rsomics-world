//! Pure-Rust bigWig/BBI reader: just enough to answer per-base value queries
//! the way pyBigWig's `values(chrom, start, end)` does (NaN where the file
//! carries no data), plus chromosome enumeration for genome-wide tiling.
//!
//! ## Origin
//!
//! The bigWig/BBI on-disk layout (header at offset 0, chromosome B-tree, R-tree
//! "cir" index, zlib-compressed data sections in bedGraph/varStep/fixedStep
//! flavours) was read from the bigtools 0.5.6 source (`src/bbi/bbiread.rs`,
//! `src/bbi/bigwigread.rs`, MIT, Jack Huey) and Jim Kent's published BBI format.
//! We carry our own reader rather than depend on bigtools because bigtools pins
//! `libdeflater = "0.13"` whose `libdeflate-sys` (C FFI, `links = "libdeflate"`)
//! collides with the workspace's `rsomics-fqgz` (`libdeflater = "1"`); two
//! crates may not link the same native library. Block inflation here uses the
//! workspace `flate2` zlib-rs backend (pure Rust), keeping this Quadrant ①.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

const BIGWIG_MAGIC: u32 = 0x888F_FC26;
const CHROM_TREE_MAGIC: u32 = 0x78CA_8C91;
const CIR_TREE_MAGIC: u32 = 0x2468_ACE0;

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
        let _zoom_levels = c.u16();
        let chromosome_tree_offset = c.u64();
        let _full_data_offset = c.u64();
        let full_index_offset = c.u64();
        let _field_count = c.u16();
        let _defined_field_count = c.u16();
        let _auto_sql_offset = c.u64();
        let _total_summary_offset = c.u64();
        let uncompress_buf_size = c.u32();

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

    /// Per-base values for `[start, end)` on `chrom`. Positions with no data in
    /// the bigWig are `NAN`. If the chromosome is absent, returns `None`.
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
