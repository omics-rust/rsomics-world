"""Stdlib-only performance-controller preparation, never a science/timing run."""

import importlib
import ast
import gzip
import io
import json
import os
from pathlib import Path
import re
import sqlite3
import struct
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest import mock
import zipfile

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / '.github/workflows/signal-real-performance.yml'


class SignalPerformanceTests(unittest.TestCase):
    def test_api_download_redirect_does_not_forward_api_credentials(self):
        module=importlib.import_module('signal_real_performance')
        def response(request, timeout):
            self.assertEqual(request.get_header('Authorization'), 'Bearer fixture-not-a-credential')
            redirected=module.urllib.request.HTTPRedirectHandler().redirect_request(
                request,None,302,'Found',{},'https://fixture.blob.core.windows.net/artifact')
            self.assertNotIn('authorization', {key.lower() for key,_ in redirected.header_items()})
            return io.BytesIO(b'{"retained":"transport fixture only"}')
        with tempfile.TemporaryDirectory(prefix='signal-perf-redirect-') as directory:
            destination=Path(directory)/'original.json'
            with mock.patch.dict(os.environ,GH_TOKEN='fixture-not-a-credential'),mock.patch.object(module.urllib.request,'urlopen',side_effect=response):
                result=module.api('actions/artifacts/1/zip',destination)
            self.assertEqual(result,{'retained':'transport fixture only'})
            self.assertEqual(destination.read_bytes(),b'{"retained":"transport fixture only"}')

    def test_receipt_requires_actual_head_blob(self):
        module=importlib.import_module('signal_real_performance')
        import hashlib
        import zlib
        with tempfile.TemporaryDirectory(prefix='signal-perf-adoption-') as directory:
            root=Path(directory); git=root/'.git'
            (git/'objects').mkdir(parents=True); (git/'refs/heads').mkdir(parents=True)
            (git/'HEAD').write_text('ref: refs/heads/main\n')
            def object_id(kind,body):
                data=kind.encode()+b' '+str(len(body)).encode()+b'\0'+body
                digest=hashlib.sha1(data).hexdigest(); path=git/'objects'/digest[:2]/digest[2:]
                path.parent.mkdir(exist_ok=True); path.write_bytes(zlib.compress(data)); return digest
            body=b'{"fixture":"membership only, not a scientific receipt"}\n'
            blob=object_id('blob',body)
            tree=object_id('tree',b'100644 adopted.json\0'+bytes.fromhex(blob))
            tree=object_id('tree',b'40000 oracles\0'+bytes.fromhex(tree))
            tree=object_id('tree',b'40000 .autopilot\0'+bytes.fromhex(tree))
            commit=object_id('commit',('tree '+tree+'\nauthor Fixture <fixture@example.invalid> 0 +0000\ncommitter Fixture <fixture@example.invalid> 0 +0000\n\nMembership fixture\n').encode())
            (git/'refs/heads/main').write_text(commit+'\n')
            path=root/'.autopilot/oracles/adopted.json'; path.parent.mkdir(parents=True); path.write_bytes(body)
            data,binding=module.committed_receipt(root,'.autopilot/oracles/adopted.json')
            self.assertEqual(data,body)
            self.assertEqual(binding,dict(selector='.autopilot/oracles/adopted.json',commit=commit,blob=blob,sha256=hashlib.sha256(body).hexdigest()))
            draft=path.with_name('draft.json'); draft.write_bytes(body)
            with self.assertRaises(ValueError): module.committed_receipt(root,'.autopilot/oracles/draft.json')
            path.write_bytes(body+b' ')
            with self.assertRaises(ValueError): module.committed_receipt(root,'.autopilot/oracles/adopted.json')

    def test_full_route_receipt_and_schedule(self):
        self.assertTrue(WORKFLOW.is_file(), 'complete signal performance workflow is absent')
        module = importlib.import_module('signal_real_performance')
        with self.assertRaises((OSError, ValueError)):
            module.load_receipt(ROOT, '.autopilot/oracles/absent-signal-native.json')
        schedule = module.pair_schedule()
        self.assertEqual(len(schedule), 30)
        self.assertEqual([r['order'] for r in schedule[:5]], [['source', 'native'], ['native', 'source'], ['source', 'native'], ['native', 'source'], ['source', 'native']])
        self.assertEqual({(r['case'], r['workers']) for r in schedule}, {(c, t) for c in ('main', 'track20', 'atac') for t in (1, 4)})
        for workers in (1, 4):
            native=module.graph('native','main',workers,'/qualified/input','/trial/native','/qualified/ELF','/pinned/python','/pinned/runtime')
            source=module.graph('source','main',workers,'/qualified/input','/trial/source',None,'/pinned/python','/pinned/runtime')
            self.assertEqual([s['name'] for s in native],[s['name'] for s in source])
            self.assertEqual(len(native),18)
            self.assertEqual([s['name'] for s in native[:9]],['filter-p0','filter-p1','filter-p2','sort-index-p0','sort-index-p1','sort-index-p2','track-p0','track-p1','track-p2'])
            self.assertEqual(sum(s['shared'] for s in native),3)
            average=next(s for s in native if s['name']=='average')['argv']
            self.assertEqual(average[3:6],['/trial/native/p0.bw','/trial/native/p1.bw','/trial/native/p2.bw'])
            matrix=next(s for s in source if s['name']=='matrix-scale')['argv']
            self.assertIn('/trial/source/p0.bw',matrix)
            self.assertNotIn('/trial/native/p0.bw',matrix)
            for side in (native,source):
                self.assertTrue(all(str(workers) in s['argv'] for s in side))
                self.assertTrue(all(s['outputs'] for s in side))
            atac=module.graph('source','atac',workers,'/input','/atac',None,'/pinned/python','/pinned/runtime')
            self.assertEqual([s['name'] for s in atac],['proper-atac','sort-index-atac','macs-fragments'])
            self.assertNotIn('--filteredOutReads',atac[-1]['argv'])
        from test_infercnv_ingestion_workflow import bash_body,steps
        text=WORKFLOW.read_text()
        self.assertNotIn('continue-on-error',text)
        for name,step in steps(text).items():
            if '        run: |\n' not in step: continue
            body=re.sub(r'\$\{\{.*?\}\}','explicit-only',bash_body(step))
            result=subprocess.run(['bash','-n'],input=body,text=True,capture_output=True)
            self.assertEqual(result.returncode,0,name+result.stderr)
        for name in ('signal_real_performance.py','signal_real_performance_check.py'):
            ast.parse((ROOT/'scripts'/name).read_text())

    def test_exact_qname_payload_union_and_membership(self):
        module=importlib.import_module('signal_real_performance')
        checker=importlib.import_module('signal_real_performance_check')
        self.assertEqual([module.bucket(name) for name in (b'mate',b'mate/1',b'mate/2')],[0,0,1])
        header=b'BAM\1'+struct.pack('<i',0)+struct.pack('<i',1)+struct.pack('<i',5)+b'chr1\0'+struct.pack('<i',100)
        def record(name,pos):
            return struct.pack('<iiIIiiii',0,pos,len(name)+1,0,0,-1,-1,0)+name+b'\0'
        records=[record(b'mate',1),record(b'mate/1',2),record(b'mate/2',3),record(b'mate',4)]
        raw=header+b''.join(struct.pack('<i',len(r))+r for r in records)
        with tempfile.TemporaryDirectory(prefix='signal-perf-partition-test-') as directory:
            outputs=[io.BytesIO() for _ in range(3)]
            state=module.partition_stream(io.BytesIO(raw),outputs,Path(directory)/'membership.sqlite')
            self.assertEqual([r['records'] for r in state['partitions']],[3,1,0])
            for index,expected in [(0,[records[0],records[1],records[3]]),(1,[records[2]]),(2,[])]:
                stream=io.BytesIO(outputs[index].getvalue())
                actual,_=checker.bam_header(stream)
                self.assertEqual(actual,header)
                self.assertEqual(list(checker.bam_records(stream)),expected)
            db=sqlite3.connect(Path(directory)/'membership.sqlite')
            try:
                self.assertEqual(db.execute('select records,bucket from qnames where qname=?',(b'mate',)).fetchone(),(2,0))
            finally:
                db.close()

    def test_streaming_numeric_classes_and_window_identities(self):
        checker=importlib.import_module('signal_real_performance_check')
        class Bounded(io.BytesIO):
            def read(self,size=-1):
                if size<0 or size>1024*1024: raise AssertionError('unbounded read')
                return super().read(size)
        def npy(values,shape):
            text=str(dict(descr='<f8',fortran_order=False,shape=shape)).encode()+b'\n'
            return b'\x93NUMPY\1\0'+struct.pack('<H',len(text))+text+b''.join(struct.pack('<d',v) for v in values)
        header=checker.npy_header(Bounded(npy([1.0],(1,1))))
        self.assertEqual(header['shape'],(1,1))
        self.assertFalse(checker.equal(float('nan'),0.0))
        self.assertFalse(checker.equal(float('inf'),-float('inf')))
        with tempfile.TemporaryDirectory(prefix='signal-perf-numbers-') as directory:
            root=Path(directory)
            def container(name,values):
                path=root/name
                with zipfile.ZipFile(path,'w',compression=zipfile.ZIP_DEFLATED) as out:
                    out.writestr('matrix.npy',npy(values,(2,2)))
                    text=b"{'descr': '<U2', 'fortran_order': False, 'shape': (2,), }\n"
                    out.writestr('labels.npy',b'\x93NUMPY\1\0'+struct.pack('<H',len(text))+text+'P0P1'.encode('utf-32-le'))
                return path
            a=container('a.npz',[1.0,float('nan'),2.0,float('inf')])
            b=container('b.npz',[1.0,float('nan'),2.0,float('inf')])
            self.assertEqual(checker.compare_npz(a,b)['cells'],4)
            bad=container('bad.npz',[1.0,0.0,2.0,float('inf')])
            with self.assertRaises(ValueError): checker.compare_npz(a,bad)
            with self.assertRaises(ValueError): checker.compare_npz(a,bad,io.StringIO(),0)
            one=root/'one.tsv'; two=root/'two.tsv'
            one.write_text("#coordinates\nchr1\t0\t10\t0.0\n")
            two.write_text("#coordinates\nchr1\t1\t10\t0.0\n")
            with self.assertRaises(ValueError): checker.compare_table(one,two,3)
        cursor=checker.RunCursor([(2,4,1.0),(7,10,0.0)])
        self.assertEqual(list(cursor.segments(0,8)),[(0,2,None),(2,4,1.0),(4,7,None),(7,8,0.0)])
        self.assertEqual(list(cursor.segments(8,10)),[(8,10,0.0)])

    def test_resource_scope_and_strict_ranges(self):
        module=importlib.import_module('signal_real_performance')
        self.assertEqual(module.resource_plan([2,3,4,5],range(8),{'MemAvailable':2*2**30},40*2**30)['memory_abort_bytes'],16*2**30)
        for cpus,memory,free in [([0,1,2,9],{'MemAvailable':2*2**30},40*2**30),([0,1,2,3],{'MemAvailable':0},40*2**30),([0,1,2,3],{'MemAvailable':2*2**30},2**30)]:
            with self.assertRaises(ValueError): module.resource_plan(cpus,range(8),memory,free)
        rows=[dict(tool=tool,wall_seconds=value,aggregate_tree_peak_bytes=None,aggregate_tree_peak_scope=None,sampled_sum_rss_peak_bytes=1 if tool=='native' else 1000)
              for tool,values in [('source',[10,11,12,13,14]),('native',[9,10,11,12,13])] for value in values]
        self.assertFalse(module.strict_decision(rows)['strict_advantage'])
        for row in rows:
            row['aggregate_tree_peak_bytes']=50 if row['tool']=='native' else 100
        self.assertFalse(module.strict_decision(rows)['strict_exact_tree_memory'])
        for row in rows: row['aggregate_tree_peak_scope']='dedicated cgroup v2 hierarchical memory.peak; anon/file/kernel usage, not RSS'
        self.assertTrue(module.strict_decision(rows)['strict_exact_tree_memory'])
        stage=module.strict_decision(rows,stage=True)
        self.assertFalse(stage['strict_advantage'])
        self.assertIsNone(stage['strict_exact_tree_memory'])
        self.assertFalse(stage['exact_tree_memory_available'])
        self.assertIsNone(stage['memory_source'])
        for row in rows:
            if row['tool']=='native': row['wall_seconds']-=8
        self.assertTrue(module.strict_decision(rows,stage=True)['strict_advantage'])
        with self.assertRaises(ValueError): module.strict_decision(rows[:-1])

    def test_carrier_safety_and_original_failure_retention(self):
        module=importlib.import_module('signal_real_performance')
        checker=importlib.import_module('signal_real_performance_check')
        for names in [('a','A'),('a','a/b'),('../escape',)]:
            stream=io.BytesIO()
            with zipfile.ZipFile(stream,'w') as archive:
                for name in names: archive.writestr(name,b'body')
            stream.seek(0)
            with zipfile.ZipFile(stream) as archive,self.assertRaises(ValueError): module.zip_members(archive)
        with tempfile.TemporaryDirectory(prefix='signal-perf-failure-') as directory:
            root=Path(directory); (root/'bin').mkdir()
            (root/'bin/failure').write_text('print("original deliberate failure"); raise SystemExit(7)\n')
            with mock.patch.object(sys,'prefix',str(root)),self.assertRaises(RuntimeError):
                checker.source_call(root/'evidence','failure','failure',[],1)
            status=json.loads((root/'evidence/source-calls/failure/command.json').read_text())
            self.assertEqual(status['status'],7)
            self.assertIn('original deliberate failure',(root/'evidence/source-calls/failure/stdout.txt').read_text())
            output=root/'occupied'; output.mkdir(); (output/'original').write_text('retain')
            with self.assertRaises(ValueError): checker.check_pair(root,ROOT,'main',1,root,root,output)
            self.assertEqual((output/'original').read_text(),'retain')
            self.assertFalse((output/'qualification.json').exists())


if __name__ == '__main__':
    unittest.main()
