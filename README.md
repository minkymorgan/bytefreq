# bytefreq-rs

bytefreq-rs is POC for a rust implementation of the ByteFreq data profiling tool, built in Rust. 
The original code is found here, if you want full features. https://github.com/minkymorgan/bytefreq

This initital implementation has limited user options, but works very fast, about 4 times faster than using MAWK with the full bytefreq.awk

It is designed to process very large pipe delimited  datasets efficiently and provide mask based data profiling statistics.
It only works for pipe delimited files, and later it may be updated to expand this.

## Features

- Process large files quickly and efficiently
- Support for tabular file formats:
  - PSV (pipe-separated values)
- it is limited to data recived on STDIN 
- Report generation
- User options for high and low grain reporting

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
