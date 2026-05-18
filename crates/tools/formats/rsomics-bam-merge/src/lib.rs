use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use noodles::bam;
use noodles::sam;
use rsomics_common::{Result, RsomicsError};

struct HeapEntry {
    record: bam::Record,
    file_idx: usize,
    tid: Option<usize>,
    pos: Option<usize>,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.tid == other.tid && self.pos == other.pos
    }
}
impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.tid, other.tid) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => match b.cmp(&a) {
                Ordering::Equal => other.pos.cmp(&self.pos),
                o => o,
            },
        }
    }
}

fn tid(r: &bam::Record) -> Option<usize> {
    r.reference_sequence_id()
        .transpose()
        .ok()
        .flatten()
}

fn pos(r: &bam::Record) -> Option<usize> {
    r.alignment_start()
        .transpose()
        .ok()
        .flatten()
        .map(|p| p.get())
}

pub fn merge_bams(inputs: &[&Path], output: &mut dyn Write) -> Result<u64> {
    if inputs.is_empty() {
        return Err(RsomicsError::InvalidInput("no input files".into()));
    }

    let mut readers: Vec<bam::io::Reader<File>> = Vec::with_capacity(inputs.len());
    let mut headers: Vec<sam::Header> = Vec::with_capacity(inputs.len());

    for path in inputs {
        let mut r = File::open(path)
            .map(bam::io::Reader::new)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
        let h = r.read_header().map_err(RsomicsError::Io)?;
        headers.push(h);
        readers.push(r);
    }

    let merged_header = headers[0].clone();

    let mut writer = bam::io::Writer::new(output);
    writer
        .write_header(&merged_header)
        .map_err(RsomicsError::Io)?;

    let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::new();

    for (i, reader) in readers.iter_mut().enumerate() {
        if let Some(result) = reader.records().next() {
            let record = result.map_err(RsomicsError::Io)?;
            let t = tid(&record);
            let p = pos(&record);
            heap.push(HeapEntry {
                record,
                file_idx: i,
                tid: t,
                pos: p,
            });
        }
    }

    let mut count: u64 = 0;

    while let Some(entry) = heap.pop() {
        writer
            .write_record(&merged_header, &entry.record)
            .map_err(RsomicsError::Io)?;
        count += 1;

        if let Some(result) = readers[entry.file_idx].records().next() {
            let record = result.map_err(RsomicsError::Io)?;
            let t = tid(&record);
            let p = pos(&record);
            heap.push(HeapEntry {
                record,
                file_idx: entry.file_idx,
                tid: t,
                pos: p,
            });
        }
    }

    Ok(count)
}
