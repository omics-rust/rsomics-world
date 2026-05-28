#[cfg(feature = "fasta")]
pub mod fasta {
    pub use rsomics_fasta_index as index;
    pub use rsomics_fasta_n50 as n50;
    pub use rsomics_fasta_stats as stats;
    pub use rsomics_fasta_translate as translate;
    pub use rsomics_fasta_utils as utils;
    pub use rsomics_fasta_validate as validate;
}

#[cfg(feature = "fastq")]
pub mod fastq {
    pub use rsomics_fastq_complexity as complexity;
    pub use rsomics_fastq_correct as correct;
    pub use rsomics_fastq_dedup as dedup;
    pub use rsomics_fastq_filter as filter;
    pub use rsomics_fastq_merge as merge;
    pub use rsomics_fastq_pair as pair;
    pub use rsomics_fastq_split as split;
    pub use rsomics_fastq_stats as stats;
    pub use rsomics_fastq_trim as trim;
    pub use rsomics_fastq_umi as umi;
    pub use rsomics_fastq_utils as utils;
    pub use rsomics_fastq_validate as validate;
    pub use rsomics_fastqc as qc;
}

#[cfg(feature = "vcf")]
pub mod vcf {
    pub use rsomics_vcf_annotate as annotate;
    pub use rsomics_vcf_filter as filter;
    pub use rsomics_vcf_isec as isec;
    pub use rsomics_vcf_merge as merge;
    pub use rsomics_vcf_norm as norm;
    pub use rsomics_vcf_query as query;
    pub use rsomics_vcf_split as split;
    pub use rsomics_vcf_stats as stats;
    pub use rsomics_vcf_utils as utils;
    pub use rsomics_vcf_validate as validate;
}

#[cfg(feature = "bam")]
pub mod bam {
    pub use rsomics_bam_coverage as coverage;
    pub use rsomics_bam_depth as depth;
    pub use rsomics_bam_flagstat as flagstat;
    pub use rsomics_bam_idxstats as idxstats;
    pub use rsomics_bam_index as index;
    pub use rsomics_bam_markdup as markdup;
    pub use rsomics_bam_merge as merge;
    pub use rsomics_bam_sort as sort;
    pub use rsomics_bam_split as split;
    pub use rsomics_bam_stats as stats;
    pub use rsomics_bam_to_fastq as to_fastq;
    pub use rsomics_bam_view as view;
    pub use rsomics_sam_to_bam as sam_to_bam;
}

#[cfg(feature = "bed")]
pub mod bed {
    pub use rsomics_bed_closest as closest;
    pub use rsomics_bed_complement as complement;
    pub use rsomics_bed_flank as flank;
    pub use rsomics_bed_getfasta as getfasta;
    pub use rsomics_bed_intersect as intersect;
    pub use rsomics_bed_jaccard as jaccard;
    pub use rsomics_bed_map as map;
    pub use rsomics_bed_merge as merge;
    pub use rsomics_bed_shift as shift;
    pub use rsomics_bed_slop as slop;
    pub use rsomics_bed_sort as sort;
    pub use rsomics_bed_subtract as subtract;
    pub use rsomics_bed_utils as utils;
    pub use rsomics_bed_window as window;
}

#[cfg(feature = "gff")]
pub mod gff {
    pub use rsomics_gff_utils as utils;
}
