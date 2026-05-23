use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use noodles::bgzf;
use noodles::core::Position;
use noodles::csi::{
    self as csi,
    binning_index::index::{
        header::{Builder as HeaderBuilder, ReferenceSequenceNames},
        reference_sequence::bin::Chunk,
    },
    binning_index::{self, index::reference_sequence::index::BinnedIndex},
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

        indexer.add_record(Some((ref_id, start, end, true)), chunk)?;
        start_pos = end_pos;
    }

    let contig_count = header.contigs().len();
    Ok(indexer.build(contig_count))
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
