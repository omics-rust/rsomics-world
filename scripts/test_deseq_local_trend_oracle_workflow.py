import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from test_infercnv_ingestion_workflow import bash_body, steps


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/deseq-local-trend-oracle.yml'


class DeseqLocalTrendOracleWorkflowTests(unittest.TestCase):
    def test_two_source_processes_retain_failures_and_propagate_status(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing local-trend source workflow')
        route = bash_body(steps(WORKFLOW.read_text())['Fit frozen cases and reload in a fresh R process'])
        source_boundary = r'''
Rscript() {
    test "$#" -eq 6 || return 91
    test "$1" = --vanilla && test -f "$2" || return 92
    test -d "$4" && test -d "$5" && test "$6" = "$EVIDENCE/local" || return 93
    source_pid=$(sh -c 'printf "%s" "$PPID"')
    printf '%s\t%s\n' "$3" "$source_pid" >> "$EVIDENCE/processes.tsv"
    printf 'source-%s-stdout\n' "$3"
    printf 'source-%s-stderr\n' "$3" >&2
    case "$3" in
        fit) return "$FIT_STATUS" ;;
        reload) return "$RELOAD_STATUS" ;;
        *) return 94 ;;
    esac
}
tee() {
    command tee "$@"
    stream_status=$?
    if (( stream_status != 0 )); then return "$stream_status"; fi
    return "$LOG_STATUS"
}
'''
        for fit_status, reload_status, log_status, expected in ((0, 0, 0, 0), (1, 0, 0, 1), (0, 2, 0, 1), (0, 0, 1, 1)):
            with self.subTest(fit=fit_status, reload=reload_status, log=log_status):
                directory = Path(tempfile.mkdtemp(prefix='deseq-local-routing-', dir=os.environ['TMPDIR']))
                (directory / 'controller').mkdir()
                (directory / 'DESeq2').mkdir()
                (directory / 'locfit').mkdir()
                environment = dict(os.environ, EVIDENCE=str(directory),
                                   FIT_STATUS=str(fit_status), RELOAD_STATUS=str(reload_status), LOG_STATUS=str(log_status))
                result = subprocess.run(['bash', '-c', source_boundary + route], cwd=ROOT,
                                        env=environment, capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, expected, result.stderr)
                calls = [line.split('\t') for line in (directory / 'processes.tsv').read_text().splitlines()]
                self.assertEqual([call[0] for call in calls], ['fit', 'reload'])
                self.assertNotEqual(calls[0][1], calls[1][1])
                self.assertEqual((directory / 'process-status.tsv').read_text(),
                                 f'fit\t{log_status or fit_status}\nreload\t{log_status or reload_status}\n')
                for phase in ('fit', 'reload'):
                    log = (directory / f'{phase}.log').read_text()
                    self.assertIn(f'source-{phase}-stdout', log)
                    self.assertIn(f'source-{phase}-stderr', log)

    def test_source_only_workflow_shell_and_pins(self):
        self.assertTrue(WORKFLOW.is_file(), 'missing local-trend source workflow')
        text = WORKFLOW.read_text()
        self.assertIn("r-version: '4.6.1'", text)
        self.assertIn('fa31b9d407f3557a6fcb7105cbd7b16a32a6fb868a7c60bc38a3e33cb7a39343', text)
        self.assertIn('https://cran.r-project.org/src/contrib/locfit_1.5-9.12.tar.gz', text)
        self.assertNotIn('continue-on-error', text)
        routed = steps(text)
        self.assertIn('if: always()', routed['Retain complete source and failed-case evidence'])
        for name, step in routed.items():
            if '        run: |\n' in step:
                with self.subTest(step=name):
                    body = re.sub(r'\$\{\{.*?\}\}', 'stub', bash_body(step))
                    result = subprocess.run(['bash', '-n'], input=body,
                                            capture_output=True, text=True, timeout=15)
                    self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
