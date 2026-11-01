#!/usr/bin/env python3
"""Manual Linux-only RSeQC junction qualification; local preparation is stdlib-only."""

import argparse
from collections import Counter
from email.parser import BytesParser
import hashlib
import importlib
import importlib.metadata
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import stat
import sys
import sysconfig
import tarfile
import tempfile
import traceback
import urllib.error
import urllib.request
import zipfile

from infercnv_trial_process import TrialProcessError, run_child
from private_artifact import encrypt, finalization_space, prepare_tool
from verify_infercnv_oracle_artifact import _archive_name, _artifact, _extract, _run


ROOT = Path(__file__).resolve().parents[1]
ASSET = ROOT / "assets/rnaseq-qc-junctions-20261101.json"
WORKFLOW = ".github/workflows/rnaseq-qc-junctions.yml"
RECIPIENT = ROOT / "assets/private-artifacts-20261101.recipient"
RECIPIENT_SHA = "8bb86dc97982fc7b64d7e33f7c3741e7f3d27a21690f48ddce9dada4a230680d"
CLOSURE = ("rnaseq_qc_junctions.py", "test_rnaseq_qc_junctions.py", "infercnv_trial_process.py",
           "private_artifact.py", "verify_infercnv_oracle_artifact.py")
CACHES = ("TMPDIR", "XDG_CACHE_HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "PIP_CACHE_DIR",
          "PYTHONPYCACHEPREFIX", "PYTHONUSERBASE", "MPLCONFIGDIR")
HEADER = "chrom\tintron_st(0-based)\tintron_end(1-based)\tread_count\tannotation"
CLASSES = ("annotated", "partial_novel", "complete_novel")


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with path.open("x") as output:
        json.dump(value, output, indent=2, sort_keys=True, allow_nan=False)
        output.write("\n")


def identity(path):
    path = Path(path)
    with path.open("rb") as source:
        return dict(bytes=path.stat().st_size, sha256=hashlib.file_digest(source, "sha256").hexdigest())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fixture_plans():
    header = "@HD\tVN:1.6\tSO:unsorted\n@SQ\tSN:chr1\tLN:10000\n@SQ\tSN:CHR1\tLN:10000\n"

    def bed(chrom, start, name="shared-name", strand="+"):
        return "\t".join(map(str, (chrom, start, start + 90, name, 0, strand,
                                   start, start + 90, 0, 2, "20,20,", "0,70,")))

    annotation = "\n".join((bed("chr1", 100), bed("chr1", 100), bed("chr1", 100, strand="-"),
                              bed("chr1", 200), bed("CHR1", 400))) + "\n"
    records = [("known", 0, "chr1", 100, 30, "20M50N20M"),
               ("repeated", 16, "chr1", 100, 255, "20M50N20M"),
               ("independent-sites-new-pair", 0, "chr1", 100, 60, "20M150N20M"),
               ("partial", 0, "chr1", 100, 60, "20M51N20M"),
               ("short", 0, "chr1", 100, 60, "20M49N20M"),
               ("multiple-N", 0, "chr1", 100, 60, "20M50N20M50N20M"),
               ("case-pooling", 0, "CHR1", 100, 60, "20M50N20M"),
               ("novel", 0, "chr1", 300, 60, "20M50N20M"),
               ("mixed", 0, "chr1", 500, 60, "5H3S10M2I5M3D4M2P5=3X50N20M4S2H")]
    records += [(name, flag, "chr1", 100, mapq, "20M50N20M") for name, flag, mapq in
                (("secondary", 256, 60), ("qcfail", 512, 60), ("duplicate", 1024, 60),
                 ("excluded-combined", 1796, 60), ("supplementary", 2048, 60), ("mapq29", 0, 29))]
    records.append(("unmapped", 4, "*", -1, 0, "*"))

    def sam(rows, paired=False):
        lines = []
        for name, flag, chrom, start, mapq, cigar in rows:
            length = 40 if cigar == "*" else sum(int(n) for n, op in re.findall(r"(\d+)([MIDNSHP=X])", cigar) if op in "MIS=X")
            span = sum(int(n) for n, op in re.findall(r"(\d+)([MIDNSHP=X])", cigar) if op in "MDN=X")
            copies = (1, 2) if paired and not flag & 4 else (0,)
            for mate in copies:
                reverse = bool(flag & 16) if mate != 2 else not bool(flag & 16)
                actual_flag = flag if not mate else (flag & ~16) | 3 | (64 if mate == 1 else 128) | (16 if reverse else 32)
                fields = (name, actual_flag, chrom, start + 1, mapq, cigar,
                          "=" if mate else "*", start + 1 if mate else 0,
                          span * (-1 if reverse else 1) if mate else 0, "A" * length, "I" * length)
                lines.append("\t".join(map(str, fields)))
        return header + "\n".join(lines) + "\n"

    intent = dict(scope="synthetic semantic qualification, not biological or performance evidence",
                  classes=list(CLASSES), independent_sites="120 and270 from separate transcript pairs; no120-270 annotation pair",
                  case="both exact header names present; source uppercase pooling",
                  support="duplicate records, multiple N events and both mates count independently",
                  filters="0x704 excluded, supplementary included, MAPQ29/30/255 and N49/50/51",
                  projection="S/I/H/P and =/X ignored; M/D/N advance; mixed CIGAR separates canonical geometry",
                  annotation="duplicate sites and names, opposite strands do not multiply support")
    plans = {"informative-single": dict(sam=sam(records), bed=annotation, intent=intent),
             "informative-paired": dict(sam=sam(records, paired=True), bed=annotation, intent=intent),
             "case-alias-only": dict(sam=sam([("alias", 0, "chr1", 100, 30, "20M50N20M")]),
                                     bed=bed("ChR1", 100) + "\n",
                                     intent=dict(source="uppercase alias participates", native="reject absent exact annotation header name")),
             "no-N": dict(sam=sam([("no-N", 0, "chr1", 100, 30, "40M")]), bed=annotation,
                          intent=dict(source="exit0, empty primary and R files, no derivative files", native="complete atomic zero tables, exit0 intentional difference")),
             "all-short": dict(sam=sam([("all-short", 0, "chr1", 100, 30, "20M49N20M")]), bed=annotation,
                               intent=dict(source="exit1, header-only primary, event-only R, no derivatives", native="complete atomic zero tables, exit0 intentional difference"))}
    return plans


def invocation_plan():
    core = ["-q", "30", "-m", "50", "--skip-plot", "--skip-bed", "--skip-interact"]
    rows = [dict(name=name, case=name, options=core, expected_exit=int(name == "all-short")) for name in fixture_plans()]
    rows.append(dict(name="informative-single-exports", case="informative-single",
                     options=core[:5], expected_exit=0))
    return rows


def source_events(plan):
    events = []
    for line in plan["sam"].splitlines():
        if line.startswith("@"):
            continue
        name, flags, chrom, start, mapq, cigar, *_ = line.split("\t")
        flags, mapq, position = int(flags), int(mapq), int(start) - 1
        for ordinal, (size, operation) in enumerate(re.findall(r"(\d+)([MIDNSHP=X])", cigar)):
            size = int(size)
            if operation == "N":
                events.append(dict(name=name, flag=flags, mapq=mapq, cigar=cigar, cigar_index=ordinal,
                                   exact_chrom=chrom, chrom=chrom.upper(), start=position, end=position + size,
                                   length=size, eligible_record=not bool(flags & 0x704) and mapq >= 30,
                                   minimum_pass=size >= 50))
            if operation in "MDN":
                position += size
    return events


def predicted(plan):
    donors, acceptors = set(), set()
    for line in plan["bed"].splitlines():
        fields = line.split("\t")
        chrom, start = fields[0].upper(), int(fields[1])
        sizes = [int(value) for value in fields[10].rstrip(",").split(",")]
        starts = [start + int(value) for value in fields[11].rstrip(",").split(",")]
        donors.update((chrom, offset + size) for offset, size in zip(starts[:-1], sizes[:-1]))
        acceptors.update((chrom, offset) for offset in starts[1:])
    events, rows = source_events(plan), {}
    counts = Counter()
    for event in events:
        if not event["eligible_record"]:
            continue
        counts["total"] += 1
        if not event["minimum_pass"]:
            counts["filtered"] += 1
            continue
        donor = (event["chrom"], event["start"]) in donors
        acceptor = (event["chrom"], event["end"]) in acceptors
        category = "annotated" if donor and acceptor else "partial_novel" if donor or acceptor else "complete_novel"
        key = (event["chrom"], event["start"], event["end"])
        if key not in rows:
            rows[key] = dict(chrom=event["chrom"].replace("CHR", "chr"), start=event["start"],
                             end=event["end"], read_count=0, annotation=category)
        rows[key]["read_count"] += 1
        counts[category] += 1
        event.update(accepted=True, annotation=category, donor_known=donor, acceptor_known=acceptor)
    return dict(scope="independent policy derivation, not an observed source result", N_events=events,
                rows=list(rows.values()), events=dict(total=counts["total"], filtered=counts["filtered"],
                **{name: counts[name] for name in CLASSES}),
                distinct=dict(total=len(rows), **{name: sum(row["annotation"] == name for row in rows.values()) for name in CLASSES}),
                annotation_sites=dict(donors=sorted(donors), acceptors=sorted(acceptors)))


def parse_primary(text, empty=False):
    if empty:
        require(text == "", "no-N primary must actually be empty")
        return []
    lines = text.splitlines()
    require(bool(lines) and lines[0] == HEADER, "source primary header differs")
    rows, keys = [], set()
    for line in lines[1:]:
        fields = line.split()
        require(len(fields) == 5, "source primary row arity differs")
        chrom, start, end, count, category = fields
        start, end, count = int(start), int(end), int(count)
        key = (chrom, start, end)
        require(0 <= start < end and count > 0 and category in CLASSES and key not in keys,
                "invalid or duplicate source primary row")
        keys.add(key)
        rows.append(dict(chrom=chrom, start=start, end=end, read_count=count, annotation=category))
    return rows


def http_response(request, output, maximum, timeout=120, receipt=None):
    try:
        return urllib.request.urlopen(request, timeout=timeout)
    except urllib.error.HTTPError as error:
        record = dict(http_status=error.code, final_url=error.geturl(),
                      headers={key: value for key, value in (error.headers or {}).items()
                               if key.lower() not in ("authorization", "proxy-authorization")},
                      body_limit_bytes=min(maximum, 2**20), body_truncated=None)
        body = Path(str(output) + ".http-error.body")
        try:
            remaining = record["body_limit_bytes"]
            with body.open("xb") as stream:
                while remaining:
                    block = error.read(min(remaining, 2**20))
                    if not block:
                        break
                    stream.write(block)
                    remaining -= len(block)
                record["body_truncated"] = bool(error.read(1)) if remaining == 0 else False
            record["body"] = dict(path=str(body), **identity(body))
        except Exception:
            record["custody_error"] = traceback.format_exc()
            error.add_note("HTTP response body custody failed: " + record["custody_error"])
        finally:
            try:
                error.close()
            except Exception:
                error.add_note("HTTP response close failed: " + traceback.format_exc())
            if receipt is not None:
                receipt.update(record)
            try:
                save(str(output) + ".http-error.json", record)
            except Exception:
                error.add_note("HTTP response receipt custody failed: " + traceback.format_exc())
        raise


def download(spec, output, maximum=None, token=None):
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    receipt = dict(url=spec["url"], expected=spec)
    limit = spec.get("bytes", spec.get("size", maximum))
    require(type(limit) is int and limit > 0, "bounded acquisition required")
    request = urllib.request.Request(spec["url"], headers={"User-Agent": "rsomics-junction-source/1"})
    if token:
        request.add_unredirected_header("Authorization", "Bearer " + token)
    failure = None
    try:
        with http_response(request, output, limit, receipt=receipt) as response, output.open("xb") as target:
            receipt.update(http_status=response.status, final_url=response.url,
                           headers={key: value for key, value in response.headers.items()
                                    if key.lower() not in ("authorization", "proxy-authorization")})
            size = 0
            while block := response.read(2**20):
                size += len(block)
                require(size <= limit, "acquisition exceeds admitted size")
                target.write(block)
        actual = identity(output)
        require(not ("bytes" in spec or "size" in spec) or actual["bytes"] == limit, "acquired size differs")
        require(not spec.get("sha256") or actual["sha256"] == spec["sha256"], "acquired SHA256 differs")
        receipt.update(status="admitted", actual=actual)
    except BaseException as error:
        failure = error
        receipt.update(status="failed-original-retained", error=traceback.format_exc())
        raise
    finally:
        try:
            save(str(output) + ".receipt.json", receipt)
        except Exception:
            if not isinstance(failure, urllib.error.HTTPError):
                raise
            failure.add_note("Acquisition receipt custody failed: " + traceback.format_exc())
    return output


def run(argv, directory, expected=0, timeout=300):
    directory.mkdir(parents=True)
    prefix = directory / "process"
    try:
        run_child([str(value) for value in argv], prefix, timeout=timeout)
    except TrialProcessError:
        report = json.loads(Path(str(prefix) + ".process.json").read_text())
        require(expected == 1 and report["returncode"] == 1 and report["status"] == "nonzero"
                and not report["timed_out"] and not report["interrupted_signal"] and report["group_empty"]
                and not report["cleanup_actions"] and not report["cleanup_error"], "unexpected source/process failure")
    report = json.loads(Path(str(prefix) + ".process.json").read_text())
    require(report["returncode"] == expected, "actual process status differs from frozen expectation")
    return report


def hosted():
    require(os.environ.get("GITHUB_ACTIONS") == "true" and os.environ.get("GITHUB_REPOSITORY") == "omics-rust/rsomics-world"
            and platform.system() == "Linux" and platform.machine() == "x86_64", "hosted Linux x86_64 required")
    require(re.fullmatch(r"[0-9a-f]{40}", os.environ["GITHUB_SHA"]), "actual hosted head missing")
    runner = Path(os.environ["RUNNER_TEMP"]).resolve(strict=True)
    session = Path(os.environ["JUNCTION_SESSION"]).resolve(strict=True)
    require(session != runner and session.is_relative_to(runner) and session.stat().st_dev == runner.stat().st_dev,
            "session escapes hosted runner-temp")
    for name in ("private", "cipher"):
        path = session / name
        require(not path.is_symlink() and stat.S_IMODE(path.stat().st_mode) == 0o700, "private directory mode differs")
    for name in CACHES:
        require(Path(os.environ[name]).resolve(strict=True).is_relative_to(session / "private"), "cache escapes private tree")
    return session


def initialize():
    require(os.environ.get("GITHUB_ACTIONS") == "true" and platform.system() == "Linux", "hosted initialization required")
    session = Path(tempfile.mkdtemp(prefix="rnaseq-junctions-", dir=Path(os.environ["RUNNER_TEMP"]).resolve(strict=True)))
    session.chmod(0o700)
    for name in ("private", "cipher"):
        (session / name).mkdir(mode=0o700)
    values = dict(JUNCTION_SESSION=str(session))
    for name in CACHES:
        path = session / "private/cache" / name
        path.mkdir(parents=True, mode=0o700)
        values[name] = str(path)
    with Path(os.environ["GITHUB_ENV"]).open("a") as output:
        for name, value in values.items():
            output.write(name + "=" + value + "\n")
    save(session / "private/request.json", dict(head_sha=os.environ["GITHUB_SHA"], run_id=os.environ["GITHUB_RUN_ID"],
         run_attempt=os.environ["GITHUB_RUN_ATTEMPT"], workflow_ref=os.environ.get("GITHUB_WORKFLOW_REF"),
         limitation="No plaintext fallback. Hard host loss, cancellation, time/space exhaustion or encryption failure can prevent retention."))


def inventory(root):
    rows = []
    for path in sorted(root.rglob("*")):
        entry = path.lstat()
        row = dict(name=path.relative_to(root).as_posix(), mode=stat.S_IMODE(entry.st_mode), mtime=int(entry.st_mtime))
        if path.is_symlink():
            require(path.resolve().is_relative_to(root.resolve()), "retained symlink escapes private tree")
            row.update(kind="symlink", target=os.readlink(path), bytes=0)
        elif path.is_file():
            row.update(kind="file", **identity(path))
        else:
            require(path.is_dir(), "unsupported evidence member")
            row.update(kind="directory", bytes=0)
        rows.append(row)
    require(len(rows) < 100000, "retained member bound exceeded")
    return rows


def capacity(private, initial=False):
    limits = json.loads(ASSET.read_text())["limits"]
    retained = sum(row["bytes"] for row in inventory(private))
    space = finalization_space(limits["retained_bytes"], limits["tar_bytes"])
    free = shutil.disk_usage(private).free
    require(retained <= limits["retained_bytes"] and free >= space["required_free_bytes"] +
            (limits["retained_bytes"] if initial else 0), "retained/streamed TAR/headroom capacity unavailable")
    return dict(retained_bytes=retained, free_bytes=free, finalization=space)


def transport(session):
    require(not RECIPIENT.is_symlink() and identity(RECIPIENT)["sha256"] == RECIPIENT_SHA, "fixed public recipient differs")
    private = session / "private"
    save(private / "capacity.json", capacity(private, initial=True))
    admission = private / "transport-admission"
    admission.mkdir()
    try:
        tool = prepare_tool("linux-amd64", private / "age-tool")
        probe = encrypt(tool, RECIPIENT, admission / "probe.age", producer=[sys.executable, "-B", "-c",
                        "import sys;sys.stdout.buffer.write(b'junction-source-transport-admission\\n')"], plaintext_limit=128,
                        context=dict(repository="omics-rust/rsomics-world", product="rsomics-rnaseq-qc", phase="transport-admission"))
        save(admission / "receipt.json", probe)
    except BaseException:
        save(admission / "failure.json", dict(status="failed-originals-retained", error=traceback.format_exc()))
        raise


def cgroup():
    return {name: (Path("/sys/fs/cgroup") / name).read_text().strip() for name in
            ("cpu.max", "memory.max", "memory.swap.max", "memory.current", "memory.peak", "memory.events")}


def admit_machine(private, policy):
    observed = cgroup()
    quota, period = observed["cpu.max"].split()
    require(quota != "max" and int(quota) == 4 * int(period) and len(os.sched_getaffinity(0)) >= 4,
            "actual four-CPU quota/affinity differs")
    require(int(observed["memory.max"]) == 8 * 2**30 and observed["memory.swap.max"] == "0", "actual8GiB/no-extra-swap differs")
    require(os.environ.get("JUNCTION_IMAGE_DIGEST") == policy["image"]["manifest_digest"] and
            os.environ.get("GCC_VERSION") == "14.3.0", "immutable image declaration differs")
    save(private / "machine.json", dict(platform=platform.platform(), uname=list(platform.uname()),
         cgroup=observed, affinity=sorted(os.sched_getaffinity(0)), limits=policy["limits"],
         address_space_limit="none added; cgroup supplies memory boundary"))
    token_url = "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/gcc:pull"
    with http_response(urllib.request.Request(token_url), private / "registry-token", 2**20, timeout=60) as response:
        token_body = response.read(2**20 + 1)
    require(len(token_body) <= 2**20, "registry token response exceeds bound")
    token = json.loads(token_body)["token"]
    for name, route, digest in (("manifest", "manifests/", policy["image"]["manifest_digest"]),
                                ("config", "blobs/", policy["image"]["config_digest"])):
        request = urllib.request.Request("https://registry-1.docker.io/v2/library/gcc/" + route + digest,
                  headers={"Accept": "application/vnd.oci.image.manifest.v1+json"})
        request.add_unredirected_header("Authorization", "Bearer " + token)
        with http_response(request, private / ("image-" + name), 2**20) as response:
            body = response.read(2**20)
        (private / ("image-" + name + ".json")).write_bytes(body)
        require("sha256:" + hashlib.sha256(body).hexdigest() == digest, "official image authority differs")
    config = json.loads((private / "image-config.json").read_text())
    require(config["architecture"] == "amd64" and config["os"] == "linux", "official image platform differs")


def accepted_source(private, policy):
    source = private / "accepted-source"
    source.mkdir()
    receipt = policy["accepted_source"]
    require(json.loads((ROOT / ".autopilot/oracles/rnaseq-qc-first-workflow-2026-10-27.json").read_text()) == receipt,
            "tracked accepted source authority differs")
    token = os.environ["GH_TOKEN"]
    base = "https://api.github.com/repos/omics-rust/rsomics-world/actions/"
    download(dict(url=base + "runs/" + str(receipt["run_id"])), source / "run.json", maximum=2**20, token=token)
    download(dict(url=base + "artifacts/" + str(receipt["artifact_id"])), source / "artifact.json", maximum=2**20, token=token)
    zipped = download(dict(url=base + "artifacts/" + str(receipt["artifact_id"]) + "/zip",
                      bytes=receipt["size_in_bytes"], sha256=receipt["artifact_sha256"]), source / "artifact.zip", token=token)
    _run(source / "run.json", receipt)
    _artifact(source / "artifact.json", receipt, zipped)
    with zipfile.ZipFile(zipped) as archive:
        names = [_archive_name(row.filename, row.is_dir()) for row in archive.infolist()]
        require(len(names) == len(set(names)) and sum(row.file_size for row in archive.infolist()) < 16 * 2**20
                and archive.testzip() is None, "accepted source ZIP member/CRC bound differs")
        _extract(archive, source / "artifact")
    acquisition = source / "artifact/acquisition"
    require(identity(acquisition / "installed-freeze.txt")["sha256"] == policy["dependency_freeze_sha256"], "accepted17-version freeze differs")
    for filename, digest in (("rseqc-5.0.5-py3-none-any.whl", policy["source"]["wheel_sha256"]),
                             ("rseqc-source.tar.gz", policy["source"]["archive_sha256"])):
        require(identity(acquisition / filename)["sha256"] == digest, "accepted source body differs")


def extract_python(archive, output):
    output.mkdir()
    with tarfile.open(archive, "r:gz") as carrier:
        members = carrier.getmembers()
        require(sum(row.size for row in members) < 768 * 2**20 and len(members) < 30000, "Python archive bound differs")
        names = {row.name.rstrip("/") for row in members}
        require(len(names) == len(members), "duplicate Python archive member")
        links = {row.name.rstrip("/") for row in members if row.issym() or row.islnk()}
        for row in members:
            name = PurePosixPath(row.name)
            require(not name.is_absolute() and ".." not in name.parts and not any("/".join(name.parts[:i]) in links
                    for i in range(1, len(name.parts))), "unsafe Python archive path")
            target = output / row.name
            require(row.isdir() or row.isfile() or row.issym(), "unsupported Python archive member")
            if row.issym():
                require((target.parent / row.linkname).resolve().is_relative_to(output.resolve()), "Python link escapes")
            elif row.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with carrier.extractfile(row) as source, target.open("xb") as stream:
                    shutil.copyfileobj(source, stream)
                target.chmod(row.mode & 0o777)
        for row in members:
            if row.issym():
                (output / row.name).parent.mkdir(parents=True, exist_ok=True)
                (output / row.name).symlink_to(row.linkname)


def verify_wheel(path, package):
    require(identity(path) == {key: package[key] for key in ("bytes", "sha256")}, "wheel body differs")
    with zipfile.ZipFile(path) as archive:
        raw = archive.read(package["metadata_member"])
        require(archive.testzip() is None, "wheel CRC differs")
    metadata = BytesParser().parsebytes(raw)
    require(hashlib.sha256(raw).hexdigest() == package["metadata_sha256"] and metadata["Version"] == package["version"]
            and metadata.get_all("Requires-Dist", []) == package["requires_dist"], "wheel METADATA differs")


def prepare_runtime(private, policy):
    runtime = private / "runtime"
    runtime.mkdir()
    original = download(policy["python"], runtime / policy["python"]["filename"])
    checksum = download(dict(url=policy["python"]["checksum_url"]), runtime / "python-checksum.txt", maximum=1024)
    require(checksum.read_text().split()[0] == policy["python"]["sha256"], "official Python checksum differs")
    extract_python(original, runtime / "portable")
    python = runtime / "portable/python/bin/python3"
    run([python, "-I", "-c", "import platform;assert platform.python_version()=='3.12.10'"], runtime / "python-version")
    run([python, "-I", "-m", "venv", "--without-pip", "--symlinks", runtime / "venv"], runtime / "venv-create")
    python = runtime / "venv/bin/python3"
    run([python, "-I", "-m", "ensurepip", "--upgrade"], runtime / "ensurepip")
    wheels = runtime / "wheels"
    wheels.mkdir()
    requirements = []
    for name, package in policy["packages"].items():
        metadata = download(dict(url=package["metadata_url"]), wheels / (name + ".pypi.json"), maximum=16 * 2**20)
        pypi = json.loads(metadata.read_text())
        require(any(row["filename"] == package["filename"] and row["digests"]["sha256"] == package["sha256"]
                    and row["size"] == package["bytes"] for row in pypi["urls"]), "fresh official PyPI record disagrees")
        path = download(package, wheels / package["filename"])
        verify_wheel(path, package)
        requirements.append(name + "==" + package["version"] + " --hash=sha256:" + package["sha256"])
        capacity(private)
    lock = runtime / "requirements.lock"
    lock.write_text("\n".join(requirements) + "\n")
    run([python, "-I", "-m", "pip", "--isolated", "install", "--no-index", "--no-deps", "--no-cache-dir",
         "--require-hashes", "--only-binary=:all:", "--find-links", wheels, "-r", lock], runtime / "wheel-install")
    run([python, "-I", "-m", "pip", "--isolated", "check"], runtime / "pip-check")
    run([python, "-I", "-m", "pip", "--isolated", "freeze", "--all"], runtime / "installed-freeze")
    run([python, "-I", "-m", "pip", "--isolated", "inspect"], runtime / "installed-inspect")
    source_archive = download(dict(url=policy["source"]["archive_url"], bytes=472500,
                              sha256=policy["source"]["archive_sha256"]), runtime / "rseqc-source.tar.gz")
    with zipfile.ZipFile(wheels / policy["packages"]["RSeQC"]["filename"]) as wheel, tarfile.open(source_archive) as source:
        for member, digest in policy["source"]["anchors"].items():
            raw = wheel.read(member)
            require(hashlib.sha256(raw).hexdigest() == digest, "pinned wheel called-source differs")
            relative = "scripts/junction_annotation.py" if member.endswith("junction_annotation.py") else "src/" + member
            original = source.extractfile("RSeQC-" + policy["source"]["commit"] + "/" + relative).read()
            require(raw.split(b"\n", 1)[1] == original.split(b"\n", 1)[1] if raw.startswith(b"#!") else raw == original,
                    "pinned GitHub called-source differs")
    run([python, "-B", __file__, "probe", "--root", private], runtime / "probe")
    save(runtime / "installed-members.json", inventory(runtime))
    capacity(private)
    return python


def runtime_probe(private):
    policy = json.loads(ASSET.read_text())
    require(platform.python_version() == "3.12.10", "actual source Python patch differs")
    versions = {name: importlib.metadata.version(name) for name in policy["packages"]}
    require(versions == {name: package["version"] for name, package in policy["packages"].items()}, "installed versions differ")
    modules = {}
    for name in ("numpy", "pysam", "bx", "pyBigWig", "logomaker", "matplotlib", "pandas", "qcmodule.SAM", "qcmodule.bam_cigar"):
        module = importlib.import_module(name)
        path = Path(module.__file__).resolve()
        require(path.is_relative_to(Path(sys.prefix).resolve()), "source/scientific module escapes admitted venv")
        modules[name] = dict(path=str(path), **identity(path))
    script = Path(sysconfig.get_path("scripts")) / "junction_annotation.py"
    wheels = private / "runtime/wheels"
    installed = {}
    for name, package in policy["packages"].items():
        distribution = importlib.metadata.distribution(name)
        with zipfile.ZipFile(wheels / package["filename"]) as archive:
            for member in archive.namelist():
                if member.endswith(("/", ".dist-info/RECORD")) or ".data/" in member:
                    continue
                actual = Path(distribution.locate_file(member))
                require(actual.is_file() and actual.read_bytes() == archive.read(member), "installed wheel body differs: " + member)
        installed[name] = dict(version=distribution.version, metadata_path=str(distribution._path))
    with zipfile.ZipFile(wheels / policy["packages"]["RSeQC"]["filename"]) as wheel:
        require(script.read_bytes().split(b"\n", 1)[1] == wheel.read("rseqc-5.0.5.data/scripts/junction_annotation.py").split(b"\n", 1)[1], "installed CLI body differs")
    libraries = {}
    for line in Path("/proc/self/maps").read_text().splitlines():
        tokens = line.split()
        if len(tokens) < 6 or not tokens[-1].startswith("/"):
            continue
        path = Path(tokens[-1]).resolve(strict=True)
        require(path.is_file(), "loaded provider missing")
        record = identity(path)
        destination = private / "runtime/loaded-providers" / (record["sha256"] + "-" + path.name)
        destination.parent.mkdir(exist_ok=True)
        if not destination.exists():
            shutil.copy2(path, destination)
        libraries[str(path)] = dict(retained=str(destination.relative_to(private)), **record)
    save(private / "runtime/runtime.json", dict(python=sys.version, executable=str(Path(sys.executable).resolve()),
         executable_identity=identity(Path(sys.executable).resolve()), modules=modules, installed=installed,
         script=dict(path=str(script), **identity(script)), versions=versions, loaded_providers=libraries,
         accepted_source_restoration=False, runtime_change="fresh admitted CPython3.12.10 and wheels; old versions retained"))
    import numpy
    numpy.show_config()


def build_inputs(private):
    import pysam

    policy = json.loads(ASSET.read_text())
    plans = fixture_plans()
    require(policy["invocations"] == invocation_plan() and set(policy["fixtures"]) == set(plans), "frozen six-call/case plan differs")
    save(private / "before-source-plan.json", dict(fixtures=plans, invocations=invocation_plan(), policy=policy))
    results = {}
    for name, plan in plans.items():
        require(plan["intent"] == policy["fixtures"][name]["intent"], "before-outcome fixture intent differs")
        directory = private / "inputs" / name
        directory.mkdir(parents=True)
        for key, filename in (("sam", "input.sam"), ("bed", "genes.bed12")):
            path = directory / filename
            path.write_text(plan[key])
            require(identity(path)["sha256"] == policy["fixtures"][name][key + "_sha256"], "frozen input text differs")
        bam = directory / "input.bam"
        pysam.sort("-@", "1", "-T", str(directory / "sort-tmp"), "-o", str(bam), str(directory / "input.sam"))
        with pysam.AlignmentFile(bam, "rb") as alignment:
            require(list(alignment.references) == ["chr1", "CHR1"] and list(alignment.lengths) == [10000, 10000], "actual BAM dictionary differs")
            records = [record.to_string() for record in alignment.fetch(until_eof=True)]
        expected_records = Counter(line for line in plan["sam"].splitlines() if not line.startswith("@"))
        require(Counter(records) == expected_records, "actual sorted BAM records differ from frozen SAM")
        ordered = dict(plan, sam="\n".join(records) + "\n")
        derived = predicted(ordered)
        save(directory / "record-cigar-inventory.json", dict(sorted_sam_records=records, derived_policy=derived,
             inputs={file.name: identity(file) for file in (directory / "input.sam", directory / "genes.bed12", bam)}))
        results[name] = (bam, directory / "genes.bed12", derived)
    return results


def parse_counters(stdout, stderr, derived, case):
    require(re.findall(r"^total = (\d+)$", stdout, re.MULTILINE) == [str(derived["events"]["total"])], "actual total-event stdout differs")
    labels = {"Total splicing  Events": ("events", "total"), "Known Splicing Events": ("events", "annotated"),
              "Partial Novel Splicing Events": ("events", "partial_novel"), "Novel Splicing Events": ("events", "complete_novel"),
              "Filtered Splicing Events": ("events", "filtered"), "Total splicing  Junctions": ("distinct", "total"),
              "Known Splicing Junctions": ("distinct", "annotated"), "Partial Novel Splicing Junctions": ("distinct", "partial_novel"),
              "Novel Splicing Junctions": ("distinct", "complete_novel")}
    observed = {}
    for label, (kind, key) in labels.items():
        matches = re.findall(r"^" + re.escape(label) + r":\s*(\d+)\s*$", stderr, re.MULTILINE)
        expected = [] if case == "no-N" or case == "all-short" and kind == "distinct" else [str(derived[kind][key])]
        require(matches == expected, "actual event/distinct counter differs: " + label)
        observed[label] = list(map(int, matches))
    if case in ("no-N", "all-short"):
        require(("No splice junction found." if case == "no-N" else "No splice read found") in stderr, "named source anomaly absent")
    require("Traceback" not in stderr and "Invalid bed line" not in stderr and "Warning:" not in stderr, "unexpected source diagnostic")
    return observed


def capture_sources(private):
    inputs = build_inputs(private)
    tool = Path(sysconfig.get_path("scripts")) / "junction_annotation.py"
    observations, failures = [], []
    for invocation in invocation_plan():
        directory = private / "observations" / invocation["name"]
        directory.mkdir(parents=True)
        bam, bed, derived = inputs[invocation["case"]]
        prefix = directory / "observed"
        argv = [sys.executable, "-B", tool, "-i", bam, "-r", bed, "-o", prefix, *invocation["options"]]
        save(directory / "before-call.json", dict(invocation=invocation, argv=list(map(str, argv)),
             input_bam=identity(bam), annotation=identity(bed), script=identity(tool), cgroup=cgroup(),
             cwd=str(Path.cwd()), environment={name: os.environ.get(name) for name in (*CACHES,
             "PYTHONNOUSERSITE", "PYTHONDONTWRITEBYTECODE", "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS",
             "MKL_NUM_THREADS", "BLIS_NUM_THREADS", "NUMEXPR_NUM_THREADS")}))
        try:
            report = run(argv, directory / "call", expected=invocation["expected_exit"])
            primary = Path(str(prefix) + ".junction.xls")
            plot = Path(str(prefix) + ".junction_plot.r")
            rows = parse_primary(primary.read_text(), empty=invocation["case"] == "no-N")
            require(rows == derived["rows"], "actual full row identities/support/classes/order differ")
            stdout = (directory / "call/process.stdout").read_text()
            stderr = (directory / "call/process.stderr").read_text()
            counters = parse_counters(stdout, stderr, derived, invocation["case"])
            exports = invocation["name"].endswith("-exports")
            expected_files = {"observed.junction.xls", "observed.junction_plot.r"}
            if exports:
                expected_files.update(("observed.junction.bed", "observed.junction.Interact.bed"))
            require({path.name for path in directory.glob("observed*")} == expected_files, "ordinary source side-effect set differs")
            text = plot.read_text()
            require(text == "" if invocation["case"] == "no-N" else
                    "events=c(" in text and (("junction=c(" in text) == (invocation["case"] != "all-short")), "source partial/ordinary R body differs")
            if exports:
                require(len((directory / "observed.junction.bed").read_text().splitlines()) == len(rows) and
                        len((directory / "observed.junction.Interact.bed").read_text().splitlines()) == len(rows) + 1,
                        "ordinary source export row count differs")
            observation = dict(name=invocation["name"], case=invocation["case"], status="observed-qualified",
                 raw_returncode=report["returncode"], expected_anomaly=invocation["case"] if invocation["case"] in ("no-N", "all-short") else None,
                 rows=rows, counters=counters, files={path.name: identity(path) for path in directory.glob("observed*")},
                 cgroup_after=cgroup())
            require(not re.search(r"^oom(?:_kill|_group_kill)? [1-9]", observation["cgroup_after"]["memory.events"], re.MULTILINE), "actual source OOM is not qualification/performance success")
            save(directory / "observation.json", observation)
            observations.append(observation)
        except Exception:
            failure = dict(name=invocation["name"], status="unqualified-originals-retained", error=traceback.format_exc(), cgroup=cgroup())
            save(directory / "failure.json", failure)
            failures.append(failure)
    result = dict(schema="rsomics.rnaseq-qc.junction-source.v1", status="qualified" if not failures else "failed",
                  runtime=identity(private / "runtime/runtime.json"), policy=identity(ASSET),
                  observations=observations, failures=failures, source_invocations=6,
                  scope="actual ParseBAM.annotate_junction CLI; no R execution or native/performance claim")
    save(private / "qualification.json", result)
    require(not failures and len(observations) == 6, "source qualification contradicted frozen policy; all available originals retained")


def execute(session):
    private = session / "private"
    policy = json.loads(ASSET.read_text())
    try:
        controller = private / "controller"
        controller.mkdir()
        (controller / "scripts").mkdir()
        for name in CLOSURE:
            shutil.copy2(ROOT / "scripts" / name, controller / "scripts" / name)
        for relative in ("assets/rnaseq-qc-junctions-20261101.json", "assets/private-artifact-tools-20261101.json",
                         "assets/private-artifacts-20261101.recipient", WORKFLOW,
                         ".autopilot/oracles/rnaseq-qc-first-workflow-2026-10-27.json"):
            target = controller / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, target)
        save(private / "controller-identities.json", inventory(controller))
        admit_machine(private, policy)
        accepted_source(private, policy)
        python = prepare_runtime(private, policy)
        run([python, "-B", __file__, "capture", "--root", private], private / "source-driver", timeout=2400)
        capacity(private)
    except BaseException:
        save(private / "failure.json", dict(status="failed", error=traceback.format_exc(), cgroup=cgroup()))
        raise


def write_tar(private, members):
    with tarfile.open(fileobj=sys.stdout.buffer, mode="w|", format=tarfile.PAX_FORMAT) as archive:
        for row in members:
            path = private / row["name"]
            member = tarfile.TarInfo(row["name"])
            member.mode, member.mtime = row["mode"], row["mtime"]
            if row["kind"] == "directory":
                member.type = tarfile.DIRTYPE
                archive.addfile(member)
            elif row["kind"] == "symlink":
                require(os.readlink(path) == row["target"], "finalized symlink changed")
                member.type, member.linkname = tarfile.SYMTYPE, row["target"]
                archive.addfile(member)
            else:
                require(identity(path) == {key: row[key] for key in ("bytes", "sha256")}, "finalized evidence body changed")
                member.size = row["bytes"]
                with path.open("rb") as source:
                    archive.addfile(member, source)


def finalize(session):
    private = session / "private"
    require(identity(RECIPIENT)["sha256"] == RECIPIENT_SHA, "fixed recipient differs")
    tool = prepare_tool("linux-amd64", private / "age-tool")
    save(private / "final-cgroup.json", cgroup())
    members = inventory(private)
    save(private / "archive-members.json", dict(members=members,
         scope="all available private originals, failures, controller, runtime and observations; excludes this self-index and sealing-time sibling diagnostics"))
    members = inventory(private)
    limits = json.loads(ASSET.read_text())["limits"]
    retained = sum(row["bytes"] for row in members)
    bound = retained + len(members) * 8192 + 10240
    require(retained <= limits["retained_bytes"] and bound <= limits["tar_bytes"], "finalization retained/TAR bound exceeded")
    require(shutil.disk_usage(private).free >= finalization_space(retained, bound)["required_free_bytes"], "finalization space unavailable")
    manifest = session / "sealed-members.json"
    save(manifest, members)
    receipt = encrypt(tool, RECIPIENT, session / "cipher/evidence.tar.age", producer=[sys.executable, "-B", __file__,
        "tar", "--root", private, "--members", manifest], plaintext_limit=bound,
        context=dict(repository="omics-rust/rsomics-world", product="rsomics-rnaseq-qc", head_sha=os.environ["GITHUB_SHA"],
                     run_id=int(os.environ["GITHUB_RUN_ID"]), run_attempt=int(os.environ["GITHUB_RUN_ATTEMPT"]),
                     workflow_path=WORKFLOW, phase="junction-source"))
    require(receipt["recipient_sha256"] == RECIPIENT_SHA, "sealed recipient differs")
    save(session / "cipher/transport.json", receipt)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("init", "transport", "execute", "probe", "capture", "finalize", "tar"))
    parser.add_argument("--root", type=Path)
    parser.add_argument("--members", type=Path)
    args = parser.parse_args()
    if args.action == "init":
        initialize()
    elif args.action in ("probe", "capture", "tar"):
        session = hosted()
        require(args.root.resolve(strict=True) == session / "private", "child evidence root differs")
        if args.action == "probe":
            runtime_probe(args.root)
        elif args.action == "capture":
            capture_sources(args.root)
        else:
            write_tar(args.root, json.loads(args.members.read_text()))
    else:
        {"transport": transport, "execute": execute, "finalize": finalize}[args.action](hosted())


if __name__ == "__main__":
    main()
