#!/usr/bin/env python3
"""Record sparse-BCF dictionary behavior without assuming oracle equivalence."""

import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path
import resource
import struct
import subprocess


def run(argv, cwd):
    try:
        result = subprocess.run(argv, capture_output=True, text=True, timeout=10, cwd=cwd)
    except subprocess.TimeoutExpired as error:
        return {
            "argv": [str(value) for value in argv],
            "status": "timeout",
            "returncode": None,
            "stdout": (error.stdout or b"").decode(errors="replace"),
            "stderr": (error.stderr or b"").decode(errors="replace"),
        }
    return {
        "argv": [str(value) for value in argv],
        "status": "completed",
        "returncode": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--oracle", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    root = args.output.resolve()
    root.relative_to(Path(os.environ["RUNNER_TEMP"]).resolve())
    root.mkdir(exist_ok=False)
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    source = root / "sparse.vcf"
    source.write_text(
        '##fileformat=VCFv4.3\n'
        '##contig=<ID=chrA,length=200,IDX=5>\n'
        '##contig=<ID=chrB,length=100,IDX=2>\n'
        '##contig=<ID=empty,length=300,IDX=9>\n'
        '#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n'
        'chrB\t10\tb1\tA\tC\t.\tPASS\t.\n'
        'chrB\t20\tb2\tA\tG\t.\tPASS\t.\n'
        'chrA\t30\ta1\tT\tA\t.\tPASS\t.\n'
    )
    bcf = root / "sparse.bcf"
    report = {
        "binary_sha256": hashlib.sha256(args.binary.read_bytes()).hexdigest(),
        "commands": [],
    }
    commands = report["commands"]
    try:
        report["oracle_version"] = run([args.oracle, "--version"], root)
        assert report["oracle_version"]["returncode"] == 0, report["oracle_version"]
        assert report["oracle_version"]["stdout"].startswith("bcftools 1.24\n"), report["oracle_version"]
        commands.append(run([args.binary, "view", "-Ob", "-o", bcf, source], root))
        assert commands[-1]["returncode"] == 0, commands[-1]
        raw = gzip.decompress(bcf.read_bytes())
        assert raw[:5] == b"BCF\x02\x02"
        header_length = struct.unpack_from("<I", raw, 5)[0]
        header = raw[9:9 + header_length].rstrip(b"\0").decode()
        assert "ID=chrB,length=100,IDX=2" in header, header
        assert "ID=chrA,length=200,IDX=5" in header, header
        assert "ID=empty,length=300,IDX=9" in header, header
        offset = 9 + header_length
        ids = []
        while offset < len(raw):
            shared, samples = struct.unpack_from("<II", raw, offset)
            ids.append(struct.unpack_from("<i", raw, offset + 8)[0])
            offset += 8 + shared + samples
        assert offset == len(raw) and ids == [2, 2, 5], (offset, len(raw), ids)
        report["raw_rids"] = ids
        report["bcf_sha256"] = hashlib.sha256(bcf.read_bytes()).hexdigest()
        commands.append(run([args.oracle, "view", "-H", bcf], root))
        assert commands[-1]["returncode"] == 0, commands[-1]
        commands.append(run([args.oracle, "index", bcf], root))
        assert commands[-1]["returncode"] == 0, commands[-1]
        for executable in (args.oracle, args.binary):
            for flags in (("--stats",), ("--stats", "--all"), ("--nrecords",)):
                commands.append(run([executable, "index", *flags, bcf], root))
        for executable, header_flag in ((args.oracle, "-H"), (args.binary, "--no-header")):
            for region in ("chrB:10", "chrA:30", "empty:1"):
                commands.append(run([executable, "view", header_flag, "-r", region, bcf], root))
        commands.append(run([args.binary, "index", "-o", root / "ours.csi", bcf], root))
    finally:
        (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
