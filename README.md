# bytefreq-rs 
### Mask Based Data Profiling, for Data Quality Assessment
**Bytefreq-rs** implements a mask based data profiling technique is one of the most efficient methods for doing data quality assessment on new unknown datasets you receive.

A "Mask" is the output of a function that generalises a string of data into a pattern, the mask, which greatly reduces the cardinality of the original values. This cardinality reduction allows you to inspect vast quantities of data quickly in a field or column, helping you to discover outliers and data quality issues in your dataset. Examples of each pattern help to validate what you can expect when you come to use the data in a use case. **bytefreq-rs** is a refactor of the original bytefreq tool found here: https://github.com/minkymorgan/bytefreq
### Features:
- Supports both JSON and Delimited tabular data formats 
- Offers modern masks: "Unicode HighGrain", and "Unicode LowGrain"
- Supports well known ASCII "HighGrain" and "LowGrain" masks 
- Produces human readable Frequency counts of the patterns/masks in your data.
- Reports a true random example of a mask, using Reservoir Sampling. 
- Handles complex json nesting, including unrolling arrays

I highly suggest you pre-parse complex csv using a decent parser, and pass clean pipe delimited values to this program. Also - this program expects a header for tabular data. (note: If there are ragged columns, this will probably error presently)

------
## Usage:


To use bytefreq-rs, install rust, clone the repo, and compile the Rust program, and check it delivers the help information.

```
$ cargo clean
$ cargo build --release
$ ./target/release/bytefreq-rs --help

Bytefreq Data Profiler 1.0

Andrew Morgan <minkymorgan@gmail.com>
A command-line tool to generate data profiling reports based on various masking strategies.

USAGE:
    bytefreq-rs [OPTIONS]

OPTIONS:
    -d, --delimiter <DELIMITER>    Sets the delimiter used to separate fields in input tabular data.
                                   Default: '|' (pipe character) [default: |]
    -f, --format <FORMAT>          Sets the format of the input data:
                                   'json' - JSON data (each line should contain a JSON object)
                                   'tabular' - Tabular data (first line should be the header)
                                   [default: tabular]
    -g, --grain <GRAIN>            Sets the grain type for masking:
                                   'H' - High grain (A for uppercase letters, a for lowercase
                                   letters, 9 for digits)
                                   'L' - Low grain (repeated pattern characters will be compressed
                                   to one)
                                   'U' - Unicode (uses Unicode general categories for masking
                                   'LU'- Low grain Unicode (repeated pattern classes compressed to
                                   one
                                   ) [default: LU]
    -h, --help                     Print help information
    -V, --version                  Print version information
```
### Usage Examples:

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
### Example Output:

```
cat testdata/source.geojson* | ./target/release/bytefreq-rs --format "json" -g "LU" |grep -v hash | column -t -s $'\t'
Data Profiling Report: 20230403 00:36:03
Examined rows: 190493
column                                    count     pattern      example                         
--------------------------------          --------  --------     --------------------------
col_00012_type                            190493    _Aa_         "Feature"                       
col_00000_geometry.coordinates[0]         164816    9_9          129.813483                      
col_00000_geometry.coordinates[0]         25677     _9_9         -6.5248824                      
col_00007_properties.number               161668    _9_9_        "1625-1"                        
col_00007_properties.number               24538     _9_          "34"                            
col_00007_properties.number               2784      _a9_9_       "ﾛ663-1"                        
col_00007_properties.number               1139      _9A_         "45A"                           
col_00007_properties.number               177       _9a_9_       "136ｲ-1"                        
col_00007_properties.number               155       _9a9_9_      "440第5-1"                       
col_00007_properties.number               17        _a_9_        "又-1"                           
col_00007_properties.number               12        _a9a9_9_     "ﾛ487第3-1"                      
col_00007_properties.number               3         _a9a_9_      "又1176ｲ-1"                      
col_00004_properties.district             190493    _            ""                              
col_00009_properties.region               190493    _            ""                              
col_00003_properties.city                 164816    _            ""                              
col_00003_properties.city                 25398     _Aa_         "Glyvrar"                       
col_00003_properties.city                 220       _Aa_ Aa_     "Nes, Vágur"                    
col_00003_properties.city                 59        _Aa Aa_      "Innan Glyvur"                  
col_00001_geometry.coordinates[1]         190493    9_9          32.838522                       
col_00006_properties.id                   190493    _            ""                              
col_00008_properties.postcode             164816    _            ""                              
col_00008_properties.postcode             25677     _9_          "740"                           
col_00002_geometry.type                   190493    _Aa_         "Point"                         
col_00010_properties.street               164816    _a_          "新馬場町"                          
col_00010_properties.street               19714     _Aa_         "Heiðagøta"                     
col_00010_properties.street               2612      _Aa Aa_      "Millum Húsa"                   
col_00010_properties.street               1970      _A Aa_       "Í Húsgarði"                    
col_00010_properties.street               672       _Aa a Aa_    "Norðuri í Bø"                  
col_00010_properties.street               387       _Aa Aa a_    "Niels Finsens gøta"            
col_00010_properties.street               119       _A_A_ Aa a_  "A.C. Evensens gøta"            
col_00010_properties.street               68        _Aa a Aa a_  "Djóna í Geil gøta"             
col_00010_properties.street               61        _Aa A_       "Uttan Á"                       
col_00010_properties.street               27        _A Aa Aa_    "Á Eystaru Hellu"               
col_00010_properties.street               25        _Aa A Aa_    "Oman Í Fjøru"                  
col_00010_properties.street               13        _A_ Aa a_    "C. Pløyens gøta"               
col_00010_properties.street               9         _Aa a a_     "Erlings jals gøta"             
col_00011_properties.unit                 190493    _            ""                       
```


### License:

Bytefreq-rs is released under the GNU General Public License v3.0. 
See the LICENSE file for more information.

