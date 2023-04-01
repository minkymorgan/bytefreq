# bytefreq-rs

bytefreq-rs is a modern, ultra-fast implementation of the ByteFreq data profiling tool, built in Rust. 

This is the first MVP to get the work started. It's a little limited, but works, and fast.

It is designed to process very large pipe delimited  datasets efficiently and provide mask based data profiling statistics.
It only works for pipe delimited files, and later it may be updated to expand this. .

## Features

- Process large files quickly and efficiently
- Support for tabular file formats:
  - PSV (pipe-separated values)
- it is limited to data recived on STDIN 
- Report generation with byte frequency statistics
- Dotted notation for nested JSON data paths

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
cargo run --release -- -i "/path/to/input/files/*"
```

Run tests:
```
cargo test
```
