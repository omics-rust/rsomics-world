#[cfg(feature = "fasta")]
pub mod fasta {
    pub use rsomics_fasta_index as index;
    pub use rsomics_fasta_locate as locate;
    pub use rsomics_fasta_n50 as n50;
    pub use rsomics_fasta_sliding as sliding;
    pub use rsomics_fasta_stats as stats;
    pub use rsomics_fasta_subseq as subseq;
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
    pub use rsomics_fastq_quality as quality;
    pub use rsomics_fastq_sample as sample;
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
    pub use rsomics_vcf_call as call;
    pub use rsomics_vcf_cnv as cnv;
    pub use rsomics_vcf_concat as concat;
    pub use rsomics_vcf_consensus as consensus;
    pub use rsomics_vcf_convert as convert;
    pub use rsomics_vcf_fill_tags as fill_tags;
    pub use rsomics_vcf_filter as filter;
    pub use rsomics_vcf_fixref as fixref;
    pub use rsomics_vcf_gtcheck as gtcheck;
    pub use rsomics_vcf_head as head;
    pub use rsomics_vcf_index as index;
    pub use rsomics_vcf_isec as isec;
    pub use rsomics_vcf_merge as merge;
    pub use rsomics_vcf_mpileup as mpileup;
    pub use rsomics_vcf_norm as norm;
    pub use rsomics_vcf_polysomy as polysomy;
    pub use rsomics_vcf_query as query;
    pub use rsomics_vcf_reheader as reheader;
    pub use rsomics_vcf_roh as roh;
    pub use rsomics_vcf_sample as sample;
    pub use rsomics_vcf_setgt as setgt;
    pub use rsomics_vcf_sort as sort;
    pub use rsomics_vcf_split as split;
    pub use rsomics_vcf_stats as stats;
    pub use rsomics_vcf_utils as utils;
    pub use rsomics_vcf_validate as validate;
    pub use rsomics_vcf_view as view;
}

#[cfg(feature = "bam")]
pub mod bam {
    pub use rsomics_bam_addreplacerg as addreplacerg;
    pub use rsomics_bam_ampliconclip as ampliconclip;
    pub use rsomics_bam_ampliconstats as ampliconstats;
    pub use rsomics_bam_bedcov as bedcov;
    pub use rsomics_bam_calmd as calmd;
    pub use rsomics_bam_cat as cat;
    pub use rsomics_bam_checksum as checksum;
    pub use rsomics_bam_collate as collate;
    pub use rsomics_bam_consensus as consensus;
    pub use rsomics_bam_coverage as coverage;
    pub use rsomics_bam_depad as depad;
    pub use rsomics_bam_depth as depth;
    pub use rsomics_bam_dict as dict;
    pub use rsomics_bam_fasta as fasta;
    pub use rsomics_bam_fixmate as fixmate;
    pub use rsomics_bam_flagstat as flagstat;
    pub use rsomics_bam_head as head;
    pub use rsomics_bam_idxstats as idxstats;
    pub use rsomics_bam_import as import;
    pub use rsomics_bam_index as index;
    pub use rsomics_bam_markdup as markdup;
    pub use rsomics_bam_merge as merge;
    pub use rsomics_bam_mpileup as mpileup;
    pub use rsomics_bam_phase as phase;
    pub use rsomics_bam_quickcheck as quickcheck;
    pub use rsomics_bam_reheader as reheader;
    pub use rsomics_bam_reset as reset;
    pub use rsomics_bam_samples as samples;
    pub use rsomics_bam_sort as sort;
    pub use rsomics_bam_split as split;
    pub use rsomics_bam_stats as stats;
    pub use rsomics_bam_targetcut as targetcut;
    pub use rsomics_bam_to_bed as to_bed;
    pub use rsomics_bam_to_fastq as to_fastq;
    pub use rsomics_bam_view as view;
    pub use rsomics_sam_to_bam as sam_to_bam;
}

#[cfg(feature = "bed")]
pub mod bed {
    pub use rsomics_bed_closest as closest;
    pub use rsomics_bed_cluster as cluster;
    pub use rsomics_bed_complement as complement;
    pub use rsomics_bed_count as count;
    pub use rsomics_bed_expand as expand;
    pub use rsomics_bed_fisher as fisher;
    pub use rsomics_bed_flank as flank;
    pub use rsomics_bed_getfasta as getfasta;
    pub use rsomics_bed_groupby as groupby;
    pub use rsomics_bed_intersect as intersect;
    pub use rsomics_bed_jaccard as jaccard;
    pub use rsomics_bed_len as len;
    pub use rsomics_bed_makewindows as makewindows;
    pub use rsomics_bed_map as map;
    pub use rsomics_bed_merge as merge;
    pub use rsomics_bed_midpoint as midpoint;
    pub use rsomics_bed_multiinter as multiinter;
    pub use rsomics_bed_overlap as overlap;
    pub use rsomics_bed_random as random;
    pub use rsomics_bed_reldist as reldist;
    pub use rsomics_bed_sample as sample;
    pub use rsomics_bed_shift as shift;
    pub use rsomics_bed_shuffle as shuffle;
    pub use rsomics_bed_slop as slop;
    pub use rsomics_bed_sort as sort;
    pub use rsomics_bed_spacing as spacing;
    pub use rsomics_bed_stats as stats;
    pub use rsomics_bed_subtract as subtract;
    pub use rsomics_bed_to_gff as to_gff;
    pub use rsomics_bed_total_bp as total_bp;
    pub use rsomics_bed_unionbedg as unionbedg;
    pub use rsomics_bed_unique as unique;
    pub use rsomics_bed_utils as utils;
    pub use rsomics_bed_validate as validate_bed;
    pub use rsomics_bed_window as window;
    pub use rsomics_bed12tobed6 as bed12tobed6;
}

#[cfg(feature = "gff")]
pub mod gff {
    pub use rsomics_gff_utils as utils;
}
