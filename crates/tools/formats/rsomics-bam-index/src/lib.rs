use std::fs::File;
use std::io;
use std::path::Path;

use noodles::bam;
use noodles::bam::bai;
use noodles::bgzf;
use noodles::core::Position;
use noodles::csi::binning_index::index::reference_sequence::bin::Chunk;
use noodles::csi::binning_index::index::reference_sequence::index::LinearIndex;
use noodles::csi::binning_index::Indexer;
use rsomics_common::{Result, RsomicsError};

pub fn index_bam(bam_path: &Path) -> Result<()> {
    let bai_path = bam_path.with_extension("bam.bai");

    let mut reader = File::open(bam_path)
        .map(bgzf::Reader::new)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bam_path.display())))?;

    let header = {
        let mut bam_reader = bam::io::Reader::from(reader);
        let h = bam_reader
            .read_header()
            .map_err(|e| RsomicsError::InvalidInput(format!("header: {e}")))?;
        reader = bam_reader.into_inner();
        h
    };

    let n_refs = header.reference_sequences().len();
    let mut indexer = Indexer::<LinearIndex>::new(14, 5);

    loop {
        let start_vpos = reader.virtual_position();
        let mut record = bam::Record::default();
        match bam::io::Reader::from(&mut reader).read_record(&mut record) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => return Err(RsomicsError::InvalidInput(format!("reading record: {e}"))),
        }
        let end_vpos = reader.virtual_position();
        let chunk = Chunk::new(start_vpos, end_vpos);

        let context = record
            .reference_sequence_id()
            .and_then(|r| r.ok())
            .map(|ref_id| {
                let start = record
                    .alignment_start()
                    .and_then(|r| r.ok())
                    .unwrap_or(Position::MIN);
                let end = record
                    .alignment_end()
                    .and_then(|r| r.ok())
                    .unwrap_or(start);
                let is_mapped = !record.flags().is_unmapped();
                (ref_id, start, end, is_mapped)
            });

        indexer
            .add_record(context, chunk)
            .map_err(|e| RsomicsError::InvalidInput(format!("indexing: {e}")))?;
    }

    let index = indexer.build(n_refs);
    bai::fs::write(&bai_path, &index)
        .map_err(|e| RsomicsError::InvalidInput(format!("writing index: {e}")))?;

    Ok(())
}
