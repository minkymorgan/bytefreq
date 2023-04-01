# bytefreq-rs

bytefreq-rs is POC for a refactor of the original ByteFreq data profiling tool. The original awk code is fast, but it needs modernisation. 
The original code is found here, if you want full features. https://github.com/minkymorgan/bytefreq

This refactor is into Rust and so far is very fast, about 4 times faster than using MAWK with the full bytefreq.awk

It is designed to process very large delimited datasets efficiently and provide mask based data profiling statistics.

## Features

- Process large files quickly and efficiently
- Support for tabular file formats:
  - PSV (pipe-separated values) this is the default. 
  - CSV (simple comma separated values)
  - TSV (tab delimited values)
- it is limited to data recived on STDIN 
- Report generation is configurable, for H high or L low grain reports

I highly suggest you parse csv using a decent parser, and pass clean pipe delimited values in. If there are ragged columns, this will probably error presently.

## Getting Started
### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install): Install Rust and Cargo to build and run the project.

### Building

Clone the repository:

```bash
git clone https://github.com/yourusername/bytefreq-rs.git
cd bytefreq-rs
```
Build the project

```
cargo build --release
```

Running

Run ByteFreq-RS on a specific file or set of files:
```
time cat BasicCompanyData-2021-02-01-part6_6_100k.pip | ./target/release/bytefreq-rs --grain "H" >out/test.output.txt 
```

Run tests:
```
cargo test
```
