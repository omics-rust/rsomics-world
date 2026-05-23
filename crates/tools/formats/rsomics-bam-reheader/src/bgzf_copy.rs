//! BGZF block-level passthrough: copy the compressed alignment blocks of a BAM
//! verbatim, re-framing only the header.
//!
//! samtools `cat`/`reheader` are near-optimal because they never inflate the
//! alignment records — `bgzf_raw_read`/`bgzf_raw_write` move the compressed
//! gzip members byte-for-byte (bam_cat.c, bam_reheader.c). Decoding every record
//! and re-deflating it (the `rsomics_bamio::raw` path) is correct but pays a
//! deflate per record, which loses to that block copy. To tie-or-beat samtools
//! we copy the same way.
//!
//! The wrinkle is the boundary: the BAM header (`magic l_text text n_ref refs…`)
//! and the first alignment record can share one BGZF block. We inflate frames
//! only until the header is consumed; the partial frame straddling the boundary
//! is re-emitted by recompressing its leftover uncompressed tail (one small
//! deflate, exactly as samtools' `bgzf_write` of `block_offset..block_length`
//! does), and every later frame is copied raw. So the deflate cost is O(header),
//! not O(records).
//!
//! A BGZF frame is a gzip member with a `BC` FEXTRA subfield carrying
//! `BSIZE = total_block_size - 1` (SAMv1 §4.1). Framing is read at the byte
//! level here — no dependency on noodles-bgzf reader internals, which do not
//! expose raw-frame access.

use std::io::{self, BufRead, Read, Write};

use flate2::Crc;
use flate2::write::DeflateEncoder;
use rsomics_common::{Result, RsomicsError};

/// Fixed BGZF gzip+FEXTRA header length (SAMv1 §4.1: 12-byte gzip header + the
/// 6-byte `BC` extra field).
const BGZF_HEADER_LEN: usize = 18;
/// gzip trailer: CRC32 + ISIZE, both u32 LE.
const GZIP_TRAILER_LEN: usize = 8;
/// Offset of the `BSIZE` u16 within the BGZF header.
const BSIZE_OFF: usize = 16;

/// The 28-byte BGZF EOF marker (an empty deflate block) htslib appends to every
/// well-formed BGZF file. cat drops each input's copy and writes exactly one at
/// the end; reheader copies through to real EOF, so the input's own marker is
/// the file's terminator.
pub const BGZF_EOF: [u8; 28] = [
    0x1f, 0x8b, 0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x06, 0x00, 0x42, 0x43, 0x02, 0x00,
    0x1b, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// One raw BGZF frame plus its decoded uncompressed length. The compressed bytes
/// are kept verbatim for passthrough; `isize` is read from the gzip trailer so a
/// frame can be skipped over the uncompressed stream without inflating it.
struct Frame {
    compressed: Vec<u8>,
    isize: usize,
}

/// Read one BGZF frame from `reader`. Returns `Ok(None)` at clean EOF.
fn read_frame<R: Read>(reader: &mut R) -> Result<Option<Frame>> {
    let mut header = [0u8; BGZF_HEADER_LEN];
    match read_full(reader, &mut header)? {
        0 => return Ok(None),
        BGZF_HEADER_LEN => {}
        _ => {
            return Err(RsomicsError::InvalidInput(
                "truncated BGZF header".to_string(),
            ));
        }
    }
    if header[0] != 0x1f || header[1] != 0x8b {
        return Err(RsomicsError::InvalidInput(
            "not a BGZF stream (bad gzip magic)".to_string(),
        ));
    }
    let bsize = u16::from_le_bytes([header[BSIZE_OFF], header[BSIZE_OFF + 1]]) as usize;
    let block_size = bsize + 1;
    if block_size < BGZF_HEADER_LEN + GZIP_TRAILER_LEN {
        return Err(RsomicsError::InvalidInput("invalid BGZF block size".into()));
    }
    let mut compressed = Vec::with_capacity(block_size);
    compressed.extend_from_slice(&header);
    compressed.resize(block_size, 0);
    reader
        .read_exact(&mut compressed[BGZF_HEADER_LEN..])
        .map_err(RsomicsError::Io)?;
    let isize = u32::from_le_bytes(compressed[block_size - 4..].try_into().unwrap()) as usize;
    Ok(Some(Frame { compressed, isize }))
}

/// Inflate one frame's deflate payload (between the 18-byte header and the
/// 8-byte trailer) into `dst`.
fn inflate_frame(frame: &Frame, dst: &mut Vec<u8>) -> Result<()> {
    use flate2::read::DeflateDecoder;
    let payload = &frame.compressed[BGZF_HEADER_LEN..frame.compressed.len() - GZIP_TRAILER_LEN];
    dst.clear();
    dst.reserve(frame.isize);
    DeflateDecoder::new(payload)
        .read_to_end(dst)
        .map_err(RsomicsError::Io)?;
    if dst.len() != frame.isize {
        return Err(RsomicsError::InvalidInput(
            "BGZF block ISIZE mismatch".to_string(),
        ));
    }
    Ok(())
}

/// Frame one buffer of uncompressed bytes into a single BGZF block written to
/// `out`. Used only for the header and the boundary frame's leftover tail — the
/// alignment records are never routed through here. `buf` must fit one block
/// (≤ 65535 uncompressed bytes after deflate framing); callers chunk to that.
fn write_bgzf_block<W: Write>(out: &mut W, buf: &[u8]) -> Result<()> {
    let mut deflated = Vec::new();
    {
        let mut enc = DeflateEncoder::new(&mut deflated, flate2::Compression::default());
        enc.write_all(buf).map_err(RsomicsError::Io)?;
        enc.finish().map_err(RsomicsError::Io)?;
    }
    let block_size = BGZF_HEADER_LEN + deflated.len() + GZIP_TRAILER_LEN;
    let bsize = u16::try_from(block_size - 1)
        .map_err(|_| RsomicsError::InvalidInput("BGZF block too large".to_string()))?;
    let mut header = [
        0x1f, 0x8b, 0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x06, 0x00, b'B', b'C', 0x02,
        0x00, 0x00, 0x00,
    ];
    header[BSIZE_OFF..BSIZE_OFF + 2].copy_from_slice(&bsize.to_le_bytes());
    out.write_all(&header).map_err(RsomicsError::Io)?;
    out.write_all(&deflated).map_err(RsomicsError::Io)?;
    let mut crc = Crc::new();
    crc.update(buf);
    out.write_all(&crc.sum().to_le_bytes())
        .map_err(RsomicsError::Io)?;
    out.write_all(&u32::try_from(buf.len()).unwrap().to_le_bytes())
        .map_err(RsomicsError::Io)?;
    Ok(())
}

/// Maximum uncompressed bytes htslib packs into one BGZF block.
const BGZF_MAX_ISIZE: usize = 0xff00;

/// Frame an arbitrary-length uncompressed buffer into one or more BGZF blocks.
pub fn write_bgzf<W: Write>(out: &mut W, mut buf: &[u8]) -> Result<()> {
    if buf.is_empty() {
        return write_bgzf_block(out, buf);
    }
    while !buf.is_empty() {
        let n = buf.len().min(BGZF_MAX_ISIZE);
        write_bgzf_block(out, &buf[..n])?;
        buf = &buf[n..];
    }
    Ok(())
}

/// Copy a BAM's alignment region — every BGZF frame after the header — verbatim
/// to `out`, dropping the trailing EOF marker. `consumed_header_len` is the
/// number of uncompressed header bytes already drained from the front of the
/// stream by the caller (via [`HeaderReader`]); the boundary frame's leftover
/// tail is recompressed, all later frames are copied byte-for-byte.
///
/// The EOF marker is held back (never written here): cat appends its own single
/// EOF after the last input, and reheader's caller writes the marker explicitly.
pub fn copy_records<R: BufRead, W: Write>(
    state: HeaderReader,
    reader: &mut R,
    out: &mut W,
) -> Result<()> {
    let HeaderReader {
        current,
        consumed,
        decoded,
    } = state;

    // Re-emit the boundary frame's uncompressed leftover (the record bytes after
    // the header within the frame that straddled the header/record boundary),
    // then raw-copy the rest. Mirrors samtools writing `block_offset..
    // block_length` via bgzf_write then `bgzf_raw_read`-ing onward. When the
    // header ended exactly on a frame boundary there is no leftover.
    if current.is_some() && consumed < decoded.len() {
        write_bgzf(out, &decoded[consumed..])?;
    }

    // Bulk-copy every remaining compressed byte (all alignment frames) without
    // parsing frame boundaries — the boundary parsing and per-frame allocation
    // was pure overhead on what is a byte-for-byte passthrough. Hold back a
    // 28-byte tail so the input's trailing BGZF EOF marker can be dropped (its
    // bytes are the last 28); samtools holds back `BGZF_EMPTY_BLOCK_SIZE` the
    // same way (bam_cat.c). A non-EOF tail (no trailing marker) is written out.
    copy_dropping_eof(reader, out)
}

/// Stream `reader` to `out` while holding the final 28 bytes back. At EOF the
/// held tail is dropped iff it equals [`BGZF_EOF`], else it is flushed. Writes
/// straight from the [`BufRead`] buffer — no intermediate read buffer, no
/// memmove — so each filled block reaches `out` with a single copy. The 28-byte
/// `carry` bridges the holdback across `fill_buf` refills.
fn copy_dropping_eof<R: BufRead, W: Write>(reader: &mut R, out: &mut W) -> Result<()> {
    const TAIL: usize = BGZF_EOF.len();

    // The carry holds the trailing ≤ TAIL bytes not yet known to be safe to
    // write (they might be the final EOF marker). It is always flushed before
    // the bytes that follow it, preserving order.
    let mut carry: Vec<u8> = Vec::with_capacity(TAIL);
    loop {
        let chunk_len = {
            let chunk = reader.fill_buf().map_err(RsomicsError::Io)?;
            if chunk.is_empty() {
                break;
            }
            let n = chunk.len();
            let total = carry.len() + n;
            if total <= TAIL {
                carry.extend_from_slice(chunk);
            } else {
                // Write all but the final TAIL bytes of (carry ++ chunk); keep
                // the last TAIL in `carry`.
                let safe = total - TAIL;
                let from_carry = carry.len().min(safe);
                if from_carry > 0 {
                    out.write_all(&carry[..from_carry])
                        .map_err(RsomicsError::Io)?;
                }
                let from_chunk = safe - from_carry;
                if from_chunk > 0 {
                    out.write_all(&chunk[..from_chunk])
                        .map_err(RsomicsError::Io)?;
                }
                carry.clear();
                carry.extend_from_slice(&chunk[from_chunk..]);
            }
            n
        };
        reader.consume(chunk_len);
    }
    if !(carry.len() == TAIL && carry == BGZF_EOF) {
        out.write_all(&carry).map_err(RsomicsError::Io)?;
    }
    Ok(())
}

/// Streams the uncompressed front of a BAM (the header region) out of a BGZF
/// file while retaining enough frame state to hand off the rest to
/// [`copy_records`] without inflating it.
///
/// The caller drains exactly the BAM header (magic + text + refs) via [`Read`],
/// then passes `self` to [`copy_records`], which re-emits the partially-consumed
/// boundary frame's tail and raw-copies the remaining compressed frames.
pub struct HeaderReader {
    /// The frame currently being drained, if any uncompressed bytes remain in it.
    current: Option<Frame>,
    /// Bytes already consumed from `decoded`.
    consumed: usize,
    /// Uncompressed content of `current`.
    decoded: Vec<u8>,
}

impl HeaderReader {
    pub fn new() -> Self {
        Self {
            current: None,
            consumed: 0,
            decoded: Vec::new(),
        }
    }
}

impl Default for HeaderReader {
    fn default() -> Self {
        Self::new()
    }
}

/// A [`Read`] adapter that pulls uncompressed bytes from a BGZF file frame by
/// frame, recording the live frame so the caller can switch to raw block copy
/// the instant the BAM header is fully read.
pub struct HeaderStream<'a, R: Read> {
    inner: &'a mut R,
    state: &'a mut HeaderReader,
}

impl<'a, R: Read> HeaderStream<'a, R> {
    pub fn new(inner: &'a mut R, state: &'a mut HeaderReader) -> Self {
        Self { inner, state }
    }
}

impl<R: Read> Read for HeaderStream<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.state.consumed >= self.state.decoded.len() {
            // Refill from the next non-empty frame.
            loop {
                let frame = match read_frame(self.inner)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
                {
                    Some(f) => f,
                    None => return Ok(0),
                };
                inflate_frame(&frame, &mut self.state.decoded)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                self.state.consumed = 0;
                self.state.current = Some(frame);
                if !self.state.decoded.is_empty() {
                    break;
                }
            }
        }
        let avail = &self.state.decoded[self.state.consumed..];
        let n = avail.len().min(buf.len());
        buf[..n].copy_from_slice(&avail[..n]);
        self.state.consumed += n;
        Ok(n)
    }
}

/// Read `buf` fully, tolerating a clean EOF before the first byte. Returns the
/// number of bytes read (0 = EOF at a frame boundary).
fn read_full<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(RsomicsError::Io(e)),
        }
    }
    Ok(filled)
}
