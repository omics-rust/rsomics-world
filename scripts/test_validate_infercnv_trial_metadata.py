import copy
from dataclasses import FrozenInstanceError, replace
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import tempfile
import unittest

from validate_infercnv_trial_metadata import TrialBindings, validate_trial_metadata


RECEIPT = "3bee1dc582a321f16cb173c757e955dc620a911a891f4051f8bae68dc856dc0b"
FIXED = {
    "creation_support": "d583b156e9a08f3422969697b4524469b8eb05bb92d6c65d41e2aa1df7990faa",
    "counter_support": "0c3870354e5ac5bafedbd018148492bb5f339714b5412c409866825b2e783956",
    "oracle_io": "5a01aeb620451d0254bbab775c53111a61eeefcaa32ac9b96ab142563ca5839d",
    "creation_checker": "0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36",
    "imported_validator": "682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222",
}
INPUTS = {
    "plain": "shipped-bundle/inputs/canonical-full.tsv",
    "gzip": "ingestion-witness/shipped_full/counts.tsv.gz",
    **{p: p for p in (
        "shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt",
        "shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt",
        "shipped-bundle/full/01.tsv", "shipped-bundle/full/01.genes.tsv",
        "shipped-bundle/full/01.cells.tsv", "ingestion-witness/shipped_full/plain/maps.tsv",
        "shipped-bundle/full/upstream/01_incoming_data.infercnv_obj")},
    "archive": "infercnv-source.tar.gz", "R/inferCNV.R": "R/inferCNV.R",
    "R/inferCNV_constants.R": "R/inferCNV_constants.R",
}
CODE = dict(trial="infercnv_ingestion_matched.R", runtime_helper="infercnv_ingestion_runtime_support.R",
            creation_support="infercnv_ingestion_matched_support.R", counter_support="infercnv_matched_support.R",
            oracle_io="infercnv_oracle_io.R", creation_checker="validate_infercnv_ingestion_measurement.py",
            imported_validator="validate_infercnv_samples_clustering_witness.py")
FUNCTIONS = ("CreateInfercnvObject", ".order_reduce", "validate_infercnv_obj")
THREADS = ("OPENBLAS_NUM_THREADS", "OMP_NUM_THREADS", "MKL_NUM_THREADS", "BLIS_NUM_THREADS",
           "VECLIB_MAXIMUM_THREADS", "NUMEXPR_NUM_THREADS", "RAYON_NUM_THREADS")
IO = ("rchar", "wchar", "syscr", "syscw", "read_bytes", "write_bytes", "cancelled_write_bytes")
REFS = ["Microglia/Macrophage", "Oligodendrocytes (non-malignant)"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def table(rows):
    return "key\tvalue\n" + "".join(f"{k}\t{v}\n" for k, v in rows.items())


class Fixture:
    def __init__(self, root, implementation="infercnv", case="plain"):
        self.root = root / "trial"
        self.root.mkdir()
        self.implementation = implementation
        self.case = case
        artifact, source, scripts = root / "artifact", root / "source", root / "scripts"
        paths = {k: (source if k.startswith("R/") else artifact) / v for k, v in INPUTS.items()}
        self.bindings = TrialBindings(case, artifact, source, root / "receipt.json", scripts, self.root,
                                      paths, {k: "a" * 64 for k in paths},
                                      {k: scripts / v for k, v in CODE.items()},
                                      dict(trial="b" * 64, runtime_helper="c" * 64, **FIXED),
                                      root / "cnv_ingestion_matched", "0.1.0", self.profile())
        for phase in ("warm", "measured"):
            (self.root / phase).mkdir()
            for name in ("expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv"):
                (self.root / phase / name).write_text("supporting export\n")
        self.metrics = dict(schema_version="1", measurement="raw_creation", implementation=implementation,
                            input_case=case, preparation_wall_seconds="1", region_wall_seconds="1",
                            region_user_seconds="0", region_system_seconds="0", region_child_user_seconds="0",
                            region_child_system_seconds="0", baseline_rss_bytes="1", returned_rss_bytes="2")
        (self.root / "metrics.tsv").write_text(table(self.metrics))
        (self.root / "io.lifetime.txt").write_text("".join(f"{key}: {i}\n" for i, key in enumerate(IO)))
        if implementation == "rust":
            self.runtime = self.native()
        else:
            self.runtime = self.r_runtime()
            self.provenance = self.r_provenance()
            for name in ("session.txt", "warm-checker.stdout", "warm-checker.stderr",
                         "measured-checker.stdout", "measured-checker.stderr"):
                (self.root / name).write_text("")
            (self.root / "clock-proc-time.txt").write_bytes(b"trusted clock\n")
            rows = ["function\tsource_path\tsource_sha256\tinstalled_text\tinstalled_text_sha256"]
            for name in FUNCTIONS:
                filename = f"function-{name}.txt"
                (self.root / filename).write_bytes((name + "\n").encode())
                rows.append(f"{name}\tR/inferCNV.R\t{'a' * 64}\t{filename}\t{sha((name + chr(10)).encode())}")
            (self.root / "functions.tsv").write_text("\n".join(rows) + "\n")
            self.packages = "Package\tVersion\tLibPath\n" + "".join(
                f"{name}\t{v['version']}\t{v['library_path']}\n" for name, v in self.bindings.r_profile["namespaces"].items())
            (self.root / "packages.tsv").write_text(self.packages)
        self.save()

    @staticmethod
    def profile():
        return dict(system=dict(sysname="Linux", release="6.1", version="test kernel", nodename="host",
                                machine="x86_64", login="unknown", user="runner", effective_user="runner"),
                    Python=dict(executable="/usr/bin/python3.12", executable_sha256="d" * 64,
                                version="3.12.3 (test)", prefix="/usr", base_prefix="/usr",
                                stdlib_json="/usr/lib/python3.12/json/__init__.py", libpython_sha256={}),
                    namespaces={k: dict(version=v, library_path="/library") for k, v in
                                zip(("infercnv", "digest", "jsonlite", "Matrix", "BiocManager", "BiocVersion"),
                                    ("1.28.0", "0.6.39", "2.0.0", "1.7-6", "1.30.27", "3.23.1"))},
                    blas="/blas.so", options_dput=['list(scipen = 0L, digits = 7L, warn = 0L, OutDec = ".")'],
                    object_options_dput=['list(chr_exclude = c("chrX", "chrY", "chrM"), max_cells_per_group = NULL, ',
                                         '    min_max_counts_per_cell = c(1, Inf), counts_md5 = "a08510e3cc05c9edd765cdf6e2858080")'],
                    clock_source=dict(path="clock-proc-time.txt", sha256=sha(b"trusted clock\n"),
                                      function_text=['.Primitive("proc.time")']),
                    functions={name: sha((name + "\n").encode()) for name in FUNCTIONS})

    def r_runtime(self):
        p = self.bindings.r_profile
        matrix = dict(type="double", classes=["matrix", "array"], dimensions=[9939, 184])
        obj = dict(object_class=["infercnv"], expression=matrix, counts=copy.deepcopy(matrix),
                   options_dput=p["object_options_dput"])
        return dict(R_version="4.6.1", Bioconductor_version="3.23", system=p["system"], locale_all="C",
                    locale={k: "C" for k in ("LC_COLLATE", "LC_CTYPE", "LC_MONETARY", "LC_NUMERIC", "LC_TIME", "LC_MESSAGES")},
                    Python=dict(executable="/usr/bin/python3.12", version="Python 3.12.3", sha256="d" * 64),
                    namespaces=p["namespaces"], thread_environment={k: "1" for k in THREADS},
                    observed_threads_before=2, observed_threads_after=3, blas=p["blas"], options_dput=p["options_dput"],
                    warm=obj, measured=copy.deepcopy(obj),
                    preparation_policy="verified raw/expected/source bytes; validated/exported warm factory result; released local warm/oracle states; gc(full=TRUE); baseline",
                    region_clock="proc.time elapsed/self/child counters around actual CreateInfercnvObject; seconds rounded down to milliseconds per installed R documentation; actual accuracy is system-dependent, not assumed nanoseconds",
                    clock_source=p["clock_source"],
                    rss_scope="baseline then returned object live before delta formatting, validation, export or post-call gc",
                    io_scope="unmodified /proc/self/io lifetime snapshot after both verified exports and rechecks, before final metadata writes; includes startup, source-proof writes, warmup, validators, exports and instrumentation; not factory-region deltas",
                    lifetime_scope="external monitor includes package startup, warm-up, independent Python validators and exports; not factory peak")

    def r_provenance(self):
        b = self.bindings
        return dict(input_case=self.case, receipt_sha256=RECEIPT, original_run=36707344893,
                    original_artifact=11092988090, source_commit="b421d9405c97a309b081ef86d455e976df93eae4",
                    paths={k: str(v) for k, v in b.input_paths.items()}, sha256=dict(b.input_hashes),
                    creation_arguments=dict(raw_counts_matrix=str(b.input_paths[self.case]),
                        gene_order_file=str(b.input_paths[INPUTS['shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt']]),
                        annotations_file=str(b.input_paths[INPUTS['shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt']]),
                        ref_group_names=REFS, delim="\t", max_cells_per_group=None,
                        min_max_counts_per_cell=[1, "Inf"], chr_exclude=["chrX", "chrY", "chrM"]),
                    code_paths={k: str(v) for k, v in b.code_paths.items()}, code_sha256_before=dict(b.code_hashes),
                    code_sha256_after=dict(b.code_hashes), fixed_code_sha256=FIXED,
                    code_authentication="external controller MUST authenticate exact trial/helper/reused-R/Python bytes against independently frozen source before process launch; self/runtime-helper hashes here are drift baselines, not circular authenticity pins")

    def native(self):
        b = self.bindings
        r = dict(schema_version="1", measurement="raw_creation", implementation="rust",
                 factory="rsomics_sc::cnv::create_input", product_version="0.1.0", operating_system="linux",
                 architecture="x86_64", build_assertions="false", input_case=self.case,
                 required_input_receipt="infercnv-ingestion-measurement-input-2026-09-30.json",
                 required_input_run_id="36707344893", required_input_artifact_id="11092988090",
                 required_input_artifact_sha256="2481188b6f3d058d154bf16df9297a9eca261c6df84280e7d16569c92da28bd8",
                 required_input_receipt_sha256=RECEIPT, artifact_root=str(b.artifact_root), counts_path=str(b.input_paths[self.case]),
                 positions_path=str(b.input_paths['shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt']),
                 annotations_path=str(b.input_paths['shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt']),
                 expected_full_path=str(b.artifact_root / "shipped-bundle/full"),
                 expected_maps_path=str(b.input_paths['ingestion-witness/shipped_full/plain/maps.tsv']),
                 input_selected_file_bytes="36378958" if self.case == "plain" else "4407153", input_decoded_count_bytes="36378958",
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
                 pid="123", executable_path=str(b.native_executable), returned_genes="9939", returned_cells="184",
                 observed_threads_before="2", observed_threads_returned="3")
        r.update({f"argv_{i}": v for i, v in enumerate(map(str, (b.native_executable, b.artifact_root, self.case, b.output_dir)))})
        r.update({f"reference_group_{i}": v for i, v in enumerate(REFS, 1)})
        r.update({f"excluded_chromosome_{i}": v for i, v in enumerate(("chrX", "chrY", "chrM"), 1)})
        r.update({k: "1" for k in THREADS if k != "VECLIB_MAXIMUM_THREADS"})
        return r

    def save(self):
        if self.implementation == "rust":
            (self.root / "runtime.tsv").write_text(table(self.runtime))
        else:
            (self.root / "runtime.json").write_text(json.dumps(self.runtime))
            (self.root / "provenance.json").write_text(json.dumps(self.provenance))

    def check(self):
        return validate_trial_metadata(self.root, self.implementation, self.bindings)


class MetadataTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()

    def fixture(self, implementation="infercnv", case="plain"):
        base = self.root / str(len(list(self.root.iterdir())))
        base.mkdir()
        return Fixture(base, implementation, case)

    def test_both_implementations_and_cases_preserve_observations_and_hash_every_file(self):
        for implementation in ("rust", "infercnv"):
            for case in ("plain", "gzip"):
                f = self.fixture(implementation, case)
                result = f.check()
                self.assertEqual((result.implementation, result.input_case), (implementation, case))
                self.assertEqual((result.observed_threads_before, result.observed_threads_after), (2, 3))
                self.assertEqual(dict(result.io_counters), dict(zip(IO, range(7))))
                self.assertEqual(dict(result.file_hashes), {str(p.relative_to(f.root)): sha(p.read_bytes())
                                                         for p in f.root.rglob("*") if p.is_file()})
                with self.assertRaises(FrozenInstanceError):
                    result.input_case = "gzip"

    def test_native_every_literal_and_binding_rejects_changes(self):
        f = self.fixture("rust")
        for key, value in list(f.runtime.items()):
            with self.subTest(key=key):
                f.runtime[key] = "invalid"
                f.save()
                with self.assertRaises(ValueError):
                    f.check()
                f.runtime[key] = value
        f.runtime["VECLIB_MAXIMUM_THREADS"] = "1"
        f.save()
        with self.assertRaises(ValueError):
            f.check()

    def test_noncanonical_and_out_of_range_native_integers(self):
        f = self.fixture("rust")
        for key in ("pid", "observed_threads_before", "observed_threads_returned"):
            for value in ("01", "+1", " 1", "0", str(2**64), "1.0", "1e0"):
                with self.subTest(key=key, value=value):
                    old = f.runtime[key]
                    f.runtime[key] = value
                    f.save()
                    with self.assertRaises(ValueError):
                        f.check()
                    f.runtime[key] = old

    def test_binding_shape_and_canonical_paths(self):
        f = self.fixture()
        changes = dict(input_case="gzip", artifact_root=Path("relative"), source_root=Path("/a/../b"),
                       receipt_path="/receipt", scripts_dir=Path("relative"), output_dir=Path("relative"),
                       input_paths={}, input_hashes={}, code_paths={}, code_hashes={},
                       native_executable=Path("relative"), native_version="", r_profile={})
        for key, value in changes.items():
            with self.subTest(key=key):
                old = f.bindings
                f.bindings = replace(old, **{key: value})
                with self.assertRaises(ValueError):
                    f.check()
                f.bindings = old

    def test_offline_root_is_distinct_from_original_host_paths(self):
        for implementation in ("rust", "infercnv"):
            f = self.fixture(implementation)
            f.bindings = replace(f.bindings, output_dir=Path("/original-host/trial"))
            if implementation == "rust":
                f.runtime["argv_3"] = "/original-host/trial"
                f.save()
            f.check()

    def test_all_named_bindings_and_full_profile_shapes_are_checked(self):
        f = self.fixture()
        for category in ("input_paths", "input_hashes", "code_paths", "code_hashes"):
            original = f.bindings
            for key in getattr(original, category):
                mapping = dict(getattr(original, category))
                mapping[key] = Path("/wrong") if category.endswith("paths") else "e" * 64
                f.bindings = replace(original, **{category: mapping})
                with self.subTest(category=category, key=key), self.assertRaises(ValueError):
                    f.check()
            f.bindings = original
        p = copy.deepcopy(f.bindings.r_profile)
        variants = []
        for key in p:
            changed = copy.deepcopy(p)
            changed[key] = None
            variants.append(changed)
        for key in p["Python"]:
            changed = copy.deepcopy(p)
            changed["Python"].pop(key)
            variants.append(changed)
        changed = copy.deepcopy(p)
        changed["Python"]["libpython_sha256"] = {"relative": "e" * 64}
        variants.append(changed)
        changed = copy.deepcopy(p)
        changed["Python"]["libpython_sha256"] = {"/libpython.so": "bad"}
        variants.append(changed)
        original = f.bindings
        for changed in variants:
            f.bindings = replace(original, r_profile=changed)
            with self.subTest(profile=changed), self.assertRaises(ValueError):
                f.check()
        f.bindings = original

    def test_positive_r_counts_and_nested_numeric_types(self):
        f = self.fixture()
        for value in (True, 0, -1, 1.0, 2**64):
            f.runtime["observed_threads_after"] = value
            f.save()
            with self.subTest(value=value), self.assertRaises(ValueError):
                f.check()
        f.runtime = f.r_runtime()
        f.runtime["warm"]["counts"]["dimensions"][0] = True
        f.save()
        with self.assertRaises(ValueError):
            f.check()
        f.runtime = f.r_runtime()
        f.provenance["creation_arguments"]["min_max_counts_per_cell"][0] = True
        f.save()
        with self.assertRaises(ValueError):
            f.check()

    def test_raw_profile_paths_reject_normalized_away_components(self):
        f = self.fixture()
        original = f.bindings
        for key in ("executable", "prefix", "base_prefix", "stdlib_json"):
            for value in ("/usr//", "/usr//python", "/usr/./python", "/usr/python/", "//usr/python"):
                profile = copy.deepcopy(original.r_profile)
                profile["Python"][key] = value
                f.bindings = replace(original, r_profile=profile)
                if key == "executable":
                    f.runtime["Python"]["executable"] = value
                f.save()
                with self.subTest(field=key, value=value), self.assertRaises(ValueError):
                    f.check()
                f.runtime["Python"]["executable"] = original.r_profile["Python"]["executable"]
        for name, record in original.r_profile["namespaces"].items():
            for value in ("/library/.", "/library//x", "/library/", "//library"):
                profile = copy.deepcopy(original.r_profile)
                profile["namespaces"][name]["library_path"] = value
                f.bindings = replace(original, r_profile=profile)
                f.runtime["namespaces"] = profile["namespaces"]
                f.save()
                old_row = f"{name}\t{record['version']}\t/library"
                new_row = f"{name}\t{record['version']}\t{value}"
                (f.root / "packages.tsv").write_text(f.packages.replace(old_row, new_row))
                with self.subTest(namespace=name, library=value), self.assertRaises(ValueError):
                    f.check()
        for value in ("/usr//lib/libpython.so", "/usr/./libpython.so", "/usr/libpython.so/", "//usr/libpython.so"):
            profile = copy.deepcopy(original.r_profile)
            profile["Python"]["libpython_sha256"] = {value: "e" * 64}
            f.bindings = replace(original, r_profile=profile)
            f.runtime["namespaces"] = original.r_profile["namespaces"]
            f.save()
            (f.root / "packages.tsv").write_text(f.packages)
            with self.subTest(mapped_library=value), self.assertRaises(ValueError):
                f.check()
        profile = copy.deepcopy(original.r_profile)
        profile["Python"]["libpython_sha256"] = {"/usr/lib/libpython.so": "e" * 64}
        f.bindings = replace(original, r_profile=profile)
        f.runtime["namespaces"] = original.r_profile["namespaces"]
        f.save()
        (f.root / "packages.tsv").write_text(f.packages)
        f.check()

    def test_r_every_leaf_and_nested_inventory_is_checked(self):
        f = self.fixture()
        def leaves(value, prefix=()):
            if isinstance(value, dict):
                for key, child in value.items():
                    yield from leaves(child, prefix + (key,))
            elif isinstance(value, list):
                for index, child in enumerate(value):
                    yield from leaves(child, prefix + (index,))
            else:
                yield prefix
        for attribute in ("runtime", "provenance"):
            original = copy.deepcopy(getattr(f, attribute))
            for path in leaves(original):
                with self.subTest(attribute=attribute, path=path):
                    changed = copy.deepcopy(original)
                    target = changed
                    for part in path[:-1]:
                        target = target[part]
                    target[path[-1]] = "wrong"
                    setattr(f, attribute, changed)
                    f.save()
                    with self.assertRaises(ValueError):
                        f.check()
            setattr(f, attribute, original)
        for path in ((), ("system",), ("Python",), ("namespaces", "infercnv"),
                     ("warm",), ("measured", "expression"), ("clock_source",)):
            f.runtime = copy.deepcopy(f.r_runtime())
            target = f.runtime
            for part in path:
                target = target[part]
            target["extra"] = "x"
            f.save()
            with self.subTest(extra=path), self.assertRaises(ValueError):
                f.check()

    def test_json_duplicate_nonfinite_bool_and_missing_keys(self):
        for filename in ("runtime.json", "provenance.json"):
            f = self.fixture()
            path = f.root / filename
            original = path.read_text()
            key = "observed_threads_before" if filename == "runtime.json" else "original_run"
            for token in ("true", "NaN", "Infinity", "-Infinity", "1.0"):
                parsed = json.loads(original)
                parsed[key] = 1
                text = json.dumps(parsed).replace(f'"{key}": 1', f'"{key}": {token}')
                path.write_text(text)
                with self.subTest(filename=filename, token=token), self.assertRaises(ValueError):
                    f.check()
            path.write_text(original[:-1] + f', "{key}": 2}}')
            with self.assertRaises(ValueError):
                f.check()
            path.write_text("{}")
            with self.assertRaises(ValueError):
                f.check()

    def test_literal_tsv_and_io_reject_bad_rows(self):
        for implementation in ("rust", "infercnv"):
            f = self.fixture(implementation)
            names = ("metrics.tsv", "runtime.tsv") if implementation == "rust" else ("metrics.tsv", "functions.tsv", "packages.tsv")
            for name in names:
                path = f.root / name
                original = path.read_text()
                for text in (original.replace("\n", "\r\n"), '"' + original, original + original.splitlines()[1] + "\n",
                             original + "bad\trow\textra\tfield\there\n"):
                    path.write_bytes(text.encode())
                    with self.subTest(name=name, text=text[:30]), self.assertRaises(ValueError):
                        f.check()
                path.write_text(original)
            path = f.root / "io.lifetime.txt"
            original = path.read_text()
            for text in (original + "rchar: 1\n", original.replace("rchar: 0", "rchar: -1"),
                         original.replace("rchar: 0", "rchar: 01"), original + "extra: 1\n",
                         original.replace("rchar: 0", "rchar: " + str(2**64)), original.replace("\n", "\r\n")):
                path.write_bytes(text.encode())
                with self.assertRaises(ValueError):
                    f.check()
                path.write_text(original)

    def test_closed_tree_rejects_before_opening_even_special_files(self):
        def timeout(signum, frame):
            raise AssertionError("special-file validation hung")
        old = signal.signal(signal.SIGALRM, timeout)
        self.addCleanup(signal.signal, signal.SIGALRM, old)
        for kind in ("missing", "extra", "nested", "symlink", "fifo", "socket", "root-symlink", "ancestor-symlink"):
            f = self.fixture()
            path = f.root / "metrics.tsv"
            if kind == "missing":
                path.unlink()
            elif kind == "extra":
                (f.root / "extra").write_text("x")
            elif kind == "nested":
                (f.root / "warm" / "extra").mkdir()
            elif kind in ("symlink", "fifo", "socket"):
                path.unlink()
                if kind == "symlink":
                    path.symlink_to(f.root / "session.txt")
                elif kind == "fifo":
                    os.mkfifo(path)
                else:
                    sock = socket.socket(socket.AF_UNIX)
                    self.addCleanup(sock.close)
                    sock.bind(str(path))
            elif kind == "root-symlink":
                alias = f.root.parent / "alias"
                alias.symlink_to(f.root, target_is_directory=True)
                f.root = alias
                f.bindings = replace(f.bindings, output_dir=alias)
            else:
                alias = self.root / ("alias" + f.root.parent.name)
                alias.symlink_to(f.root.parent, target_is_directory=True)
                f.root = alias / "trial"
                f.bindings = replace(f.bindings, output_dir=f.root)
            signal.alarm(3)
            try:
                with self.subTest(kind=kind), self.assertRaises(ValueError):
                    f.check()
            finally:
                signal.alarm(0)

    def test_function_clock_changes_cannot_supply_their_own_authenticity(self):
        for name in ("clock-proc-time.txt", "function-.order_reduce.txt"):
            f = self.fixture()
            (f.root / name).write_bytes(b"changed\n")
            if name.startswith("clock"):
                f.runtime["clock_source"] = dict(path=name, sha256=sha(b"changed\n"), function_text=['.Primitive("proc.time")'])
                f.save()
            else:
                path = f.root / "functions.tsv"
                path.write_text(path.read_text().replace(sha(b".order_reduce\n"), sha(b"changed\n")))
            with self.assertRaises(ValueError):
                f.check()

    def test_package_names_can_repeat_in_distinct_libraries_but_namespace_library_must_match(self):
        f = self.fixture()
        path = f.root / "packages.tsv"
        path.write_text(f.packages + "infercnv\t0.1\t/other-library\n")
        f.check()
        path.write_text(f.packages.replace("infercnv\t1.28.0\t/library", "infercnv\t1.28.0\t/wrong"))
        with self.assertRaises(ValueError):
            f.check()

    def test_metadata_learned_proof_path_is_not_opened(self):
        f = self.fixture()
        f.runtime["clock_source"] = dict(f.runtime["clock_source"], path="/dev/zero")
        f.save()
        with self.assertRaises(ValueError):
            f.check()
        f.runtime = f.r_runtime()
        f.save()
        path = f.root / "functions.tsv"
        path.write_text(path.read_text().replace("function-.order_reduce.txt", "/dev/zero"))
        with self.assertRaises(ValueError):
            f.check()

    def test_rejects_wrong_selector_and_missing_inventory_keys(self):
        f = self.fixture()
        for implementation in ("R", "native", None, True):
            with self.subTest(implementation=implementation), self.assertRaises(ValueError):
                validate_trial_metadata(f.root, implementation, f.bindings)
        for attribute in ("runtime", "provenance"):
            original = copy.deepcopy(getattr(f, attribute))
            for key in original:
                changed = copy.deepcopy(original)
                changed.pop(key)
                setattr(f, attribute, changed)
                f.save()
                with self.subTest(attribute=attribute, key=key), self.assertRaises(ValueError):
                    f.check()
            setattr(f, attribute, original)
        f.save()
        path = f.root / "runtime.json"
        text = path.read_text().replace('"sysname": "Linux"', '"sysname": "Linux", "sysname": "Linux"')
        path.write_text(text)
        with self.assertRaises(ValueError):
            f.check()

    def test_json_is_utf8_not_auto_detected_utf16(self):
        f = self.fixture()
        path = f.root / "runtime.json"
        path.write_bytes(path.read_text().encode("utf-16"))
        with self.assertRaises(ValueError):
            f.check()


if __name__ == "__main__":
    unittest.main()
