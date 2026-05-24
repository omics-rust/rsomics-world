//! Smoke tests for rsomics-derep-prefix.
//!
//! These tests verify basic functionality without requiring vsearch on PATH.

use rsomics_derep_prefix::derep_prefix;

fn run(input: &str, sizein: bool, minseqlength: usize) -> Vec<(String, u64)> {
    let mut reader = std::io::Cursor::new(input.as_bytes());
    let (clusters, _) = derep_prefix(&mut reader, sizein, minseqlength, 50000).unwrap();
    clusters
        .into_iter()
        .map(|c| (c.label, c.abundance))
        .collect()
}

#[test]
fn simple_prefix_merge() {
    let fa = "\
>long\n\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGGGG\n\
>prefix\n\
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGGGG\n";
    // Exact match: both collapse.
    let res = run(fa, false, 32);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].1, 2);
}

#[test]
fn prefix_shorter_merges_into_longer() {
    // short is a prefix of long; both >= minseqlength=1.
    let long = "ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT"; // 44
    let pref = "ACGTACGTACGTACGTACGTACGTACGTACGTACGT"; // 36 = prefix of long
    let fa = format!(">long\n{long}\n>pref\n{pref}\n");
    let res = run(&fa, false, 1);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].0, "long");
    assert_eq!(res[0].1, 2);
}

#[test]
fn minseqlength_filters_prefix() {
    // pref is < 32 nt → filtered; long survives alone.
    let long = "ACGTACGTACGTACGTACGTACGTACGTACGTACGT"; // 36
    let pref = "ACGTACGTACGTACGTACGT"; // 20, filtered
    let fa = format!(">long\n{long}\n>pref\n{pref}\n");
    let res = run(&fa, false, 32);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].0, "long");
    assert_eq!(res[0].1, 1); // pref was discarded, not merged
}

#[test]
fn u_equals_t_for_matching_case_preserved() {
    // RNA long (U), DNA prefix (T lowercase) — should match after normalisation.
    let long_rna = "AAACCCGUUUGGGAAACCCGUUUGGGAAACCCGUUUGGGAAACCCGUUUGGG"; // 51, U
    let pref_dna = "aaacccgtttgggaaacccgtttgggaaacccgtttggg"; // 39, lowercase T
    let fa = format!(">long_rna\n{long_rna}\n>pref_dna\n{pref_dna}\n");
    let res = run(&fa, false, 1);
    assert_eq!(res.len(), 1);
    // Representative is long_rna (longer); its label is "long_rna".
    assert_eq!(res[0].0, "long_rna");
    assert_eq!(res[0].1, 2);
}

#[test]
fn sizein_abundances_summed() {
    let fa = "\
>long;size=3\n\
GGGGAAAACCCCTTTTGGGGAAAACCCCTTTTGGGGAAAACCCCTTTTGGGGAAAACCCCTTTTGGGG\n\
>pref;size=2\n\
GGGGAAAACCCCTTTTGGGGAAAACCCCTTTTGGGG\n";
    // sizein: long=3, pref=2; pref merges in → total=5.
    let res = run(fa, true, 1);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].1, 5);
}

#[test]
fn sort_descending_abundance() {
    // Three independent sequences with abundances 1, 3, 2 → sorted 3,2,1.
    let a = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"; // 52
    let b = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC"; // 52
    let c = "TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT"; // 52
    let fa = format!(">a;size=1\n{a}\n>b;size=3\n{b}\n>c;size=2\n{c}\n");
    let res = run(&fa, true, 1);
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].1, 3);
    assert_eq!(res[1].1, 2);
    assert_eq!(res[2].1, 1);
}

#[test]
fn exact_duplicate_merged() {
    let seq = "ACGTACGTACGTACGTACGTACGTACGTACGTACGT"; // 36
    let fa = format!(">dup1\n{seq}\n>dup2\n{seq}\n>dup3\n{seq}\n");
    let res = run(&fa, false, 1);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].1, 3);
}

#[test]
fn empty_input_returns_empty() {
    let res = run("", false, 32);
    assert!(res.is_empty());
}

#[test]
fn independent_seqs_not_merged() {
    let fa = "\
>seqa\n\
ACGTACGTACGTACGTACGTACGTACGTACGTACGT\n\
>seqb\n\
TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT\n";
    let res = run(fa, false, 1);
    assert_eq!(res.len(), 2);
}
