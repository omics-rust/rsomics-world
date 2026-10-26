import gzip
import io
from pathlib import Path
import tarfile
import tempfile
import unittest

from prepare_infercnv_10x_fixture import prepare


class GbmFixtureTests(unittest.TestCase):
    def test_sparse_coordinates_become_exact_tsv_without_transposing_or_relabeling_cells(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            members = {
                "features.tsv.gz": "ENSG2\tG2\tGene Expression\nENSG1\tG1\tGene Expression\nENSG3\tG3\tGene Expression\n",
                "barcodes.tsv.gz": "b1\nb2\nb3\n",
                "matrix.mtx.gz": "%%MatrixMarket matrix coordinate integer general\n% toy\n3 3 5\n1 1 3\n2 1 7\n3 2 5\n1 3 2\n2 3 11\n",
            }
            matrix = root / "matrix.tar.gz"
            with tarfile.open(matrix, "w:gz") as archive:
                directory = tarfile.TarInfo("filtered_feature_bc_matrix/")
                directory.type = tarfile.DIRTYPE
                archive.addfile(directory)
                for name, text in members.items():
                    data = gzip.compress(text.encode())
                    member = tarfile.TarInfo("filtered_feature_bc_matrix/" + name)
                    member.size = len(data)
                    archive.addfile(member, io.BytesIO(data))
            gtf = root / "genes.gtf.gz"
            with gzip.open(gtf, "wt") as target:
                for gene, chromosome, start in [("ENSG1.2", "chr1", 10), ("ENSG2.3", "chr2", 20), ("ENSG3.1", "chrM", 30)]:
                    target.write(f'{chromosome}\ttest\tgene\t{start}\t{start + 2}\t.\t+\t.\tgene_id "{gene}";\n')
            output = root / "prepared"
            manifest = prepare(matrix, gtf, output)
            self.assertEqual((output / "counts.tsv").read_text(),
                             "gene\tb1\tb2\tb3\nENSG2\t3\t0\t2\nENSG1\t7\t0\t11\nENSG3\t0\t5\t0\n")
            self.assertEqual((output / "positions.tsv").read_text(),
                             "ENSG2\tchr2\t20\t22\nENSG1\tchr1\t10\t12\nENSG3\tchrM\t30\t32\n")
            self.assertEqual((output / "annotations.tsv").read_text(),
                             "b1\tgbm_unclassified\nb2\tgbm_unclassified\nb3\tgbm_unclassified\n")
            self.assertEqual((manifest["features"], manifest["cells"], manifest["nonzero_entries"],
                              manifest["count_sum"], manifest["matched_genes"]), (3, 3, 5, 28, 3))


if __name__ == "__main__":
    unittest.main()
