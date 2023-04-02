# bytefreq-rs 
## Mask Based Data Profiling for Data Quality

bytefreq-rs is a refactor of the original ByteFreq data profiling tool written in awk. 
The original code is found here, if you want full features. https://github.com/minkymorgan/bytefreq

This implementation written in Rust and is very fast.

## Features

- Supports JSON and tabular data formats
- High, low, and Unicode grain masking options
- Frequency distribution of patterns in columns
- Reservoir sampling for example value generation

I highly suggest you pre-parse complex csv using a decent parser, and pass clean pipe delimited values to this program. 
(If there are ragged columns, this will probably error presently)

Usage:
------

To use bytefreq-rs, compile the Rust program and execute it with the desired options:

```
$ cargo build --release

$ ./target/release/bytefreq-rs [OPTIONS]

$ ./target/release/bytefreq-rs --help

Options:
--------

-g, --grain GRAIN
    Sets the grain type for masking:
    - 'H' for high grain
    - 'L' for low grain
    - 'U' for Unicode (default)

-d, --delimiter DELIMITER
    Sets the delimiter used to separate fields in input data (default: '|')

-f, --format FORMAT
    Sets the format of the input data:
    - 'json' for JSON data
    - 'tabular' for tabular data (default)

--help
    Displays the help message with detailed explanations for each option

```
Examples:
---------

1. Process a tabular data file with default options (Unicode grain, '|' delimiter):
```
$ cat testdata/test1.pip | ./target/release/bytefreq-rs
```
2. Process a JSON data file with low grain masking:
```
$ cat testdata/test2.json | ./target/release/bytefreq-rs -f "json" -g "L"
```
3. Process a tabular data file with a custom delimiter and high grain masking:
```
$ cat testdata/test3.tsv | ./target/release/bytefreq-rs -d "\t" -g "H"
```
4. Display the help message:
```
$ ./target/release/bytefreq-rs --help
```
License:
--------

Bytefreq-rs is released under the GNU General Public License v3.0. 
See the LICENSE file for more information.

