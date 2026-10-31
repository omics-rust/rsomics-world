"""Bounded complete-product/source-consumer validation; scientific imports are hosted-only."""

import argparse
import ast
from contextlib import ExitStack
import gzip
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import sqlite3
import struct
import sys
import zipfile

from infercnv_source_snapshot import _hash_file
import signal_first_workflow_oracle as oracle

CHUNK=1024*1024
MATRIX_KEYS=('sample_boundaries','group_boundaries','sample_labels','group_labels')
PARAMETER_KEYS=('upstream','downstream','body','bin size','unscaled 5 prime','unscaled 3 prime','ref point','missing data as zero','bin avg type')


def check(condition,message):
    if not condition: raise ValueError(message)


def save(path,value):
    Path(path).parent.mkdir(parents=True,exist_ok=True)
    with Path(path).open('x') as out:
        json.dump(oracle.encode(value),out,sort_keys=True,indent=2,allow_nan=False); out.write('\n')


def load(path):
    with Path(path).open() as stream: return json.load(stream)


def equal(a,b,absolute=1e-12,relative=1e-12):
    if math.isnan(a) or math.isnan(b): return math.isnan(a) and math.isnan(b)
    if math.isinf(a) or math.isinf(b): return a==b
    return abs(a-b)<=absolute+relative*abs(b)


def exact(stream,size):
    check(0<=size<=16*CHUNK,'bounded binary field exceeded')
    data=stream.read(size)
    check(len(data)==size,'truncated binary input')
    return data


def bam_header(stream):
    head=exact(stream,8)
    check(head[:4]==b'BAM\1','not a BAM stream')
    length=struct.unpack('<i',head[4:])[0]
    text=exact(stream,length)
    raw_count=exact(stream,4); count=struct.unpack('<i',raw_count)[0]
    check(0<count<=262144,'BAM reference count exceeds private bound')
    chunks=[head,text,raw_count]; refs=[]
    for _ in range(count):
        size_raw=exact(stream,4); size=struct.unpack('<i',size_raw)[0]
        check(1<size<=4096,'BAM reference-name size differs')
        name=exact(stream,size); length_raw=exact(stream,4)
        check(name[-1:]==b'\0' and b'\0' not in name[:-1],'BAM reference name is not exact NUL-terminated')
        refs.append([name[:-1].decode('utf8'),struct.unpack('<i',length_raw)[0]])
        chunks.extend([size_raw,name,length_raw])
    return b''.join(chunks),refs


def bam_records(stream):
    while raw:=stream.read(4):
        check(len(raw)==4,'truncated BAM record-size word')
        size=struct.unpack('<i',raw)[0]
        check(32<=size<=16*CHUNK,'BAM record allocation exceeds bound')
        yield exact(stream,size)


def record_fields(record):
    ref,pos,bin_mq_name,flag_cigar,sequence,mate,mpos,tlen=struct.unpack('<iiIIiiii',record[:32])
    name_size=bin_mq_name&255; cigar_size=flag_cigar&65535
    check(name_size>0 and 32+name_size+4*cigar_size<=len(record),'invalid raw BAM field extent')
    qname=record[32:32+name_size]
    check(qname[-1:]==b'\0' and b'\0' not in qname[:-1],'invalid exact BAM QNAME bytes')
    cigar=[(v&15,v>>4) for (v,) in struct.iter_unpack('<I',record[32+name_size:32+name_size+4*cigar_size])]
    return dict(reference=ref,position=pos,flags=flag_cigar>>16,qname=qname[:-1],
        mapq=(bin_mq_name>>8)&255,cigar=cigar,sequence_length=sequence,mate_reference=mate,mate_position=mpos,tlen=tlen)


def npy_header(stream):
    magic=exact(stream,8); check(magic[:6]==b'\x93NUMPY','not NPY')
    version=tuple(magic[6:])
    check(version in ((1,0),(2,0),(3,0)),'unrecognized NPY version')
    width=2 if version==(1,0) else 4
    size=int.from_bytes(exact(stream,width),'little')
    check(size<=65536,'NPY header exceeds64KiB')
    header=ast.literal_eval(exact(stream,size).decode('utf8' if version==(3,0) else 'latin1').strip())
    check(header['fortran_order'] is False and isinstance(header['shape'],tuple) and
          all(type(n) is int and n>=0 for n in header['shape']),'NPY shape/layout differs')
    return header


class Npz:
    def __init__(self,path):
        self.stack=ExitStack()
        archive=self.stack.enter_context(zipfile.ZipFile(path))
        check(len(archive.namelist())==2 and set(archive.namelist())=={'matrix.npy','labels.npy'},'NPZ schema differs')
        self.stream=self.stack.enter_context(archive.open('matrix.npy'))
        self.header=npy_header(self.stream)
        check(self.header['descr']=='<f8' and len(self.header['shape'])==2,'summary must be row-major f64 matrix')
        with archive.open('labels.npy') as stream:
            labels=npy_header(stream)
            check(len(labels['shape'])==1 and labels['shape'][0]==self.header['shape'][1],'summary label shape differs')
            import re
            check(re.fullmatch('<U[0-9]+',labels['descr']),'summary label dtype differs')
            width=int(labels['descr'][2:])*4
            check(width<=4096 and labels['shape'][0]<=4096,'label allocation bound exceeded')
            self.labels=[exact(stream,width).decode('utf-32-le').rstrip('\0') for _ in range(labels['shape'][0])]
            check(stream.read(1)==b'','extra NPY label bytes')
    def values(self):
        remaining=math.prod(self.header['shape'])
        while remaining:
            count=min(remaining,8192)
            data=exact(self.stream,count*8)
            yield from (value for (value,) in struct.iter_unpack('<d',data))
            remaining-=count
        check(self.stream.read(1)==b'','extra NPY matrix bytes')
    def __enter__(self): return self
    def __exit__(self,*args): self.stack.close()


def compare_npz(first,second,differences=None,allowed_column=None):
    compared=different=0
    with Npz(first) as a,Npz(second) as b:
        check(a.header==b.header and a.labels==b.labels,'NPZ dtype/shape/labels differ')
        columns=a.header['shape'][1]
        for index,(left,right) in enumerate(zip(a.values(),b.values(),strict=True)):
            compared+=1
            if not equal(left,right):
                check(allowed_column is not None and index%columns==allowed_column,'NPZ categorical/f64 cell differs')
                check(differences is not None,'every attributed producer-delta cell requires retained evidence')
                differences.write(json.dumps(oracle.encode(dict(index=index,row=index//columns,column=index%columns,source=left,native=right)))+'\n')
                different+=1
        return dict(shape=a.header['shape'],labels=a.labels,cells=compared,attributed_difference_cells=different)


def line_fields(stream):
    while line:=stream.readline(2*CHUNK):
        check(len(line)<2*CHUNK,'text row exceeds2MiB bound')
        yield line.rstrip('\r\n').split('\t')


def compare_table(first,second,identity_columns,header_lines=1,absolute=1e-12,allowed_column=None,differences=None):
    rows=values=0
    with Path(first).open() as a,Path(second).open() as b:
        for _ in range(header_lines):
            check(a.readline()==b.readline(),'table header/schema differs')
        for left,right in itertools.zip_longest(line_fields(a),line_fields(b)):
            check(left is not None and right is not None and len(left)==len(right) and
                  left[:identity_columns]==right[:identity_columns],'table identities/shape/order differ')
            for col,(x,y) in enumerate(zip(left[identity_columns:],right[identity_columns:])):
                values+=1
                if not equal(float(x),float(y),absolute=absolute):
                    check(allowed_column==col and differences is not None,'table categorical/numeric cell differs')
                    differences.write(json.dumps(oracle.encode(dict(row=rows,identity=left[:identity_columns],
                        column=col,source=float(x),native=float(y))))+'\n')
            rows+=1
    return dict(rows=rows,values=values)


def scalar_parameter(header,key):
    value=header.get(key)
    return value[0] if isinstance(value,list) and value and all(x==value[0] for x in value) else value


def compare_matrix(first,second):
    rows=cells=0
    with gzip.open(first,'rt') as a,gzip.open(second,'rt') as b:
        ah=a.readline(); bh=b.readline()
        check(ah.startswith('@') and bh.startswith('@'),'matrix schema header absent')
        ah=json.loads(ah[1:]); bh=json.loads(bh[1:])
        check(all(ah[key]==bh[key] for key in MATRIX_KEYS),'matrix sample/group schemas differ')
        check(all(scalar_parameter(ah,key)==scalar_parameter(bh,key) for key in PARAMETER_KEYS),'matrix window/mean/missingness parameters differ')
        columns=ah['sample_boundaries'][-1]
        for left,right in itertools.zip_longest(line_fields(a),line_fields(b)):
            check(left is not None and right is not None and len(left)==len(right)==columns+6 and left[:6]==right[:6],
                  'matrix complete row identities/order differ')
            for x,y in zip(left[6:],right[6:]):
                check(equal(float(x),float(y),absolute=0.5e-6),'six-decimal matrix cell/nonfinite differs')
                cells+=1
            rows+=1
        check(rows==ah['group_boundaries'][-1],'matrix group-boundary population differs')
    return dict(rows=rows,cells=cells,groups=ah['group_labels'],samples=ah['sample_labels'])


def pinned_runtime(root,world):
    from signal_source_on_native import hosted_runtime
    root=Path(root); world=Path(world)
    state=hosted_runtime(root/'runtime',world/'assets/signal-first-runtime-20261031.json')
    import pysam
    check(pysam.__samtools_version__=='1.21','actual installed dispatcher is not samtools1.21')
    wrappers={}
    for name,module in oracle.ENTRYPOINTS.items():
        path=root/'runtime/venv/bin'/name
        text=path.read_text()
        check('from deeptools.'+module+' import main' in text,'installed source CLI entry point differs: '+name)
        wrappers[name]=_hash_file(path)
    state.update(console_scripts=wrappers,samtools_version=pysam.__samtools_version__)
    path=root/'evidence/runtime-before.json'
    if not path.exists(): save(path,state)
    else: check(load(path)==state,'source runtime bodies changed')
    return state


def source_call(output,name,entry,args,workers):
    target=Path(output)/'source-calls'/name
    script=Path(sys.prefix)/'bin'/entry
    command=[sys.executable,'-B',str(script),*map(str,args),'-p',str(workers)]
    oracle.capture(target,command)
    return target


def bam_compare(paths,output,case):
    database=sqlite3.connect(output/'exact-bam-payloads.sqlite')
    database.execute('pragma cache_size=-8192'); database.execute('pragma journal_mode=persist')
    database.execute('create table payloads(context text,payload blob,original integer default 0,source_pass integer default 0,native_pass integer default 0,source_sorted integer default 0,native_sorted integer default 0,source_reject integer default 0,native_reject integer default 0,primary key(context,payload)) without rowid')
    counts={}; dictionaries={}
    def consume(context,column,path):
        count=mapped=0
        with gzip.open(path,'rb') as stream:
            header,refs=bam_header(stream)
            if context in dictionaries: check(dictionaries[context]==refs,'BAM reference identities/order differ')
            else: dictionaries[context]=refs
            for payload in bam_records(stream):
                fields=record_fields(payload)
                mapped+=not fields['flags']&4; count+=1
                database.execute(f'insert into payloads(context,payload,{column}) values(?,?,1) on conflict(context,payload) do update set {column}={column}+1',(context,payload))
                if count%50000==0: database.commit()
        database.commit()
        counts[context+'-'+column]=dict(records=count,mapped=mapped,header_sha256=hashlib.sha256(header).hexdigest())
    for context,items in paths.items():
        for column,path in items.items(): consume(context,column,path)
    if case=='main':
        bad=database.execute('select count(*) from payloads where source_pass!=native_pass or source_sorted!=source_pass or native_sorted!=native_pass or source_reject!=native_reject or original!=source_pass+source_reject').fetchone()[0]
    else:
        bad=database.execute('select count(*) from payloads where source_pass!=native_pass or source_sorted!=source_pass or native_sorted!=native_pass or source_reject!=native_reject').fetchone()[0]
    check(bad==0,'exact BAM record multiset/filter/rejected/sort preservation differs')
    database.close()
    return dict(counts=counts,dictionaries=dictionaries,exact_raw_payload_multisets=True,
                header_policy='full references exact; truthful tool PG/CL and sort metadata retained, not forged/equated')


def metrics(path):
    with Path(path).open() as stream:
        check(stream.readline()=='#bamFilterReads --filterMetrics\n' and
              stream.readline()=='#File\tReads Remaining\tTotal Initial Reads\n','filter metrics schema differs')
        row=stream.readline().rstrip('\n').split('\t')
        check(len(row)==3 and stream.read(1)=='','filter metrics population differs')
        return dict(label=row[0],remaining=int(row[1]),initial=int(row[2]))


def filter_metrics(source,native,stem,counts,context):
    a=metrics(source/(stem+'.metrics.tsv')); b=metrics(native/(stem+'.metrics.tsv'))
    check(a==b,'actual filter metrics differ')
    p=counts[context+'-source_pass']['records']; r=counts[context+'-source_reject']['records']
    dropped=a['remaining']-p
    check(dropped>=0 and a['initial']==p+r+dropped,'filter/rejected/shift-drop populations differ')
    report=load(native/((('filter-'+stem) if stem!='atac' else 'proper-atac')+'.report.json'))
    check(report['initial_records']==a['initial'] and report['metrics_remaining']==a['remaining'] and
          report['passed_alignments']==p and report['shift_dropped']==dropped and report['filtered']==r,
          'native observed counters differ from actual source record/metric populations')
    return dict(metrics=a,passed=p,rejected=r,shift_dropped=dropped)


def bw_runs(handle,chrom,length,block=1000000):
    end_seen=0
    for start in range(0,length,block):
        for left,right,value in handle.intervals(chrom,start,min(length,start+block)) or ():
            if right<=end_seen: continue
            check(left>=end_seen and left<right<=length,'BW intervals overlap or escape dictionary')
            yield (left,right,float(value))
            end_seen=right


def compare_bw(first,second):
    import pyBigWig
    count=0
    with pyBigWig.open(str(first)) as a,pyBigWig.open(str(second)) as b:
        dictionary=list(a.chroms().items())
        check(dictionary==list(b.chroms().items()),'BW dictionaries/order differ')
        for chrom,length in dictionary:
            for left,right in itertools.zip_longest(bw_runs(a,chrom,length),bw_runs(b,chrom,length)):
                check(left is not None and right is not None and left[:2]==right[:2] and
                      equal(left[2],right[2],absolute=0,relative=0),'BW six-significant-digit/float32 run population differs')
                count+=1
    return dict(intervals=count,dictionary=dictionary)


def source_writer(native,output):
    import pyBigWig
    records=0
    with pyBigWig.open(str(native)) as source,pyBigWig.open(str(output),'w') as target:
        dictionary=list(source.chroms().items()); target.addHeader(dictionary,maxZooms=10)
        for chrom,length in dictionary:
            buffered=[]
            for row in bw_runs(source,chrom,length):
                buffered.append(row)
                if len(buffered)==16384:
                    target.addEntries([chrom]*len(buffered),[r[0] for r in buffered],ends=[r[1] for r in buffered],values=[r[2] for r in buffered])
                    records+=len(buffered); buffered.clear()
            if buffered:
                target.addEntries([chrom]*len(buffered),[r[0] for r in buffered],ends=[r[1] for r in buffered],values=[r[2] for r in buffered])
                records+=len(buffered)
    return dict(path=str(output),sha256=_hash_file(output),intervals=records,
                diagnostic_only=True,replaces_native_graph_input=False)


def stats_diagnostic(native,comparator,output):
    import pyBigWig
    queries=0
    with pyBigWig.open(str(native)) as a,pyBigWig.open(str(comparator)) as b,gzip.open(output,'xt') as out:
        check(a.chroms()==b.chroms(),'diagnostic writer dictionary differs')
        for chrom,length in a.chroms().items():
            windows=[(0,length),(min(17,length-1),min(1813,length)),(max(0,length-1003),length)]
            for start in range(10000000,length,10000000): windows.append((max(0,start-100),min(length,start+100)))
            for start,end in windows:
                for exact_query in (False,True):
                    av=a.stats(chrom,start,end,type='mean',exact=exact_query)
                    bv=b.stats(chrom,start,end,type='mean',exact=exact_query)
                    check(len(av)==len(bv) and all((x is None)==(y is None) and (x is None or equal(x,y)) for x,y in zip(av,bv)),
                          'actual same-stream default/exact diagnostic statistics differ')
                    out.write(json.dumps(oracle.encode(dict(chrom=chrom,start=start,end=end,exact=exact_query,native=av,source_writer=bv)))+'\n')
                    queries+=1
    return queries


class RunCursor:
    def __init__(self,rows):
        self.rows=iter(rows); self.current=next(self.rows,None)
    def segments(self,start,end):
        cursor=start
        while self.current is not None and self.current[1]<=start:
            self.current=next(self.rows,None)
        while cursor<end:
            if self.current is None or self.current[0]>=end:
                yield cursor,end,None; return
            left,right,value=self.current
            if cursor<left:
                stop=min(left,end); yield cursor,stop,None; cursor=stop; continue
            stop=min(right,end); yield cursor,stop,value; cursor=stop
            if cursor>=right: self.current=next(self.rows,None)


def average_tail_proof(source,native,output,workers):
    import pyBigWig
    from deeptools import writeBedGraph_bam_and_bw as writer
    from deeptools.bigwigAverage import average
    source=Path(source); native=Path(native)
    added_bases=added_tiles=tasks=all_zero=0
    with pyBigWig.open(str(source/'AVG.bw')) as original,pyBigWig.open(str(native/'AVG.bw')) as corrected,gzip.open(output/'zero-tail-attribution.jsonl.gz','xt') as audit:
        check(list(original.chroms().items())==list(corrected.chroms().items()),'AVG dictionaries differ')
        for chrom,length in original.chroms().items():
            old=RunCursor(bw_runs(original,chrom,length)); new=RunCursor(bw_runs(corrected,chrom,length))
            for start in range(0,length,10000000):
                end=min(start+10000000,length); tasks+=1
                result=writer.writeBedGraph_worker(chrom,start,end,50,0,[(str(source/f'p{i}.bw'),'bigwig') for i in range(3)],
                    average,{'scaleFactors':[1.0,0.5,2.0]},extendPairedEnds=False,smoothLength=0,
                    skipZeroOverZero=False,missingDataAsZero=True,fixedStep=True)
                missing=[]; last_zero=None; seen=0; nonnegative=True
                with open(result[3]) as stream:
                    for row in line_fields(stream):
                        check(len(row)==4 and row[0]==chrom,'actual source fixed-step identity differs')
                        left,right=int(row[1]),int(row[2]); value=struct.unpack('<f',struct.pack('<f',float(row[3])))[0]
                        check(left==start+seen*50 and right==min(left+50,end),'actual source fixed-step grid differs')
                        check(math.isfinite(value) and value>=0,'chosen main profile cannot invoke signed-loss/nonfinite exemptions')
                        nonnegative &= value>=0
                        last_zero=(left if last_zero is None else last_zero) if value==0 else None
                        check(all(v is not None and equal(v,value,absolute=0,relative=0) for _,_,v in new.segments(left,right)),
                              'corrected native AVG differs from actual unmodified source scientific tile value')
                        for a,b,v in old.segments(left,right):
                            if v is None:
                                check(value==0,'source missing nonzero AVG value is not an approved tail exception')
                                if missing and missing[-1][1]==a: missing[-1][1]=b
                                else: missing.append([a,b])
                                added_bases+=b-a; added_tiles+=1
                            else: check(equal(v,value,absolute=0,relative=0),'independent source AVG differs outside named tail omission')
                        seen+=1
                check(seen==(end-start+49)//50,'source fixed-step task population differs')
                check(not missing or last_zero is not None and all(a>=last_zero and b<=end for a,b in missing),
                      'missing-zero delta is not confined to this actual source task terminal-zero run')
                all_zero+=last_zero==start
                audit.write(json.dumps(dict(chrom=chrom,start=start,end=end,source_fixed_step_file=result[3],
                    source_fixed_step_sha256=_hash_file(result[3]),source_tiles=seen,last_zero_run_start=last_zero,
                    independently_missing_zero_spans=missing,nonnegative=nonnegative,
                    incompatibility='SIGNAL-356-ZERO-TAIL' if missing else None))+'\n')
    return dict(tasks=tasks,added_zero_bases=added_bases,affected_tile_segments=added_tiles,all_zero_tasks=all_zero,
        actual_source_scientific_function='writeBedGraph_bam_and_bw.writeBedGraph_worker fixedStep=True + bigwigAverage.average',
        scientific_code_patched=False,diagnostic_only=True,graph_inputs_replaced=False,signed_loss_triggered=False)


def normalization(source,native,stem,entry,args,output):
    request=dict(kind='normalization',entrypoint=entry,args=list(map(str,args)),producer='actual timed source counterpart')
    result=oracle.science_action(request)
    save(output/(stem+'-source-population.json'),result)
    record=load(native/(stem+'.report.json'))
    factors=[float(x) for x in result['scale_factors']]
    check(len(record['factors'])==len(factors) and all(equal(a,b) for a,b in zip(record['factors'],factors)),
          'observed native normalization factors differ from actual source functions')
    details=[record['scale']] if entry=='bamCoverage' else record['scaling']['populations']
    check(len(details)==len(result['inputs']),'normalization population cardinality differs')
    for native_row,source_row in zip(details,result['inputs']):
        check(native_row['mapped_total']==source_row['mapped'] and equal(native_row['kept'],float(source_row['kept_reads'])),
              'observed native source-selected kept/mapped population differs')
    return dict(factors=factors,mapped=[r['mapped'] for r in result['inputs']],kept=[float(r['kept_reads']) for r in result['inputs']])


def same_native_consumers(root,source,native,output,workers):
    import signal_real_performance as perf
    inputs=Path(root)/'evidence/prepared'
    same=output/'same-native'; same.mkdir()
    comparator=output/'source-writer'; comparator.mkdir()
    graph=perf.graph('source','main',workers,inputs,same,None,sys.executable,Path(root)/'runtime')
    for name in ('p0','p1','p2','CBAM','CBW','AVG'):
        source_writer(native/(name+'.bw'),comparator/(name+'.bw'))
        compare_bw(native/(name+'.bw'),comparator/(name+'.bw'))
        stats_diagnostic(native/(name+'.bw'),comparator/(name+'.bw'),output/(name+'-default-exact.jsonl.gz'))
    receipts={}
    for row in graph:
        if not row['name'].startswith(('summary-bw-','matrix-')): continue
        argv=row['argv']
        entry=Path(argv[2]).name
        for lane,input_root in [('same-native',native),('source-writer',comparator)]:
            dest=same if lane=='same-native' else comparator
            args=[str(input_root/Path(value).name) if value.endswith('.bw') else str(dest/Path(value).name)
                  if value.startswith(str(same)+os.sep) else value for value in argv[3:-2]]
            source_call(output,row['name']+'-'+lane,entry,args,workers)
        name=row['name']
        if name.startswith('summary'):
            receipts[name]=compare_npz(native/(name+'.npz'),same/(name+'.npz'))
            compare_npz(same/(name+'.npz'),comparator/(name+'.npz'))
            compare_table(native/(name+'.tsv'),same/(name+'.tsv'),3)
            compare_table(same/(name+'.tsv'),comparator/(name+'.tsv'),3)
        else:
            receipts[name]=compare_matrix(native/(name+'.gz'),same/(name+'.gz'))
            compare_matrix(same/(name+'.gz'),comparator/(name+'.gz'))
            compare_table(native/(name+'.tsv'),same/(name+'.tsv'),0,3,absolute=0)
            compare_table(same/(name+'.tsv'),comparator/(name+'.tsv'),0,3,absolute=0)
            check(receipts[name]['rows']>=50000,'performance matrix retained fewer than50000 real rows')
    return receipts


def check_pair(root,world,case,workers,source,native,output,qualification=False):
    root=Path(root); source=Path(source)/'products'; native=Path(native)/'products'; output=Path(output)
    check(output.is_dir() and not any(output.iterdir()),'fresh existing comparison product directory required')
    pinned_runtime(root,world)
    before={str(p):_hash_file(p) for directory in (source,native) for p in directory.iterdir() if p.is_file()}
    result={}
    inputs=root/'evidence/prepared'
    if case=='main':
        paths={f'p{i}':dict(original=inputs/f'p{i}.bam',source_pass=source/f'p{i}.filtered.bam',
            native_pass=native/f'p{i}.filtered.bam',source_sorted=source/f'p{i}.sorted.bam',native_sorted=native/f'p{i}.sorted.bam',
            source_reject=source/f'p{i}.rejected.bam',native_reject=native/f'p{i}.rejected.bam') for i in range(3)}
        result['bam']=bam_compare(paths,output,case)
        result['filters']={f'p{i}':filter_metrics(source,native,f'p{i}',result['bam']['counts'],f'p{i}') for i in range(3)}
        for name in ('p0','p1','p2','CBAM','CBW'): result[name]=compare_bw(source/(name+'.bw'),native/(name+'.bw'))
        result['average_tail']=average_tail_proof(source,native,output,workers)
        for i in range(3):
            stem=f'track-p{i}'
            result[stem]=normalization(source,native,stem,'bamCoverage',['-b',source/f'p{i}.sorted.bam','-o',output/'unused.bw','--binSize','50','--normalizeUsing','CPM','-p',workers],output)
            if qualification:
                stem=f'p{i}-without-dedup'
                source_call(output,stem,'alignmentSieve',['-b',inputs/f'p{i}.bam','-o',output/(stem+'.bam'),
                    '--filteredOutReads',output/(stem+'.rejected.bam'),'--filterMetrics',output/(stem+'.metrics.tsv'),
                    '--minMappingQuality','20','--samFlagExclude','2816','--genomeChunkLength','1000000'],workers)
                nondedup=metrics(output/(stem+'.metrics.tsv'))
                result.setdefault('actual_positional_duplicate_removals',{})[f'p{i}']=nondedup['remaining']-result['filters'][f'p{i}']['metrics']['remaining']
                check(result['actual_positional_duplicate_removals'][f'p{i}']>=0,'source duplicate-removal population is negative')
        result['compare-bam']=normalization(source,native,'compare-bam','bamCompare',['-b1',source/'p0.sorted.bam','-b2',source/'p1.sorted.bam','-o',output/'unused.bw','--binSize','50','--operation','ratio','--scaleFactorsMethod','readCount','-p',workers],output)
        for kind in ('bam','bw'):
            for mode in ('bins','regions'):
                name='summary-'+kind+'-'+mode
                if kind=='bam':
                    result[name]=compare_npz(source/(name+'.npz'),native/(name+'.npz'))
                    compare_table(source/(name+'.tsv'),native/(name+'.tsv'),3)
                    compare_table(source/(name+'.factors.tsv'),native/(name+'.factors.tsv'),1)
                else:
                    with gzip.open(output/(name+'-npz-producer-deltas.jsonl.gz'),'xt') as differences:
                        result[name]=compare_npz(source/(name+'.npz'),native/(name+'.npz'),differences,2)
                    with gzip.open(output/(name+'-raw-producer-deltas.jsonl.gz'),'xt') as differences:
                        compare_table(source/(name+'.tsv'),native/(name+'.tsv'),3,allowed_column=2,differences=differences)
        for mode in ('reference','scale'):
            name='matrix-'+mode
            result[name]=compare_matrix(source/(name+'.gz'),native/(name+'.gz'))
            check(result[name]['rows']>=50000,'matrix retained fewer than50000 original rich rows')
            compare_table(source/(name+'.tsv'),native/(name+'.tsv'),0,3,absolute=0)
            result[name]['sorted_bed']=dict(source_sha256=_hash_file(source/(name+'.bed')),
                native_sha256=_hash_file(native/(name+'.bed')),acceptance='separate pending human policy; observational only')
        result['same_native_complete_consumers_and_source_writer']=same_native_consumers(root,source,native,output,workers)
    elif case=='track20':
        result['full_run_population']=compare_bw(source/'whole20.bw',native/'whole20.bw')
        result['normalization']=normalization(source,native,'track20','bamCoverage',['-b',inputs/'whole.bam','-o',output/'unused.bw','--binSize','20','--normalizeUsing','CPM','-p',workers],output)
        comparator=output/'source-writer.bw'; source_writer(native/'whole20.bw',comparator)
        compare_bw(native/'whole20.bw',comparator)
        result['default_exact_queries']=stats_diagnostic(native/'whole20.bw',comparator,output/'default-exact.jsonl.gz')
    elif case=='atac':
        paths={'atac':dict(source_pass=source/'atac.filtered.bam',native_pass=native/'atac.filtered.bam',
            source_sorted=source/'atac.sorted.bam',native_sorted=native/'atac.sorted.bam',
            source_reject=source/'atac.rejected.bam',native_reject=native/'atac.rejected.bam')}
        result['bam']=bam_compare(paths,output,case)
        result['filter']=filter_metrics(source,native,'atac',result['bam']['counts'],'atac')
        rows=0
        with (source/'fragments.bed').open() as a,(native/'fragments.bed').open() as b:
            for left,right in itertools.zip_longest(line_fields(a),line_fields(b)):
                check(left is not None and right is not None and left==right and len(left)==3 and int(left[1])<int(left[2]),'actual MACS3-column fragment rows/order differ')
                rows+=1
        result['fragment_rows']=rows
        check(result['fragment_rows']>0,'useful ATAC/MACS branch is empty')
    else: raise ValueError('unknown fixed case')
    check(all(_hash_file(p)==sha for p,sha in before.items()),'an independent timed graph product was changed/replaced by the checker')
    result.update(passed=True,case=case,workers=workers,source_products=str(source),native_products=str(native),
        all_timed_products_unchanged=True,normal_approximation_preserved=True,
        approved_deltas={'SIGNAL-356-ZERO-TAIL':'only actual source terminal-zero omissions proven by original fixed-step scientific worker; all differing downstream AVG cells retained and checked against same-native source consumers',
            'SIGNAL-356-ARITHMETIC-LOSS':'not triggered in this fixed nonnegative ratio/average profile; no exemption',
            'SIGNAL-356-FILTER-BED-REJECTED':'not triggered: prior BAM filter retains rejected BAM; final fragment stage has no second rejected request'},
        sorted_bed='pending optional public-default policy, not numerical matrix acceptance',release=False)
    save(output/'qualification.json',result)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('root','world','source','native','output'):
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--case',choices=['main','track20','atac'],required=True)
    parser.add_argument('--workers',type=int,choices=[1,4],required=True)
    parser.add_argument('--qualification',action='store_true')
    args=parser.parse_args()
    check_pair(args.root,args.world,args.case,args.workers,args.source,args.native,args.output,args.qualification)


if __name__=='__main__':
    main()
