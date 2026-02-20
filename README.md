# bytefreq  
### Mask Based Data Profiling
**Bytefreq** implements a mask based data profiling technique that is one of the most efficient methods for doing data quality assessment on new unknown datasets you receive.

A "Mask" is the output of a function that generalises a string of data into a pattern, the mask, which greatly reduces the cardinality of the original values. This cardinality reduction allows you to inspect vast quantities of data quickly in a field or column, helping you to discover outliers and data quality issues in your dataset. Examples of each pattern help to validate what you can expect when you come to use the data in a use case. **bytefreq** is a refactor of the original bytefreq tool found here: https://github.com/minkymorgan/bytefreq
### Features:
- Produces two report formats: Data Profiling, and Byte Frequency reports
- Supports both complex nested JSON and delimited tabular data formats (including CSV)
- **Proper CSV parsing** using industry-standard parser - handles quoted fields, embedded commas, and escape sequences
- Native Excel file support (.xlsx, .xls, .xlsb, .ods) with optional feature flag
- Native Apache Parquet file support with nested struct and array handling via optional feature flag
- Offers modern masks: "HU: HighGrain Unicode", and "LU: LowGrain Unicode"
- Supports well known ASCII "HighGrain" and "LowGrain" masks
- Produces human readable frequency counts of the patterns/masks in your data
- Reports a true random example of a mask, using Reservoir Sampling
- Handles complex JSON nesting, including unrolling arrays
- Byte frequency reports support Unicode, plus the non-printable control characters you need for DQ studies, like LF / CR
- Configurable header row selection for files with metadata or multi-line headers

### Author:
**Andrew J Morgan**

This library implements a mask based data profiler technique is described in our book found here https://www.amazon.co.uk/Mastering-Spark-Science-Andrew-Morgan/dp/1785882147 and subsequently improved upon.

Questions can be directed to me at:
minkymorgan at gmail dot com

### License:

Bytefreq is released under the GNU General Public License v3.0.
See the LICENSE file for more information.

------
## Usage:


To use bytefreq, install rust, clone the repo, and compile the Rust program, and check it delivers the help information.

On my latest read, the installation for rust suggests the following, as found here: https://www.rust-lang.org/tools/install
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then continue to get, build, and install the tool:

```
$ git clone git@github.com:minkymorgan/bytefreq.git
$ cd bytefreq
$ cargo clean
$ cargo build --release
$ cargo install --path .
$ bytefreq --help
Bytefreq Data Profiler 1.0
Andrew Morgan <minkymorgan@gmail.com>
A command-line tool to generate data profiling reports based on various masking strategies.

USAGE:
    bytefreq [OPTIONS]

OPTIONS:
    -a, --remove-array-numbers <REMOVE_ARRAY_NUMBERS>
            Remove array numbers when set to true [default: false]

    -d, --delimiter <DELIMITER>
            Sets the delimiter used to separate fields in input tabular data.
            Default: '|' (pipe character) [default: |]

    -e, --enhanced-output
            Output the processed tabular data in JSON format when set to true.

    -E, --flat-enhanced
            Formats the enhanced output in a flattened format

    -f, --format <FORMAT>
            Sets the format of the input data:
            'json' - JSON data (each line should contain a JSON object)
            'tabular' - Tabular data (first line should be the header)
            'excel' - Excel file (.xlsx, .xls, .xlsb, .ods) - requires --excel-path
            'parquet' - Parquet file (.parquet) - requires --parquet-path [default: tabular]

    -g, --grain <GRAIN>
            Sets the grain type for masking:
            'H' - High grain (A for uppercase letters, a for lowercase letters, 9 for digits)
            'L' - Low grain (repeated pattern characters will be compressed to one)
            'U' - Unicode (uses Unicode general categories for masking
            'LU'- Low grain Unicode (repeated pattern classes compressed to one
            ) [default: LU]

    -h, --help
            Print help information

    -p, --pathdepth <PATHDEPTH>
            Sets the depth for JSON paths (applicable for JSON data only). [default: 9]

    -r, --report <REPORT>
            Sets the type of report to generate:
            'DQ' - Data Quality (default)
            'CP' - Character Profiling [default: DQ]

    -V, --version
            Print version information
```




### Usage Examples:
Here are some examples to run, using the testdata provided. These should help you to understand the range of options and tools.
Note you may want to install jq if you haven't, it helps when formatting json outputs: https://github.com/jqlang/jq
Typically on a mac, you might try: 
```
brew install jq
```

#### Examples

1. Process a tabular data file with default options (Unicode grain, '|' delimiter):
```
$ cat testdata/test1.pip | ./target/release/bytefreq
```

2. Process a JSON data file with low grain masking:
```
$ cat testdata/test2.json | ./target/release/bytefreq -f "json" -g "L"
```

3. Process a tabular data file with a custom delimiter and high grain masking:
```
$ cat testdata/test3.tsv | ./target/release/bytefreq -d "\t" -g "H"
```

4. Process a CSV file (proper CSV parsing handles quoted fields with embedded commas):
```
$ cat yourfile.csv | ./target/release/bytefreq -d ","
```

### Processing CSV Files

**Bytefreq uses proper CSV parsing** for all delimited data, which correctly handles:
- Quoted fields containing the delimiter (e.g., `"Smith, John"` won't be split)
- Escaped quotes within quoted fields
- Multi-line quoted fields (though each complete record should be on one line)

This means you can process Excel-exported CSV files directly without pre-processing:

```bash
# Process a comma-delimited CSV file
cat yourfile.csv | ./target/release/bytefreq -d ","

# The tool automatically handles fields like:
# "Last Name, First Name","Address, including city","Department"
# Which will be correctly parsed as 3 fields, not 5
```

**Note:** The CSV parser follows RFC 4180 standards. If you encounter issues with non-standard CSV formats, you may need to use external tools like `csvkit` to normalize the data first.

### Processing Microsoft Excel Files

**Bytefreq now supports native Excel file reading!** Build with the `--features excel` flag to enable support for .xlsx, .xls, .xlsb, and .ods formats.

#### Building with Excel Support:

```bash
cargo build --release --features excel
```

#### Using Excel Files:

Process an Excel file directly using the `-f excel` flag:

```bash
# Process the first sheet (default, header on row 0)
./target/release/bytefreq -f excel --excel-path yourfile.xlsx

# Process a specific sheet by index (0-based)
./target/release/bytefreq -f excel --excel-path yourfile.xlsx --sheet 1

# Process a specific sheet by name
./target/release/bytefreq -f excel --excel-path yourfile.xls --sheet-name "Data Sheet"

# If headers are not on row 0 (e.g., file has metadata/labels first)
./target/release/bytefreq -f excel --excel-path yourfile.xlsx --sheet-name "Data" --header-row 1
```

**Important Notes:**
- Excel files often contain multiple sheets - metadata, data, and reference tables
- Use `--sheet` or `--sheet-name` to select the correct data sheet
- By default, row 0 is treated as the header. Use `--header-row N` if headers are on a different row
- Common scenario: Files with a title/label in row 0 and actual headers in row 1 should use `--header-row 1`
- All Excel data is converted internally to pipe-delimited format before processing

#### Alternative: Command-Line Converters

If you cannot build with the Excel feature, you can still process Excel files using external conversion tools:

Using `xlsx2csv` (install via: `pip install xlsx2csv`):
```bash
xlsx2csv -d "|" yourfile.xlsx | bytefreq
```

Using Python pandas:
```bash
python -c "import pandas; pandas.read_excel('yourfile.xlsx').to_csv('/dev/stdout', sep='|', index=False)" | bytefreq
```

### Processing Apache Parquet Files

**Bytefreq supports native Parquet file reading!** Build with the `--features parquet` flag to enable support for .parquet files. Parquet data is processed through the JSON pipeline, which means nested structs and arrays are handled natively with dot-notation paths.

#### Building with Parquet Support:

```bash
cargo build --release --features parquet

# Or with both Excel and Parquet support:
cargo build --release --features excel,parquet
```

#### Using Parquet Files:

```bash
# Profile a Parquet file
./target/release/bytefreq -f parquet --parquet-path yourfile.parquet

# With low-grain Unicode masking
./target/release/bytefreq -f parquet --parquet-path yourfile.parquet -g LU

# Enhanced JSON output with data quality rules
./target/release/bytefreq -f parquet --parquet-path yourfile.parquet -e
```

#### Nested Data Handling:

Parquet files with nested struct columns produce dot-notation paths, just like JSON:

```
column                          	count   	pattern 	example
--------------------------------	--------	--------	--------------------------------
col_00000_id                    	3       	9       	 2
col_00001_user.address.city     	2       	"A"     	 "NYC"
col_00001_user.address.city     	1       	"Aa"    	 "Chicago"
col_00002_user.address.zip      	3       	"9"     	 "10001"
col_00003_user.name             	3       	"Aa"    	 "Charlie"
```

Parquet list/array columns produce indexed paths (use `-a` to collapse indices):

```bash
# With array indices: tags[0], tags[1], scores[0], scores[1]
./target/release/bytefreq -f parquet --parquet-path data.parquet

# Collapsed: tags[], scores[]
./target/release/bytefreq -f parquet --parquet-path data.parquet -a
```

**Supported Parquet types:** Int8/16/32/64, UInt8/16/32/64, Float32/64, Utf8, Boolean, Null, Struct (nested objects), List/LargeList (arrays), Timestamp (ISO8601), Date32/64, LargeUtf8.

### Example Output:

```
cat testdata/*geo* | bytefreq -f json -a true -e | jq . | head -100




## here's an example with formatted output using the column command:
cat testdata/source.geojson* | bytefreq --format "json" -g "LU" |grep -v hash | column -t -s $'\t'
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

```
cat testdata/BasicCompanyData* | bytefreq -e | jq . | head -100

{
  "Accounts.AccountCategory": {
    "HU": "AAAAAAA",
    "LU": "A",
    "Rules": {
      "string_length": 7
    },
    "raw": "DORMANT"
  },
  "Accounts.AccountRefDay": {
    "HU": "99",
    "LU": "9",
    "Rules": {
      "is_numeric": true,
      "string_length": 2
    },
    "raw": "30"
  },
  "Accounts.AccountRefMonth": {
    "HU": "9",
    "LU": "9",
    "Rules": {
      "is_numeric": true,
      "string_length": 1
    },
    "raw": "9"
  },
  "Accounts.LastMadeUpDate": {
    "HU": "99_99_9999",
    "LU": "9_9_9",
    "Rules": {
      "std_date": "2019-09-30",
      "string_length": 10
    },
    "raw": "30/09/2019"
  },
  "Accounts.NextDueDate": {
    "HU": "99_99_9999",
    "LU": "9_9_9",
    "Rules": {
      "std_date": "2021-06-30",
      "string_length": 10
    },
    "raw": "30/06/2021"
  },
  "CompanyCategory": {
    "HU": "Aaaaaaa Aaaaaaa Aaaaaaa",
    "LU": "Aa Aa Aa",
    "Rules": {
      "string_length": 23
    },
    "raw": "Private Limited Company"

...


// for json data, the enhance works with a flatten option. two examples show the difference:

cat testdata/*geo* | bytefreq -f json -e | jq . | head -50
{
  "geometry": {
    "coordinates": [
      -6.8533376,
      62.2774559
    ],
    "type": {
      "HU": "Aaaaa",
      "LU": "Aa",
      "Rules": {
        "string_length": 5
      },
      "raw": "Point"
    }
  },
  "properties": {
    "city": {
      "HU": "Aaaaaaaaaaaaa",
      "LU": "Aa",
      "Rules": {
        "string_length": 13
      },
      "raw": "Oyndarfjørður"
    },
    "district": {
      "HU": "",
      "LU": "_",
      "Rules": {
        "string_length": 0
      },
      "raw": ""
    },
    "hash": {
      "HU": "99a999a9a9999a99",
      "LU": "9a9a9a9a9",
      "Rules": {
        "string_length": 16
      },
      "raw": "79f233a8e3912d05"
    },
    "id": {
      "HU": "",
      "LU": "_",
      "Rules": {
        "string_length": 0
      },
      "raw": ""
    },
    "number": {
      "HU": "9",

// and the second example is:

{
  "geometry.coordinates.0": -6.8533376,
  "geometry.coordinates.1": 62.2774559,
  "geometry.type.HU": "Aaaaa",
  "geometry.type.LU": "Aa",
  "geometry.type.Rules.string_length": 5,
  "geometry.type.raw": "Point",
  "properties.city.HU": "Aaaaaaaaaaaaa",
  "properties.city.LU": "Aa",
  "properties.city.Rules.string_length": 13,
  "properties.city.raw": "Oyndarfjørður",
  "properties.district.HU": "",
  "properties.district.LU": "_",
  "properties.district.Rules.string_length": 0,
  "properties.district.raw": "",
  "properties.hash.HU": "99a999a9a9999a99",
  "properties.hash.LU": "9a9a9a9a9",
  "properties.hash.Rules.string_length": 16,
  "properties.hash.raw": "79f233a8e3912d05",
  "properties.id.HU": "",
  "properties.id.LU": "_",
  "properties.id.Rules.string_length": 0,
  "properties.id.raw": "",
  "properties.number.HU": "9",
  "properties.number.LU": "9",
  "properties.number.Rules.is_numeric": true,
  "properties.number.Rules.string_length": 1,
  "properties.number.raw": "4",
  "properties.postcode.HU": "999",
  "properties.postcode.LU": "9",
  "properties.postcode.Rules.is_numeric": true,
  "properties.postcode.Rules.string_length": 3,
  "properties.postcode.raw": "690",
  "properties.region.HU": "",
  "properties.region.LU": "_",
  "properties.region.Rules.string_length": 0,
  "properties.region.raw": "",
  "properties.street.HU": "Aaaaaaaaaaaa",
  "properties.street.LU": "Aa",
  "properties.street.Rules.string_length": 12,
  "properties.street.raw": "Sýnarbrekkan",
  "properties.unit.HU": "",
  "properties.unit.LU": "_",
  "properties.unit.Rules.string_length": 0,
  "properties.unit.raw": "",
  "type.HU": "Aaaaaaa",
  "type.LU": "Aa",
  "type.Rules.string_length": 7,
  "type.raw": "Feature"
}


# ByteFreq(uency) reports using the CP option. This is the only solution I know of that does this properly


cat testdata/*geo* | bytefreq -f json -r CP 

char                           count     description      name
--------                       --------  ---------------  ---------------
\u{a}                          190493    \n               LF - Line Feed
\u{20}                         7630                       SPACE
\u{22}                         9524650   \"               QUOTATION MARK
\u{2c}                         2286136   ,                COMMA
\u{2d}                         190493    -                HYPHEN-MINUS
\u{2e}                         381237    .                FULL STOP
\u{30}                         487056    0                DIGIT ZERO
\u{31}                         872758    1                DIGIT ONE
\u{32}                         735372    2                DIGIT TWO
\u{33}                         733375    3                DIGIT THREE
\u{34}                         448046    4                DIGIT FOUR
\u{35}                         457303    5                DIGIT FIVE
\u{36}                         528031    6                DIGIT SIX
\u{37}                         523090    7                DIGIT SEVEN
\u{38}                         525587    8                DIGIT EIGHT
\u{39}                         597565    9                DIGIT NINE
\u{3a}                         2666902   :                COLON
\u{41}                         1460      A                LATIN CAPITAL LETTER A
\u{42}                         3245      B                LATIN CAPITAL LETTER B
\u{43}                         218       C                LATIN CAPITAL LETTER C
\u{44}                         781       D                LATIN CAPITAL LETTER D
\u{45}                         1180      E                LATIN CAPITAL LETTER E
\u{46}                         192936    F                LATIN CAPITAL LETTER F
\u{47}                         3213      G                LATIN CAPITAL LETTER G
\u{48}                         5964      H                LATIN CAPITAL LETTER H
\u{49}                         218       I                LATIN CAPITAL LETTER I
\u{4a}                         512       J                LATIN CAPITAL LETTER J
\u{4b}                         4764      K                LATIN CAPITAL LETTER K
\u{4c}                         2034      L                LATIN CAPITAL LETTER L
\u{4d}                         2089      M                LATIN CAPITAL LETTER M
\u{4e}                         2061      N                LATIN CAPITAL LETTER N
\u{4f}                         1043      O                LATIN CAPITAL LETTER O
\u{50}                         191055    P                LATIN CAPITAL LETTER P
\u{52}                         1281      R                LATIN CAPITAL LETTER R
\u{53}                         8095      S                LATIN CAPITAL LETTER S
\u{54}                         9206      T                LATIN CAPITAL LETTER T
\u{55}                         1210      U                LATIN CAPITAL LETTER U
\u{56}                         4171      V                LATIN CAPITAL LETTER V
\u{57}                         10        W                LATIN CAPITAL LETTER W
\u{59}                         97        Y                LATIN CAPITAL LETTER Y
\u{5a}                         10        Z                LATIN CAPITAL LETTER Z
\u{5b}                         190493    [                LEFT SQUARE BRACKET
\u{5d}                         190493    ]                RIGHT SQUARE BRACKET
\u{61}                         810981    a                LATIN SMALL LETTER A
\u{62}                         384574    b                LATIN SMALL LETTER B
\u{63}                         951886    c                LATIN SMALL LETTER C
\u{64}                         958604    d                LATIN SMALL LETTER D
\u{65}                         2878541   e                LATIN SMALL LETTER E
\u{66}                         193116    f                LATIN SMALL LETTER F
\u{67}                         409765    g                LATIN SMALL LETTER G
\u{68}                         387698    h                LATIN SMALL LETTER H
\u{69}                         1733743   i                LATIN SMALL LETTER I
\u{6a}                         4992      j                LATIN SMALL LETTER J
\u{6b}                         17048     k                LATIN SMALL LETTER K
\u{6c}                         15121     l                LATIN SMALL LETTER L
\u{6d}                         385533    m                LATIN SMALL LETTER M
\u{6e}                         975946    n                LATIN SMALL LETTER N
\u{6f}                         1533200   o                LATIN SMALL LETTER O
\u{70}                         953599    p                LATIN SMALL LETTER P
\u{72}                         1767157   r                LATIN SMALL LETTER R
\u{73}                         1163670   s                LATIN SMALL LETTER S
\u{74}                         2682559   t                LATIN SMALL LETTER T
\u{75}                         596650    u                LATIN SMALL LETTER U
\u{76}                         27839     v                LATIN SMALL LETTER V
\u{79}                         769198    y                LATIN SMALL LETTER Y
\u{7a}                         14        z                LATIN SMALL LETTER Z
\u{7b}                         571479    {                LEFT CURLY BRACKET
\u{7d}                         571479    }                RIGHT CURLY BRACKET
\u{c1}                         1548      Á                LATIN CAPITAL LETTER A WITH ACUTE
\u{c6}                         99        Æ                LATIN CAPITAL LETTER AE
\u{cd}                         892       Í                LATIN CAPITAL LETTER I WITH ACUTE
\u{d3}                         169       Ó                LATIN CAPITAL LETTER O WITH ACUTE
\u{d8}                         130       Ø                LATIN CAPITAL LETTER O WITH STROKE
\u{da}                         192       Ú                LATIN CAPITAL LETTER U WITH ACUTE
\u{e1}                         7245      á                LATIN SMALL LETTER A WITH ACUTE
\u{e6}                         660       æ                LATIN SMALL LETTER AE
\u{ed}                         7792      í                LATIN SMALL LETTER I WITH ACUTE
\u{f0}                         11168     ð                LATIN SMALL LETTER ETH
\u{f3}                         8125      ó                LATIN SMALL LETTER O WITH ACUTE
\u{f8}                         11961     ø                LATIN SMALL LETTER O WITH STROKE
\u{fa}                         1813      ú                LATIN SMALL LETTER U WITH ACUTE
\u{fc}                         14        ü                LATIN SMALL LETTER U WITH DIAERESIS
\u{fd}                         539       ý                LATIN SMALL LETTER Y WITH ACUTE
\u{3005}                       2133      々               IDEOGRAPHIC ITERATION MARK
\u{304b}                       262       か               HIRAGANA LETTER KA
\u{304c}                       681       が               HIRAGANA LETTER GA
\u{304d}                       262       き               HIRAGANA LETTER KI
\u{304f}                       80        く               HIRAGANA LETTER KU
\u{3053}                       22        こ               HIRAGANA LETTER KO
\u{3055}                       102       さ               HIRAGANA LETTER SA
\u{3058}                       136       じ               HIRAGANA LETTER ZI
\u{3064}                       204       つ               HIRAGANA LETTER TU
\u{3068}                       104       と               HIRAGANA LETTER TO
\u{306a}                       216       な               HIRAGANA LETTER NA
\u{306e}                       4134      の               HIRAGANA LETTER NO
\u{3073}                       112       び               HIRAGANA LETTER BI
\u{307e}                       112       ま               HIRAGANA LETTER MA
\u{307f}                       156       み               HIRAGANA LETTER MI
\u{3082}                       52        も               HIRAGANA LETTER MO
\u{3089}                       80        ら               HIRAGANA LETTER RA
\u{30a4}                       256       イ               KATAKANA LETTER I
\u{30a6}                       13        ウ               KATAKANA LETTER U
\u{30a8}                       27        エ               KATAKANA LETTER E
\u{30b1}                       1045      ケ               KATAKANA LETTER KE
\u{30b5}                       89        サ               KATAKANA LETTER SA
\u{30b7}                       89        シ               KATAKANA LETTER SI
\u{30b9}                       26        ス               KATAKANA LETTER SU
\u{30c0}                       152       ダ               KATAKANA LETTER DA
\u{30c1}                       1         チ               KATAKANA LETTER TI
\u{30c3}                       21        ッ               KATAKANA LETTER SMALL TU
\u{30c4}                       280       ツ               KATAKANA LETTER TU
\u{30c6}                       13        テ               KATAKANA LETTER TE
\u{30c8}                       27        ト               KATAKANA LETTER TO
\u{30c9}                       241       ド               KATAKANA LETTER DO
\u{30ca}                       5         ナ               KATAKANA LETTER NA
\u{30cc}                       429       ヌ               KATAKANA LETTER NU
\u{30cd}                       27        ネ               KATAKANA LETTER NE
\u{30ce}                       2675      ノ               KATAKANA LETTER NO
\u{30cf}                       13        ハ               KATAKANA LETTER HA
\u{30dc}                       13        ボ               KATAKANA LETTER BO
\u{30de}                       5         マ               KATAKANA LETTER MA
\u{30df}                       27        ミ               KATAKANA LETTER MI
\u{30e4}                       152       ヤ               KATAKANA LETTER YA
\u{30e9}                       152       ラ               KATAKANA LETTER RA
\u{30ea}                       6         リ               KATAKANA LETTER RI
\u{30ed}                       20        ロ               KATAKANA LETTER RO
\u{30f3}                       192       ン               KATAKANA LETTER N
\u{30f6}                       1         ヶ               KATAKANA LETTER SMALL KE
\u{30fc}                       94        ー               KATAKANA-HIRAGANA PROLONGED SOUND MARK
\u{4e00}                       5566      一               CJK UNIFIED IDEOGRAPH-4E00
\u{4e01}                       18433     丁               CJK UNIFIED IDEOGRAPH-4E01
\u{4e03}                       229       七               CJK UNIFIED IDEOGRAPH-4E03
\u{4e07}                       117       万               CJK UNIFIED IDEOGRAPH-4E07
\u{4e09}                       6214      三               CJK UNIFIED IDEOGRAPH-4E09
\u{4e0a}                       5505      上               CJK UNIFIED IDEOGRAPH-4E0A
\u{4e0b}                       3712      下               CJK UNIFIED IDEOGRAPH-4E0B
\u{4e0e}                       171       与               CJK UNIFIED IDEOGRAPH-4E0E
\u{4e16}                       475       世               CJK UNIFIED IDEOGRAPH-4E16
\u{4e18}                       480       丘               CJK UNIFIED IDEOGRAPH-4E18
\u{4e19}                       1625      丙               CJK UNIFIED IDEOGRAPH-4E19
\u{4e26}                       981       並               CJK UNIFIED IDEOGRAPH-4E26
\u{4e2d}                       5178      中               CJK UNIFIED IDEOGRAPH-4E2D
\u{4e32}                       522       串               CJK UNIFIED IDEOGRAPH-4E32
\u{4e38}                       500       丸               CJK UNIFIED IDEOGRAPH-4E38
\u{4e3b}                       148       主               CJK UNIFIED IDEOGRAPH-4E3B
\u{4e45}                       5508      久               CJK UNIFIED IDEOGRAPH-4E45
\u{4e4b}                       1701      之               CJK UNIFIED IDEOGRAPH-4E4B
\u{4e57}                       6         乗               CJK UNIFIED IDEOGRAPH-4E57
\u{4e59}                       1687      乙               CJK UNIFIED IDEOGRAPH-4E59
\u{4e71}                       58        乱               CJK UNIFIED IDEOGRAPH-4E71
\u{4e7e}                       109       乾               CJK UNIFIED IDEOGRAPH-4E7E
\u{4e80}                       175       亀               CJK UNIFIED IDEOGRAPH-4E80
\u{4e8c}                       7137      二               CJK UNIFIED IDEOGRAPH-4E8C
\u{4e94}                       599       五               CJK UNIFIED IDEOGRAPH-4E94
\u{4e95}                       1570      井               CJK UNIFIED IDEOGRAPH-4E95
\u{4eac}                       225       京               CJK UNIFIED IDEOGRAPH-4EAC
\u{4eba}                       107       人               CJK UNIFIED IDEOGRAPH-4EBA
\u{4ec1}                       91        仁               CJK UNIFIED IDEOGRAPH-4EC1
\u{4eca}                       1291      今               CJK UNIFIED IDEOGRAPH-4ECA
\u{4ecf}                       89        仏               CJK UNIFIED IDEOGRAPH-4ECF
\u{4ed9}                       184       仙               CJK UNIFIED IDEOGRAPH-4ED9
\u{4ee3}                       1185      代               CJK UNIFIED IDEOGRAPH-4EE3
\u{4ee5}                       267       以               CJK UNIFIED IDEOGRAPH-4EE5
\u{4ef2}                       71        仲               CJK UNIFIED IDEOGRAPH-4EF2
\u{4f0a}                       330       伊               CJK UNIFIED IDEOGRAPH-4F0A
\u{4f1a}                       50        会               CJK UNIFIED IDEOGRAPH-4F1A
\u{4f3c}                       136       似               CJK UNIFIED IDEOGRAPH-4F3C
\u{4f4f}                       98        住               CJK UNIFIED IDEOGRAPH-4F4F
\u{4f50}                       3222      佐               CJK UNIFIED IDEOGRAPH-4F50
\u{4fdd}                       1889      保               CJK UNIFIED IDEOGRAPH-4FDD
\u{4ff5}                       171       俵               CJK UNIFIED IDEOGRAPH-4FF5
\u{5009}                       590       倉               CJK UNIFIED IDEOGRAPH-5009
\u{5143}                       1168      元               CJK UNIFIED IDEOGRAPH-5143
\u{5148}                       22        先               CJK UNIFIED IDEOGRAPH-5148
\u{5149}                       115       光               CJK UNIFIED IDEOGRAPH-5149
\u{514d}                       7700      免               CJK UNIFIED IDEOGRAPH-514D
\u{5165}                       108       入               CJK UNIFIED IDEOGRAPH-5165
\u{516b}                       387       八               CJK UNIFIED IDEOGRAPH-516B
\u{516d}                       290       六               CJK UNIFIED IDEOGRAPH-516D
\u{5185}                       3274      内               CJK UNIFIED IDEOGRAPH-5185
\u{5186}                       7         円               CJK UNIFIED IDEOGRAPH-5186
\u{51a8}                       81        冨               CJK UNIFIED IDEOGRAPH-51A8
\u{51b6}                       11        冶               CJK UNIFIED IDEOGRAPH-51B6
\u{51fa}                       991       出               CJK UNIFIED IDEOGRAPH-51FA
\u{5206}                       768       分               CJK UNIFIED IDEOGRAPH-5206
\u{5207}                       291       切               CJK UNIFIED IDEOGRAPH-5207
\u{5208}                       154       刈               CJK UNIFIED IDEOGRAPH-5208
\u{5225}                       26        別               CJK UNIFIED IDEOGRAPH-5225
\u{524d}                       742       前               CJK UNIFIED IDEOGRAPH-524D
\u{526f}                       114       副               CJK UNIFIED IDEOGRAPH-526F
\u{52a0}                       1302      加               CJK UNIFIED IDEOGRAPH-52A0
\u{52a9}                       62        助               CJK UNIFIED IDEOGRAPH-52A9
\u{52d2}                       55        勒               CJK UNIFIED IDEOGRAPH-52D2
\u{52dd}                       139       勝               CJK UNIFIED IDEOGRAPH-52DD
\u{52e2}                       6         勢               CJK UNIFIED IDEOGRAPH-52E2
\u{5316}                       484       化               CJK UNIFIED IDEOGRAPH-5316
\u{5317}                       2329      北               CJK UNIFIED IDEOGRAPH-5317
\u{533a}                       100       区               CJK UNIFIED IDEOGRAPH-533A
\u{5341}                       80        十               CJK UNIFIED IDEOGRAPH-5341
\u{5343}                       1692      千               CJK UNIFIED IDEOGRAPH-5343
\u{5354}                       154       協               CJK UNIFIED IDEOGRAPH-5354
\u{5357}                       1367      南               CJK UNIFIED IDEOGRAPH-5357
\u{535a}                       36        博               CJK UNIFIED IDEOGRAPH-535A
\u{5378}                       66        卸               CJK UNIFIED IDEOGRAPH-5378
\u{539a}                       10        厚               CJK UNIFIED IDEOGRAPH-539A
\u{539f}                       8588      原               CJK UNIFIED IDEOGRAPH-539F
\u{53a8}                       843       厨               CJK UNIFIED IDEOGRAPH-53A8
\u{53b3}                       1625      厳               CJK UNIFIED IDEOGRAPH-53B3
\u{53c8}                       107       又               CJK UNIFIED IDEOGRAPH-53C8
\u{53cd}                       146       反               CJK UNIFIED IDEOGRAPH-53CD
\u{53d6}                       33        取               CJK UNIFIED IDEOGRAPH-53D6
\u{53e3}                       3495      口               CJK UNIFIED IDEOGRAPH-53E3
\u{53e4}                       1529      古               CJK UNIFIED IDEOGRAPH-53E4
\u{53f0}                       1283      台               CJK UNIFIED IDEOGRAPH-53F0
\u{5408}                       3         合               CJK UNIFIED IDEOGRAPH-5408
\u{5409}                       2606      吉               CJK UNIFIED IDEOGRAPH-5409
\u{540c}                       116       同               CJK UNIFIED IDEOGRAPH-540C
\u{540d}                       236       名               CJK UNIFIED IDEOGRAPH-540D
\u{5411}                       433       向               CJK UNIFIED IDEOGRAPH-5411
\u{5439}                       90        吹               CJK UNIFIED IDEOGRAPH-5439
\u{548c}                       1556      和               CJK UNIFIED IDEOGRAPH-548C
\u{5584}                       100       善               CJK UNIFIED IDEOGRAPH-5584
\u{56db}                       2560      四               CJK UNIFIED IDEOGRAPH-56DB
\u{56f2}                       132       囲               CJK UNIFIED IDEOGRAPH-56F2
\u{56fd}                       2218      国               CJK UNIFIED IDEOGRAPH-56FD
\u{5712}                       521       園               CJK UNIFIED IDEOGRAPH-5712
\u{571f}                       1257      土               CJK UNIFIED IDEOGRAPH-571F
\u{5730}                       414       地               CJK UNIFIED IDEOGRAPH-5730
\u{5742}                       1570      坂               CJK UNIFIED IDEOGRAPH-5742
\u{574a}                       201       坊               CJK UNIFIED IDEOGRAPH-574A
\u{576a}                       18        坪               CJK UNIFIED IDEOGRAPH-576A
\u{5782}                       1         垂               CJK UNIFIED IDEOGRAPH-5782
\u{57cb}                       5         埋               CJK UNIFIED IDEOGRAPH-57CB
\u{57ce}                       1426      城               CJK UNIFIED IDEOGRAPH-57CE
\u{5800}                       919       堀               CJK UNIFIED IDEOGRAPH-5800
\u{5802}                       36        堂               CJK UNIFIED IDEOGRAPH-5802
\u{5824}                       141       堤               CJK UNIFIED IDEOGRAPH-5824
\u{5834}                       3143      場               CJK UNIFIED IDEOGRAPH-5834
\u{5854}                       654       塔               CJK UNIFIED IDEOGRAPH-5854
\u{585a}                       291       塚               CJK UNIFIED IDEOGRAPH-585A
\u{5869}                       152       塩               CJK UNIFIED IDEOGRAPH-5869
\u{58eb}                       26        士               CJK UNIFIED IDEOGRAPH-58EB
\u{58f1}                       162       壱               CJK UNIFIED IDEOGRAPH-58F1
\u{591a}                       3236      多               CJK UNIFIED IDEOGRAPH-591A
\u{5927}                       8268      大               CJK UNIFIED IDEOGRAPH-5927
\u{5929}                       1592      天               CJK UNIFIED IDEOGRAPH-5929
\u{592a}                       297       太               CJK UNIFIED IDEOGRAPH-592A
\u{592b}                       12        夫               CJK UNIFIED IDEOGRAPH-592B
\u{592e}                       8         央               CJK UNIFIED IDEOGRAPH-592E
\u{5948}                       198       奈               CJK UNIFIED IDEOGRAPH-5948
\u{5965}                       117       奥               CJK UNIFIED IDEOGRAPH-5965
\u{5973}                       219       女               CJK UNIFIED IDEOGRAPH-5973
\u{5a66}                       12        婦               CJK UNIFIED IDEOGRAPH-5A66
\u{5b09}                       609       嬉               CJK UNIFIED IDEOGRAPH-5B09
\u{5b50}                       970       子               CJK UNIFIED IDEOGRAPH-5B50
\u{5b57}                       2350      字               CJK UNIFIED IDEOGRAPH-5B57
\u{5b87}                       2342      宇               CJK UNIFIED IDEOGRAPH-5B87
\u{5b89}                       159       安               CJK UNIFIED IDEOGRAPH-5B89
\u{5b95}                       206       宕               CJK UNIFIED IDEOGRAPH-5B95
\u{5b97}                       783       宗               CJK UNIFIED IDEOGRAPH-5B97
\u{5b9d}                       44        宝               CJK UNIFIED IDEOGRAPH-5B9D
\u{5ba4}                       71        室               CJK UNIFIED IDEOGRAPH-5BA4
\u{5bae}                       1143      宮               CJK UNIFIED IDEOGRAPH-5BAE
\u{5bb6}                       2783      家               CJK UNIFIED IDEOGRAPH-5BB6
\u{5bbf}                       1148      宿               CJK UNIFIED IDEOGRAPH-5BBF
\u{5bc4}                       63        寄               CJK UNIFIED IDEOGRAPH-5BC4
\u{5bcc}                       1195      富               CJK UNIFIED IDEOGRAPH-5BCC
\u{5bfa}                       1519      寺               CJK UNIFIED IDEOGRAPH-5BFA
\u{5bff}                       96        寿               CJK UNIFIED IDEOGRAPH-5BFF
\u{5c0f}                       10971     小               CJK UNIFIED IDEOGRAPH-5C0F
\u{5c3b}                       294       尻               CJK UNIFIED IDEOGRAPH-5C3B
\u{5c3d}                       31        尽               CJK UNIFIED IDEOGRAPH-5C3D
\u{5c3e}                       6184      尾               CJK UNIFIED IDEOGRAPH-5C3E
\u{5c4b}                       1052      屋               CJK UNIFIED IDEOGRAPH-5C4B
\u{5c71}                       7113      山               CJK UNIFIED IDEOGRAPH-5C71
\u{5c90}                       81        岐               CJK UNIFIED IDEOGRAPH-5C90
\u{5ca1}                       2211      岡               CJK UNIFIED IDEOGRAPH-5CA1
\u{5ca9}                       944       岩               CJK UNIFIED IDEOGRAPH-5CA9
\u{5cb3}                       1449      岳               CJK UNIFIED IDEOGRAPH-5CB3
\u{5cf0}                       502       峰               CJK UNIFIED IDEOGRAPH-5CF0
\u{5cf6}                       3392      島               CJK UNIFIED IDEOGRAPH-5CF6
\u{5d0e}                       6388      崎               CJK UNIFIED IDEOGRAPH-5D0E
\u{5d29}                       367       崩               CJK UNIFIED IDEOGRAPH-5D29
\u{5ddd}                       9957      川               CJK UNIFIED IDEOGRAPH-5DDD
\u{5de5}                       52        工               CJK UNIFIED IDEOGRAPH-5DE5
\u{5de6}                       334       左               CJK UNIFIED IDEOGRAPH-5DE6
\u{5df1}                       1087      己               CJK UNIFIED IDEOGRAPH-5DF1
\u{5dfb}                       169       巻               CJK UNIFIED IDEOGRAPH-5DFB
\u{5e02}                       500       市               CJK UNIFIED IDEOGRAPH-5E02
\u{5e03}                       505       布               CJK UNIFIED IDEOGRAPH-5E03
\u{5e2b}                       150       師               CJK UNIFIED IDEOGRAPH-5E2B
\u{5e38}                       586       常               CJK UNIFIED IDEOGRAPH-5E38
\u{5e3d}                       64        帽               CJK UNIFIED IDEOGRAPH-5E3D
\u{5e61}                       112       幡               CJK UNIFIED IDEOGRAPH-5E61
\u{5e72}                       31        干               CJK UNIFIED IDEOGRAPH-5E72
\u{5e73}                       6054      平               CJK UNIFIED IDEOGRAPH-5E73
\u{5e78}                       125       幸               CJK UNIFIED IDEOGRAPH-5E78
\u{5e83}                       323       広               CJK UNIFIED IDEOGRAPH-5E83
\u{5e84}                       273       庄               CJK UNIFIED IDEOGRAPH-5E84
\u{5e8a}                       537       床               CJK UNIFIED IDEOGRAPH-5E8A
\u{5e95}                       334       底               CJK UNIFIED IDEOGRAPH-5E95
\u{5e9a}                       350       庚               CJK UNIFIED IDEOGRAPH-5E9A
\u{5ea7}                       53        座               CJK UNIFIED IDEOGRAPH-5EA7
\u{5eb5}                       531       庵               CJK UNIFIED IDEOGRAPH-5EB5
\u{5efa}                       36        建               CJK UNIFIED IDEOGRAPH-5EFA
\u{5f01}                       193       弁               CJK UNIFIED IDEOGRAPH-5F01
\u{5f0f}                       453       式               CJK UNIFIED IDEOGRAPH-5F0F
\u{5f15}                       172       引               CJK UNIFIED IDEOGRAPH-5F15
\u{5f25}                       86        弥               CJK UNIFIED IDEOGRAPH-5F25
\u{5f53}                       26        当               CJK UNIFIED IDEOGRAPH-5F53
\u{5f62}                       427       形               CJK UNIFIED IDEOGRAPH-5F62
\u{5f66}                       34        彦               CJK UNIFIED IDEOGRAPH-5F66
\u{5f7c}                       290       彼               CJK UNIFIED IDEOGRAPH-5F7C
\u{5f8c}                       13        後               CJK UNIFIED IDEOGRAPH-5F8C
\u{5fa1}                       945       御               CJK UNIFIED IDEOGRAPH-5FA1
\u{5fb3}                       351       徳               CJK UNIFIED IDEOGRAPH-5FB3
\u{5fc3}                       44        心               CJK UNIFIED IDEOGRAPH-5FC3
\u{5fd7}                       1348      志               CJK UNIFIED IDEOGRAPH-5FD7
\u{6069}                       83        恩               CJK UNIFIED IDEOGRAPH-6069
\u{6075}                       10        恵               CJK UNIFIED IDEOGRAPH-6075
\u{611b}                       206       愛               CJK UNIFIED IDEOGRAPH-611B
\u{6148}                       83        慈               CJK UNIFIED IDEOGRAPH-6148
\u{620a}                       379       戊               CJK UNIFIED IDEOGRAPH-620A
\u{6210}                       3         成               CJK UNIFIED IDEOGRAPH-6210
\u{6238}                       4504      戸               CJK UNIFIED IDEOGRAPH-6238
\u{6247}                       40        扇               CJK UNIFIED IDEOGRAPH-6247
\u{624b}                       1643      手               CJK UNIFIED IDEOGRAPH-624B
\u{624d}                       10        才               CJK UNIFIED IDEOGRAPH-624D
\u{6298}                       999       折               CJK UNIFIED IDEOGRAPH-6298
\u{6307}                       614       指               CJK UNIFIED IDEOGRAPH-6307
\u{6426}                       1         搦               CJK UNIFIED IDEOGRAPH-6426
\u{6469}                       198       摩               CJK UNIFIED IDEOGRAPH-6469
\u{647a}                       71        摺               CJK UNIFIED IDEOGRAPH-647A
\u{6559}                       19        教               CJK UNIFIED IDEOGRAPH-6559
\u{6577}                       546       敷               CJK UNIFIED IDEOGRAPH-6577
\u{6587}                       61        文               CJK UNIFIED IDEOGRAPH-6587
\u{6589}                       193       斉               CJK UNIFIED IDEOGRAPH-6589
\u{65b0}                       2709      新               CJK UNIFIED IDEOGRAPH-65B0
\u{65b9}                       1998      方               CJK UNIFIED IDEOGRAPH-65B9
\u{65e5}                       2806      日               CJK UNIFIED IDEOGRAPH-65E5
\u{65e9}                       527       早               CJK UNIFIED IDEOGRAPH-65E9
\u{65ed}                       55        旭               CJK UNIFIED IDEOGRAPH-65ED
\u{660c}                       100       昌               CJK UNIFIED IDEOGRAPH-660C
\u{660e}                       351       明               CJK UNIFIED IDEOGRAPH-660E
\u{661f}                       640       星               CJK UNIFIED IDEOGRAPH-661F
\u{6625}                       205       春               CJK UNIFIED IDEOGRAPH-6625
\u{662d}                       76        昭               CJK UNIFIED IDEOGRAPH-662D
\u{6642}                       316       時               CJK UNIFIED IDEOGRAPH-6642
\u{666f}                       64        景               CJK UNIFIED IDEOGRAPH-666F
\u{6674}                       96        晴               CJK UNIFIED IDEOGRAPH-6674
\u{66d9}                       42        曙               CJK UNIFIED IDEOGRAPH-66D9
\u{66f2}                       64        曲               CJK UNIFIED IDEOGRAPH-66F2
\u{66fd}                       5         曽               CJK UNIFIED IDEOGRAPH-66FD
\u{66ff}                       21        替               CJK UNIFIED IDEOGRAPH-66FF
\u{6708}                       213       月               CJK UNIFIED IDEOGRAPH-6708
\u{6709}                       3881      有               CJK UNIFIED IDEOGRAPH-6709
\u{6728}                       7481      木               CJK UNIFIED IDEOGRAPH-6728
\u{672b}                       139       末               CJK UNIFIED IDEOGRAPH-672B
\u{672c}                       4490      本               CJK UNIFIED IDEOGRAPH-672C
\u{672d}                       47        札               CJK UNIFIED IDEOGRAPH-672D
\u{6749}                       64        杉               CJK UNIFIED IDEOGRAPH-6749
\u{6751}                       1553      村               CJK UNIFIED IDEOGRAPH-6751
\u{6756}                       1468      杖               CJK UNIFIED IDEOGRAPH-6756
\u{6765}                       2166      来               CJK UNIFIED IDEOGRAPH-6765
\u{676d}                       622       杭               CJK UNIFIED IDEOGRAPH-676D
\u{6771}                       3474      東               CJK UNIFIED IDEOGRAPH-6771
\u{6775}                       290       杵               CJK UNIFIED IDEOGRAPH-6775
\u{677e}                       3451      松               CJK UNIFIED IDEOGRAPH-677E
\u{6797}                       84        林               CJK UNIFIED IDEOGRAPH-6797
\u{679d}                       7         枝               CJK UNIFIED IDEOGRAPH-679D
\u{67cf}                       183       柏               CJK UNIFIED IDEOGRAPH-67CF
\u{67da}                       610       柚               CJK UNIFIED IDEOGRAPH-67DA
\u{67f3}                       120       柳               CJK UNIFIED IDEOGRAPH-67F3
\u{67ff}                       200       柿               CJK UNIFIED IDEOGRAPH-67FF
\u{6804}                       758       栄               CJK UNIFIED IDEOGRAPH-6804
\u{6817}                       210       栗               CJK UNIFIED IDEOGRAPH-6817
\u{6822}                       10        栢               CJK UNIFIED IDEOGRAPH-6822
\u{6839}                       1125      根               CJK UNIFIED IDEOGRAPH-6839
\u{6843}                       102       桃               CJK UNIFIED IDEOGRAPH-6843
\u{6851}                       297       桑               CJK UNIFIED IDEOGRAPH-6851
\u{685c}                       478       桜               CJK UNIFIED IDEOGRAPH-685C
\u{685f}                       41        桟               CJK UNIFIED IDEOGRAPH-685F
\u{6876}                       42        桶               CJK UNIFIED IDEOGRAPH-6876
\u{6881}                       35        梁               CJK UNIFIED IDEOGRAPH-6881
\u{6885}                       101       梅               CJK UNIFIED IDEOGRAPH-6885
\u{68a8}                       59        梨               CJK UNIFIED IDEOGRAPH-68A8
\u{68b6}                       43        梶               CJK UNIFIED IDEOGRAPH-68B6
\u{68da}                       242       棚               CJK UNIFIED IDEOGRAPH-68DA
\u{68ee}                       57        森               CJK UNIFIED IDEOGRAPH-68EE
\u{690d}                       268       植               CJK UNIFIED IDEOGRAPH-690D
\u{690e}                       227       椎               CJK UNIFIED IDEOGRAPH-690E
\u{693f}                       43        椿               CJK UNIFIED IDEOGRAPH-693F
\u{6960}                       82        楠               CJK UNIFIED IDEOGRAPH-6960
\u{6a29}                       574       権               CJK UNIFIED IDEOGRAPH-6A29
\u{6a2a}                       1060      横               CJK UNIFIED IDEOGRAPH-6A2A
\u{6a2b}                       262       樫               CJK UNIFIED IDEOGRAPH-6A2B
\u{6a3a}                       11        樺               CJK UNIFIED IDEOGRAPH-6A3A
\u{6a3d}                       38        樽               CJK UNIFIED IDEOGRAPH-6A3D
\u{6a4b}                       924       橋               CJK UNIFIED IDEOGRAPH-6A4B
\u{6a58}                       59        橘               CJK UNIFIED IDEOGRAPH-6A58
\u{6b4c}                       203       歌               CJK UNIFIED IDEOGRAPH-6B4C
\u{6b63}                       242       正               CJK UNIFIED IDEOGRAPH-6B63
\u{6b66}                       489       武               CJK UNIFIED IDEOGRAPH-6B66
\u{6b73}                       22        歳               CJK UNIFIED IDEOGRAPH-6B73
\u{6bcd}                       205       母               CJK UNIFIED IDEOGRAPH-6BCD
\u{6bd4}                       763       比               CJK UNIFIED IDEOGRAPH-6BD4
\u{6bdb}                       112       毛               CJK UNIFIED IDEOGRAPH-6BDB
\u{6c0f}                       61        氏               CJK UNIFIED IDEOGRAPH-6C0F
\u{6c34}                       1089      水               CJK UNIFIED IDEOGRAPH-6C34
\u{6c38}                       551       永               CJK UNIFIED IDEOGRAPH-6C38
\u{6c5f}                       3873      江               CJK UNIFIED IDEOGRAPH-6C5F
\u{6c60}                       412       池               CJK UNIFIED IDEOGRAPH-6C60
\u{6c72}                       95        汲               CJK UNIFIED IDEOGRAPH-6C72
\u{6c96}                       262       沖               CJK UNIFIED IDEOGRAPH-6C96
\u{6cb3}                       608       河               CJK UNIFIED IDEOGRAPH-6CB3
\u{6cb9}                       122       油               CJK UNIFIED IDEOGRAPH-6CB9
\u{6cc9}                       348       泉               CJK UNIFIED IDEOGRAPH-6CC9
\u{6cca}                       739       泊               CJK UNIFIED IDEOGRAPH-6CCA
\u{6cd5}                       121       法               CJK UNIFIED IDEOGRAPH-6CD5
\u{6d0b}                       40        洋               CJK UNIFIED IDEOGRAPH-6D0B
\u{6d17}                       50        洗               CJK UNIFIED IDEOGRAPH-6D17
\u{6d25}                       7503      津               CJK UNIFIED IDEOGRAPH-6D25
\u{6d5c}                       4009      浜               CJK UNIFIED IDEOGRAPH-6D5C
\u{6d66}                       7784      浦               CJK UNIFIED IDEOGRAPH-6D66
\u{6d6a}                       7         浪               CJK UNIFIED IDEOGRAPH-6D6A
\u{6d77}                       2796      海               CJK UNIFIED IDEOGRAPH-6D77
\u{6d78}                       84        浸               CJK UNIFIED IDEOGRAPH-6D78
\u{6df1}                       1015      深               CJK UNIFIED IDEOGRAPH-6DF1
\u{6df5}                       323       淵               CJK UNIFIED IDEOGRAPH-6DF5
\u{6e05}                       90        清               CJK UNIFIED IDEOGRAPH-6E05
\u{6e15}                       130       渕               CJK UNIFIED IDEOGRAPH-6E15
\u{6e21}                       496       渡               CJK UNIFIED IDEOGRAPH-6E21
\u{6e2f}                       40        港               CJK UNIFIED IDEOGRAPH-6E2F
\u{6e4a}                       1089      湊               CJK UNIFIED IDEOGRAPH-6E4A
\u{6e56}                       46        湖               CJK UNIFIED IDEOGRAPH-6E56
\u{6e6f}                       332       湯               CJK UNIFIED IDEOGRAPH-6E6F
\u{6e80}                       336       満               CJK UNIFIED IDEOGRAPH-6E80
\u{6e9d}                       336       溝               CJK UNIFIED IDEOGRAPH-6E9D
\u{6ed1}                       283       滑               CJK UNIFIED IDEOGRAPH-6ED1
\u{6edd}                       69        滝               CJK UNIFIED IDEOGRAPH-6EDD
\u{6f5c}                       58        潜               CJK UNIFIED IDEOGRAPH-6F5C
\u{6f5f}                       356       潟               CJK UNIFIED IDEOGRAPH-6F5F
\u{6f6e}                       82        潮               CJK UNIFIED IDEOGRAPH-6F6E
\u{702c}                       2841      瀬               CJK UNIFIED IDEOGRAPH-702C
\u{7070}                       6         灰               CJK UNIFIED IDEOGRAPH-7070
\u{7089}                       29        炉               CJK UNIFIED IDEOGRAPH-7089
\u{70ba}                       594       為               CJK UNIFIED IDEOGRAPH-70BA
\u{70cf}                       64        烏               CJK UNIFIED IDEOGRAPH-70CF
\u{7121}                       852       無               CJK UNIFIED IDEOGRAPH-7121
\u{713c}                       1142      焼               CJK UNIFIED IDEOGRAPH-713C
\u{718a}                       394       熊               CJK UNIFIED IDEOGRAPH-718A
\u{7236}                       208       父               CJK UNIFIED IDEOGRAPH-7236
\u{7247}                       554       片               CJK UNIFIED IDEOGRAPH-7247
\u{725f}                       56        牟               CJK UNIFIED IDEOGRAPH-725F
\u{7267}                       1040      牧               CJK UNIFIED IDEOGRAPH-7267
\u{72e9}                       8         狩               CJK UNIFIED IDEOGRAPH-72E9
\u{732a}                       186       猪               CJK UNIFIED IDEOGRAPH-732A
\u{733f}                       38        猿               CJK UNIFIED IDEOGRAPH-733F
\u{7389}                       4         玉               CJK UNIFIED IDEOGRAPH-7389
\u{738b}                       277       王               CJK UNIFIED IDEOGRAPH-738B
\u{7396}                       370       玖               CJK UNIFIED IDEOGRAPH-7396
\u{73fe}                       647       現               CJK UNIFIED IDEOGRAPH-73FE
\u{7434}                       2394      琴               CJK UNIFIED IDEOGRAPH-7434
\u{751f}                       67        生               CJK UNIFIED IDEOGRAPH-751F
\u{7530}                       14796     田               CJK UNIFIED IDEOGRAPH-7530
\u{7532}                       2164      甲               CJK UNIFIED IDEOGRAPH-7532
\u{753a}                       132846    町               CJK UNIFIED IDEOGRAPH-753A
\u{754c}                       67        界               CJK UNIFIED IDEOGRAPH-754C
\u{7551}                       192       畑               CJK UNIFIED IDEOGRAPH-7551
\u{7559}                       594       留               CJK UNIFIED IDEOGRAPH-7559
\u{755d}                       154       畝               CJK UNIFIED IDEOGRAPH-755D
\u{7566}                       40        畦               CJK UNIFIED IDEOGRAPH-7566
\u{767b}                       59        登               CJK UNIFIED IDEOGRAPH-767B
\u{767d}                       1706      白               CJK UNIFIED IDEOGRAPH-767D
\u{767e}                       302       百               CJK UNIFIED IDEOGRAPH-767E
\u{7686}                       427       皆               CJK UNIFIED IDEOGRAPH-7686
\u{76bf}                       135       皿               CJK UNIFIED IDEOGRAPH-76BF
\u{76db}                       1         盛               CJK UNIFIED IDEOGRAPH-76DB
\u{76e4}                       12        盤               CJK UNIFIED IDEOGRAPH-76E4
\u{76ee}                       17446     目               CJK UNIFIED IDEOGRAPH-76EE
\u{76f8}                       1371      相               CJK UNIFIED IDEOGRAPH-76F8
\u{770c}                       7         県               CJK UNIFIED IDEOGRAPH-770C
\u{771f}                       635       真               CJK UNIFIED IDEOGRAPH-771F
\u{77e2}                       522       矢               CJK UNIFIED IDEOGRAPH-77E2
\u{77e5}                       67        知               CJK UNIFIED IDEOGRAPH-77E5
\u{77f3}                       4903      石               CJK UNIFIED IDEOGRAPH-77F3
\u{7802}                       20        砂               CJK UNIFIED IDEOGRAPH-7802
\u{7834}                       1006      破               CJK UNIFIED IDEOGRAPH-7834
\u{78ef}                       191       磯               CJK UNIFIED IDEOGRAPH-78EF
\u{7947}                       236       祇               CJK UNIFIED IDEOGRAPH-7947
\u{795e}                       2230      神               CJK UNIFIED IDEOGRAPH-795E
\u{798f}                       2714      福               CJK UNIFIED IDEOGRAPH-798F
\u{79cb}                       100       秋               CJK UNIFIED IDEOGRAPH-79CB
\u{79e9}                       208       秩               CJK UNIFIED IDEOGRAPH-79E9
\u{7a17}                       294       稗               CJK UNIFIED IDEOGRAPH-7A17
\u{7a32}                       122       稲               CJK UNIFIED IDEOGRAPH-7A32
\u{7acb}                       294       立               CJK UNIFIED IDEOGRAPH-7ACB
\u{7af9}                       1122      竹               CJK UNIFIED IDEOGRAPH-7AF9
\u{7aff}                       323       竿               CJK UNIFIED IDEOGRAPH-7AFF
\u{7b2c}                       168       第               CJK UNIFIED IDEOGRAPH-7B2C
\u{7b51}                       13        筑               CJK UNIFIED IDEOGRAPH-7B51
\u{7b52}                       79        筒               CJK UNIFIED IDEOGRAPH-7B52
\u{7b95}                       2         箕               CJK UNIFIED IDEOGRAPH-7B95
\u{7b99}                       19        箙               CJK UNIFIED IDEOGRAPH-7B99
\u{7bc9}                       62        築               CJK UNIFIED IDEOGRAPH-7BC9
\u{7c60}                       1255      籠               CJK UNIFIED IDEOGRAPH-7C60
\u{7c73}                       11        米               CJK UNIFIED IDEOGRAPH-7C73
\u{7c95}                       29        粕               CJK UNIFIED IDEOGRAPH-7C95
\u{7d3a}                       43        紺               CJK UNIFIED IDEOGRAPH-7D3A
\u{7d44}                       682       組               CJK UNIFIED IDEOGRAPH-7D44
\u{7db2}                       313       網               CJK UNIFIED IDEOGRAPH-7DB2
\u{7dbf}                       184       綿               CJK UNIFIED IDEOGRAPH-7DBF
\u{7dd1}                       106       緑               CJK UNIFIED IDEOGRAPH-7DD1
\u{7dd2}                       1         緒               CJK UNIFIED IDEOGRAPH-7DD2
\u{7f8e}                       95        美               CJK UNIFIED IDEOGRAPH-7F8E
\u{7fbd}                       218       羽               CJK UNIFIED IDEOGRAPH-7FBD
\u{8077}                       88        職               CJK UNIFIED IDEOGRAPH-8077
\u{80a5}                       60        肥               CJK UNIFIED IDEOGRAPH-80A5
\u{8155}                       18        腕               CJK UNIFIED IDEOGRAPH-8155
\u{8208}                       8         興               CJK UNIFIED IDEOGRAPH-8208
\u{821f}                       575       舟               CJK UNIFIED IDEOGRAPH-821F
\u{8239}                       2303      船               CJK UNIFIED IDEOGRAPH-8239
\u{826f}                       3459      良               CJK UNIFIED IDEOGRAPH-826F
\u{8292}                       291       芒               CJK UNIFIED IDEOGRAPH-8292
\u{82b1}                       260       花               CJK UNIFIED IDEOGRAPH-82B1
\u{82b3}                       29        芳               CJK UNIFIED IDEOGRAPH-82B3
\u{82d7}                       376       苗               CJK UNIFIED IDEOGRAPH-82D7
\u{82e5}                       591       若               CJK UNIFIED IDEOGRAPH-82E5
\u{8302}                       785       茂               CJK UNIFIED IDEOGRAPH-8302
\u{8349}                       132       草               CJK UNIFIED IDEOGRAPH-8349
\u{8352}                       1146      荒               CJK UNIFIED IDEOGRAPH-8352
\u{8377}                       39        荷               CJK UNIFIED IDEOGRAPH-8377
\u{837b}                       59        荻               CJK UNIFIED IDEOGRAPH-837B
\u{83c5}                       93        菅               CJK UNIFIED IDEOGRAPH-83C5
\u{83f0}                       42        菰               CJK UNIFIED IDEOGRAPH-83F0
\u{8429}                       597       萩               CJK UNIFIED IDEOGRAPH-8429
\u{8449}                       523       葉               CJK UNIFIED IDEOGRAPH-8449
\u{84b2}                       234       蒲               CJK UNIFIED IDEOGRAPH-84B2
\u{8535}                       281       蔵               CJK UNIFIED IDEOGRAPH-8535
\u{85e4}                       273       藤               CJK UNIFIED IDEOGRAPH-85E4
\u{8679}                       72        虹               CJK UNIFIED IDEOGRAPH-8679
\u{868a}                       630       蚊               CJK UNIFIED IDEOGRAPH-868A
\u{86ce}                       11        蛎               CJK UNIFIED IDEOGRAPH-86CE
\u{86e4}                       38        蛤               CJK UNIFIED IDEOGRAPH-86E4
\u{86ed}                       65        蛭               CJK UNIFIED IDEOGRAPH-86ED
\u{8823}                       286       蠣               CJK UNIFIED IDEOGRAPH-8823
\u{884c}                       243       行               CJK UNIFIED IDEOGRAPH-884C
\u{897f}                       5274      西               CJK UNIFIED IDEOGRAPH-897F
\u{898b}                       5822      見               CJK UNIFIED IDEOGRAPH-898B
\u{899a}                       28        覚               CJK UNIFIED IDEOGRAPH-899A
\u{89aa}                       89        親               CJK UNIFIED IDEOGRAPH-89AA
\u{89d2}                       26        角               CJK UNIFIED IDEOGRAPH-89D2
\u{89e6}                       957       触               CJK UNIFIED IDEOGRAPH-89E6
\u{8a08}                       212       計               CJK UNIFIED IDEOGRAPH-8A08
\u{8a2a}                       866       訪               CJK UNIFIED IDEOGRAPH-8A2A
\u{8abf}                       643       調               CJK UNIFIED IDEOGRAPH-8ABF
\u{8acf}                       866       諏               CJK UNIFIED IDEOGRAPH-8ACF
\u{8c37}                       753       谷               CJK UNIFIED IDEOGRAPH-8C37
\u{8c46}                       487       豆               CJK UNIFIED IDEOGRAPH-8C46
\u{8c4a}                       40        豊               CJK UNIFIED IDEOGRAPH-8C4A
\u{8c9d}                       879       貝               CJK UNIFIED IDEOGRAPH-8C9D
\u{8cc0}                       614       賀               CJK UNIFIED IDEOGRAPH-8CC0
\u{8cd1}                       12        賑               CJK UNIFIED IDEOGRAPH-8CD1
\u{8d64}                       1011      赤               CJK UNIFIED IDEOGRAPH-8D64
\u{8d8a}                       2259      越               CJK UNIFIED IDEOGRAPH-8D8A
\u{8def}                       625       路               CJK UNIFIED IDEOGRAPH-8DEF
\u{8e0a}                       65        踊               CJK UNIFIED IDEOGRAPH-8E0A
\u{8f9b}                       71        辛               CJK UNIFIED IDEOGRAPH-8F9B
\u{8fba}                       408       辺               CJK UNIFIED IDEOGRAPH-8FBA
\u{8fbb}                       31        辻               CJK UNIFIED IDEOGRAPH-8FBB
\u{8fbc}                       76        込               CJK UNIFIED IDEOGRAPH-8FBC
\u{8fce}                       1116      迎               CJK UNIFIED IDEOGRAPH-8FCE
\u{8feb}                       52        迫               CJK UNIFIED IDEOGRAPH-8FEB
\u{901a}                       146       通               CJK UNIFIED IDEOGRAPH-901A
\u{9053}                       931       道               CJK UNIFIED IDEOGRAPH-9053
\u{90ce}                       29        郎               CJK UNIFIED IDEOGRAPH-90CE
\u{90e8}                       711       部               CJK UNIFIED IDEOGRAPH-90E8
\u{90f7}                       19310     郷               CJK UNIFIED IDEOGRAPH-90F7
\u{90fd}                       268       都               CJK UNIFIED IDEOGRAPH-90FD
\u{91cc}                       4016      里               CJK UNIFIED IDEOGRAPH-91CC
\u{91cd}                       1403      重               CJK UNIFIED IDEOGRAPH-91CD
\u{91ce}                       7179      野               CJK UNIFIED IDEOGRAPH-91CE
\u{91d1}                       342       金               CJK UNIFIED IDEOGRAPH-91D1
\u{91dc}                       101       釜               CJK UNIFIED IDEOGRAPH-91DC
\u{91dd}                       2531      針               CJK UNIFIED IDEOGRAPH-91DD
\u{9262}                       190       鉢               CJK UNIFIED IDEOGRAPH-9262
\u{9280}                       7         銀               CJK UNIFIED IDEOGRAPH-9280
\u{9285}                       20        銅               CJK UNIFIED IDEOGRAPH-9285
\u{92ad}                       33        銭               CJK UNIFIED IDEOGRAPH-92AD
\u{9326}                       75        錦               CJK UNIFIED IDEOGRAPH-9326
\u{935b}                       11        鍛               CJK UNIFIED IDEOGRAPH-935B
\u{938c}                       29        鎌               CJK UNIFIED IDEOGRAPH-938C
\u{93e1}                       503       鏡               CJK UNIFIED IDEOGRAPH-93E1
\u{9577}                       2137      長               CJK UNIFIED IDEOGRAPH-9577
\u{9580}                       298       門               CJK UNIFIED IDEOGRAPH-9580
\u{9593}                       1211      間               CJK UNIFIED IDEOGRAPH-9593
\u{95a2}                       38        関               CJK UNIFIED IDEOGRAPH-95A2
\u{963f}                       155       阿               CJK UNIFIED IDEOGRAPH-963F
\u{9663}                       586       陣               CJK UNIFIED IDEOGRAPH-9663
\u{9670}                       153       陰               CJK UNIFIED IDEOGRAPH-9670
\u{9678}                       162       陸               CJK UNIFIED IDEOGRAPH-9678
\u{967d}                       45        陽               CJK UNIFIED IDEOGRAPH-967D
\u{96c4}                       14        雄               CJK UNIFIED IDEOGRAPH-96C4
\u{96f2}                       291       雲               CJK UNIFIED IDEOGRAPH-96F2
\u{970a}                       100       霊               CJK UNIFIED IDEOGRAPH-970A
\u{9752}                       887       青               CJK UNIFIED IDEOGRAPH-9752
\u{9762}                       140       面               CJK UNIFIED IDEOGRAPH-9762
\u{97f3}                       353       音               CJK UNIFIED IDEOGRAPH-97F3
\u{9808}                       1781      須               CJK UNIFIED IDEOGRAPH-9808
\u{982d}                       161       頭               CJK UNIFIED IDEOGRAPH-982D
\u{98a8}                       978       風               CJK UNIFIED IDEOGRAPH-98A8
\u{98ef}                       355       飯               CJK UNIFIED IDEOGRAPH-98EF
\u{98fd}                       29        飽               CJK UNIFIED IDEOGRAPH-98FD
\u{9928}                       20        館               CJK UNIFIED IDEOGRAPH-9928
\u{9996}                       251       首               CJK UNIFIED IDEOGRAPH-9996
\u{9999}                       713       香               CJK UNIFIED IDEOGRAPH-9999
\u{99ac}                       778       馬               CJK UNIFIED IDEOGRAPH-99AC
\u{99c4}                       132       駄               CJK UNIFIED IDEOGRAPH-99C4
\u{9ad8}                       4322      高               CJK UNIFIED IDEOGRAPH-9AD8
\u{9aea}                       729       髪               CJK UNIFIED IDEOGRAPH-9AEA
\u{9b3c}                       197       鬼               CJK UNIFIED IDEOGRAPH-9B3C
\u{9b41}                       22        魁               CJK UNIFIED IDEOGRAPH-9B41
\u{9b5a}                       79        魚               CJK UNIFIED IDEOGRAPH-9B5A
\u{9bdb}                       146       鯛               CJK UNIFIED IDEOGRAPH-9BDB
\u{9ce5}                       49        鳥               CJK UNIFIED IDEOGRAPH-9CE5
\u{9cf4}                       354       鳴               CJK UNIFIED IDEOGRAPH-9CF4
\u{9d28}                       53        鴨               CJK UNIFIED IDEOGRAPH-9D28
\u{9d5c}                       143       鵜               CJK UNIFIED IDEOGRAPH-9D5C
\u{9db4}                       89        鶴               CJK UNIFIED IDEOGRAPH-9DB4
\u{9df2}                       213       鷲               CJK UNIFIED IDEOGRAPH-9DF2
\u{9e7f}                       1906      鹿               CJK UNIFIED IDEOGRAPH-9E7F
\u{9ea6}                       37        麦               CJK UNIFIED IDEOGRAPH-9EA6
\u{9eb9}                       6         麹               CJK UNIFIED IDEOGRAPH-9EB9
\u{9ed2}                       2009      黒               CJK UNIFIED IDEOGRAPH-9ED2
\u{9f8d}                       214       龍               CJK UNIFIED IDEOGRAPH-9F8D
\u{ff72}                       165       ｲ                HALFWIDTH KATAKANA LETTER I
\u{ff84}                       1         ﾄ                HALFWIDTH KATAKANA LETTER TO
\u{ff86}                       7         ﾆ                HALFWIDTH KATAKANA LETTER NI
\u{ff8a}                       20        ﾊ                HALFWIDTH KATAKANA LETTER HA
\u{ff8d}                       1         ﾍ                HALFWIDTH KATAKANA LETTER HE
\u{ff8e}                       3         ﾎ                HALFWIDTH KATAKANA LETTER HO
\u{ff9b}                       181       ﾛ                HALFWIDTH KATAKANA LETTER RO
--------END OF REPORT--------

```
