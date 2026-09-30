# somen

[![crates.io](https://img.shields.io/crates/v/somen.svg)](https://crates.io/crates/somen)
[![Docs.rs](https://docs.rs/somen/badge.svg)](https://docs.rs/somen)
[![CI status](https://github.com/chusj-pigu/somen/actions/workflows/ci.yml/badge.svg)](https://github.com/chusj-pigu/somen/actions/workflows/ci.yml)

**somen** attempts to provide specification-compliant (when applicable)
implementations of libraries for handling various bioinformatics file formats.
It currently supports BAM 1.6, BCF 2.2, BED, BGZF, CRAM 3.0/3.1, CSI, FASTA,
FASTQ, GFF3, GTF 2.2, htsget 1.3, POD5, refget 2.0, SAM 1.6, tabix, and VCF 4.3/4.4.

## Usage

somen's API is still considered experimental.

somen includes parsers for multiple file formats. For convenience, the
top-level `somen` crate exposes the existing noodles parser crates as
[features]. For example, to work with the BAM format, enable the `bam` feature.

```sh
cargo add somen --features bam
```

Each enabled feature can then be imported by its re-exported name, e.g.,

```rust
use somen::bam;
```

[crates.io]: https://crates.io/
[features]: https://doc.rust-lang.org/cargo/reference/features.html

POD5 is available separately as `somen-pod5` and is not a `somen` feature.

### Feature flags

Individual crates may have optional features that can be enabled using feature
flags.

  * `async`: Enables asynchronous I/O with [Tokio]. (BAM, BCF, BGZF, CRAM, CSI,
    FASTA, FASTQ, GFF, SAM, tabix, and VCF)
  * `libdeflate`: Use [libdeflate] to encode and decode DEFLATE streams. (BGZF
    and CRAM)

[Tokio]: https://tokio.rs/
[libdeflate]: https://github.com/ebiggers/libdeflate

## Examples

Each crate may have its own examples directory, and all examples are runnable
as an application. After cloning the repository, run `cargo run --release
--example` for a list of available examples. Use the example name as the option
argument and append program arguments to the command, e.g.,

```bash
cargo run --release --example bam_write > sample.bam
cargo run --release --example bam_read_header sample.bam
```
