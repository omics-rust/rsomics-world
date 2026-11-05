"""Assemble publishable file bytes from a completed native original audit."""

import argparse
import json
from pathlib import Path
import re

from audit_native_delivery import HOSTS, MAX_ZIP, artifact_name, body_identity, zip_files
from native_delivery import identity, json_body, require, write_new


def stage_release(audit_dir, out):
    audit_body = (audit_dir / "AUDITED.json").read_bytes()
    audit = json.loads(audit_body)
    require(audit["schema"] == "rsomics.native-original-audit.v1"
            and audit["release_authorized"] is False, "not an original audit receipt")
    product, version = audit["product"], audit["version"]
    head, run_id, attempt = audit["head"], audit["run_id"], audit["attempt"]
    require(re.fullmatch(r"[a-z0-9][a-z0-9-]*", product)
            and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version)
            and audit["repository"] == f"omics-rust/{product}", "invalid release identity")
    require(re.fullmatch(r"[0-9a-f]{40}", head) and type(run_id) is int and run_id > 0
            and type(attempt) is int and attempt > 0, "invalid audit run identity")
    require(audit["targets"].keys() == HOSTS.keys(), "expected four audited native targets")

    prefix = f"{product}-{version}"
    source_name = f"{prefix}.crate"
    files = {}
    for target, row in sorted(audit["targets"].items()):
        require(row["artifact_name"] == artifact_name(product, target, head, attempt),
                f"audit artifact name differs: {target}")
        archive = audit_dir / f"{target}.zip"
        require(archive.stat().st_size == row["zip"]["bytes"] <= MAX_ZIP
                and identity(archive) == row["zip"], f"original ZIP identity differs: {target}")
        contents = zip_files(archive)
        carrier_name = f"{prefix}-{target}.tar.gz"
        source, carrier = contents[source_name], contents[carrier_name]
        require(body_identity(source) == row["source"], f"source identity differs: {target}")
        require(body_identity(carrier) == row["carrier"], f"carrier identity differs: {target}")
        require(files.setdefault(source_name, source) == source, "source archives differ")
        files[carrier_name] = carrier

    files["SHA256SUMS"] = "".join(
        f"{body_identity(body)['sha256']}  {name}\n" for name, body in sorted(files.items())
    ).encode()
    receipt = {key: audit[key] for key in ("repository", "product", "version", "head", "run_id", "attempt")}
    receipt.update(schema="rsomics.native-release-stage.v1", audit=body_identity(audit_body),
                   assets={name: body_identity(body) for name, body in files.items()},
                   release_authorized=False)
    out.mkdir()
    for name, body in files.items():
        write_new(out / name, body)
    write_new(out / "STAGED.json", json_body(receipt))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    stage_release(args.audit.resolve(strict=True), args.out.resolve())
    print(identity(args.out / "STAGED.json")["sha256"])


if __name__ == "__main__":
    main()
