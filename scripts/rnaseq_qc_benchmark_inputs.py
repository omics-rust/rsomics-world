#!/usr/bin/env python3
"""Acquire and qualify the declared real RNA-seq benchmark profiles."""

import argparse
from collections import Counter
from contextlib import nullcontext
from datetime import datetime, timezone
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import urllib.request


BAM_URL = "https://data.cyverse.org/dav-anon/iplant/home/liguow/RSeQC/Pairend_nonStrandSpecific_36mer_Human_hg19.bam"
BED_URL = "https://master.dl.sourceforge.net/project/rseqc/BED/Human_Homo_sapiens/hg19_RefSeq.bed.gz"
BAM_BYTES = 437662663
BAM_MD5 = "ba014f6b397b8a29c456b744237a12de"
BED_BYTES = 2567192
BED_MD5 = "b7df344f3787415eaa4f24c23f498632"
BED_SHA256 = "eb7070cfb52b9011c132346fd27baffb8a7acf3c034ebf996cebd52fbb52aa19"
CIGAR_OPERATIONS = "MIDNSHP=XB"


def write_payload(source, output, expected_bytes, expected_md5, expected_sha256=None):
    output = Path(output)
    size, md5, sha256 = 0, hashlib.md5(), hashlib.sha256()
    with output.open("xb") as destination:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            size += len(block)
            if size > expected_bytes:
                raise ValueError(f"download exceeds declared size: {size} > {expected_bytes}")
            destination.write(block)
            md5.update(block)
            sha256.update(block)
    result = dict(path=str(output), bytes=size, md5=md5.hexdigest(), sha256=sha256.hexdigest())
    if size != expected_bytes or result["md5"] != expected_md5 or (
            expected_sha256 is not None and result["sha256"] != expected_sha256):
        raise ValueError(f"download identity differs: {result}")
    return result


def file_identity(path):
    size, md5, sha256 = 0, hashlib.md5(), hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            size += len(block)
            md5.update(block)
            sha256.update(block)
    return dict(path=str(path), bytes=size, md5=md5.hexdigest(), sha256=sha256.hexdigest())


def save_json(path, value):
    with Path(path).open("x", encoding="utf-8") as destination:
        json.dump(value, destination, indent=2, sort_keys=True)
        destination.write("\n")


def download(url, output, expected_bytes, expected_md5, expected_sha256=None):
    record = dict(requested_url=url, acquired_at=datetime.now(timezone.utc).isoformat(),
                  identity_verified=False)
    try:
        request = urllib.request.Request(url, headers={"Accept-Encoding": "identity"})
        with urllib.request.urlopen(request, timeout=120) as response:
            record.update(resolved_url=response.geturl(), status=response.status,
                          headers={key: response.headers.get(key) for key in (
                              "Content-Length", "Content-Range", "Content-Type", "Content-Encoding",
                              "Last-Modified", "ETag", "Date")})
            if response.status != 200 or response.headers.get("Content-Range") is not None:
                raise ValueError("complete original requires HTTP200 without Content-Range")
            length = response.headers.get("Content-Length")
            if length is not None and int(length) != expected_bytes:
                raise ValueError(f"HTTP Content-Length differs: {length} != {expected_bytes}")
            record.update(write_payload(response, output, expected_bytes, expected_md5, expected_sha256))
            record["identity_verified"] = True
    except (OSError, ValueError) as error:
        record["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        save_json(Path(str(output) + ".http.json"), record)
    return record


def derive_annotation(original, references, genes, exclusions, profile):
    if profile not in ("chr21", "whole"):
        raise ValueError(f"unknown benchmark profile: {profile}")
    source_counts, selected_counts, excluded_counts = Counter(), Counter(), Counter()
    names, mrna_lengths, strands, multi_exon = Counter(), Counter(), Counter(), 0
    with gzip.open(original, "rb") as source, Path(genes).open("xb") as selected, \
            Path(exclusions).open("x", encoding="utf-8") as excluded:
        for number, line in enumerate(source, 1):
            if not line.strip() or line.startswith((b"#", b"track", b"browser")):
                continue
            fields = line.decode("utf-8").split()
            if len(fields) < 12:
                raise ValueError(f"BED12 line {number} has fewer than12 columns")
            chromosome, start, end = fields[0], int(fields[1]), int(fields[2])
            sizes = [int(value) for value in fields[10].rstrip(",").split(",")]
            offsets = [int(value) for value in fields[11].rstrip(",").split(",")]
            blocks = int(fields[9])
            if not 0 <= start < end or fields[5] not in ("+", "-") or \
                    not start <= int(fields[6]) <= int(fields[7]) <= end or \
                    blocks <= 0 or blocks != len(sizes) or blocks != len(offsets):
                raise ValueError(f"invalid BED12 geometry at line {number}")
            exons = [(start + offset, start + offset + size) for offset, size in zip(offsets, sizes)]
            if any(size <= 0 or offset < 0 for size, offset in zip(sizes, offsets)) or \
                    exons[0][0] != start or exons[-1][1] != end or \
                    any(left[1] > right[0] for left, right in zip(exons, exons[1:])):
                raise ValueError(f"invalid BED12 exons at line {number}")
            source_counts[chromosome] += 1
            if chromosome in references and end > references[chromosome]:
                raise ValueError(f"BED12 line {number} exceeds BAM reference bounds: {chromosome}:{end}")
            reason = "reference_absent_from_bam" if chromosome not in references else (
                "outside_chr21_profile" if profile == "chr21" and chromosome != "chr21" else None)
            if reason is not None:
                excluded_counts[reason] += 1
                excluded.write(json.dumps(dict(source_line=number, chromosome=chromosome, reason=reason,
                                               row=line.decode("utf-8").rstrip("\r\n"))) + "\n")
                continue
            selected.write(line)
            selected_counts[chromosome] += 1
            names[fields[3]] += 1
            mrna_lengths[sum(sizes)] += 1
            strands[fields[5]] += 1
            multi_exon += blocks > 1
    return dict(source_transcripts=sum(source_counts.values()), source_contigs=dict(sorted(source_counts.items())),
                selected_transcripts=sum(selected_counts.values()), selected_contigs=dict(sorted(selected_counts.items())),
                excluded_transcripts=sum(excluded_counts.values()), exclusion_reasons=dict(sorted(excluded_counts.items())),
                distinct_names=len(names), duplicate_name_rows=sum(value - 1 for value in names.values()),
                mrna_length_at_least_100=sum(count for length, count in mrna_lengths.items() if length >= 100),
                strands=dict(sorted(strands.items())), multi_exon=multi_exon,
                annotation=file_identity(genes), exclusions=file_identity(exclusions))


def empty_record_counts():
    return dict(records=0, references=Counter(), flags=Counter(), query_lengths=Counter(),
                cigar_operations=Counter(), cigar_bases=Counter())


def observe_record(counts, record, chromosome):
    counts["records"] += 1
    counts["references"][chromosome] += 1
    counts["flags"][str(record.flag)] += 1
    counts["query_lengths"][str(record.query_length)] += 1
    for operation, length in record.cigartuples or ():
        name = CIGAR_OPERATIONS[operation]
        counts["cigar_operations"][name] += 1
        counts["cigar_bases"][name] += length


def ordered_counts(counts):
    return {name: dict(sorted(value.items())) if isinstance(value, Counter) else value
            for name, value in counts.items()}


def prepare_inputs(output: Path, profile: str) -> dict:
    if profile not in ("chr21", "whole"):
        raise ValueError(f"unknown benchmark profile: {profile}")
    output = Path(output).resolve()
    runner_temp = os.environ.get("RUNNER_TEMP")
    if platform.system() != "Linux" or os.environ.get("GITHUB_ACTIONS") != "true" or not runner_temp or \
            not output.is_relative_to(Path(runner_temp).resolve(strict=True)):
        raise ValueError("complete real inputs must be prepared under hosted Linux RUNNER_TEMP")
    import pysam

    output.mkdir(parents=True, exist_ok=False)
    originals = output / "originals"
    originals.mkdir()
    original_bam, original_bed = originals / "official.bam", originals / "hg19_RefSeq.bed.gz"
    bam_receipt = download(BAM_URL, original_bam, BAM_BYTES, BAM_MD5)
    bed_receipt = download(BED_URL, original_bed, BED_BYTES, BED_MD5, BED_SHA256)
    bam, genes = output / "paired.bam", output / "genes.bed12"
    source_counts, selected_counts = empty_record_counts(), empty_record_counts()
    previous = None
    with pysam.AlignmentFile(str(original_bam), "rb") as source:
        references = dict(zip(source.references, source.lengths))
        if len(references) != 25 or references.get("chr21") != 48129895:
            raise ValueError("official BAM reference dictionary differs from qualified header")
        header = output / "bam-header.sam"
        with header.open("x", encoding="utf-8") as destination:
            destination.write(str(source.header))
        selection = pysam.AlignmentFile(str(bam), "wb", template=source) if profile == "chr21" else nullcontext()
        with selection as selected:
            for record in source.fetch(until_eof=True):
                chromosome = source.get_reference_name(record.reference_id) if record.reference_id >= 0 else "*"
                if record.reference_id >= 0 and record.reference_start >= 0:
                    coordinate = record.reference_id, record.reference_start
                    if previous is not None and coordinate < previous:
                        raise ValueError(f"official BAM is not coordinate-monotonic: {coordinate} < {previous}")
                    previous = coordinate
                observe_record(source_counts, record, chromosome)
                if profile == "whole" or chromosome == "chr21":
                    observe_record(selected_counts, record, chromosome)
                    if selected is not None:
                        selected.write(record)
    save_json(output / "record-counts.json", dict(source=ordered_counts(source_counts),
                                                selected=ordered_counts(selected_counts)))
    if selected_counts["records"] <= 10000:
        raise ValueError("declared profile has no nontrivial real-data population (>10000 records required)")
    if profile == "whole":
        os.link(original_bam, bam)
        bam_identity = dict(bam_receipt, path=str(bam))
    else:
        bam_identity = file_identity(bam)
    annotation = derive_annotation(original_bed, references, genes, output / "annotation-exclusions.jsonl", profile)
    expected_rows = 754 if profile == "chr21" else 58384
    if annotation["source_transcripts"] != 61177 or annotation["selected_transcripts"] != expected_rows or \
            annotation["exclusion_reasons"].get("reference_absent_from_bam") != 2793:
        raise ValueError(f"official annotation qualification differs: {annotation}")
    pysam.index(str(original_bam))
    original_index = Path(str(original_bam) + ".bai")
    if profile == "whole":
        os.link(original_index, Path(str(bam) + ".bai"))
    else:
        pysam.index(str(bam))
    result = dict(schema_version=1, profile=profile, bam=str(bam), genes=str(genes),
                  records=selected_counts["records"], source=dict(bam=bam_receipt, annotation_gzip=bed_receipt,
                  records=ordered_counts(source_counts), bam_index=file_identity(original_index)),
                  derivative=dict(bam=bam_identity, annotation=annotation,
                  records=ordered_counts(selected_counts), bam_index=file_identity(Path(str(bam) + ".bai"))),
                  header=dict(file=file_identity(header), references=references),
                  record_selection="all records unchanged" if profile == "whole" else "all exact chr21 records; no flag filtering",
                  coordinate_check="nondecreasing reference-id/start for all records with assigned coordinates",
                  preparation_scope="untimed acquisition/indexing/qualification; not a benchmark or eligibility claim")
    save_json(output / "qualification.json", result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--profile", choices=("chr21", "whole"), required=True)
    arguments = parser.parse_args()
    print(json.dumps(prepare_inputs(arguments.output, arguments.profile), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
