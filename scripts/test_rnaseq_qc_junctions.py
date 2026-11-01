"""One connected, stdlib-only source-witness preparation contract."""

import ast
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import urllib.error
import urllib.request
import urllib.response


ROOT = Path(__file__).resolve().parents[1]


class JunctionPreparation(unittest.TestCase):
    def test_complete_source_preparation_contract(self):
        controller = ROOT / "scripts/rnaseq_qc_junctions.py"
        workflow = ROOT / ".github/workflows/rnaseq-qc-junctions.yml"
        self.assertTrue(controller.is_file(), "junction source controller is not prepared")
        self.assertTrue(workflow.is_file(), "manual junction source workflow is not prepared")
        spec = importlib.util.spec_from_file_location("junction_preparation", controller)
        driver = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(driver)
        policy = json.loads(driver.ASSET.read_text())
        self.assertEqual(policy["schema"], "rsomics.rnaseq-qc.junction-source-policy.v1")
        self.assertEqual(policy["python"]["version"], "3.12.10")
        self.assertEqual(len(policy["packages"]), 18)
        self.assertEqual({name: package["version"] for name, package in policy["packages"].items()},
                         {"bx-python": "0.15.1", "contourpy": "1.4.0", "cycler": "0.12.1", "fonttools": "4.67.0",
                          "kiwisolver": "1.5.1", "logomaker": "0.8.7", "matplotlib": "3.11.2", "numpy": "2.2.6",
                          "packaging": "26.3", "pandas": "3.1.0", "pillow": "13.0.0", "pyBigWig": "0.3.26",
                          "pyparsing": "3.3.3", "pysam": "0.24.0", "python-dateutil": "3.0.0", "six": "1.17.0",
                          "tzdata": "2026.5", "RSeQC": "5.0.5"})
        self.assertEqual(policy["source"]["wheel_sha256"],
                         "a1fca6d455886b19bc2d58aa1e13b233977a2c835f676ef93bd0c6f4099b8452")
        self.assertEqual(policy["accepted_source"]["artifact_id"], 12740753197)
        for package in policy["packages"].values():
            self.assertRegex(package["sha256"], r"^[0-9a-f]{64}$")
            self.assertRegex(package["metadata_sha256"], r"^[0-9a-f]{64}$")
            self.assertGreater(package["bytes"], 1000)
            self.assertTrue(package["url"].startswith("https://files.pythonhosted.org/"))
        plans = driver.fixture_plans()
        self.assertEqual(list(plans), ["informative-single", "informative-paired",
                                      "case-alias-only", "no-N", "all-short"])
        for name, plan in plans.items():
            self.assertTrue(plan["intent"])
            self.assertIn("@SQ\tSN:chr1\t", plan["sam"])
            self.assertIn("@SQ\tSN:CHR1\t", plan["sam"])
            self.assertEqual(hashlib.sha256(plan["sam"].encode()).hexdigest(),
                             policy["fixtures"][name]["sam_sha256"])
            self.assertEqual(hashlib.sha256(plan["bed"].encode()).hexdigest(),
                             policy["fixtures"][name]["bed_sha256"])
        informative = plans["informative-single"]["sam"]
        for cigar in ("20M49N20M", "20M50N20M", "20M51N20M", "20M50N20M50N20M",
                      "5H3S10M2I5M3D4M2P5=3X50N20M4S2H"):
            self.assertIn(cigar, informative)
        flags = {int(line.split("\t")[1]) for line in informative.splitlines() if not line.startswith("@")}
        self.assertTrue({4, 256, 512, 1024, 1796, 2048}.issubset(flags))
        invocations = driver.invocation_plan()
        self.assertEqual(policy["invocations"], invocations)
        self.assertEqual(len(invocations), 6)
        self.assertEqual([row["case"] for row in invocations[:5]], list(plans))
        self.assertEqual([row["expected_exit"] for row in invocations], [0, 0, 0, 0, 1, 0])
        for row in invocations[:5]:
            self.assertEqual(row["options"], ["-q", "30", "-m", "50", "--skip-plot", "--skip-bed", "--skip-interact"])
        self.assertEqual(invocations[-1]["options"], ["-q", "30", "-m", "50", "--skip-plot"])
        self.assertEqual(driver.parse_primary("", empty=True), [])
        header = "chrom\tintron_st(0-based)\tintron_end(1-based)\tread_count\tannotation\n"
        self.assertEqual(driver.parse_primary(header), [])
        rows = driver.parse_primary(header + "CHR1\t120\t170\t2\tannotated\n")
        self.assertEqual(rows[0]["read_count"], 2)
        with self.assertRaises(ValueError):
            driver.parse_primary(header + "CHR1\t120\t170\t0\tannotated\n")
        self.assertEqual(driver.source_events(plans["no-N"]), [])
        self.assertTrue(all(event["length"] == 49 for event in driver.source_events(plans["all-short"])))
        events = driver.source_events(plans["informative-single"])
        self.assertTrue(any(not event["eligible_record"] for event in events))
        self.assertTrue(any(event["name"] == "supplementary" and event["eligible_record"] for event in events))
        derived = driver.predicted(plans["informative-single"])
        self.assertEqual({row["annotation"] for row in derived["rows"]}, set(driver.CLASSES))
        self.assertEqual(len(derived["annotation_sites"]["donors"]), 3)
        self.assertTrue(any(row["start"] == 120 and row["end"] == 270 and row["annotation"] == "annotated"
                            for row in derived["rows"]))
        mixed, = [event for event in derived["N_events"] if event["name"] == "mixed"]
        self.assertEqual((mixed["start"], mixed["end"]), (522, 572))
        paired = driver.predicted(plans["informative-paired"])
        self.assertEqual(paired["events"], {name: 2 * count for name, count in derived["events"].items()})
        short = driver.predicted(plans["all-short"])
        self.assertEqual((short["events"]["total"], short["events"]["filtered"], short["rows"]), (1, 1, []))
        self.assertEqual(driver.predicted(plans["case-alias-only"])["rows"][0]["annotation"], "annotated")
        retained = Path(tempfile.mkdtemp(prefix="junction-preparation-contract-"))
        def redirected_download(request, timeout):
            self.assertEqual(request.get_header("Authorization"), "Bearer fake-preparation-sentinel")
            redirected = urllib.request.HTTPRedirectHandler().redirect_request(
                request, None, 302, "Found", {}, "https://artifact-storage.invalid/payload")
            self.assertIsNone(redirected.get_header("Authorization"), "API authorization leaked onto redirected storage request")
            return urllib.response.addinfourl(io.BytesIO(b"ok"), {}, redirected.full_url, 200)

        with patch.object(driver.urllib.request, "urlopen", side_effect=redirected_download):
            driver.download(dict(url="https://api.github.invalid/artifact"), retained / "redirect-body", maximum=2,
                            token="fake-preparation-sentinel")
        for name, url in (("download-error", "https://pypi.invalid/denied"),
                          ("registry-token", "https://auth.docker.invalid/token"),
                          ("registry-manifest", "https://registry.invalid/manifest"),
                          ("registry-config", "https://registry.invalid/config")):
            failure = urllib.error.HTTPError(url, 403, "Forbidden", {"X-Request-Id": "fake-denial", "Authorization": "never-retain"},
                                             io.BytesIO(b"denied-original-body"))
            output = retained / name
            with patch.object(driver.urllib.request, "urlopen", side_effect=failure), self.assertRaises(urllib.error.HTTPError) as raised:
                if name == "download-error":
                    driver.download(dict(url=url), output, maximum=5)
                else:
                    driver.http_response(urllib.request.Request(url), output, maximum=5)
            self.assertIs(raised.exception, failure)
            self.assertEqual(Path(str(output) + ".http-error.body").read_bytes(), b"denie")
            custody = json.loads(Path(str(output) + ".http-error.json").read_text())
            self.assertEqual((custody["http_status"], custody["final_url"], custody["body_truncated"]), (403, url, True))
            self.assertEqual(custody["headers"], {"X-Request-Id": "fake-denial"})
            if name == "download-error":
                download_receipt = json.loads(Path(str(output) + ".receipt.json").read_text())
                self.assertEqual((download_receipt["http_status"], download_receipt["body_truncated"]), (403, True))
        short_error = urllib.error.HTTPError("https://fake.invalid/short", 503, "Unavailable", {}, io.BytesIO(b"bad"))
        with patch.object(driver.urllib.request, "urlopen", side_effect=short_error), self.assertRaises(urllib.error.HTTPError):
            driver.http_response(urllib.request.Request(short_error.url), retained / "short-error", maximum=5)
        self.assertFalse(json.loads((retained / "short-error.http-error.json").read_text())["body_truncated"])
        fake_manifest, fake_config = b"fake-manifest", b'{"architecture":"amd64","os":"linux"}'
        registry_policy = dict(policy, image=dict(policy["image"],
             manifest_digest="sha256:" + hashlib.sha256(fake_manifest).hexdigest(),
             config_digest="sha256:" + hashlib.sha256(fake_config).hexdigest()))
        for failed_phase in ("token", "manifest", "config"):
            registry_root = retained / ("actual-registry-wiring-" + failed_phase)
            registry_root.mkdir()
            failures = []
            def registry_response(request, timeout):
                url = request.full_url
                phase = "token" if "auth.docker.io" in url else "manifest" if "/manifests/" in url else "config"
                if phase != "token":
                    self.assertEqual(request.get_header("Authorization"), "Bearer fake-registry-only-in-memory")
                    redirected = urllib.request.HTTPRedirectHandler().redirect_request(
                        request, None, 302, "Found", {}, "https://fake-registry-storage.invalid/blob")
                    self.assertIsNone(redirected.get_header("Authorization"))
                if phase == failed_phase:
                    failure = urllib.error.HTTPError(url, 502, "Fake provider failure", {"X-Request-Id": "registry-denial"}, io.BytesIO(b"registry-error-original"))
                    failures.append(failure)
                    raise failure
                body = b'{"token":"fake-registry-only-in-memory"}' if phase == "token" else fake_manifest if phase == "manifest" else fake_config
                return urllib.response.addinfourl(io.BytesIO(body), {}, url, 200)

            with patch.object(driver, "cgroup", return_value={"cpu.max": "400000 100000", "memory.max": str(8 * 2**30), "memory.swap.max": "0"}), \
                    patch.object(driver.os, "sched_getaffinity", return_value={0, 1, 2, 3}, create=True), \
                    patch.dict(driver.os.environ, {"JUNCTION_IMAGE_DIGEST": registry_policy["image"]["manifest_digest"], "GCC_VERSION": "14.3.0"}), \
                    patch.object(driver.urllib.request, "urlopen", side_effect=registry_response), self.assertRaises(urllib.error.HTTPError) as raised:
                driver.admit_machine(registry_root, registry_policy)
            self.assertIs(raised.exception, failures[0])
            stem = "registry-token" if failed_phase == "token" else "image-" + failed_phase
            self.assertEqual((registry_root / (stem + ".http-error.body")).read_bytes(), b"registry-error-original")
            self.assertFalse(json.loads((registry_root / (stem + ".http-error.json")).read_text())["body_truncated"])
            for path in registry_root.rglob("*"):
                if path.is_file():
                    self.assertNotIn(b"fake-registry-only-in-memory", path.read_bytes())
        def probe_failure(tool, recipient, output, **kwargs):
            for suffix, body in ((".age.stderr", b"fake-age-diagnostic"), (".producer.stderr", b"fake-producer-diagnostic"),
                                 (".partial", b"fake-failed-cipher-partial")):
                Path(str(output) + suffix).write_bytes(body)
            raise RuntimeError("fake preparation probe failure")

        session = retained / "probe-session"
        for name in ("private", "cipher"):
            (session / name).mkdir(parents=True)
        with patch.object(driver, "capacity", return_value={}), patch.object(driver, "prepare_tool", return_value=retained), \
                patch.object(driver, "encrypt", side_effect=probe_failure), self.assertRaisesRegex(RuntimeError, "fake preparation probe failure"):
            driver.transport(session)
        probe_members = {row["name"] for row in driver.inventory(session / "private")}
        self.assertTrue({"transport-admission/probe.age.age.stderr", "transport-admission/probe.age.producer.stderr",
                         "transport-admission/probe.age.partial", "transport-admission/failure.json"}.issubset(probe_members))
        self.assertEqual(list((session / "cipher").iterdir()), [])
        successful = retained / "successful-probe-session"
        for name in ("private", "cipher"):
            (successful / name).mkdir(parents=True)
        def completed_probe(tool, recipient, output, **kwargs):
            output.write_bytes(b"fake-completed-probe-cipher")
            Path(str(output) + ".age.stderr").write_bytes(b"fake-completed-age-diagnostic")
            Path(str(output) + ".producer.stderr").write_bytes(b"fake-completed-producer-diagnostic")
            return {"preparation_adapter_only": True}

        with patch.object(driver, "capacity", return_value={}), patch.object(driver, "prepare_tool", return_value=retained), \
                patch.object(driver, "encrypt", side_effect=completed_probe):
            driver.transport(successful)
        self.assertTrue({"transport-admission/probe.age", "transport-admission/probe.age.age.stderr",
                         "transport-admission/probe.age.producer.stderr", "transport-admission/receipt.json"}.issubset(
                             {row["name"] for row in driver.inventory(successful / "private")}))
        report = driver.run([sys.executable, "-B", "-c", "import sys;print('retained named exit');sys.exit(1)"],
                            retained / "named-nonzero", expected=1)
        self.assertEqual(report["status"], "nonzero")
        self.assertTrue((retained / "named-nonzero/process.stdout").read_text().startswith("retained named exit"))
        with self.assertRaises(ValueError):
            driver.run([sys.executable, "-B", "-c", "raise SystemExit(2)"], retained / "unexpected-nonzero", expected=1)
        self.assertEqual(json.loads((retained / "unexpected-nonzero/process.process.json").read_text())["returncode"], 2)
        members = driver.inventory(retained)
        archive_body = io.BytesIO()
        archive_stream = io.TextIOWrapper(archive_body)
        with contextlib.redirect_stdout(archive_stream):
            driver.write_tar(retained, members)
        with tarfile.open(fileobj=io.BytesIO(archive_body.getvalue())) as archive:
            self.assertEqual(set(archive.getnames()), {row["name"] for row in members})
            self.assertEqual(archive.extractfile("named-nonzero/process.stdout").read(), b"retained named exit\n")
            self.assertEqual(archive.extractfile("probe-session/private/transport-admission/probe.age.partial").read(), b"fake-failed-cipher-partial")
            self.assertEqual(archive.extractfile("probe-session/private/transport-admission/probe.age.age.stderr").read(), b"fake-age-diagnostic")
            self.assertEqual(archive.extractfile("successful-probe-session/private/transport-admission/probe.age.producer.stderr").read(), b"fake-completed-producer-diagnostic")
        changed = [dict(row, sha256="0" * 64) if row["kind"] == "file" else row for row in members]
        with contextlib.redirect_stdout(io.TextIOWrapper(io.BytesIO())), self.assertRaises(ValueError):
            driver.write_tar(retained, changed)
        self.assertEqual(hashlib.sha256(driver.RECIPIENT.read_bytes()).hexdigest(), driver.RECIPIENT_SHA)
        self.assertFalse(policy["privacy"]["plaintext_upload"])
        space = driver.finalization_space(2 * 2**30, 3 * 2**30)
        self.assertEqual(space["headroom_bytes"], 8 * 2**30)
        self.assertGreater(space["required_free_bytes"], 11 * 2**30)
        text = controller.read_text()
        tree = ast.parse(text)
        self.assertNotIn("resource", {node.names[0].name for node in ast.walk(tree) if isinstance(node, ast.Import)})
        for token in ("run_child", "prepare_tool", "finalization_space", "producer=", "--require-hashes",
                      "--no-index", "--no-deps", "runtime.json", "qualification.json", "memory.events"):
            self.assertIn(token, text)
        flow = workflow.read_text()
        for token in ("workflow_dispatch:", "contents: read", "actions: read", "timeout-minutes: 90",
                      "--cpus=4", "--memory=8g", "--memory-swap=8g", "if: always()",
                      "evidence.tar.age", "transport.json", "persist-credentials: false"):
            self.assertIn(token, flow)
        self.assertNotRegex(flow, r"(?m)^  (?:push|pull_request|schedule):")
        upload = flow.split("uses: actions/upload-artifact@", 1)[1]
        self.assertNotIn("${{ env.EVIDENCE }}", upload)
        self.assertNotIn("setup-python", flow)
        self.assertNotIn("Rscript", flow)
        self.assertEqual(policy["limits"], {"cpus": 4, "memory_bytes": 8 * 2**30,
                         "retained_bytes": 2 * 2**30, "tar_bytes": 3 * 2**30,
                         "headroom_bytes": 8 * 2**30, "per_call_seconds": 300})


if __name__ == "__main__":
    unittest.main()
