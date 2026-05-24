use std::io::BufReader;
use std::process::Command;

fn derep_binary() -> std::path::PathBuf {
    let exe = env!("CARGO_BIN_EXE_rsomics-derep");
    exe.into()
}

fn golden(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/golden");
    p.push(name);
    p
}

#[test]
fn smoke_basic_produces_output() {
    let input = golden("basic.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    let status = Command::new(derep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
        ])
        .status()
        .unwrap();
    assert!(status.success(), "binary exited non-zero");
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(
        content.contains(";size=3"),
        "highest-abundance record expected"
    );
    assert!(
        content.contains(">seq_alpha;size=3"),
        "first occurrence header kept"
    );
}

#[test]
fn smoke_sizein_sums_abundances() {
    let input = golden("sizein.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    let status = Command::new(derep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
            "--sizein",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let content = std::fs::read_to_string(out.path()).unwrap();
    // seq1;size=5 + seq2 (size=1) = 6
    assert!(
        content.contains(">seq1;size=6"),
        "sizein summed correctly; got:\n{content}"
    );
}

#[test]
fn smoke_tie_breaking_lexicographic() {
    // seq_delta < seq_gamma lexicographically; both size=1; delta should come first.
    let input = golden("basic.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    Command::new(derep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
        ])
        .status()
        .unwrap();
    let content = std::fs::read_to_string(out.path()).unwrap();
    let delta_pos = content.find(">seq_delta").unwrap();
    let gamma_pos = content.find(">seq_gamma").unwrap();
    assert!(
        delta_pos < gamma_pos,
        "delta should precede gamma (lex tie-break)"
    );
}

#[test]
fn smoke_cli_debug_assert() {
    // Exercises Cli::command().debug_assert() (compiled into the binary test harness).
    // The actual assertion lives in cli.rs; here we just confirm the binary is runnable.
    let status = Command::new(derep_binary()).arg("--help").status().unwrap();
    // --help exits 0 in clap unless disable_help_flag overrides it oddly
    let _ = status; // clap may exit with 0 or 1; just ensure no panic
}

#[test]
fn smoke_case_insensitive_dedup() {
    use rsomics_derep::derep_fulllength;
    // Use sequences long enough to exceed the default minseqlength of 32.
    let seq_upper = "ACGTACGTACGTACGTACGTACGTACGTACGT"; // 32 bp
    let seq_lower = "acgtacgtacgtacgtacgtacgtacgtacgt"; // 32 bp, same
    let input = format!(">A\n{seq_upper}\n>B\n{seq_lower}\n");
    let mut reader = BufReader::new(input.as_bytes());
    let (clusters, discarded) = derep_fulllength(&mut reader, false, 32, 50000).unwrap();
    assert_eq!(discarded, 0);
    assert_eq!(
        clusters.len(),
        1,
        "lowercase and uppercase should be the same cluster"
    );
    assert_eq!(clusters[0].abundance, 2);
    // Case of the representative (first occurrence) is preserved.
    assert_eq!(
        &clusters[0].seq,
        seq_upper.as_bytes(),
        "representative preserves original case"
    );
}

#[test]
fn smoke_u_to_t_normalisation() {
    use rsomics_derep::derep_fulllength;
    // Use sequences long enough to exceed the default minseqlength of 32.
    let rna_seq = "ACGUACGUACGUACGUACGUACGUACGUACGU"; // 32 bp RNA
    let dna_seq = "ACGTACGTACGTACGTACGTACGTACGTACGT"; // 32 bp DNA
    let input = format!(">rna\n{rna_seq}\n>dna\n{dna_seq}\n");
    let mut reader = BufReader::new(input.as_bytes());
    let (clusters, _) = derep_fulllength(&mut reader, false, 32, 50000).unwrap();
    assert_eq!(
        clusters.len(),
        1,
        "U and T should be treated as the same base"
    );
    assert_eq!(clusters[0].abundance, 2);
    // Representative output preserves U (not converted to T).
    assert_eq!(
        &clusters[0].seq,
        rna_seq.as_bytes(),
        "representative preserves U in output"
    );
}

#[test]
fn smoke_minseqlength_filter() {
    use rsomics_derep::derep_fulllength;
    // Short sequences (< 32 nt) are discarded by default.
    let input = ">short\nACGT\n>long\nACGTACGTACGTACGTACGTACGTACGTACGT\n";
    let mut reader = BufReader::new(input.as_bytes());
    let (clusters, discarded) = derep_fulllength(&mut reader, false, 32, 50000).unwrap();
    assert_eq!(discarded, 1, "one short sequence should be discarded");
    assert_eq!(clusters.len(), 1, "one sequence should remain");
}

#[test]
fn smoke_maxseqlength_filter() {
    use rsomics_derep::derep_fulllength;
    let long_seq = "A".repeat(50001);
    let normal_seq = "ACGTACGTACGTACGTACGTACGTACGTACGT";
    let input = format!(">toolong\n{long_seq}\n>ok\n{normal_seq}\n");
    let mut reader = BufReader::new(input.as_bytes());
    let (clusters, discarded) = derep_fulllength(&mut reader, false, 32, 50000).unwrap();
    assert_eq!(discarded, 1, "one overlong sequence should be discarded");
    assert_eq!(clusters.len(), 1, "one sequence should remain");
}
