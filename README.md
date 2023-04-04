# bytefreq-rs 
### Mask Based Data Profiling, for Data Quality Assessment
**Bytefreq-rs** implements a mask based data profiling technique that is one of the most efficient methods for doing data quality assessment on new unknown datasets you receive.

A "Mask" is the output of a function that generalises a string of data into a pattern, the mask, which greatly reduces the cardinality of the original values. This cardinality reduction allows you to inspect vast quantities of data quickly in a field or column, helping you to discover outliers and data quality issues in your dataset. Examples of each pattern help to validate what you can expect when you come to use the data in a use case. **bytefreq-rs** is a refactor of the original bytefreq tool found here: https://github.com/minkymorgan/bytefreq
### Features:
- Produces two report formats: Data Profiling, and Byte Frequency reports 
- Supports both complex nested JSON and Delimited tabular data formats 
- Offers modern masks: "HU: HighGrain Unicode", and "LU: LowGrain Unicode"
- Supports well known ASCII "HighGrain" and "LowGrain" masks 
- Produces human readable Frequency counts of the patterns/masks in your data.
- Reports a true random example of a mask, using Reservoir Sampling. 
- Handles complex json nesting, including unrolling arrays. 
- Byte frequency reports supports Unicode, as well as control characts like LF / CR

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
Data Profiling Report: 20230403 00:55:13
Examined rows: 190493
column                                    count     pattern      example                         
--------------------------------          --------  --------     --------------------------------
col_00007_properties.number               161668    "9-9"        "375-1"                         
col_00007_properties.number               24538     "9"          "8"                             
col_00007_properties.number               2784      "a9-9"       "丙551-1"                        
col_00007_properties.number               1139      "9A"         "89C"                           
col_00007_properties.number               177       "9a-9"       "2334ｲ-1"                       
col_00007_properties.number               155       "9a9-9"      "17第2-1"                        
col_00007_properties.number               17        "a-9"        "又又-1"                          
col_00007_properties.number               12        "a9a9-9"     "ﾛ487第3-1"                      
col_00007_properties.number               3         "a9a-9"      "又729ｲ-1"                       
col_00009_properties.region               190493    "            ""                              
col_00010_properties.street               164816    "a"          "城山町"                           
col_00010_properties.street               19714     "Aa"         "Skálavegur"                    
col_00010_properties.street               2612      "Aa Aa"      "Við Svartá"                    
col_00010_properties.street               1970      "A Aa"       "Á Fløttinum"                   
col_00010_properties.street               672       "Aa a Aa"    "Norðuri í Sundum"              
col_00010_properties.street               387       "Aa Aa a"    "Jónas Broncks gøta"            
col_00010_properties.street               119       "A.A. Aa a"  "R.C. Effersøes gøta"           
col_00010_properties.street               68        "Aa a Aa a"  "Djóna í Geil gøta"             
col_00010_properties.street               61        "Aa A"       "Handan Á"                      
col_00010_properties.street               27        "A Aa Aa"    "Á Eystaru Hellu"               
col_00010_properties.street               25        "Aa A Aa"    "Oman Á Bakka"                  
col_00010_properties.street               13        "A. Aa a"    "C. Pløyens gøta"               
col_00010_properties.street               9         "Aa a a"     "Suðuri í lægd"                 
col_00011_properties.unit                 190493    "            ""                              
col_00012_type                            190493    "Aa"         "Feature"                       
col_00001_geometry.coordinates[1]         190493    9.9          62.0171126                      
col_00002_geometry.type                   190493    "Aa"         "Point"                         
col_00000_geometry.coordinates[0]         164816    9.9          129.826488                      
col_00000_geometry.coordinates[0]         25677     -9.9         -6.724438                       
col_00003_properties.city                 164816    "            ""                              
col_00003_properties.city                 25398     "Aa"         "Hósvík"                        
col_00003_properties.city                 220       "Aa, Aa"     "Nes, Vágur"                    
col_00003_properties.city                 59        "Aa Aa"      "Undir Gøtueiði"                
col_00004_properties.district             190493    "            ""                              
col_00006_properties.id                   190493    "            ""                              
col_00008_properties.postcode             164816    "            ""                              
col_00008_properties.postcode             25677     "9"          "730"                           
```


### License:

Bytefreq-rs is released under the GNU General Public License v3.0. 
See the LICENSE file for more information.

