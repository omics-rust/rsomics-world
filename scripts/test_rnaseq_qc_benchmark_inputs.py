import gzip
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest

from rnaseq_qc_benchmark_inputs import derive_annotation, write_payload


class RealInputTests(unittest.TestCase):
    def test_download_stream_retains_bytes_and_rejects_wrong_identity(self):
        payload = b"actual\nbytes\n"
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "input.bam"
            result = write_payload(io.BytesIO(payload), target, len(payload),
                                   hashlib.md5(payload).hexdigest(), hashlib.sha256(payload).hexdigest())
            self.assertEqual(target.read_bytes(), b"actual\nbytes\n")
            self.assertEqual(result["bytes"], 13)
            for name, data, size, digest in [("corrupt", b"changed\nbytes\n", 14, hashlib.md5(payload).hexdigest()),
                                              ("truncated", b"actual\n", 13, hashlib.md5(payload).hexdigest())]:
                with self.subTest(name=name), self.assertRaises(ValueError):
                    write_payload(io.BytesIO(data), Path(directory) / name, size, digest)

    def test_annotation_profiles_preserve_rows_and_record_exclusions(self):
        rows = [
            b"chr21\t10\t30\tduplicate\t0\t+\t10\t30\t0\t2\t5,10,\t0,10,\n",
            b"chr1\t100\t150\tother\t0\t-\t100\t150\t0\t1\t50,\t0,\n",
            b"chr21\t10\t30\tduplicate\t0\t+\t10\t30\t0\t2\t5,10,\t0,10,\n",
            b"chrUn\t0\t20\tmissing\t0\t+\t0\t20\t0\t1\t20,\t0,\n",
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original = root / "genes.bed.gz"
            original.write_bytes(gzip.compress(b"".join(rows)))
            for profile, expected, reasons in [
                ("whole", b"".join(rows[:3]), ["reference_absent_from_bam"]),
                ("chr21", rows[0] + rows[2], ["outside_chr21_profile", "reference_absent_from_bam"]),
            ]:
                with self.subTest(profile=profile):
                    genes, excluded = root / (profile + ".bed"), root / (profile + ".jsonl")
                    result = derive_annotation(original, {"chr21": 100, "chr1": 200}, genes, excluded, profile)
                    self.assertEqual(genes.read_bytes(), expected)
                    ledger = [json.loads(line) for line in excluded.read_text().splitlines()]
                    self.assertEqual([row["reason"] for row in ledger], reasons)
                    self.assertEqual(ledger[-1]["source_line"], 4)
                    self.assertEqual(ledger[-1]["row"], rows[-1].decode().rstrip("\n"))
                    self.assertEqual(result["source_transcripts"], 4)
                    self.assertEqual(result["selected_transcripts"], 3 if profile == "whole" else 2)
                    self.assertEqual(result["excluded_transcripts"], 1 if profile == "whole" else 2)

    def test_annotation_exceeding_bam_bounds_is_not_silently_filtered(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original = root / "genes.bed.gz"
            original.write_bytes(gzip.compress(b"chr21\t10\t130\tbad\t0\t+\t10\t130\t0\t1\t120,\t0,\n"))
            with self.assertRaisesRegex(ValueError, "reference bounds"):
                derive_annotation(original, {"chr21": 100}, root / "genes.bed", root / "excluded.jsonl", "whole")


if __name__ == "__main__":
    unittest.main()
