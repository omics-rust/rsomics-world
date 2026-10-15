"""Offline owner of the fixed, authenticated raw-ingestion paired campaign."""

import argparse
from collections.abc import Mapping
from dataclasses import fields, is_dataclass
import gzip
import hashlib
import importlib
import json
import math
import os
from pathlib import Path
import platform
import re
import stat
import sys
import tomllib
import zipfile


ROOT = Path(__file__).resolve().parent.parent
SCRIPTS = ROOT / "scripts"
INPUT_RECEIPT_SHA = "3bee1dc582a321f16cb173c757e955dc620a911a891f4051f8bae68dc856dc0b"
ORIGINAL_RECEIPT_SHA = "41284ea567fd2c41a48e22eef3c9063a59944607828ebc9cdea8a3f6da82e526"
NATIVE_RECEIPT_SHA = "4be244bb087c074171d05e1dc02904c891957ab96d90523e03ccf26a79f2fbb1"
INSTALLED_RECEIPT_SHA = "8785cb31c96d7b09497bb5b7af1a73ee9d4b2406d8940e4a9cc9bbf27470ee4a"
CONTEXT_KEYS = {
    "schema_version", "artifact_root", "r_source_root", "input_receipt",
    "original_receipt", "original_run_json", "original_artifact_json",
    "original_zip", "native_receipt", "native_run_json", "native_jobs_json",
    "native_snapshot_dir", "native_source_root", "cargo_artifacts",
    "cargo_target_dir", "python", "rscript", "r_executable", "gnu_time",
    "rustc", "contract_root", "installed_contract_receipt", "code_sha256",
}
DIRECTORIES = {"artifact_root", "r_source_root", "native_snapshot_dir", "native_source_root",
               "cargo_target_dir", "contract_root"}
FIXED_CODE = {
    "scripts/infercnv_trial_process.py": "37c1d558524ebc6ec48dd90d16dc36d231ccece431e894e2a534a69f088b2444",
    "scripts/infercnv_python_identity.py": "654cd3aede60d5aff366b96ad85cd5ecc0a8e6130a35905c752e973f99eb07ab",
    "scripts/validate_infercnv_trial_metadata.py": "2a2b5dd64169aab03b9ba3c4e987144497de2433091a035ddc3fd7811e59e771",
    "scripts/infercnv_ingestion_campaign.py": "212bf00ad68c59e652f220eacdf76350ca4e56d48752c3b1592fd46dba5c4dea",
    "scripts/validate_infercnv_ingestion_measurement.py": "0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36",
    "scripts/validate_infercnv_samples_clustering_witness.py": "682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222",
    "scripts/verify_infercnv_ingestion_snapshot.py": "426c6ab0ae9abd9f0198d6de4af11717f67c2ab5d44f3d9999b0ec06ac1a10b2",
    "scripts/verify_infercnv_measurement_snapshot.py": "24748f98933c24bd9886d2725eba72be734a5c7c6c71cf910af1cf43353fc8b1",
    "scripts/infercnv_source_snapshot.py": "095ebb67a2d9a050c20b6e876f2b0c9cfd26fcbf8f0ce3601a8124b292b553f2",
    "scripts/verify_infercnv_oracle_artifact.py": "651ac3082d92f952bf026f134bfb0482f50b118dcfab367a497bbb0fe863571f",
    "scripts/infercnv_ingestion_matched.R": "eb09009172c905363407f49c63bfa164da8d32a29ea8ec8a17a8701dc6e231d0",
    "scripts/infercnv_ingestion_runtime_support.R": "bcbbd0e3b6a2d053562345cd9ee59a4ff3e9dc4c8d4e6e2a7db66097d5787418",
    "scripts/infercnv_ingestion_matched_support.R": "d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa",
    "scripts/infercnv_matched_support.R": "0c3870354e5ac5bafedbd018148492bb5f339714b5412c409866825b2e783956",
    "scripts/infercnv_oracle_io.R": "5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d",
    ".autopilot/oracles/infercnv-ingestion-measurement-input-2026-09-30.json": INPUT_RECEIPT_SHA,
    ".autopilot/oracles/infercnv-samples-clustering-witness-2026-09-30.json": ORIGINAL_RECEIPT_SHA,
    ".autopilot/oracles/infercnv-ingestion-trial-candidate-2026-10-15.json": NATIVE_RECEIPT_SHA,
    ".autopilot/oracles/infercnv-installed-creation-contracts-2026-10-15.json": INSTALLED_RECEIPT_SHA,
}
CODE_NAMES = frozenset(FIXED_CODE) | {"scripts/measure_infercnv_ingestion.py"}
IDENTITY_KEYS = {"executable", "executable_sha256", "version", "prefix", "base_prefix",
                 "stdlib_json", "libpython_sha256"}
THREAD_KEYS = ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS", "BLIS_NUM_THREADS",
               "VECLIB_MAXIMUM_THREADS", "NUMEXPR_NUM_THREADS", "RAYON_NUM_THREADS")
NAMESPACES = ("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")
TIME_FORMAT = "wall_seconds\t%e\nuser_seconds\t%U\nsystem_seconds\t%S\nmaximum_rss_kib\t%M\nexit_status\t%x"
TIME_KEYS = ("wall_seconds", "user_seconds", "system_seconds", "maximum_rss_kib", "exit_status")
HASH = re.compile(r"[0-9a-f]{64}\Z")


def _require(condition, message):
    if not condition:
        raise ValueError(message)


def _pairs(items):
    result = {}
    for key, value in items:
        _require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _json(raw):
    def invalid(value):
        raise ValueError(f"nonfinite JSON constant: {value}")
    return json.loads(raw.decode("utf-8"), object_pairs_hook=_pairs, parse_constant=invalid)


def _physical(value, directory=False, new=False):
    _require(type(value) is str and value and not any(ord(c) < 32 or ord(c) == 127 for c in value),
             "invalid physical path")
    path = Path(value)
    _require(path.is_absolute() and str(path) == value and not value.startswith("//") and
             ".." not in path.parts, f"noncanonical physical path: {value}")
    for ancestor in path.parents:
        _require(stat.S_ISDIR(ancestor.lstat().st_mode), f"non-directory/symlink ancestor: {ancestor}")
    if new:
        _require(not os.path.lexists(path), f"output already exists: {path}")
    else:
        mode = path.lstat().st_mode
        _require(stat.S_ISDIR(mode) if directory else stat.S_ISREG(mode), f"nonregular physical path: {path}")
        _require(path.resolve(strict=True) == path, f"aliased physical path: {path}")
    return path


def _sha256(path):
    path = _physical(str(path))
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _read_json(path):
    return _json(_physical(str(path)).read_bytes())


def _plain(value):
    if is_dataclass(value):
        return {field.name: _plain(getattr(value, field.name)) for field in fields(value)}
    if isinstance(value, Mapping):
        return {key: _plain(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_plain(item) for item in value]
    return str(value) if isinstance(value, Path) else value


def _write_json(path, value):
    with path.open("x", encoding="utf-8") as target:
        json.dump(_plain(value), target, sort_keys=True, allow_nan=False, indent=2)
        target.write("\n")


def _load_context(path):
    context = _read_json(path)
    _require(type(context) is dict and set(context) == CONTEXT_KEYS, "context keys differ")
    _require(type(context["schema_version"]) is int and context["schema_version"] == 1, "context schema differs")
    code = context["code_sha256"]
    _require(type(code) is dict and set(code) == CODE_NAMES, "context code name set differs")
    _require(all(type(value) is str and HASH.fullmatch(value) for value in code.values()), "invalid context code hash")
    for key in CONTEXT_KEYS - {"schema_version", "code_sha256"}:
        _physical(context[key], directory=key in DIRECTORIES)
    _require(context["native_source_root"] != context["r_source_root"], "native and R source roots coincide")
    return context


def _guard_imports():
    _require("PYTHONPATH" not in os.environ and "PYTHONHOME" not in os.environ, "Python environment injection is forbidden")
    _require(sys.dont_write_bytecode, "owner must disable bytecode (-B)")
    for path in SCRIPTS.rglob("*"):
        _require(path.suffix not in (".pyc", ".pyo"), f"shadowing bytecode: {path}")


def _check_code(context):
    _guard_imports()
    _require(set(context["code_sha256"]) == CODE_NAMES, "context code name set differs")
    for name, digest in FIXED_CODE.items():
        _require(context["code_sha256"][name] == digest, f"independent fixed code pin differs: {name}")
    actual = {name: _sha256(ROOT / name) for name in sorted(CODE_NAMES)}
    _require(actual == context["code_sha256"], "code bytes differ from independently supplied pins")
    for name in CODE_NAMES:
        if name.endswith(".py"):
            module = sys.modules.get(Path(name).stem)
            if module is not None:
                _require(Path(module.__file__) == ROOT / name, f"shadowed imported module: {name}")
    return actual


def _module(name):
    _guard_imports()
    for relative, digest in FIXED_CODE.items():
        if relative.endswith(".py"):
            _require(_sha256(ROOT / relative) == digest, f"unsafe local dependency: {relative}")
    module = importlib.import_module(name)
    _require(Path(module.__file__) == SCRIPTS / (name + ".py"), f"shadowed dependency: {name}")
    return module


def _check_receipt(path, expected):
    _require(_sha256(path) == expected, f"physical receipt hash differs: {path}")
    return _read_json(path)


def _check_inputs(c):
    receipt = _check_receipt(c["input_receipt"], INPUT_RECEIPT_SHA)
    _check_receipt(c["original_receipt"], ORIGINAL_RECEIPT_SHA)
    original = _module("verify_infercnv_oracle_artifact")
    admitted = original._receipt(c["original_receipt"])
    original._run(c["original_run_json"], admitted)
    original._artifact(c["original_artifact_json"], admitted, c["original_zip"])
    _require(original._sha256(c["original_zip"]) == admitted["artifact_sha256"], "original ZIP digest differs")
    root = Path(c["artifact_root"])
    with zipfile.ZipFile(c["original_zip"]) as archive:
        names = original._validate_archive(archive)
        _require(archive.testzip() is None, "original ZIP CRC differs")
        actual = set()
        for path in root.rglob("*"):
            if path.is_dir() and not path.is_symlink():
                continue
            _physical(str(path))
            actual.add(path.relative_to(root).as_posix())
        _require(actual == names, "existing original extraction inventory differs")
        original._manifest(root / original.BUNDLE, admitted["manifest_sha256"], names)
    metadata = _module("validate_infercnv_trial_metadata")
    expected = {"plain": receipt["plain"]["sha256"], "gzip": receipt["gzip"]["sha256"],
                "archive": receipt["source_archive_sha256"], **receipt["files"], **receipt["factory_source"]}
    hashes = {}
    for key, name in metadata.INPUT_RELATIVE.items():
        path = Path(c["r_source_root"] if key.startswith("R/") else c["artifact_root"]) / name
        hashes[key] = _sha256(path)
    _require(hashes == expected, "raw/auxiliary/expected/R-source hashes differ")
    for case in ("plain", "gzip"):
        _require((root / receipt[case]["path"]).stat().st_size == receipt[case]["bytes"], f"{case} input size differs")
    decoded_hash, decoded_bytes = hashlib.sha256(), 0
    with gzip.open(root / receipt["gzip"]["path"], "rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            decoded_bytes += len(chunk)
            _require(decoded_bytes <= receipt["gzip"]["decoded_bytes"], "decoded gzip exceeds admitted size")
            decoded_hash.update(chunk)
    _require(decoded_bytes == receipt["gzip"]["decoded_bytes"] == receipt["plain"]["bytes"] and
             decoded_hash.hexdigest() == receipt["gzip"]["decoded_sha256"] == hashes["plain"],
             "decoded gzip hash/size differs")
    paths = {key: Path(c["r_source_root"] if key.startswith("R/") else c["artifact_root"]) / name
             for key, name in metadata.INPUT_RELATIVE.items()}
    return {"sha256": hashes, "paths": {key: str(path) for key, path in paths.items()},
            "file_bytes": {key: path.stat().st_size for key, path in paths.items()},
            "decoded_bytes": decoded_bytes, "decoded_sha256": decoded_hash.hexdigest(),
            "admission_sha256": {key: _sha256(c[key]) for key in
                                 ("input_receipt", "original_receipt", "original_run_json", "original_artifact_json", "original_zip")}}


def _check_native(c):
    receipt = _check_receipt(c["native_receipt"], NATIVE_RECEIPT_SHA)
    raw = _module("verify_infercnv_ingestion_snapshot")
    raw._raw_jobs(receipt, _read_json(c["native_run_json"]), _read_json(c["native_jobs_json"]))
    result = _module("verify_infercnv_measurement_snapshot").verify_extracted(
        c["native_receipt"], c["native_snapshot_dir"], c["native_source_root"])
    raw._raw_cargo(Path(c["native_source_root"]))
    _require(result["verified_files"] == 149, "native source file count differs")
    cargo = tomllib.loads((Path(c["native_source_root"]) / "Cargo.toml").read_text(encoding="utf-8"))
    return dict(result, native_version=cargo["package"]["version"],
                admission_sha256={key: _sha256(c[key]) for key in ("native_receipt", "native_run_json", "native_jobs_json")})


def _cargo_executable(c):
    rows = _physical(c["cargo_artifacts"]).read_bytes().splitlines()
    _require(bool(rows), "empty Cargo artifact log")
    records = [_json(row) for row in rows]
    _require(all(type(row) is dict for row in records), "invalid Cargo message")
    finishes = [row for row in records if row.get("reason") == "build-finished"]
    _require(len(finishes) == 1 and finishes[0].get("success") is True, "Cargo build did not uniquely succeed")
    _require(not any(row.get("reason") == "compiler-message" and row.get("message", {}).get("level")
                     in ("error", "failure-note") for row in records), "Cargo compiler failure")
    matches = [row for row in records if row.get("reason") == "compiler-artifact" and
               row.get("target", {}).get("name") == "cnv_ingestion_matched"]
    _require(len(matches) == 1, "Cargo bench artifact is missing or ambiguous")
    row = matches[0]
    source = Path(c["native_source_root"])
    target, profile = row.get("target", {}), row.get("profile", {})
    _require(row.get("manifest_path") == str(source / "Cargo.toml") and target.get("kind") == ["bench"] and
             target.get("crate_types") == ["bin"] and target.get("src_path") == str(source / "benches/cnv_ingestion_matched.rs") and
             row.get("features") == ["infercnv-measurement"] and profile.get("opt_level") == "3" and
             profile.get("test") is False and profile.get("debug_assertions") is False,
             "Cargo artifact target/features/profile differs")
    executable = _physical(row.get("executable"))
    _require(executable.is_relative_to(Path(c["cargo_target_dir"])) and os.access(executable, os.X_OK) and
             type(row.get("filenames")) is list and str(executable) in row["filenames"],
             "Cargo executable outside target or nonexecutable/unlisted")
    return executable


def _check_executables(c, native):
    _require(native == _cargo_executable(c), "Cargo executable changed")
    paths = {key: Path(c[key]) for key in ("python", "rscript", "r_executable", "gnu_time", "rustc")}
    paths["native"] = native
    for path in paths.values():
        _physical(str(path))
        _require(os.access(path, os.X_OK), f"not executable: {path}")
    return {"sha256": {key: _sha256(path) for key, path in paths.items()},
            "paths": {key: str(path) for key, path in paths.items()}, "cargo_artifacts_sha256": _sha256(c["cargo_artifacts"])}


def _check_host():
    _require(sys.version_info[:2] == (3, 12), "campaign requires actual owner Python 3.12")
    _require(platform.system() == "Linux" and platform.machine() == "x86_64", "campaign requires native Linux x86_64")
    for name in ("maps", "io", "smaps_rollup", "status"):
        _require((Path("/proc/self") / name).is_file(), "campaign requires live procfs")


def _environment(c):
    env = dict(os.environ)
    env.update({key: "1" for key in THREAD_KEYS})
    env.update(LC_ALL="C", LANG="C", PYTHONDONTWRITEBYTECODE="1")
    env["PATH"] = str(Path(c["python"]).parent) + os.pathsep + env.get("PATH", os.defpath)
    return env


def _command(argv, prefix, env):
    return _module("infercnv_trial_process").run_child([str(value) for value in argv], prefix, timeout=1200, env=env)


def _stdout(prefix):
    return _physical(str(prefix) + ".stdout").read_bytes().decode("utf-8")


def _compare_identity(actual, expected):
    _require(type(actual) is dict and set(actual) == IDENTITY_KEYS and type(expected) is dict and
             set(expected) == IDENTITY_KEYS, "full Python identity fields differ")
    _require(all(type(actual[key]) is str for key in IDENTITY_KEYS - {"libpython_sha256"}) and
             type(actual["libpython_sha256"]) is dict and actual == expected, "full Python identity differs")


def _identity_line(text):
    value = _json(text.encode("utf-8"))
    _require(text == json.dumps(value, sort_keys=True) + "\n", "identity must be one sorted JSON line")
    return value


def _runtime_probes(c, output, phase, env, expected=None):
    _require(_sha256(SCRIPTS / "infercnv_python_identity.py") == c["code_sha256"]["scripts/infercnv_python_identity.py"],
             "unsafe identity probe code")
    result, failures = {}, []
    try:
        identity = _module("infercnv_python_identity").capture_identity()
        _require(identity["executable"] == c["python"] and re.match(r"3\.12\.[0-9]+(?:\s|$)", identity["version"]),
                 "owner Python differs from admitted Python 3.12")
        result["Python"] = identity
    except Exception as exc:
        failures.append(f"owner: {type(exc).__name__}: {exc}")
    script = SCRIPTS / "infercnv_python_identity.py"
    shell_prefix = output / (phase + "-python-shell")
    quote = json.dumps
    packages = ",".join(quote(name) for name in NAMESPACES)
    code = (f'for (p in c({packages})) if (!requireNamespace(p, quietly=TRUE)) stop("namespace absent");'
            f'if (!identical(normalizePath(file.path(R.home(),"bin","exec","R"),mustWork=TRUE),{quote(c["r_executable"])})) stop("R executable differs");'
            'p <- Sys.which("python3"); if (!nzchar(p)) stop("python3 absent");'
            f'p <- normalizePath(p,mustWork=TRUE); if (!identical(p,{quote(c["python"])})) stop("Python executable differs");'
            f's <- system2(p,c("-I","-B",shQuote({quote(str(script))})),stdout="",stderr="");'
            'if (!identical(s,0L)) stop("Python identity child failed");')
    r_prefix = output / (phase + "-python-R")
    for key, argv, prefix in (("shell", [c["python"], "-I", "-B", script], shell_prefix),
                              ("R", [c["rscript"], "--vanilla", "-e", code], r_prefix)):
        try:
            _command(argv, prefix, env)
            result[key] = _identity_line(_stdout(prefix))
        except Exception as exc:
            failures.append(f"{key}: {type(exc).__name__}: {exc}")
    _require(not failures, "runtime probe failed: " + "; ".join(failures))
    for key in ("shell", "R"):
        _compare_identity(result[key], result["Python"])
    if expected is not None:
        for key in ("Python", "shell", "R"):
            _compare_identity(result[key], expected[key])
    return result


def _versions(c, output, phase, env):
    result, failures = {}, []
    commands = {"rustc": [c["rustc"], "-vV"], "Rscript": [c["rscript"], "--version"],
                "R": [c["r_executable"], "--version"], "gnu_time": [c["gnu_time"], "--version"]}
    for key, argv in commands.items():
        prefix = output / f"{phase}-version-{key}"
        try:
            _command(argv, prefix, env)
            result[key] = _stdout(prefix)
        except Exception as exc:
            failures.append(f"{key}: {type(exc).__name__}: {exc}")
    _require(not failures, "version probes failed: " + "; ".join(failures))
    rust = dict(line.split(": ", 1) for line in result["rustc"].splitlines()[1:] if ": " in line)
    _require(rust.get("release") == "1.91.0" and rust.get("host") == "x86_64-unknown-linux-gnu", "actual rustc release/host differs")
    _require(re.match(r"Rscript \(R\) version 4\.6\.1(?:\s|$)", result["Rscript"]) and
             re.match(r"R version 4\.6\.1(?:\s|$)", result["R"]), "actual R/Rscript version differs")
    _require("GNU Time" in result["gnu_time"] or "GNU time" in result["gnu_time"], "actual time is not GNU time")
    return result


def _bindings(c, case, output, inputs, native, native_version, profile):
    metadata = _module("validate_infercnv_trial_metadata")
    paths = {key: Path(c["r_source_root"] if key.startswith("R/") else c["artifact_root"]) / name
             for key, name in metadata.INPUT_RELATIVE.items()}
    code_paths = {key: SCRIPTS / name for key, name in metadata.CODE_NAMES.items()}
    return metadata.TrialBindings(case, Path(c["artifact_root"]), Path(c["r_source_root"]), Path(c["input_receipt"]),
        SCRIPTS, output, paths, inputs["sha256"], code_paths,
        {key: c["code_sha256"]["scripts/" + name] for key, name in metadata.CODE_NAMES.items()}, native, native_version, profile)


def _metadata(root, implementation, bindings):
    return _module("validate_infercnv_trial_metadata").validate_trial_metadata(root, implementation, bindings)


def _factory_results(root, c):
    return _module("validate_infercnv_ingestion_measurement").validate_factory_results(
        root, Path(c["artifact_root"]) / "shipped-bundle/full",
        Path(c["artifact_root"]) / "ingestion-witness/shipped_full/plain/maps.tsv")


def _build_profile(c, identity, inputs, native, native_version):
    installed = _check_receipt(c["installed_contract_receipt"], INSTALLED_RECEIPT_SHA)
    runtime = _read_json(Path(c["contract_root"]) / "runtime.json")
    proofs = installed["installed_source_proofs"]
    profile = {key: runtime[key] for key in ("system", "namespaces", "blas", "options_dput")}
    profile.update(Python=identity, object_options_dput=runtime["measured"]["options_dput"],
                   clock_source=proofs["clock_source"], functions=proofs["functions"])
    bindings = _bindings(c, "plain", Path(c["contract_root"]), inputs, native, native_version, profile)
    _metadata(Path(c["contract_root"]), "infercnv", bindings)
    _factory_results(Path(c["contract_root"]), c)
    return profile


def _factory_argv(c, native, slot, trial):
    if slot.implementation == "rust":
        return [str(native), c["artifact_root"], slot.input_case, str(trial)]
    return [c["rscript"], "--vanilla", str(SCRIPTS / "infercnv_ingestion_matched.R"), c["artifact_root"],
            slot.input_case, c["r_source_root"], c["input_receipt"], str(SCRIPTS), str(trial)]


def _parse_time(text):
    _require(type(text) is str and text.endswith("\n") and "\r" not in text, "invalid GNU time text")
    rows = [row.split("\t") for row in text[:-1].split("\n")]
    _require(len(rows) == 5 and all(len(row) == 2 for row in rows) and tuple(row[0] for row in rows) == TIME_KEYS,
             "GNU time fields differ")
    values = dict(rows)
    result = {}
    for key in TIME_KEYS[:3]:
        raw = values[key]
        _require(re.fullmatch(r"[0-9]+(?:\.[0-9]+)?", raw) is not None, f"invalid literal GNU time {key}")
        value = float(raw)
        _require(math.isfinite(value) and value >= 0, f"invalid finite GNU time {key}")
        result[key] = value
    rss = values["maximum_rss_kib"]
    _require(re.fullmatch(r"[1-9][0-9]{0,19}", rss) is not None and int(rss) <= 2**64 - 1,
             "GNU time RSS outside positive u64")
    _require(values["exit_status"] == "0", "GNU time child exit status differs")
    return dict(result, maximum_rss_kib=int(rss), exit_status=0)


def _process_record(prefix, completed, argv, previous_end):
    record = _read_json(Path(str(prefix) + ".process.json"))
    _require(completed.argv == tuple(argv) and completed.returncode == 0 and completed.group_empty and
             type(completed.start_ns) is int and type(completed.end_ns) is int and
             previous_end <= completed.start_ns <= completed.end_ns, "invalid/nonserial owned process completion")
    expected = {"argv": argv, "pid": completed.pid, "returncode": 0, "start_ns": completed.start_ns,
                "end_ns": completed.end_ns, "status": "completed", "timed_out": False,
                "interrupted_signal": None, "error": None, "leftovers_detected": False,
                "cleanup_error": None, "final_group_alive": False, "group_empty": True}
    _require(all(type(record.get(key)) is type(value) and record[key] == value for key, value in expected.items()),
             "missing or inconsistent process evidence")
    return record


def _machine():
    return {"uname": list(platform.uname()), "cpu_count": os.cpu_count(), "load_average": list(os.getloadavg()),
            "affinity": sorted(os.sched_getaffinity(0)),
            "procfs": {name: (Path("/proc") / name).read_text(encoding="utf-8")
                       for name in ("cpuinfo", "meminfo", "loadavg", "version")}}


def _run_slots(c, output, inputs, native, native_version, profile, env, accepted):
    campaign = _module("infercnv_ingestion_campaign")
    samples, previous_end = [], 0
    for index, slot in enumerate(campaign.expected_schedule()):
        name = f"{index:02d}-{slot.input_case}-pair{slot.pair}-order{slot.order}-{slot.implementation}"
        trial, prefix = output / (name + "-result"), output / name
        _physical(str(trial), new=True)
        time_path = output / (name + ".time.tsv")
        with time_path.open("x", encoding="utf-8"):
            pass
        factory_argv = _factory_argv(c, native, slot, trial)
        argv = [c["gnu_time"], "-f", TIME_FORMAT, "-o", str(time_path), "--", *factory_argv]
        completed = _command(argv, prefix, env)
        process = _process_record(prefix, completed, argv, previous_end)
        previous_end = completed.end_ns
        whole = _parse_time(time_path.read_bytes().decode("ascii"))
        bindings = _bindings(c, slot.input_case, trial, inputs, native, native_version, profile)
        metadata = _metadata(trial, slot.implementation, bindings)
        results = _factory_results(trial, c)
        metrics = campaign.parse_region_metrics((trial / "metrics.tsv").read_bytes().decode("utf-8"))
        _require((metrics.input_case, metrics.implementation) == (slot.input_case, slot.implementation), "slot metrics differ")
        evidence = {"slot": slot, "trial_dir": trial, "factory_argv": factory_argv, "process": process,
                    "whole_process": whole, "metadata": metadata, "metrics": metrics, "creation_checks": results,
                    "controller_sha256": {str(path): _sha256(path) for path in
                                          (time_path, *(Path(str(prefix) + suffix) for suffix in (".stdout", ".stderr", ".process.json")))}}
        _write_json(output / (name + ".accepted.json"), evidence)
        accepted.append(_plain(evidence))
        samples.append(campaign.Sample(slot, trial, 0))
    return samples


def run_campaign(context_path: Path, output_dir: Path) -> dict:
    """Run exactly one serial campaign; preserve partial evidence on every failure."""
    context_path = _physical(str(context_path))
    c = _load_context(context_path)
    output = _physical(str(output_dir), new=True)
    output.mkdir()
    context_bytes = context_path.read_bytes()
    with (output / "context.json").open("xb") as target:
        target.write(context_bytes)
    context_hash = hashlib.sha256(context_bytes).hexdigest()
    before, accepted, postchecks = {}, [], []
    samples, native, error = [], None, None
    env = _environment(c)
    try:
        _check_host()
        before["code"] = _check_code(c)
        before["inputs"] = _check_inputs(c)
        before["source"] = _check_native(c)
        native = _cargo_executable(c)
        before["executables"] = _check_executables(c, native)
        before["runtime"] = _runtime_probes(c, output, "before", env)
        before["versions"] = _versions(c, output, "before", env)
        before["r_profile"] = _build_profile(c, before["runtime"]["Python"], before["inputs"], native, before["source"]["native_version"])
        before["machine"] = _machine()
        before["environment"] = {key: env[key] for key in (*THREAD_KEYS, "PATH", "LC_ALL", "LANG", "PYTHONDONTWRITEBYTECODE", "LD_LIBRARY_PATH") if key in env}
        before["context_sha256"] = context_hash
        before["context_trust"] = "workflow-supplied trust input; byte/hash retention does not prove workflow provenance"
        _write_json(output / "admission.json", before)
        samples = _run_slots(c, output, before["inputs"], native, before["source"]["native_version"], before["r_profile"], env, accepted)
    except BaseException as exc:
        error = exc
    finally:
        def unchanged(label, value):
            if label in before:
                _require(value == before[label], f"postcheck {label} drift")
            return value
        def context_check():
            _require(_sha256(context_path) == context_hash and _load_context(context_path) == c, "context drift")
            return context_hash
        checks = [("context", context_check), ("code", lambda: unchanged("code", _check_code(c))),
                  ("inputs", lambda: unchanged("inputs", _check_inputs(c))),
                  ("source", lambda: unchanged("source", _check_native(c))),
                  ("installed_contract", lambda: _check_receipt(c["installed_contract_receipt"], INSTALLED_RECEIPT_SHA)),
                  ("executables", lambda: unchanged("executables", _check_executables(c, native or _cargo_executable(c)))),
                  ("runtime", lambda: _runtime_probes(c, output, "after", env, before.get("runtime"))),
                  ("versions", lambda: unchanged("versions", _versions(c, output, "after", env))),
                  ("machine", _machine)]
        for label, check in checks:
            record = {"check": label}
            try:
                record.update(status="passed", result=_plain(check()))
            except BaseException as exc:
                record.update(status="failed", error=f"{type(exc).__name__}: {exc}")
            postchecks.append(record)
            try:
                _write_json(output / ("postcheck-" + label + ".json"), record)
            except BaseException as exc:
                record.update(status="failed", evidence_error=f"{type(exc).__name__}: {exc}")
    failures = [record for record in postchecks if record["status"] == "failed"]
    if error is not None or failures:
        failure = error if error is not None else ValueError(
            "campaign postchecks failed: " + ", ".join(record["check"] for record in failures))
        try:
            _write_json(output / "failure.json", {"error": f"{type(error).__name__}: {error}" if error is not None else None,
                                                  "accepted_slots": len(accepted), "postcheck_failures": failures})
        except BaseException as exc:
            failure.add_note(f"could not retain failure.json: {type(exc).__name__}: {exc}")
        raise failure
    try:
        for evidence in accepted:
            for name, digest in evidence["metadata"]["file_hashes"].items():
                _require(_sha256(Path(evidence["trial_dir"]) / name) == digest, "accepted trial evidence drift")
            for name, digest in evidence["controller_sha256"].items():
                _require(_sha256(name) == digest, "accepted process evidence drift")
        summary = _module("infercnv_ingestion_campaign").accept_campaign(samples,
            Path(c["artifact_root"]) / "shipped-bundle/full",
            Path(c["artifact_root"]) / "ingestion-witness/shipped_full/plain/maps.tsv")
        result = dict(schema_version=1, campaign=_plain(summary), admission=before, postchecks=postchecks, slots=accepted,
                      strict_wall_advantage=summary.strict_wall_advantage,
                      metric_scope="warm canonical ingestion only; whole-process maximum RSS is not factory/aggregate-descendant peak; lifetime I/O is not factory delta")
        _write_json(output / "summary.json", result)
        return result
    except BaseException as exc:
        try:
            _write_json(output / "failure.json", {"error": f"{type(exc).__name__}: {exc}", "accepted_slots": len(accepted), "postcheck_failures": []})
        except BaseException as report_error:
            exc.add_note(f"could not retain failure.json: {type(report_error).__name__}: {report_error}")
        raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--context", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        result = run_campaign(args.context, args.output_dir)
    except Exception as exc:
        parser.exit(1, f"ingestion campaign failed: {type(exc).__name__}: {exc}\n")
    if not result["strict_wall_advantage"]:
        parser.exit(1, "complete campaign retained; strict wall advantage failed\n")
    print(f"accepted matched raw-ingestion campaign: {args.output_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
