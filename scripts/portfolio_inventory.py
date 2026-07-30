#!/usr/bin/env python3
"""Generate a provisional rsomics crate portfolio ledger from local clones.

Mechanical columns come directly from Cargo manifests and repository contents.
Routing columns are deliberately labelled provisional: they are a first-pass
triage for human review, not an archive or migration decision.
"""

from __future__ import annotations

import argparse
import csv
import re
import tomllib
from collections import Counter, defaultdict
from pathlib import Path


DEFAULT_CLONES = Path("/Volumes/KIOXIA/Documents/omics-rust")
DEFAULT_OUTPUT = Path("docs/00-overview/portfolio-inventory.tsv")


UPSTREAM_PATTERNS: list[tuple[str, str]] = [
    ("bbduk", r"\bbbduk\b|\bbbtools\b"),
    ("featurecounts", r"\bfeaturecounts\b|\bsubread\b"),
    ("infercnv", r"\binfercnv\b"),
    ("freesasa", r"\bfreesasa\b"),
    ("liftover", r"\bliftover\b|\bucsc chain\b"),
    ("matrixstats", r"\bmatrixstats\b"),
    ("trimal", r"\btrimal\b"),
    ("dendropy", r"\bdendropy\b"),
    ("qvalue", r"\bbioconductor qvalue\b|\bstorey'?s q-values?\b"),
    ("tabix", r"\btabix\b"),
    ("tmalign", r"\btm-?align\b|\btmalign\b"),
    ("scikit-allel", r"\bscikit[- ]allel\b"),
    ("samtools", r"\bsamtools\b"),
    ("bcftools", r"\bbcftools\b"),
    ("bedtools", r"\bbedtools\b"),
    ("seqkit", r"\bseqkit\b"),
    ("plink", r"\bplink(?:\s*2)?\b"),
    ("scanpy", r"\bscanpy\b"),
    ("rseqc", r"\brseqc\b"),
    ("deseq2", r"\bdeseq2\b"),
    ("edger", r"\bedger\b"),
    ("limma", r"\blimma\b"),
    ("deeptools", r"\bdeeptools\b"),
    ("scipy", r"\bscipy\b"),
    ("networkx", r"\bnetworkx\b"),
    ("scikit-bio", r"\bscikit[- ]bio\b|\bskbio\b"),
    ("statsmodels", r"\bstatsmodels\b"),
    ("scikit-learn", r"\bscikit[- ]learn\b|\bsklearn\b"),
    ("scikit-image", r"\bscikit[- ]image\b|\bskimage\b"),
    ("biopython", r"\bbiopython\b|\bbio\."),
    ("vcftools", r"\bvcftools\b"),
    ("fastp", r"\bfastp\b"),
    ("csvtk", r"\bcsvtk\b"),
    ("emboss", r"\bemboss\b"),
    ("datamash", r"\bdatamash\b"),
    ("picard", r"\bpicard\b"),
    ("fastqc", r"\bfastqc\b"),
    ("macs", r"\bmacs(?:2|3)?\b"),
    ("methyldackel", r"\bmethyldackel\b"),
    ("minimap2", r"\bminimap2\b"),
    ("vsearch", r"\bvsearch\b"),
    ("sourmash", r"\bsourmash\b"),
    ("skani", r"\bskani\b"),
]


def read_manifest(path: Path) -> tuple[dict, str]:
    try:
        with path.open("rb") as handle:
            return tomllib.load(handle), ""
    except (OSError, tomllib.TOMLDecodeError) as error:
        return {}, str(error).replace("\t", " ").replace("\n", " ")


def dependency_names(value: object) -> set[str]:
    found: set[str] = set()
    if not isinstance(value, dict):
        return found
    for key, child in value.items():
        if key in {"dependencies", "dev-dependencies", "build-dependencies"}:
            if isinstance(child, dict):
                found.update(str(name).replace("_", "-") for name in child)
        elif isinstance(child, dict):
            found.update(dependency_names(child))
    return found


def text_for(crate_dir: Path, manifest: dict) -> str:
    pieces = [str(manifest.get("package", {}).get("description", ""))]
    for filename in ("README.md", "Cargo.toml"):
        path = crate_dir / filename
        if path.is_file():
            pieces.append(path.read_text(errors="replace"))
    tests = crate_dir / "tests"
    if tests.is_dir():
        for path in tests.glob("*.rs"):
            pieces.append(path.read_text(errors="replace"))
    return "\n".join(pieces).lower()


def detect_upstreams(text: str) -> list[str]:
    return [name for name, pattern in UPSTREAM_PATTERNS if re.search(pattern, text)]


def contains_any(name: str, words: set[str]) -> bool:
    tokens = set(name.removeprefix("rsomics-").split("-"))
    return bool(tokens & words)


def route(name: str, upstreams: list[str], layer: str) -> tuple[str, str, str]:
    if layer == "A":
        return "foundation", name, "high"

    primary = upstreams[0] if upstreams else ""
    suffix = name.removeprefix("rsomics-")

    explicit_routes = {
        "rsomics-bed-expand": ("tabular", "rsomics-table"),
        "rsomics-bed-groupby": ("tabular", "rsomics-table"),
        "rsomics-bed-maskfasta": ("intervals", "rsomics-bed"),
        "rsomics-cell-filter": ("single-cell", "rsomics-sc"),
        "rsomics-count-matrix": ("bulk-expression", "rsomics-expression"),
        "rsomics-de-volcano": ("bulk-expression", "rsomics-expression"),
        "rsomics-fasta-index": ("sequence-indexing", "rsomics-index"),
        "rsomics-fasta-mask": ("intervals", "rsomics-bed"),
        "rsomics-fm-search": ("sequence-indexing", "rsomics-index"),
        "rsomics-gc-windows": ("sequence-utilities", "rsomics-seq"),
        "rsomics-hmm-decode": ("sequence-models", "rsomics-model"),
        "rsomics-kmer-dist": ("metagenomics", "rsomics-sketch"),
        "rsomics-nj-tree": ("phylogenetics", "rsomics-phylo"),
        "rsomics-pvalue-adjust": ("statistics", "rsomics-stats"),
        "rsomics-sample-sheet": ("workflow-utilities", "rsomics-workflow"),
        "rsomics-seacr": ("epigenomics", "rsomics-peak"),
        "rsomics-upgma": ("phylogenetics", "rsomics-phylo"),
        "rsomics-windowed-ld": ("population-genetics", "rsomics-popgen"),
    }
    if name in explicit_routes:
        area, target = explicit_routes[name]
        return area, target, "medium"

    upstream_set = set(upstreams)

    if suffix.startswith("peak-") or suffix in {"find-peaks", "macs"}:
        return "epigenomics", "rsomics-peak", "high"
    if "rseqc" in upstream_set:
        return "transcriptomics-qc", "rsomics-rnaseq-qc", "high"
    if "deeptools" in upstream_set:
        return "epigenomic-signal", "rsomics-signal", "high"
    if "picard" in upstream_set:
        return "transcriptomics-qc", "rsomics-rnaseq-qc", "medium"
    if "bbduk" in upstream_set:
        return "read-preprocessing", "rsomics-fastq-preprocess", "high"
    if "fastp" in upstream_set:
        return "read-preprocessing", "rsomics-fastq-preprocess", "high"
    if "fastqc" in upstream_set:
        return "read-preprocessing", "rsomics-fastq-qc", "high"
    if suffix.startswith("vcf-"):
        if contains_any(name, {"tajima", "sfs", "pi", "fst", "ld", "hardy"}):
            return "population-genetics", "rsomics-popgen", "high"
        return "variation", "rsomics-vcf", "high"
    if suffix.startswith("bam-"):
        return "alignment-formats", "rsomics-bam", "high"
    if suffix.startswith("bed-") or suffix == "bed12-to-bed6":
        return "intervals", "rsomics-bed", "high"
    if suffix.startswith("sc-"):
        return "single-cell", "rsomics-sc", "high"
    if suffix.startswith("plink-"):
        return "population-genetics", "rsomics-plink", "high"
    if "featurecounts" in upstream_set:
        return "bulk-expression", "rsomics-count", "high"
    if "infercnv" in upstream_set:
        return "single-cell", "rsomics-sc", "high"
    if upstream_set & {"freesasa", "tmalign"}:
        return "protein-structure", "rsomics-structure", "high"
    if "liftover" in upstream_set:
        return "coordinate-conversion", "rsomics-liftover", "high"
    if upstream_set & {"matrixstats", "qvalue"}:
        return "statistics", "rsomics-stats", "high"
    if upstream_set & {"trimal", "dendropy"}:
        return "phylogenetics", "rsomics-phylo", "high"
    if "tabix" in upstream_set:
        return "sequence-indexing", "rsomics-index", "high"
    if "scikit-allel" in upstream_set:
        return "population-genetics", "rsomics-popgen", "high"
    if "bcftools" in upstream_set:
        return "variation", "rsomics-vcf", "high"
    if "bedtools" in upstream_set:
        return "intervals", "rsomics-bed", "high"
    if "seqkit" in upstream_set:
        return "sequence-utilities", "rsomics-seq", "high"
    if "plink" in upstream_set:
        return "population-genetics", "rsomics-plink", "high"
    if "scanpy" in upstream_set:
        return "single-cell", "rsomics-sc", "high"
    if "deseq2" in upstream_set:
        return "bulk-expression", "rsomics-deseq", "high"
    if "edger" in upstream_set:
        return "bulk-expression", "rsomics-edger", "high"
    if "limma" in upstream_set:
        return "bulk-expression", "rsomics-limma", "high"
    if "scikit-bio" in upstream_set:
        if contains_any(
            name, {"alr", "clr", "ilr", "aitchison", "ancom", "composition"}
        ):
            return "compositional", "rsomics-composition", "high"
        if contains_any(name, {"tree", "phylo", "unifrac", "faith", "tipdist"}):
            return "phylogenetics", "rsomics-phylo", "high"
        return "ecology", "rsomics-ecology", "high"
    if "vcftools" in upstream_set:
        if contains_any(name, {"tajima", "sfs", "pi", "fst", "ld", "hardy"}):
            return "population-genetics", "rsomics-popgen", "high"
        return "variation", "rsomics-vcf", "high"
    if upstream_set & {"csvtk", "datamash"}:
        return "tabular", "rsomics-table", "high"
    if "networkx" in upstream_set:
        return "graph-algorithms", "rsomics-graph", "high"
    if "scikit-learn" in upstream_set:
        target = "rsomics-sc" if suffix.startswith("sc-") else "rsomics-ml"
        area = "single-cell" if suffix.startswith("sc-") else "machine-learning"
        return area, target, "high"
    if "scikit-image" in upstream_set:
        return "bioimage-review", "rsomics-image", "high"
    if "biopython" in upstream_set:
        target = (
            "rsomics-structure"
            if suffix.startswith(("pdb-", "dssp", "tm-align"))
            else "rsomics-seq"
        )
        area = (
            "protein-structure"
            if target == "rsomics-structure"
            else "sequence-utilities"
        )
        return area, target, "high"
    if "emboss" in upstream_set:
        return "sequence-utilities", "rsomics-seq", "high"
    if "samtools" in upstream_set:
        return "alignment-formats", "rsomics-bam", "high"
    if primary == "samtools":
        return "alignment-formats", "rsomics-bam", "high"
    if primary == "bcftools":
        return "variation", "rsomics-vcf", "high"
    if primary == "bedtools":
        return "intervals", "rsomics-bed", "high"
    if primary == "seqkit":
        return "sequence-utilities", "rsomics-seq", "high"
    if primary == "plink":
        return "population-genetics", "rsomics-plink", "high"
    if primary == "scanpy":
        return "single-cell", "rsomics-sc", "high"
    if primary == "deseq2":
        return "bulk-expression", "rsomics-deseq", "high"
    if primary == "edger":
        return "bulk-expression", "rsomics-edger", "high"
    if primary == "limma":
        return "bulk-expression", "rsomics-limma", "high"
    if primary in {"csvtk", "datamash"}:
        return "tabular", "rsomics-table", "high"
    if primary == "networkx":
        return "graph-algorithms", "rsomics-graph", "high"
    if primary in {"scipy", "statsmodels"}:
        return "statistics", "rsomics-stats", "high"
    if primary == "scikit-learn":
        target = "rsomics-sc" if suffix.startswith("sc-") else "rsomics-ml"
        area = "single-cell" if suffix.startswith("sc-") else "machine-learning"
        return area, target, "medium"
    if primary == "scikit-image":
        return "bioimage-review", "rsomics-image", "medium"
    if primary == "scikit-bio":
        if contains_any(
            name, {"alr", "clr", "ilr", "aitchison", "ancom", "composition"}
        ):
            return "compositional", "rsomics-composition", "medium"
        if contains_any(name, {"tree", "phylo", "unifrac", "faith", "tipdist"}):
            return "phylogenetics", "rsomics-phylo", "medium"
        return "ecology", "rsomics-ecology", "medium"
    if primary == "vcftools":
        if contains_any(name, {"tajima", "sfs", "pi", "fst", "ld", "hardy"}):
            return "population-genetics", "rsomics-popgen", "medium"
        return "variation", "rsomics-vcf", "medium"
    if primary == "minimap2":
        return "alignment", "rsomics-minimap2", "high"
    if primary == "macs":
        return "epigenomics", "rsomics-peak", "high"
    if primary == "methyldackel":
        return "epigenomics", "rsomics-methyl", "high"
    if primary == "biopython":
        target = (
            "rsomics-structure"
            if suffix.startswith(("pdb-", "dssp", "tm-align"))
            else "rsomics-seq"
        )
        area = (
            "protein-structure"
            if target == "rsomics-structure"
            else "sequence-utilities"
        )
        return area, target, "medium"
    if primary == "emboss":
        return "sequence-utilities", "rsomics-seq", "medium"
    if primary in {"vsearch", "sourmash", "skani"}:
        return "metagenomics", "rsomics-metagenomics", "medium"

    if suffix.startswith("bam-") or suffix in {"sam-to-bam"}:
        return "alignment-formats", "rsomics-bam", "medium"
    if suffix.startswith("bed-") or suffix == "bed12-to-bed6":
        return "intervals", "rsomics-bed", "medium"
    if suffix.startswith("vcf-"):
        return "variation", "rsomics-vcf", "medium"
    if suffix.startswith(("fasta-", "fastq-", "fastx-", "seq-")):
        preprocess = {
            "correct",
            "dedup",
            "filter",
            "merge",
            "quality",
            "trim",
            "umi",
        }
        if contains_any(name, preprocess):
            return "read-preprocessing", "rsomics-fastq-preprocess", "medium"
        return "sequence-utilities", "rsomics-seq", "medium"
    if suffix.startswith("gff-") or contains_any(name, {"annotation", "transcript"}):
        return "annotation", "rsomics-annotation", "medium"
    if suffix.startswith("tsv-"):
        return "tabular", "rsomics-table", "medium"
    if suffix.startswith(("deseq-", "edger-", "limma-")):
        family = suffix.split("-", 1)[0]
        return "bulk-expression", f"rsomics-{family}", "medium"
    if suffix.startswith("sc-") or contains_any(name, {"cellranger", "barcode"}):
        return "single-cell", "rsomics-sc", "medium"
    if suffix.startswith("plink-"):
        return "population-genetics", "rsomics-plink", "medium"
    if suffix.startswith("popgen-") or contains_any(
        name, {"tajima", "haplotype", "ehh"}
    ):
        return "population-genetics", "rsomics-popgen", "medium"
    if suffix.startswith("tree-") or contains_any(
        name, {"phylo", "upgma", "newick", "unifrac"}
    ):
        return "phylogenetics", "rsomics-phylo", "medium"
    if suffix.startswith("pdb-") or contains_any(
        name, {"dssp", "protein", "peptide", "amino"}
    ):
        return "protein-structure", "rsomics-structure", "medium"
    if contains_any(name, {"kraken", "tax", "metagenome", "derep"}):
        return "metagenomics", "rsomics-metagenomics", "medium"
    if contains_any(name, {"peak", "atac", "chip", "methyl", "bigwig", "wig"}):
        return "epigenomics", "rsomics-signal", "medium"
    if contains_any(
        name, {"graph", "centrality", "clique", "connectivity", "spanning"}
    ):
        return "graph-algorithms", "rsomics-graph", "low"
    if contains_any(name, {"stat", "test", "entropy", "correlation", "regression"}):
        return "statistics", "rsomics-stats", "low"
    return "unassigned", "", "low"


def suggested_action(
    name: str,
    layer: str,
    inbound: int,
    upstreams: list[str],
    target: str,
    confidence: str,
) -> str:
    if layer == "A":
        return "keep-core-candidate" if inbound >= 2 else "internalize-or-merge-review"
    if not target:
        return "quarantine-review"
    if name == target:
        return "keep-product-candidate"
    if name.endswith("-utils"):
        return "keep-as-consolidation-seed"
    if target in {
        "rsomics-stats",
        "rsomics-graph",
        "rsomics-ml",
        "rsomics-image",
        "rsomics-model",
    }:
        return "merge-or-core-review"
    if confidence == "low":
        return "manual-routing-review"
    return f"merge-into:{target}"


def source_facts(crate_dir: Path) -> tuple[int, int]:
    files = (
        list((crate_dir / "src").rglob("*.rs")) if (crate_dir / "src").is_dir() else []
    )
    lines = 0
    for path in files:
        try:
            lines += len(path.read_text(errors="replace").splitlines())
        except OSError:
            pass
    return len(files), lines


def has_origin(crate_dir: Path) -> bool:
    readme = crate_dir / "README.md"
    return (
        readme.is_file()
        and re.search(r"(?im)^##\s+origin\b", readme.read_text(errors="replace"))
        is not None
    )


def tsv_safe(value: object) -> str:
    return str(value).replace("\t", " ").replace("\r", " ").replace("\n", " ")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--clones", type=Path, default=DEFAULT_CLONES)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    manifests: dict[str, tuple[Path, dict, str]] = {}
    dependency_index: Counter[str] = Counter()
    dependents: dict[str, set[str]] = defaultdict(set)

    for crate_dir in sorted(args.clones.glob("rsomics-*")):
        manifest_path = crate_dir / "Cargo.toml"
        if not manifest_path.is_file():
            continue
        manifest, error = read_manifest(manifest_path)
        package = manifest.get("package", {})
        name = str(package.get("name", crate_dir.name)).replace("_", "-")
        manifests[name] = (crate_dir, manifest, error)
        dependencies = dependency_names(manifest)
        dependency_index.update(dependencies)
        for dependency in dependencies:
            dependents[dependency].add(name)

    routing: dict[str, tuple[str, str, str, list[str], list[str], str]] = {}
    for name, (crate_dir, manifest, _) in manifests.items():
        package = manifest.get("package", {})
        has_main = (crate_dir / "src/main.rs").is_file() or bool(manifest.get("bin"))
        has_lib = (crate_dir / "src/lib.rs").is_file() or bool(manifest.get("lib"))
        layer = "A" if has_lib and not has_main else "B"
        text = text_for(crate_dir, manifest)
        upstream_mentions = detect_upstreams(text)
        description = str(package.get("description", "")).lower()
        upstreams = detect_upstreams(description) or upstream_mentions
        area, target, confidence = route(name, upstreams, layer)
        routing[name] = (area, target, confidence, upstreams, upstream_mentions, layer)

    fields = [
        "crate",
        "version",
        "layer",
        "git_repo",
        "source_files",
        "source_loc",
        "inbound_rsomics_dependents",
        "inbound_target_family_count",
        "inbound_target_families",
        "has_tests",
        "has_compat",
        "has_benches",
        "has_origin",
        "upstream_families",
        "upstream_mentions",
        "area_provisional",
        "target_family_provisional",
        "target_kind_provisional",
        "routing_confidence",
        "suggested_action_provisional",
        "manifest_error",
    ]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fields, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        for name, (crate_dir, manifest, error) in sorted(manifests.items()):
            package = manifest.get("package", {})
            area, target, confidence, upstreams, upstream_mentions, layer = routing[
                name
            ]
            consumer_families = sorted(
                {
                    routing[dependent][1]
                    if routing[dependent][5] == "B" and routing[dependent][1]
                    else dependent
                    for dependent in dependents[name]
                    if dependent in routing
                }
            )
            if layer == "A":
                target_kind = "foundation"
            elif area in {
                "statistics",
                "graph-algorithms",
                "machine-learning",
                "bioimage-review",
                "sequence-models",
            }:
                target_kind = "capability-pool"
            else:
                target_kind = "product"
            action = suggested_action(
                name,
                layer,
                dependency_index[name],
                upstreams,
                target,
                confidence,
            )
            source_files, source_loc = source_facts(crate_dir)
            row = {
                "crate": name,
                "version": package.get("version", ""),
                "layer": layer,
                "git_repo": (crate_dir / ".git").exists(),
                "source_files": source_files,
                "source_loc": source_loc,
                "inbound_rsomics_dependents": dependency_index[name],
                "inbound_target_family_count": len(consumer_families),
                "inbound_target_families": "|".join(consumer_families),
                "has_tests": (crate_dir / "tests").is_dir(),
                "has_compat": (crate_dir / "tests/compat.rs").is_file(),
                "has_benches": (crate_dir / "benches").is_dir(),
                "has_origin": has_origin(crate_dir),
                "upstream_families": "|".join(upstreams),
                "upstream_mentions": "|".join(upstream_mentions),
                "area_provisional": area,
                "target_family_provisional": target,
                "target_kind_provisional": target_kind,
                "routing_confidence": confidence,
                "suggested_action_provisional": action,
                "manifest_error": error or "-",
            }
            writer.writerow({key: tsv_safe(value) for key, value in row.items()})

    print(f"wrote {len(manifests)} rows to {args.output}")


if __name__ == "__main__":
    main()
