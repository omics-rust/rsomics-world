//! `samtools cat` port: concatenate BAMs by copying compressed BGZF blocks
//! verbatim.
//!
//! cat is IO-bound by design — samtools never inflates the alignment records, it
//! moves the compressed gzip members byte-for-byte (`bgzf_raw_read` /
//! `bgzf_raw_write`, bam_cat.c). The only re-framed bytes are the single output
//! header. This port does the same via [`bgzf_copy`]: each input's header is
//! inflated and dropped, one merged header is written once, and every alignment
//! frame is copied raw. So the per-record deflate that the
//! [`rsomics_bamio::raw`] edit path pays (and that loses to samtools) never
//! happens here.
//!
//! Header rule (bam_cat.c `cat_check_merge_hdr`): the first file's header is the
//! base; @RG lines present in later files but not the base are appended; @SQ and
//! everything else come from the first file unchanged. When no @RG merge is
//! needed — the dominant case of concatenating shards of one alignment — the
//! first header's raw uncompressed bytes are re-emitted verbatim, so the output
//! header is byte-identical to samtools `cat --no-PG`.

mod bgzf_copy;

use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use noodles::bam;
use noodles::sam::Header;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

use bgzf_copy::{HeaderReader, HeaderStream, copy_records, write_bgzf};

/// Buffer size for the raw input stream under the frame reader: one ~256 KiB
/// refill amortises the per-frame (~64 KiB) reads down to fewer syscalls than
/// samtools' default BGZF read path, which is the lever for a same-machine win
/// on an otherwise IO-bound copy.
const READ_BUFFER: usize = 1024 * 1024;
const WRITE_BUFFER: usize = 1024 * 1024;

#[derive(Debug, Default, Clone, Serialize)]
pub struct CatStats {
    pub inputs: u64,
}

#[derive(Debug, Clone)]
pub struct CatOpts {
    /// Header source file (`-h`): use this header instead of the first input's.
    pub header_file: Option<PathBuf>,
    /// Omit the @PG line (`--no-PG`). Always set in compat runs.
    pub no_pg: bool,
}

/// Read just the uncompressed BAM header from `path`, returning the parsed
/// [`Header`] and the raw header bytes (everything before the first alignment
/// record: `magic l_text text n_ref refs`). The raw bytes let the writer
/// re-emit the first file's header verbatim when no @RG merge is needed.
fn read_header_bytes(path: &Path) -> Result<(Header, Vec<u8>)> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut reader = BufReader::with_capacity(READ_BUFFER, file);
    let mut state = HeaderReader::new();
    let header = {
        let stream = HeaderStream::new(&mut reader, &mut state);
        let mut bam_reader = bam::io::Reader::from(stream);
        bam_reader.read_header().map_err(RsomicsError::Io)?
    };
    let raw = state.into_consumed_header();
    Ok((header, raw))
}

/// bam_cat.c `cat_check_merge_hdr`: append @RG lines from `src` that the base
/// header does not already carry (matched by ID). Returns whether any line was
/// added — when nothing is added the base header is emitted verbatim.
fn merge_read_groups(base: &mut Header, src: &Header) -> bool {
    let mut added = false;
    for (id, map) in src.read_groups() {
        if !base.read_groups().contains_key(id) {
            base.read_groups_mut().insert(id.clone(), map.clone());
            added = true;
        }
    }
    added
}

pub fn cat(inputs: &[PathBuf], output_path: Option<&Path>, opts: &CatOpts) -> Result<CatStats> {
    if inputs.is_empty() {
        return Err(RsomicsError::InvalidInput("no input BAM files".to_string()));
    }

    // Build the output header: from -h if given, else the first input's header.
    // RG lines from every input are merged in; @SQ etc. stay as the base's.
    let (mut out_header, base_raw, mut verbatim) = match &opts.header_file {
        Some(hf) => {
            let (h, _raw) = read_header_bytes(hf)?;
            (h, Vec::new(), false)
        }
        None => {
            let (h, raw) = read_header_bytes(&inputs[0])?;
            (h, raw, true)
        }
    };

    for path in &inputs[1..] {
        let (h, _raw) = read_header_bytes(path)?;
        if merge_read_groups(&mut out_header, &h) {
            verbatim = false;
        }
    }

    let pg = (!opts.no_pg).then(pg_line);
    if pg.is_some() {
        verbatim = false;
    }

    match output_path {
        Some(path) => {
            let file = File::create(path).map_err(|e| {
                RsomicsError::InvalidInput(format!("creating {}: {e}", path.display()))
            })?;
            let mut out = std::io::BufWriter::with_capacity(WRITE_BUFFER, file);
            let stats = write_all(
                inputs,
                &mut out,
                &out_header,
                &base_raw,
                verbatim,
                pg.as_deref(),
            )?;
            out.flush().map_err(RsomicsError::Io)?;
            Ok(stats)
        }
        None => {
            let stdout = std::io::stdout();
            let mut out = std::io::BufWriter::with_capacity(WRITE_BUFFER, stdout.lock());
            let stats = write_all(
                inputs,
                &mut out,
                &out_header,
                &base_raw,
                verbatim,
                pg.as_deref(),
            )?;
            out.flush().map_err(RsomicsError::Io)?;
            Ok(stats)
        }
    }
}

/// samtools writes a `@PG ID:samtools PN:samtools` line; we record our own tool
/// so a downstream reader sees the provenance. The exact text differs from
/// samtools (different program), so compat runs pass `--no-PG`.
fn pg_line() -> String {
    format!(
        "@PG\tID:rsomics-bam-cat\tPN:rsomics-bam-cat\tVN:{}\n",
        env!("CARGO_PKG_VERSION")
    )
}

fn write_all<W: Write>(
    inputs: &[PathBuf],
    out: &mut W,
    out_header: &Header,
    base_raw: &[u8],
    verbatim: bool,
    pg: Option<&str>,
) -> Result<CatStats> {
    write_output_header(out, out_header, base_raw, verbatim, pg)?;

    for path in inputs {
        let file = File::open(path)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
        let mut reader = BufReader::with_capacity(READ_BUFFER, file);
        let mut state = HeaderReader::new();
        {
            let stream = HeaderStream::new(&mut reader, &mut state);
            let mut bam_reader = bam::io::Reader::from(stream);
            bam_reader.read_header().map_err(RsomicsError::Io)?;
        }
        copy_records(state, &mut reader, out)?;
    }

    // One trailing BGZF EOF marker for the whole concatenation.
    out.write_all(&bgzf_copy::BGZF_EOF)
        .map_err(RsomicsError::Io)?;

    Ok(CatStats {
        inputs: inputs.len() as u64,
    })
}

/// Emit the output BAM header as one or more BGZF blocks. When `verbatim` (no
/// @RG merge, no @PG, header straight from the first input) the first file's
/// raw header bytes are re-framed unchanged — byte-identical to samtools
/// `cat --no-PG`. Otherwise the merged header is serialised via the BAM writer.
fn write_output_header<W: Write>(
    out: &mut W,
    out_header: &Header,
    base_raw: &[u8],
    verbatim: bool,
    pg: Option<&str>,
) -> Result<()> {
    if verbatim {
        write_bgzf(out, base_raw)?;
        return Ok(());
    }

    // Serialise the (possibly @RG-merged) header into BAM header bytes, then
    // optionally splice the @PG line in before re-framing as BGZF.
    let mut buf = Vec::new();
    {
        let mut hw = bam::io::Writer::new(&mut buf);
        hw.write_header(out_header).map_err(RsomicsError::Io)?;
    }
    let raw = strip_bgzf_to_uncompressed(&buf)?;
    let raw = match pg {
        Some(line) => splice_pg(&raw, line)?,
        None => raw,
    };
    write_bgzf(out, &raw)?;
    Ok(())
}

/// The BAM writer emits a BGZF stream (header block(s) + EOF). Inflate it back
/// to the raw uncompressed header bytes so it can be re-framed alongside our
/// own block writer. Header-sized, so the inflate cost is negligible.
fn strip_bgzf_to_uncompressed(bgzf: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut out = Vec::new();
    noodles::bgzf::io::Reader::new(bgzf)
        .read_to_end(&mut out)
        .map_err(RsomicsError::Io)?;
    Ok(out)
}

/// Insert `pg` (a full `@PG…\n` line) into the raw header text, after the last
/// existing header line and before the binary `n_ref`/refs block. Mirrors
/// htslib appending @PG to the text region.
fn splice_pg(raw: &[u8], pg: &str) -> Result<Vec<u8>> {
    // Layout: magic(4) l_text(4) text(l_text) n_ref(4) refs…
    let l_text = u32::from_le_bytes(raw[4..8].try_into().unwrap()) as usize;
    let text_start = 8;
    let text_end = text_start + l_text;
    let text = &raw[text_start..text_end];
    let mut new_text = Vec::with_capacity(text.len() + pg.len());
    new_text.extend_from_slice(text);
    if !new_text.ends_with(b"\n") && !new_text.is_empty() {
        new_text.push(b'\n');
    }
    new_text.extend_from_slice(pg.as_bytes());

    let mut out = Vec::with_capacity(raw.len() + pg.len());
    out.extend_from_slice(&raw[..4]); // magic
    out.extend_from_slice(&u32::try_from(new_text.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&new_text);
    out.extend_from_slice(&raw[text_end..]); // n_ref + refs
    Ok(out)
}
