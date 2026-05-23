use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use noodles::bgzf;
use noodles::core::Position;
use noodles::csi::{
    self as csi,
    binning_index::index::{
        ReferenceSequence,
        header::{Builder as HeaderBuilder, ReferenceSequenceNames},
        reference_sequence::bin::Chunk,
    },
    binning_index::{self, BinningIndex, index::reference_sequence::index::BinnedIndex},
};
use noodles::tabix;
use noodles::vcf::{self, Header};

/// Which on-disk format to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    /// Coordinate-sorted index (.csi). The `bcftools index` default.
    Csi,
    /// Tabix index (.tbi). tabix -p vcf preset.
    Tbi,
}

/// CSI binning parameters htslib (and thus `bcftools index`) writes by default:
/// 14-bit minimum interval (16 kbp leaves) over 6 binning levels.
const CSI_MIN_SHIFT: u8 = 14;
const CSI_DEPTH: u8 = 6;

/// Build and write an index for a bgzipped VCF.
///
/// `src` is the `.vcf.gz` input; `dst` is the path to write the index.  Caller
/// is responsible for not-overwriting checks if `--force` is absent.
pub fn index_vcf(src: &Path, dst: &Path, kind: IndexKind) -> io::Result<()> {
    match kind {
        IndexKind::Csi => {
            let idx = build_csi(src)?;
            csi::fs::write(dst, &idx)
        }
        IndexKind::Tbi => {
            let idx = build_tbi(src)?;
            tabix::fs::write(dst, &idx)
        }
    }
}

// CSI: BinnedIndex with htslib's default min_shift/depth (bcftools index default).
fn build_csi(src: &Path) -> io::Result<csi::Index> {
    let (header, mut reader) = open(src)?;

    // Collect contig names in declaration order so the CSI header carries the name→id map.
    // bcftools / htslib resolve a region string like "chr1:1-100000" by looking up the contig
    // name in the CSI aux block's reference_sequence_names list.
    let ref_names: ReferenceSequenceNames = header
        .contigs()
        .keys()
        .map(|k| bstr::BString::from(k.as_str()))
        .collect();

    let csi_header = HeaderBuilder::vcf()
        .set_reference_sequence_names(ref_names)
        .build();

    // BinnedIndex is the CSI-native index sub-type (vs LinearIndex for tabix).
    let mut indexer =
        binning_index::Indexer::<BinnedIndex>::new(CSI_MIN_SHIFT, CSI_DEPTH).set_header(csi_header);

    let contig_count = header.contigs().len();

    // Per-reference linear index, mirroring htslib's `lidx`/`l.offset[]` (hts.c
    // `insert_to_l`). noodles' BinnedIndex stores each bin's loffset from the
    // record's own bin only; htslib derives every bin's loffset from this 16 kbp
    // linear index, so a spanning structural-variant record (e.g. `<DEL>` with a
    // far `END=`) lowers the loffset of every window it overlaps. Without that
    // propagation a region query overlapping the SV's span but not its start would
    // prune the SV's chunk on the `chunk.end <= loffset` test htslib applies.
    let mut linear_indices: Vec<LinearIndexBuilder> = (0..contig_count)
        .map(|_| LinearIndexBuilder::default())
        .collect();

    let mut line = Vec::new();
    let mut start_pos = reader.virtual_position();

    while read_line(&mut reader, &mut line)? != 0 {
        let end_pos = reader.virtual_position();
        let chunk = Chunk::new(start_pos, end_pos);

        let (ref_name, start, end) = parse_interval(&line, &header)?;
        let ref_id = header.contigs().get_index_of(ref_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("contig '{ref_name}' not declared in VCF header"),
            )
        })?;

        linear_indices[ref_id].insert(start, end, start_pos);
        indexer.add_record(Some((ref_id, start, end, true)), chunk)?;
        start_pos = end_pos;
    }

    let index = indexer.build(contig_count);
    Ok(apply_linear_loffsets(&index, &mut linear_indices))
}

/// htslib's linear index for one reference: a 16 kbp-window array of the earliest
/// (smallest) record start voffset reaching each window (hts.c `lidx_t`).
#[derive(Default)]
struct LinearIndexBuilder {
    offsets: Vec<Option<bgzf::VirtualPosition>>,
}

impl LinearIndexBuilder {
    /// Mirror of htslib `insert_to_l`: fill windows `beg>>min_shift ..=
    /// (end-1)>>min_shift` with `offset`, but only where still unset. Records
    /// arrive in start order, so the first record to reach a window carries the
    /// minimum start voffset of any record overlapping it.
    fn insert(&mut self, start: Position, end: Position, offset: bgzf::VirtualPosition) {
        let shift = usize::from(CSI_MIN_SHIFT);
        let beg = (usize::from(start) - 1) >> shift;
        let end = (usize::from(end) - 1) >> shift;

        if self.offsets.len() < end + 1 {
            self.offsets.resize(end + 1, None);
        }

        for window in &mut self.offsets[beg..=end] {
            window.get_or_insert(offset);
        }
    }

    /// Mirror of the backfill loop in htslib `update_loff`: walk right→left and
    /// give every still-unset window the value of its right neighbour, so each
    /// window holds the smallest start voffset of any record at-or-after it.
    fn finish(&mut self) {
        for i in (0..self.offsets.len().saturating_sub(1)).rev() {
            if self.offsets[i].is_none() {
                self.offsets[i] = self.offsets[i + 1];
            }
        }
    }

    /// The loffset htslib assigns a bin: `lidx[hts_bin_bot(bin)]`, i.e. the linear
    /// index entry for the bin's lowest-coordinate covered window. Out-of-range
    /// (no record reached that window) yields the default (0), matching htslib's
    /// `bot_bin < lidx->n ? lidx->offset[bot_bin] : 0`.
    fn loffset(&self, bin_id: usize) -> bgzf::VirtualPosition {
        let bot = hts_bin_bot(bin_id, CSI_DEPTH);
        self.offsets.get(bot).copied().flatten().unwrap_or_default()
    }
}

/// First bin id on level `l` of the binning tree (hts.h `hts_bin_first`).
fn hts_bin_first(level: u32) -> usize {
    ((1usize << (3 * level)) - 1) / 7
}

/// Level of a bin in the binning tree (hts.h `hts_bin_level`): parent walks to 0.
fn hts_bin_level(mut bin: usize) -> u32 {
    let mut level = 0;
    while bin != 0 {
        bin = (bin - 1) >> 3;
        level += 1;
    }
    level
}

/// Lowest-coordinate linear-index window covered by `bin` (hts.h `hts_bin_bot`):
/// `(bin - hts_bin_first(level)) << ((n_lvls - level) * 3)`.
fn hts_bin_bot(bin: usize, depth: u8) -> usize {
    let level = hts_bin_level(bin);
    let n_lvls = u32::from(depth);
    (bin - hts_bin_first(level)) << ((n_lvls - level) * 3)
}

/// Replace every bin's loffset with htslib's linear-index-derived value
/// (`update_loff` in hts.c), closing the spanning-SV gap in noodles' per-bin
/// loffset. The bin chunk lists, metadata, and binning parameters built by the
/// indexer are preserved; only the `BinnedIndex` loffset map of each reference is
/// rebuilt. O(bins) over an already-collected linear index — no hot-path cost.
fn apply_linear_loffsets(
    index: &csi::Index,
    linear_indices: &mut [LinearIndexBuilder],
) -> csi::Index {
    let min_shift = index.min_shift();
    let depth = index.depth();
    let header = index.header().cloned();
    let unplaced = index.unplaced_unmapped_record_count();

    let reference_sequences: Vec<ReferenceSequence<BinnedIndex>> = index
        .reference_sequences()
        .iter()
        .zip(linear_indices.iter_mut())
        .map(|(reference_sequence, linear_index)| {
            linear_index.finish();

            let bins = reference_sequence.bins().clone();
            let loffsets: BinnedIndex = bins
                .keys()
                .map(|&bin_id| (bin_id, linear_index.loffset(bin_id)))
                .collect();
            let metadata = csi_metadata(reference_sequence).cloned();

            ReferenceSequence::new(bins, loffsets, metadata)
        })
        .collect();

    let mut builder = csi::Index::builder()
        .set_min_shift(min_shift)
        .set_depth(depth)
        .set_reference_sequences(reference_sequences);

    if let Some(header) = header {
        builder = builder.set_header(header);
    }
    if let Some(count) = unplaced {
        builder = builder.set_unplaced_unmapped_record_count(count);
    }

    builder.build()
}

/// Read a reference sequence's metadata pseudo-bin through the trait the CSI
/// reference-sequence type implements.
fn csi_metadata(
    reference_sequence: &ReferenceSequence<BinnedIndex>,
) -> Option<&csi::binning_index::index::reference_sequence::Metadata> {
    use csi::binning_index::ReferenceSequence as _;
    reference_sequence.metadata()
}

// TBI: LinearIndex, tabix VCF preset.
fn build_tbi(src: &Path) -> io::Result<tabix::Index> {
    let (header, mut reader) = open(src)?;

    let mut indexer = tabix::index::Indexer::default();
    indexer.set_header(HeaderBuilder::vcf().build());

    let mut line = Vec::new();
    let mut start_pos = reader.virtual_position();

    while read_line(&mut reader, &mut line)? != 0 {
        let end_pos = reader.virtual_position();
        let chunk = Chunk::new(start_pos, end_pos);

        let (ref_name, start, end) = parse_interval(&line, &header)?;
        indexer.add_record(ref_name, start, end, chunk)?;
        start_pos = end_pos;
    }

    Ok(indexer.build())
}

/// Read the header through noodles (for the contig map and file format), then
/// hand back the underlying BGZF reader positioned at the first data record so
/// the index loop can do a minimal CHROM/POS/REF parse off the raw lines.
fn open(src: &Path) -> io::Result<(Header, bgzf::io::Reader<File>)> {
    let mut reader = File::open(src)
        .map(bgzf::io::Reader::new)
        .map(vcf::io::Reader::new)?;
    let header = reader.read_header()?;
    Ok((header, reader.into_inner()))
}

/// Read one record line (without the trailing newline) into `dst`.  Mirrors the
/// std `read_until` semantics noodles itself relies on, so `virtual_position()`
/// after the call lands on the byte after the line feed — the chunk boundary.
fn read_line<R>(reader: &mut R, dst: &mut Vec<u8>) -> io::Result<usize>
where
    R: BufRead,
{
    const LINE_FEED: u8 = b'\n';
    const CARRIAGE_RETURN: u8 = b'\r';

    dst.clear();
    match reader.read_until(LINE_FEED, dst)? {
        0 => Ok(0),
        n => {
            if dst.last() == Some(&LINE_FEED) {
                dst.pop();
                if dst.last() == Some(&CARRIAGE_RETURN) {
                    dst.pop();
                }
            }
            Ok(n)
        }
    }
}

/// Extract the CHROM, start, and end of a record's index interval from a raw
/// VCF data line.
///
/// htslib indexes each record over the 1-based inclusive span from `POS` to
/// `POS + rlen - 1`, where `rlen` is the REF allele length for the common
/// SNV/indel case.  For VCF < 4.5 an INFO `END` value, when larger, extends the
/// span (the reach htslib uses for symbolic structural-variant records).
/// Parsing CHROM, POS, REF, and an `END=` scan of INFO avoids materialising the
/// alternate alleles, FORMAT keys, and per-sample genotypes a full record decode
/// would touch — none of which affect the index interval.
fn parse_interval<'a>(
    line: &'a [u8],
    header: &Header,
) -> io::Result<(&'a str, Position, Position)> {
    let chrom_end = memchr::memchr(b'\t', line).ok_or_else(|| invalid("VCF record missing POS"))?;
    let ref_name = std::str::from_utf8(&line[..chrom_end])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let after_chrom = &line[chrom_end + 1..];
    let pos_end =
        memchr::memchr(b'\t', after_chrom).ok_or_else(|| invalid("VCF record missing ID"))?;

    // POS == 0 is the telomere-start sentinel htslib treats specially; noodles'
    // own indexer rejects records with no valid 1-based start, so we do too.
    let pos = parse_usize(&after_chrom[..pos_end])?;
    let start =
        Position::try_from(pos).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Skip ID, land on REF.
    let after_pos = &after_chrom[pos_end + 1..];
    let id_end =
        memchr::memchr(b'\t', after_pos).ok_or_else(|| invalid("VCF record missing REF"))?;
    let after_id = &after_pos[id_end + 1..];
    let ref_len =
        memchr::memchr(b'\t', after_id).ok_or_else(|| invalid("VCF record missing ALT"))?;
    if ref_len == 0 {
        return Err(invalid("invalid reference bases length"));
    }

    // span end = POS + rlen - 1; END (VCF < 4.5) extends it when larger.
    let mut end = start
        .checked_add(ref_len - 1)
        .ok_or_else(|| invalid("position overflow"))?;
    if let Some(end_pos) = info_end(after_id, header)? {
        let end_position = Position::try_from(end_pos)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        end = end.max(end_position);
    }

    Ok((ref_name, start, end))
}

/// For VCF < 4.5, htslib derives the record's reach from the INFO `END` field
/// when present.  Returns the parsed 1-based inclusive end coordinate, or `None`
/// when END is absent or the file format is >= 4.5 (where END is deprecated and
/// the reach comes from REF/SVLEN instead).  Scans only the INFO column for an
/// `END=` key — no full INFO parse.
fn info_end(after_id: &[u8], header: &Header) -> io::Result<Option<usize>> {
    let ff = header.file_format();
    if (ff.major(), ff.minor()) >= (4, 5) {
        return Ok(None);
    }

    // after_id starts at REF; INFO is the 4th tab field beyond it
    // (REF, ALT, QUAL, FILTER, INFO).
    let Some(info) = nth_tab_field(after_id, 4) else {
        return Ok(None);
    };
    if info == b"." {
        return Ok(None);
    }

    match find_info_value(info, b"END") {
        Some(value) => Ok(Some(parse_usize(value)?)),
        None => Ok(None),
    }
}

/// Find the value of `key` inside a VCF INFO field (`;`-delimited `KEY=VALUE`
/// pairs).  Returns the raw value bytes if the key is found.
fn find_info_value<'a>(info: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    for entry in info.split(|&b| b == b';') {
        if let Some(eq) = memchr::memchr(b'=', entry)
            && &entry[..eq] == key
        {
            return Some(&entry[eq + 1..]);
        }
    }
    None
}

/// Return the `n`-th tab-delimited field of `src` beyond the current position
/// (0-based), stopping at the next tab or end of slice.
fn nth_tab_field(src: &[u8], n: usize) -> Option<&[u8]> {
    let mut rest = src;
    for _ in 0..n {
        let i = memchr::memchr(b'\t', rest)?;
        rest = &rest[i + 1..];
    }
    let end = memchr::memchr(b'\t', rest).unwrap_or(rest.len());
    Some(&rest[..end])
}

fn parse_usize(bytes: &[u8]) -> io::Result<usize> {
    let s =
        std::str::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    s.parse::<usize>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn invalid(msg: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}
