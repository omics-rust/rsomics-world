# rsomics-bamio

Layer-A primitive: a parallel-BGZF BAM reader shared by the `rsomics-bam-*`
tool family.

`samtools` inflates BGZF on a single thread by default. A single-threaded
pure-Rust reader (zlib-rs) loses to htslib's libdeflate inner loop, so reading
BAM through a worker pool is what puts the rsomics BAM tools ahead of
`samtools` default invocations on multi-core hosts.

```rust
use rsomics_bamio::open_parallel;

let mut reader = open_parallel(path)?;
let header = reader.read_header()?;
for result in reader.records() {
    let record = result?;
    // ...
}
```

License: MIT OR Apache-2.0.
