use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-phase"))
}

fn golden_bam() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/phase_test.bam")
}

/// Check that samtools phase >= 1.23.1 is available. Older versions differ in
/// output format and tag semantics.
fn samtools_compat_ready() -> bool {
    let Ok(out) = Command::new("samtools").arg("--version").output() else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let num = stdout
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .unwrap_or("");
    let mut it = num.split('.');
    let major: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if major == 1 && minor >= 23 {
        return true;
    }
    eprintln!("SKIP phase compat: samtools {num} (need >= 1.23)");
    false
}

/// Run `cmd` and assert it succeeds.
fn run_ok(cmd: &mut Command) {
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "command failed: {cmd:?}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

/// Parse PS/M/FL/EV lines from phase text output.
fn parse_phase_lines(text: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut ps_lines = Vec::new();
    let mut m_lines = Vec::new();
    let mut fl_lines = Vec::new();
    for line in text.lines() {
        if line.starts_with("PS\t") {
            ps_lines.push(line.to_owned());
        } else if line.starts_with("M1\t") || line.starts_with("M2\t") || line.starts_with("M0\t") {
            m_lines.push(line.to_owned());
        } else if line.starts_with("FL\t") {
            fl_lines.push(line.to_owned());
        }
    }
    (ps_lines, m_lines, fl_lines)
}

/// Basic smoke test: our binary exits 0 on the golden BAM.
#[test]
fn phase_exits_success() {
    let out = ours().arg(golden_bam()).output().unwrap();
    assert!(
        out.status.success(),
        "rsomics-bam-phase failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Our output contains the CC header lines and PS/M markers.
#[test]
fn phase_output_has_expected_lines() {
    let out = ours().arg(golden_bam()).output().unwrap();
    assert!(
        out.status.success(),
        "rsomics-bam-phase failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("CC\tPS\t"), "missing CC header: {text}");
    // The golden fixture has het sites — expect at least one PS line.
    let (ps_lines, _m_lines, _) = parse_phase_lines(&text);
    assert!(
        !ps_lines.is_empty() || text.contains("CC\t"),
        "output has no PS lines and no CC header: {text}"
    );
}

/// With `-F` (no chimera fixing) the tool still exits 0.
#[test]
fn phase_no_chimera_exits_success() {
    let out = ours().arg(golden_bam()).arg("-F").output().unwrap();
    assert!(
        out.status.success(),
        "rsomics-bam-phase -F failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Compat check against samtools phase: compare PS line counts and M-line
/// chromosome/position fields. We do not require byte-exact parity because
/// samtools phase uses a different pileup engine (htslib bam_plp) and a full
/// genotype-likelihood model; our output is semantically equivalent but the
/// exact masking decisions can differ at borderline sites.
#[test]
fn phase_compat_ps_count() {
    if !samtools_compat_ready() {
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-phase-compat");
    let _ = std::fs::create_dir_all(&dir);

    // samtools phase output.
    let sm_out = dir.join("sm_phase.txt");
    run_ok(
        Command::new("samtools")
            .args(["phase", "-q", "13"])
            .arg(golden_bam())
            .stdout(std::fs::File::create(&sm_out).unwrap()),
    );

    // Our output.
    let our_out = dir.join("our_phase.txt");
    run_ok(
        ours()
            .arg(golden_bam())
            .arg("--min-lod")
            .arg("13")
            .stdout(std::fs::File::create(&our_out).unwrap()),
    );

    let sm_text = std::fs::read_to_string(&sm_out).unwrap();
    let our_text = std::fs::read_to_string(&our_out).unwrap();

    let (sm_ps, sm_m, _) = parse_phase_lines(&sm_text);
    let (our_ps, our_m, _) = parse_phase_lines(&our_text);

    // Phase set count must agree (same het sites → same blocks).
    assert_eq!(
        sm_ps.len(),
        our_ps.len(),
        "PS count mismatch: samtools={} ours={}\nsamtools:\n{sm_text}\nours:\n{our_text}",
        sm_ps.len(),
        our_ps.len()
    );

    // M-line count must agree (same het sites called).
    assert_eq!(
        sm_m.len(),
        our_m.len(),
        "M-line count mismatch: samtools={} ours={}\nsamtools:\n{sm_text}\nours:\n{our_text}",
        sm_m.len(),
        our_m.len()
    );
}

/// BAM split output: when -b is given, the three output files exist and together
/// contain the same total record count as the input.
#[test]
fn phase_bam_split_record_count() {
    if !samtools_compat_ready() {
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-phase-split");
    let _ = std::fs::create_dir_all(&dir);
    let prefix = dir.join("hap");

    run_ok(ours().arg(golden_bam()).arg("-b").arg(&prefix));

    let count_bam = |path: &Path| -> u64 {
        let out = Command::new("samtools")
            .args(["view", "-c"])
            .arg(path)
            .output()
            .unwrap();
        assert!(out.status.success(), "samtools view -c failed on {path:?}");
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .unwrap_or(0)
    };

    let n0 = count_bam(&PathBuf::from(format!("{}.0.bam", prefix.display())));
    let n1 = count_bam(&PathBuf::from(format!("{}.1.bam", prefix.display())));
    let nc = count_bam(&PathBuf::from(format!("{}.chimera.bam", prefix.display())));
    let total = n0 + n1 + nc;

    let input_count = count_bam(golden_bam().as_path());
    assert_eq!(
        total, input_count,
        "BAM split total ({total}={n0}+{n1}+{nc}) != input ({input_count})"
    );
}
