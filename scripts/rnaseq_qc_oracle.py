#!/usr/bin/env python3
"""Capture actual RSeQC5.0.5 five-metric workflows on informative BAM fixtures."""

import argparse
import csv
import hashlib
import importlib.metadata
import io
import json
import math
import os
from pathlib import Path
import platform
import re
import shutil
import sys
import sysconfig

from infercnv_trial_process import TrialInterrupted, TrialProcessError, run_child


SCRIPTS = {"mapping": "bam_stat.py", "strandedness": "infer_experiment.py",
           "distribution": "read_distribution.py", "coverage": "geneBody_coverage.py",
           "integrity": "tin.py"}
ELIGIBLE_TIN = ("plus", "minus", "entropy_positive", "eligible_zero")


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def query_length(cigar):
    return 50 if cigar == "*" else sum(int(size) for size, op in re.findall(r"(\d+)([MIDNSHP=X])", cigar)
                                      if op in "MIS=X")


def fixture_plan(profile):
    if profile not in ("single", "paired"):
        raise ValueError("unknown fixture layout")
    genes = [
        ("plus", 10000, 11200, "+", 10100, 11100, "400,400,", "0,800,"),
        ("minus", 40000, 41200, "-", 40100, 41100, "400,400,", "0,800,"),
        ("entropy_positive", 70000, 71200, "+", 70000, 71200, "100,100,", "0,1100,"),
        ("eligible_zero", 100000, 101200, "+", 100000, 101200, "100,100,", "0,1100,"),
        ("empty", 130000, 131200, "+", 130000, 131200, "400,400,", "0,800,"),
        ("ambiguous_plus", 160000, 160300, "+", 160000, 160300, "300,", "0,"),
        ("ambiguous_minus", 160200, 160500, "-", 160200, 160500, "300,", "0,"),
    ]
    records = []

    def add(name, start, cigar="75M", flag=0, mapq=60):
        records.append(dict(name=name, chromosome="chr1", start=start, cigar=cigar,
                            flag=flag, mapq=mapq, mate_start=None, template_length=0))

    for gene, start, reverse in (("plus", 10000, 0), ("minus", 40000, 16)):
        for exon in (0, 800):
            for index, offset in enumerate(range(5, 300, 25)):
                add(f"{gene}-{exon}-{index}", start + exon + offset, flag=reverse)
    for name, start, cigar in (
        ("boundary", 10370, "60M"), ("soft-clip", 10370, "30S50M"),
        ("eqx", 10110, "25=5X20="), ("splice", 10370, "30M400N40M"),
        ("insertion", 10070, "20M10I30M"), ("deletion", 10120, "20M10D30M"),
        ("hard-clip", 10160, "5H50M5H"), ("padding", 10200, "25M5P25M"),
        ("ambiguous", 160225, "50M"), ("intergenic", 180000, "50M"),
    ):
        add(name, start, cigar)
    for name, flag, mapq in (("qc-fail", 512, 60), ("duplicate", 1024, 60),
                             ("secondary", 256, 60), ("combined-flags", 1792, 60),
                             ("supplementary", 2048, 60), ("low-mapq", 0, 29)):
        add(name, 10250, "50M", flag, mapq)
    add("entropy-exon", 70000, "1M")
    for index in range(11):
        add(f"entropy-intron-{index}", 70200 + index * 3, "1M")
        add(f"zero-intron-{index}", 100200 + index * 3, "1M")
    add("unmapped", -1, "*", 4, 0)
    if profile == "paired":
        pairs = []
        for record in records:
            if record["flag"] & 4:
                pairs.append(record)
                continue
            reverse = bool(record["flag"] & 16)
            inherited = record["flag"] & ~16
            span = sum(int(size) for size, op in re.findall(r"(\d+)([MIDNSHP=X])", record["cigar"])
                       if op in "MDN=X")
            for ordinal, orientation in ((1, reverse), (2, not reverse)):
                flag = inherited | 3 | (64 if ordinal == 1 else 128)
                flag |= 16 if orientation else 32
                pairs.append(dict(record, flag=flag, mate_start=record["start"],
                                  template_length=-span if orientation else span))
        records = pairs
    bed = []
    for name, start, end, strand, thick_start, thick_end, sizes, offsets in genes:
        bed.append("\t".join(map(str, ("chr1", start, end, name, 0, strand, thick_start,
                                      thick_end, 0, len(sizes.rstrip(",").split(",")), sizes, offsets))))
    return dict(layout=profile, bed12=bed, records=records,
                intent=dict(eligible_TIN=list(ELIGIBLE_TIN),
                            entropy_positive="one sampled exonic base;12 distinct starts",
                            eligible_zero="11 distinct intronic starts, no sampled exonic coverage",
                            empty="no reads; excluded from sample-level TIN summary",
                            ambiguous="overlapping opposite-strand transcripts",
                            scope="synthetic semantic fixture, not representative RNA-seq or performance evidence"))


def build_fixture(directory, profile):
    import pysam

    plan = fixture_plan(profile)
    directory.mkdir(parents=True)
    bed, sam, bam = (directory / name for name in ("genes.bed12", f"{profile}.sam", f"{profile}.bam"))
    bed.write_text("\n".join(plan["bed12"]) + "\n")
    with sam.open("w") as output:
        output.write("@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:chr1\tLN:1000000\n")
        for record in plan["records"]:
            unmapped = bool(record["flag"] & 4)
            length = query_length(record["cigar"])
            mate = record["mate_start"]
            fields = (record["name"], record["flag"], "*" if unmapped else record["chromosome"],
                      record["start"] + 1, record["mapq"], record["cigar"],
                      "*" if mate is None else "=", 0 if mate is None else mate + 1,
                      record["template_length"], "A" * length, "I" * length)
            output.write("\t".join(map(str, fields)) + "\n")
    pysam.sort("-T", str(directory / "sort-tmp"), "-o", str(bam), str(sam))
    pysam.index(str(bam))
    with pysam.AlignmentFile(str(bam), "rb") as alignment:
        observed = sum(1 for _ in alignment.fetch(until_eof=True))
    if observed != len(plan["records"]):
        raise ValueError("generated BAM record count differs from the fixture plan")
    (directory / "fixture.json").write_text(json.dumps(plan, indent=2) + "\n")
    return bam, bed, observed


def number(value):
    result = float(value)
    if not math.isfinite(result):
        raise ValueError("nonfinite upstream numeric output")
    return result


def parse_coverage(text):
    rows = list(csv.reader(io.StringIO(text), delimiter="\t"))
    header = ["Percentile", *map(str, range(1, 101))]
    if len(rows) != 2 or rows[0] != header or len(rows[1]) != 101:
        raise ValueError("missing complete100-point actual gene-body coverage")
    values = [number(value) for value in rows[1][1:]]
    if min(values) < 0 or max(values) <= 0:
        raise ValueError("actual gene-body coverage has no positive signal")
    return dict(sample=rows[1][0], coverage=values)


def parse_integrity(tin, summary):
    rows = list(csv.DictReader(io.StringIO(tin), delimiter="\t"))
    scores = {row["geneID"]: number(row["TIN"]) for row in rows}
    if len(scores) != len(rows) or not set((*ELIGIBLE_TIN, "empty")).issubset(scores):
        raise ValueError("actual TIN transcript identities are missing or duplicated")
    summaries = list(csv.DictReader(io.StringIO(summary), delimiter="\t"))
    if len(summaries) != 1 or scores["entropy_positive"] <= 0 or scores["eligible_zero"] != 0:
        raise ValueError("actual TIN lacks the informative positive-entropy-zero/eligible-zero cases")
    observed = {key: number(summaries[0][column]) for key, column in
                (("mean", "TIN(mean)"), ("median", "TIN(median)"), ("stdev", "TIN(stdev)"))}
    expected = math.fsum(scores[name] for name in ELIGIBLE_TIN) / len(ELIGIBLE_TIN)
    if not math.isclose(observed["mean"], expected, rel_tol=1e-12, abs_tol=1e-12):
        raise ValueError("actual TIN summary does not include the eligible zero transcript")
    return dict(transcripts=rows, summary=observed, eligible_transcripts=list(ELIGIBLE_TIN))


def parse_stdout(metric, text, profile):
    if metric == "mapping":
        values = {match[1].strip(): int(match[2]) for line in text.splitlines()
                  if (match := re.fullmatch(r"(.+?(?:\s+|:))(\d+)\s*", line))}
        if values.get("Total records:", 0) <= 0 or "mapq >= mapq_cut (unique):" not in values:
            raise ValueError("missing informative actual mapping statistics")
        return values
    if metric == "strandedness":
        layout = "PairEnd" if profile == "paired" else "SingleEnd"
        fractions = {match[1]: number(match[2]) for line in text.splitlines()
                     if (match := re.fullmatch(r"(Fraction of reads .+):\s*(\S+)", line))}
        if f"This is {layout} Data" not in text or len(fractions) != 3:
            raise ValueError("actual library layout has no complete numeric strandedness result")
        if any(value < 0 or value > 1 for value in fractions.values()):
            raise ValueError("invalid actual strandedness fraction")
        return dict(layout=layout, fractions=fractions)
    totals, groups = {}, {}
    for line in text.splitlines():
        match = re.fullmatch(r"(Total Reads|Total Tags|Total Assigned Tags)\s+(\d+)\s*", line)
        if match:
            totals[match[1]] = int(match[2])
        fields = line.split()
        if len(fields) == 4 and fields[0] in ("CDS_Exons", "5'UTR_Exons", "3'UTR_Exons", "Introns",
                                             "TSS_up_1kb", "TSS_up_5kb", "TSS_up_10kb",
                                             "TES_down_1kb", "TES_down_5kb", "TES_down_10kb"):
            groups[fields[0]] = dict(bases=int(fields[1]), tags=int(fields[2]), tags_per_kb=number(fields[3]))
    if len(totals) != 3 or len(groups) != 10 or totals["Total Tags"] <= 0:
        raise ValueError("missing complete actual annotation distribution")
    return dict(totals=totals, groups=groups)


def run_profile(root, profile, tools):
    directory = root / "cases" / profile
    bam, bed, count = build_fixture(directory / "inputs", profile)
    outcomes = {}
    for metric, script in tools.items():
        own = directory / metric
        own.mkdir()
        arguments = ["-i", str(bam)]
        if metric != "mapping":
            arguments += ["-r", str(bed)]
        if metric in ("mapping", "strandedness"):
            arguments += ["-q", "30"]
        if metric == "strandedness":
            arguments += ["-s", "200000"]
        if metric == "coverage":
            arguments += ["-l", "100", "--skip-plot", "-o", str(own / "coverage")]
        if metric == "integrity":
            arguments += ["-c", "10", "-n", "100", "-o", str(own)]
        command = [sys.executable, str(script), *arguments]
        previous = Path.cwd()
        try:
            os.chdir(own)
            child = run_child(command, own / "process", timeout=300)
            if metric == "coverage":
                observed = parse_coverage((own / "coverage.geneBodyCoverage.txt").read_text())
            elif metric == "integrity":
                observed = parse_integrity((own / f"{profile}.tin.xls").read_text(),
                                           (own / f"{profile}.summary.txt").read_text())
            else:
                observed = parse_stdout(metric, (own / "process.stdout").read_text(), profile)
                if metric == "mapping" and observed["Total records:"] != count:
                    raise ValueError("mapping total differs from the actual generated BAM")
            (own / "observed.json").write_text(json.dumps(observed, indent=2) + "\n")
            outcomes[metric] = dict(status="success", argv=command, returncode=child.returncode,
                                    owner_wall_seconds=(child.end_ns - child.start_ns) / 1e9,
                                    observed=str((own / "observed.json").relative_to(root)))
        except TrialInterrupted:
            raise
        except (OSError, ValueError, KeyError, TrialProcessError) as error:
            outcomes[metric] = dict(status="failed", argv=command, error=str(error))
        finally:
            os.chdir(previous)
    return dict(status="complete" if all(row["status"] == "success" for row in outcomes.values())
                else "captured_with_failures", alignment_records=count, metrics=outcomes)


def capture(output, acquisition):
    output = Path(output).absolute()
    acquisition = Path(acquisition).resolve(strict=True)
    output.mkdir(parents=True, exist_ok=False)
    metadata = output / "metadata"
    metadata.mkdir()
    distribution = importlib.metadata.distribution("RSeQC")
    if distribution.version != "5.0.5":
        raise ValueError("installed RSeQC distribution is not the declared5.0.5 oracle")
    tools = {metric: Path(sysconfig.get_path("scripts")) / name for metric, name in SCRIPTS.items()}
    for metric, script in tools.items():
        if not script.is_file():
            raise ValueError(f"installed RSeQC script missing: {script}")
        shutil.copyfile(script, metadata / script.name)
    shutil.copyfile(acquisition, metadata / "acquisition.json")
    for name in (Path(__file__).name, "test_rnaseq_qc_oracle.py", "infercnv_trial_process.py"):
        shutil.copyfile(Path(__file__).with_name(name), metadata / name)
    import pysam

    provenance = dict(package="RSeQC", version=distribution.version, python=sys.version,
                      executable=sys.executable, machine=platform.uname()._asdict(),
                      pysam_version=pysam.__version__, samtools_version=pysam.__samtools_version__,
                      dependencies=sorted((item.metadata["Name"], item.version)
                                          for item in importlib.metadata.distributions()),
                      acquisition_sha256=sha256(acquisition),
                      scripts={metric: dict(path=str(script), sha256=sha256(script))
                               for metric, script in tools.items()})
    (metadata / "runtime.json").write_text(json.dumps(provenance, indent=2) + "\n")
    cases = {profile: run_profile(output, profile, tools) for profile in ("single", "paired")}
    failed = [name for name, case in cases.items() if case["status"] != "complete"]
    files = {str(path.relative_to(output)): sha256(path)
             for path in sorted(output.rglob("*")) if path.is_file()}
    witness = dict(schema_version=1, status="captured_with_failures" if failed else "complete",
                   scope="actual installed five-metric synthetic semantics; not native acceptance or representative performance",
                   source_commit="0d2e017723c5c4b41eb3b938e1631692d187de1f", package=provenance,
                   failed_cases=failed, cases=cases, files=files)
    target = output / "witness.json"
    target.write_text(json.dumps(witness, indent=2) + "\n")
    files["witness.json"] = sha256(target)
    (output / "sha256-manifest.tsv").write_text("sha256\tpath\n" + "".join(
        f"{digest}\t{name}\n" for name, digest in sorted(files.items())))
    return witness


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--acquisition", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = capture(args.output, args.acquisition)
    except (OSError, ValueError, importlib.metadata.PackageNotFoundError) as error:
        parser.exit(1, f"RNA-seq QC oracle failed: {error}\n")
    if result["failed_cases"]:
        parser.exit(1, "actual source failures retained: " + ", ".join(result["failed_cases"]) + "\n")
    print("Captured actual RSeQC5.0.5 five-metric workflows for single/paired BAM fixtures")


if __name__ == "__main__":
    main()
