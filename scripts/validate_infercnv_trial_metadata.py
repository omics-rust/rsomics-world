"""Private metadata gate; caller owns input admission and process chronology."""

from collections.abc import Mapping
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import stat
from types import MappingProxyType

from infercnv_ingestion_campaign import parse_region_metrics


RECEIPT_SHA = "3bee1dc582a321f16cb173c757e955dc620a911a891f4051f8bae68dc856dc0b"
INPUT_RELATIVE = {
    "plain": "shipped-bundle/inputs/canonical-full.tsv",
    "gzip": "ingestion-witness/shipped_full/counts.tsv.gz",
    **{name: name for name in (
        "shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt",
        "shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt",
        "shipped-bundle/full/01.tsv", "shipped-bundle/full/01.genes.tsv", "shipped-bundle/full/01.cells.tsv",
        "ingestion-witness/shipped_full/plain/maps.tsv",
        "shipped-bundle/full/upstream/01_incoming_data.infercnv_obj")},
    "archive": "infercnv-source.tar.gz", "R/inferCNV.R": "R/inferCNV.R",
    "R/inferCNV_constants.R": "R/inferCNV_constants.R",
}
CODE_NAMES = dict(trial="infercnv_ingestion_matched.R", runtime_helper="infercnv_ingestion_runtime_support.R",
                  creation_support="infercnv_ingestion_matched_support.R", counter_support="infercnv_matched_support.R",
                  oracle_io="infercnv_oracle_io.R", creation_checker="validate_infercnv_ingestion_measurement.py",
                  imported_validator="validate_infercnv_samples_clustering_witness.py")
FIXED_CODE = {
    "creation_support": "d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa",
    "counter_support": "0c3870354e5ac5bafedbd018148492bb5f339714b5412c409866825b2e783956",
    "oracle_io": "5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d",
    "creation_checker": "0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36",
    "imported_validator": "682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222",
}
REFERENCES = ["Microglia/Macrophage", "Oligodendrocytes (non-malignant)"]
EXCLUSIONS = ["chrX", "chrY", "chrM"]
FUNCTIONS = ("CreateInfercnvObject", ".order_reduce", "validate_infercnv_obj")
NAMESPACES = ("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion")
THREAD_KEYS = ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS", "BLIS_NUM_THREADS",
               "VECLIB_MAXIMUM_THREADS", "NUMEXPR_NUM_THREADS", "RAYON_NUM_THREADS")
LOCALES = ("LC_COLLATE", "LC_CTYPE", "LC_MONETARY", "LC_NUMERIC", "LC_TIME", "LC_MESSAGES")
SYSTEM_KEYS = ("sysname", "release", "version", "nodename", "machine", "login", "user", "effective_user")
IO_KEYS = ("rchar", "wchar", "syscr", "syscw", "read_bytes", "write_bytes", "cancelled_write_bytes")
PHASE_FILES = ("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv")
R_FILES = ("clock-proc-time.txt", "function-.order_reduce.txt", "function-CreateInfercnvObject.txt",
           "function-validate_infercnv_obj.txt", "functions.tsv", "io.lifetime.txt", "measured-checker.stderr",
           "measured-checker.stdout", "warm-checker.stderr", "warm-checker.stdout", "metrics.tsv",
           "packages.tsv", "provenance.json", "runtime.json", "session.txt")
R_POLICIES = dict(
    preparation_policy="verified raw/expected/source bytes; validated/exported warm factory result; released local warm/oracle states; gc(full=TRUE); baseline",
    region_clock="proc.time elapsed/self/child counters around actual CreateInfercnvObject; seconds rounded down to milliseconds per installed R documentation; actual accuracy is system-dependent, not assumed nanoseconds",
    rss_scope="baseline then returned object live before delta formatting, validation, export or post-call gc",
    io_scope="unmodified /proc/self/io lifetime snapshot after both verified exports and rechecks, before final metadata writes; includes startup, source-proof writes, warmup, validators, exports and instrumentation; not factory-region deltas",
    lifetime_scope="external monitor includes package startup, warm-up, independent Python validators and exports; not factory peak")
AUTHENTICATION = "external controller MUST authenticate exact trial/helper/reused-R/Python bytes against independently frozen source before process launch; self/runtime-helper hashes here are drift baselines, not circular authenticity pins"


@dataclass(frozen=True)
class TrialBindings:
    input_case: str
    artifact_root: Path
    source_root: Path
    receipt_path: Path
    scripts_dir: Path
    output_dir: Path
    input_paths: Mapping[str, Path]
    input_hashes: Mapping[str, str]
    code_paths: Mapping[str, Path]
    code_hashes: Mapping[str, str]
    native_executable: Path
    native_version: str
    r_profile: Mapping


@dataclass(frozen=True)
class CheckedMetadata:
    implementation: str
    input_case: str
    observed_threads_before: int
    observed_threads_after: int
    io_counters: Mapping[str, int]
    file_hashes: Mapping[str, str]


def _require(condition, message):
    if not condition:
        raise ValueError(message)


def _keys(value, keys, label):
    _require(isinstance(value, Mapping) and set(value) == set(keys), f"{label} keys differ")


def _equal(actual, expected, label):
    if isinstance(expected, Mapping):
        _keys(actual, expected, label)
        for key, value in expected.items():
            _equal(actual[key], value, f"{label}.{key}")
    elif type(expected) is list:
        _require(type(actual) is list and len(actual) == len(expected), f"{label} list shape differs")
        for index, value in enumerate(expected):
            _equal(actual[index], value, f"{label}[{index}]")
    else:
        _require(type(actual) is type(expected) and actual == expected, f"{label} differs")


def _text(value, label):
    _require(type(value) is str and bool(value) and not any(c in value for c in "\x00\r\n\t"), f"invalid {label}")


def _path(value, label):
    _require(isinstance(value, Path), f"{label} must be Path")
    raw = str(value)
    _text(raw, label)
    _require(value.is_absolute() and not raw.startswith("//") and ".." not in value.parts, f"noncanonical {label}")


def _profile_path(value, label):
    _text(value, label)
    path = Path(value)
    _require(str(path) == value, f"noncanonical {label}")
    _path(path, label)


def _hash(value, label):
    _require(type(value) is str and re.fullmatch("[0-9a-f]{64}", value) is not None, f"invalid {label} hash")


def _uint(value, label, zero=False, literal=False):
    if literal:
        _require(type(value) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}", value) is not None,
                 f"invalid canonical decimal {label}")
        value = int(value)
    _require(type(value) is int and (0 if zero else 1) <= value <= 2**64 - 1, f"invalid u64 {label}")
    return value


def _string_list(value, label):
    _require(type(value) is list and bool(value), f"invalid {label} lines")
    for line in value:
        _require(type(line) is str and "\x00" not in line and "\r" not in line and "\n" not in line,
                 f"invalid {label} line")


def _profile(profile):
    _keys(profile, ("system", "Python", "namespaces", "blas", "options_dput", "object_options_dput", "clock_source", "functions"), "R profile")
    _keys(profile["system"], SYSTEM_KEYS, "profile system")
    for key, value in profile["system"].items():
        _text(value, f"profile system {key}")
    _require(profile["system"]["sysname"] == "Linux" and profile["system"]["machine"] == "x86_64", "profile host differs")
    python = profile["Python"]
    _keys(python, ("executable", "executable_sha256", "version", "prefix", "base_prefix", "stdlib_json", "libpython_sha256"), "full Python identity")
    for key in ("executable", "prefix", "base_prefix", "stdlib_json"):
        _profile_path(python[key], f"Python {key}")
    _hash(python["executable_sha256"], "Python executable")
    _require(type(python["version"]) is str and re.match(r"3\.12\.[0-9]+(?:\s|$)", python["version"]) is not None,
             "Python full version differs")
    _require(isinstance(python["libpython_sha256"], Mapping), "invalid libpython mapping")
    for path, digest in python["libpython_sha256"].items():
        _profile_path(path, "libpython")
        _hash(digest, "libpython")
    _keys(profile["namespaces"], NAMESPACES, "profile namespaces")
    for package, record in profile["namespaces"].items():
        _keys(record, ("version", "library_path"), f"namespace {package}")
        _text(record["version"], "namespace version")
        _profile_path(record["library_path"], "namespace library")
    _require(profile["namespaces"]["infercnv"]["version"] == "1.28.0", "infercnv profile version differs")
    _text(profile["blas"], "BLAS")
    for key in ("options_dput", "object_options_dput"):
        _string_list(profile[key], key)
    clock = profile["clock_source"]
    _keys(clock, ("path", "sha256", "function_text"), "profile clock")
    _equal(clock["path"], "clock-proc-time.txt", "profile clock path")
    _hash(clock["sha256"], "profile clock")
    _equal(clock["function_text"], ['.Primitive("proc.time")'], "profile clock primitive")
    _keys(profile["functions"], FUNCTIONS, "profile functions")
    for name, digest in profile["functions"].items():
        _hash(digest, name)


def _bindings(bindings):
    _require(type(bindings) is TrialBindings, "invalid trial bindings")
    _require(type(bindings.input_case) is str and bindings.input_case in ("plain", "gzip"), "invalid binding case")
    for name in ("artifact_root", "source_root", "receipt_path", "scripts_dir", "output_dir", "native_executable"):
        _path(getattr(bindings, name), name)
    _text(bindings.native_version, "native version")
    for name, keys in (("input_paths", INPUT_RELATIVE), ("input_hashes", INPUT_RELATIVE),
                       ("code_paths", CODE_NAMES), ("code_hashes", CODE_NAMES)):
        collection = getattr(bindings, name)
        _keys(collection, keys, name)
        for key, value in collection.items():
            (_path if name.endswith("paths") else _hash)(value, f"{name}.{key}")
    for key, relative in INPUT_RELATIVE.items():
        base = bindings.source_root if key.startswith("R/") else bindings.artifact_root
        _require(bindings.input_paths[key] == base / relative, f"input path binding differs: {key}")
    proposal = bindings.code_paths["trial"].parent
    for key, filename in CODE_NAMES.items():
        base = proposal if key in ("trial", "runtime_helper", "creation_support") else bindings.scripts_dir
        _require(bindings.code_paths[key] == base / filename, f"code path binding differs: {key}")
    for key, digest in FIXED_CODE.items():
        _equal(bindings.code_hashes[key], digest, f"fixed binding {key}")
    _profile(bindings.r_profile)


def _inventory(root, implementation):
    _path(root, "trial root")
    for directory in reversed((root, *root.parents)):
        _require(stat.S_ISDIR(directory.lstat().st_mode), f"non-directory/symlink ancestor: {directory}")
    expected_files = set(R_FILES if implementation == "infercnv" else ("metrics.tsv", "runtime.tsv", "io.lifetime.txt"))
    expected_files.update(f"{phase}/{name}" for phase in ("warm", "measured") for name in PHASE_FILES)
    seen_files, seen_dirs = set(), set()
    def walk(directory):
        for entry in directory.iterdir():
            name = entry.relative_to(root).as_posix()
            mode = entry.lstat().st_mode
            if stat.S_ISDIR(mode):
                _require(name in ("warm", "measured"), f"unexpected directory: {name}")
                seen_dirs.add(name)
                walk(entry)
            else:
                _require(stat.S_ISREG(mode), f"nonregular/symlink output: {name}")
                seen_files.add(name)
    walk(root)
    _require(seen_dirs == {"warm", "measured"} and seen_files == expected_files, "closed trial file inventory differs")
    return sorted(expected_files)


def _bytes(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
    with os.fdopen(descriptor, "rb") as stream:
        _require(stat.S_ISREG(os.fstat(stream.fileno()).st_mode), "output changed to nonregular file")
        return stream.read()


def _rows(raw, header, label):
    text = raw.decode("utf-8")
    _require("\r" not in text and "\x00" not in text, f"invalid literal {label}")
    lines = text.split("\n")
    if lines[-1] == "":
        lines.pop()
    _require(bool(lines) and lines[0].split("\t") == header, f"{label} header differs")
    rows = [line.split("\t") for line in lines[1:]]
    _require(bool(rows) and all(len(row) == len(header) and all(row) for row in rows), f"malformed {label} rows")
    return rows


def _table(raw, label):
    rows = _rows(raw, ["key", "value"], label)
    _require(len({row[0] for row in rows}) == len(rows), f"duplicate {label} key")
    return dict(rows)


def _json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            _require(key not in result, f"duplicate JSON key: {key}")
            result[key] = value
        return result
    def invalid(value):
        raise ValueError(f"nonfinite JSON constant: {value}")
    return json.loads(raw.decode("utf-8"), object_pairs_hook=pairs, parse_constant=invalid)


def _native_expected(b):
    paths = b.input_paths
    result = dict(schema_version="1", measurement="raw_creation", implementation="rust",
        factory="rsomics_sc::cnv::create_input", product_version=b.native_version, operating_system="linux",
        architecture="x86_64", build_assertions="false", input_case=b.input_case,
        required_input_receipt="infercnv-ingestion-measurement-input-2026-09-30.json",
        required_input_run_id="36707344893", required_input_artifact_id="11092988090",
        required_input_artifact_sha256="2481188b6f3d058d154bf16df9297a9eca261c6df84280e7d16569c92da28bd8",
        required_input_receipt_sha256=RECEIPT_SHA, artifact_root=str(b.artifact_root), counts_path=str(paths[b.input_case]),
        positions_path=str(paths["shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt"]),
        annotations_path=str(paths["shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt"]),
        expected_full_path=str(b.artifact_root / "shipped-bundle/full"),
        expected_maps_path=str(paths["ingestion-witness/shipped_full/plain/maps.tsv"]),
        input_selected_file_bytes="36378958" if b.input_case == "plain" else "4407153", input_decoded_count_bytes="36378958",
        decoded_bytes_binding="canonical plain file length; gzip equality checked by external controller",
        input_authentication="external controller must verify receipt paths and hashes before and after all trials",
        max_numeric_bytes="268435456", max_record_bytes="1048576", min_counts_per_cell="1", max_counts_per_cell="Inf",
        count_representation="native f64 column-major", numeric_parity_scope="frozen canonical receipt; not arbitrary R-decimal parsing",
        factory_warmups_per_process="1", measured_factory_calls_per_process="1",
        warm_regime="filesystem and allocation warm; no controlled cold cache",
        reclamation_before_baseline="drop warm CreatedInput and ExpectedCreation; no allocator trimming",
        retained_preparation="paths/config/runtime metadata and possible allocator-retained pages",
        cpu_counter_scope="getrusage self and reaped children; bracketing calls outside wall timer",
        rss_scope="baseline and live return snapshots; neither is a transient factory peak",
        whole_process_metrics="external monitor; includes warmup validation and exports",
        thread_observation="procfs snapshots; no claim of continuous single-thread execution",
        io_snapshot_scope="whole-process lifetime after both validated exports before metadata writes",
        io_rchar_wchar="bytes from read/write-like syscalls including instrumentation",
        io_syscr_syscw="read/write-like syscall counts",
        io_read_write_bytes="Linux storage-layer accounting; cached physical reads may be zero",
        io_cancelled_write_bytes="Linux accounting of dirty writes cancelled before storage",
        executable_path=str(b.native_executable), returned_genes="9939", returned_cells="184")
    result.update({f"argv_{index}": str(value) for index, value in enumerate((b.native_executable, b.artifact_root, b.input_case, b.output_dir))})
    result.update({f"reference_group_{index}": value for index, value in enumerate(REFERENCES, 1)})
    result.update({f"excluded_chromosome_{index}": value for index, value in enumerate(EXCLUSIONS, 1)})
    result.update({key: "1" for key in THREAD_KEYS if key != "VECLIB_MAXIMUM_THREADS"})
    return result


def _native(raw, bindings):
    runtime = _table(raw, "native runtime")
    expected = _native_expected(bindings)
    observed = ("pid", "observed_threads_before", "observed_threads_returned")
    _keys(runtime, (*expected, *observed), "native runtime")
    for key, value in expected.items():
        _equal(runtime[key], value, key)
    counters = [_uint(runtime[key], key, literal=True) for key in observed]
    return counters[1], counters[2]


def _r_expected(b):
    p = b.r_profile
    matrix = dict(type="double", classes=["matrix", "array"], dimensions=[9939, 184])
    obj = dict(object_class=["infercnv"], expression=matrix, counts=matrix, options_dput=p["object_options_dput"])
    return dict(R_version="4.6.1", Bioconductor_version="3.23", system=p["system"], locale_all="C",
                locale={key: "C" for key in LOCALES},
                Python=dict(executable=p["Python"]["executable"], version="Python " + p["Python"]["version"].split()[0],
                            sha256=p["Python"]["executable_sha256"]),
                namespaces=p["namespaces"], thread_environment={key: "1" for key in THREAD_KEYS},
                blas=p["blas"], options_dput=p["options_dput"], warm=obj, measured=obj,
                clock_source=p["clock_source"], **R_POLICIES)


def _r_provenance(b):
    return dict(input_case=b.input_case, receipt_sha256=RECEIPT_SHA, original_run=36707344893,
        original_artifact=11092988090, source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
        paths={key: str(path) for key, path in b.input_paths.items()}, sha256=b.input_hashes,
        creation_arguments=dict(raw_counts_matrix=str(b.input_paths[b.input_case]),
            gene_order_file=str(b.input_paths["shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt"]),
            annotations_file=str(b.input_paths["shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt"]),
            ref_group_names=REFERENCES, delim="\t", max_cells_per_group=None,
            min_max_counts_per_cell=[1, "Inf"], chr_exclude=EXCLUSIONS),
        code_paths={key: str(path) for key, path in b.code_paths.items()},
        code_sha256_before=b.code_hashes, code_sha256_after=b.code_hashes, fixed_code_sha256=FIXED_CODE,
        code_authentication=AUTHENTICATION)


def _r(files, bindings):
    runtime = _json(files["runtime.json"])
    expected = _r_expected(bindings)
    _keys(runtime, (*expected, "observed_threads_before", "observed_threads_after"), "R runtime")
    for key, value in expected.items():
        _equal(runtime[key], value, f"R runtime {key}")
    before, after = (_uint(runtime[key], key) for key in ("observed_threads_before", "observed_threads_after"))
    _equal(_json(files["provenance.json"]), _r_provenance(bindings), "R provenance")
    _equal(hashlib.sha256(files["clock-proc-time.txt"]).hexdigest(), bindings.r_profile["clock_source"]["sha256"], "clock proof bytes")
    functions = _rows(files["functions.tsv"], ["function", "source_path", "source_sha256", "installed_text", "installed_text_sha256"], "functions")
    wanted = [[name, "R/inferCNV.R", bindings.input_hashes["R/inferCNV.R"], f"function-{name}.txt",
               bindings.r_profile["functions"][name]] for name in FUNCTIONS]
    _equal(functions, wanted, "ordered function proof table")
    for name in FUNCTIONS:
        _equal(hashlib.sha256(files[f"function-{name}.txt"]).hexdigest(), bindings.r_profile["functions"][name], f"{name} proof bytes")
    packages = _rows(files["packages.tsv"], ["Package", "Version", "LibPath"], "packages")
    _require(len({(name, path) for name, version, path in packages}) == len(packages), "duplicate installed package/library")
    for row in packages:
        for value in row:
            _text(value, "installed package field")
    for name, record in bindings.r_profile["namespaces"].items():
        _require([name, record["version"], record["library_path"]] in packages, f"loaded namespace absent from packages: {name}")
    return before, after


def _io(raw):
    text = raw.decode("ascii")
    _require("\r" not in text, "CR in lifetime I/O")
    lines = text.split("\n")
    if lines[-1] == "":
        lines.pop()
    result = {}
    for line in lines:
        match = re.fullmatch(r"([a-z_]+): ([0-9]+)", line)
        _require(match is not None, "malformed lifetime I/O row")
        key, value = match.groups()
        _require(key not in result, "duplicate lifetime I/O counter")
        result[key] = _uint(value, key, zero=True, literal=True)
    _keys(result, IO_KEYS, "lifetime I/O")
    return result


def validate_trial_metadata(root: Path, implementation: str, bindings: TrialBindings) -> CheckedMetadata:
    """Check admitted bindings and closed output metadata, not process ownership.

    Expected mappings are immutable by caller contract. The full Python profile
    is shape-checked; its mapped-library identity needs a separate owner probe.
    """
    _require(type(implementation) is str and implementation in ("rust", "infercnv"), "invalid implementation")
    _bindings(bindings)
    names = _inventory(root, implementation)
    files = {name: _bytes(root / name) for name in names}
    _require(b"\r" not in files["metrics.tsv"], "CR in literal metrics")
    metrics = parse_region_metrics(files["metrics.tsv"].decode("utf-8"))
    _require(metrics.implementation == implementation and metrics.input_case == bindings.input_case,
             "metrics implementation/case binding differs")
    before, after = _native(files["runtime.tsv"], bindings) if implementation == "rust" else _r(files, bindings)
    io = _io(files["io.lifetime.txt"])
    hashes = {name: hashlib.sha256(raw).hexdigest() for name, raw in files.items()}
    return CheckedMetadata(implementation, bindings.input_case, before, after, MappingProxyType(io), MappingProxyType(hashes))
