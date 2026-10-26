#!/usr/bin/env python3
"""Prepare the public GBM MEX fixture for the existing strict-TSV CLI."""

import argparse
import gzip
import io
import json
import mmap
from pathlib import Path
import re
import struct
import tarfile

from validate_infercnv_shipped_oracle import file_sha256


PREFIX = "filtered_feature_bc_matrix/"
GROUP = "gbm_unclassified"
U32 = struct.Struct("<I")


def records(archive, name):
    source = archive.extractfile(PREFIX + name)
    if source is None:
        raise ValueError(f"missing regular matrix member: {name}")
    return io.TextIOWrapper(gzip.GzipFile(fileobj=source), encoding="utf-8")


def gene_coordinates(gtf, wanted):
    positions = {}
    with gzip.open(gtf, "rt", encoding="utf-8") as source:
        for line in source:
            if line.startswith("#"):
                continue
            fields = line.rstrip("\n").split("\t", 8)
            if len(fields) != 9:
                raise ValueError("invalid GTF record")
            if fields[2] != "gene":
                continue
            match = re.search(r'gene_id "([^\"]+)";', fields[8])
            if match is None:
                raise ValueError("GTF gene has no gene_id")
            gene = re.sub(r"\.[0-9]+(?=(_PAR_Y)?$)", "", match[1])
            if gene not in wanted:
                continue
            if gene in positions:
                raise ValueError(f"ambiguous normalized GTF gene: {gene}")
            start, end = int(fields[3]), int(fields[4])
            if not 0 < start <= end:
                raise ValueError(f"invalid GTF coordinates: {gene}")
            positions[gene] = (fields[0], start, end)
    if not positions:
        raise ValueError("no feature IDs match the supplied GTF")
    return positions


def unique_ids(values):
    if not values or len(set(values)) != len(values) or any(not value for value in values):
        raise ValueError("feature/barcode IDs must be nonempty and unique")


def prepare(matrix_archive, gtf, output):
    matrix_archive, gtf = Path(matrix_archive).resolve(strict=True), Path(gtf).resolve(strict=True)
    source_hashes = {str(path): file_sha256(path) for path in (matrix_archive, gtf)}
    output = Path(output).absolute()
    output.mkdir(parents=True, exist_ok=False)
    with tarfile.open(matrix_archive, "r:gz") as archive:
        expected = {PREFIX + name for name in ("features.tsv.gz", "barcodes.tsv.gz", "matrix.mtx.gz")}
        members = archive.getmembers()
        files = [member for member in members if member.isfile()]
        if {member.name for member in files} != expected or len(files) != 3 or any(
                not member.isfile() and not (member.isdir() and member.name.rstrip("/") == PREFIX.rstrip("/"))
                for member in members):
            raise ValueError("fixture archive must contain exactly the three regular MEX files")
        with records(archive, "features.tsv.gz") as source:
            features = [line.rstrip("\n").split("\t") for line in source]
        if any(len(row) != 3 or row[2] != "Gene Expression" for row in features):
            raise ValueError("fixture requires only Gene Expression features")
        genes = [row[0] for row in features]
        with records(archive, "barcodes.tsv.gz") as source:
            cells = [line.rstrip("\n") for line in source]
        unique_ids(genes)
        unique_ids(cells)
        if any("\t" in cell or "\r" in cell for cell in cells):
            raise ValueError("invalid barcode TSV field")
        positions = gene_coordinates(gtf, set(genes))
        source_cell_totals = [0] * len(cells)
        source_gene_totals = [0] * len(genes)
        with records(archive, "matrix.mtx.gz") as source:
            if source.readline().rstrip("\n") != "%%MatrixMarket matrix coordinate integer general":
                raise ValueError("fixture requires integer general coordinate Matrix Market")
            dimensions = next(line for line in source if not line.startswith("%"))
            rows, columns, nonzeros = map(int, dimensions.split())
            if (rows, columns) != (len(genes), len(cells)) or nonzeros <= 0:
                raise ValueError("matrix dimensions differ from feature/barcode files")
            with (output / "counts.row-major.u32").open("x+b") as backing:
                backing.truncate(rows * columns * U32.size)
                with mmap.mmap(backing.fileno(), 0) as dense:
                    actual_nonzeros = 0
                    for line in source:
                        row, column, count = map(int, line.split())
                        if not (1 <= row <= rows and 1 <= column <= columns and 0 < count <= 2**32 - 1):
                            raise ValueError("invalid nonzero matrix entry")
                        offset = ((row - 1) * columns + column - 1) * U32.size
                        if U32.unpack_from(dense, offset)[0] != 0:
                            raise ValueError("duplicate matrix coordinate")
                        U32.pack_into(dense, offset, count)
                        source_cell_totals[column - 1] += count
                        source_gene_totals[row - 1] += count
                        actual_nonzeros += 1
                    if actual_nonzeros != nonzeros:
                        raise ValueError("matrix nonzero cardinality differs from its header")
                    exported_cell_totals = [0] * columns
                    exported_nonzeros = 0
                    unpack_row = struct.Struct(f"<{columns}I")
                    with (output / "counts.tsv").open("x", encoding="utf-8", newline="\n") as target:
                        target.write("gene\t" + "\t".join(cells) + "\n")
                        for row, gene in enumerate(genes):
                            values = unpack_row.unpack_from(dense, row * unpack_row.size)
                            if sum(values) != source_gene_totals[row]:
                                raise ValueError("exported gene count total differs")
                            for column, value in enumerate(values):
                                if value:
                                    exported_cell_totals[column] += value
                                    exported_nonzeros += 1
                            target.write(gene + "\t" + "\t".join(map(str, values)) + "\n")
                    if exported_cell_totals != source_cell_totals or exported_nonzeros != nonzeros:
                        raise ValueError("exported barcode totals/nonzero cardinality differ")
    with (output / "positions.tsv").open("x", encoding="utf-8", newline="\n") as target:
        for gene in genes:
            if gene in positions:
                chromosome, start, end = positions[gene]
                target.write(f"{gene}\t{chromosome}\t{start}\t{end}\n")
    with (output / "annotations.tsv").open("x", encoding="utf-8", newline="\n") as target:
        target.writelines(f"{cell}\t{GROUP}\n" for cell in cells)
    with (output / "unmatched-genes.tsv").open("x", encoding="utf-8", newline="\n") as target:
        target.writelines(f"{gene}\n" for gene in genes if gene not in positions)
    if any(file_sha256(Path(path)) != digest for path, digest in source_hashes.items()):
        raise ValueError("source changed during conversion")
    manifest = dict(
        status="complete", scope="fixture conversion, not native 10x support or a CLI benchmark",
        source_sha256=source_hashes, features=rows, cells=columns, nonzero_entries=nonzeros,
        count_sum=sum(source_cell_totals), matched_genes=len(positions),
        unmatched_genes=rows - len(positions), coordinate_basis="GTF 1-based inclusive gene spans",
        id_join="feature ID to GTF gene_id with numeric version removed; PAR_Y suffix preserved",
        count_transform="none; all original rows, barcodes and integer counts retained",
        annotations=f"all cells in {GROUP}; no normal/tumor cell labels inferred",
        proposed_reference="all cells; computational scaling only, not biological CNV validation",
        intermediate="counts.row-major.u32 retained, little-endian gene-major u32",
        output_sha256={name: file_sha256(output / name) for name in
                       ("counts.tsv", "positions.tsv", "annotations.tsv", "unmatched-genes.tsv")},
    )
    with (output / "manifest.json").open("x") as target:
        target.write(json.dumps(manifest, indent=2) + "\n")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--matrix-archive", type=Path, required=True)
    parser.add_argument("--gtf", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(prepare(args.matrix_archive, args.gtf, args.output), indent=2))


if __name__ == "__main__":
    main()
