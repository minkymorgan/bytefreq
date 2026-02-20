# How to Profile Parquet Files with Bytefreq

## Quick Start

### 1. Build with Parquet support

```bash
cargo build --release --features parquet
```

Or install locally:

```bash
cargo install --path . --features parquet
```

### 2. Profile a Parquet file

```bash
./target/release/bytefreq -f parquet --parquet-path yourfile.parquet
```

## Examples

### Flat Parquet files

Profile a simple columnar Parquet file:

```bash
$ ./target/release/bytefreq -f parquet --parquet-path testdata/sample_flat.parquet

Data Profiling Report: 20260220 01:55:25
Examined rows: 0

column                          	count   	pattern 	example
--------------------------------	--------	--------	--------------------------------
col_00000_active	5       	a       	 false
col_00001_id	5       	9       	 3
col_00002_name	5       	"Aa"    	 "Diana"
col_00003_score	5       	9.9     	 92.3
```

### Nested struct columns

Parquet files with nested structs automatically produce dot-notation paths:

```bash
$ ./target/release/bytefreq -f parquet --parquet-path testdata/sample_nested.parquet

column                          	count   	pattern 	example
--------------------------------	--------	--------	--------------------------------
col_00000_id	3       	9       	 2
col_00001_user.address.city	2       	"A"     	 "NYC"
col_00001_user.address.city	1       	"Aa"    	 "Chicago"
col_00002_user.address.zip	3       	"9"     	 "10001"
col_00003_user.name	3       	"Aa"    	 "Charlie"
```

This is the same path format used for JSON data (`user.address.city`), so all existing JSON features work with Parquet nested data.

### Array/List columns

Parquet list columns produce indexed array paths:

```bash
$ ./target/release/bytefreq -f parquet --parquet-path testdata/sample_arrays.parquet

column                          	count   	pattern 	example
--------------------------------	--------	--------	--------------------------------
col_00000_id	3       	9       	 1
col_00001_scores[0]	3       	9       	 88
col_00002_scores[1]	2       	9       	 87
col_00003_tags[0]	3       	"a"     	 "python"
col_00004_tags[1]	3       	"a"     	 "data"
col_00005_scores[2]	1       	9       	 79
col_00006_tags[2]	1       	"a"     	 "api"
```

Use `-a` to collapse array indices for aggregate analysis:

```bash
$ ./target/release/bytefreq -f parquet --parquet-path testdata/sample_arrays.parquet -a

column                          	count   	pattern 	example
--------------------------------	--------	--------	--------------------------------
col_00000_id	3       	9       	 2
col_00001_scores[]	6       	9       	 88
col_00002_tags[]	7       	"a"     	 "python"
```

### Masking options

All masking grain options work with Parquet:

```bash
# High grain (individual character patterns)
./target/release/bytefreq -f parquet --parquet-path data.parquet -g H

# Low grain (compressed patterns)
./target/release/bytefreq -f parquet --parquet-path data.parquet -g L

# High grain Unicode (default)
./target/release/bytefreq -f parquet --parquet-path data.parquet -g HU

# Low grain Unicode
./target/release/bytefreq -f parquet --parquet-path data.parquet -g LU
```

### Enhanced output with data quality rules

The enhanced output mode (`-e`) produces JSON with masking and data quality assertions:

```bash
$ ./target/release/bytefreq -f parquet --parquet-path testdata/sample_flat.parquet -e | jq . | head -20

{
  "active": true,
  "id": {
    "HU": "9",
    "LU": "9",
    "Rules": {
      "is_numeric": true,
      "string_length": 1
    },
    "raw": 1
  },
  "name": {
    "HU": "Aaaaa",
    "LU": "Aa",
    "Rules": {
      "string_length": 5
    },
    "raw": "Alice"
  }
}
```

Use `-E` for flattened enhanced output.

### Character profiling

Byte frequency analysis works on Parquet data too:

```bash
./target/release/bytefreq -f parquet --parquet-path data.parquet -r CP
```

## Architecture Note

Parquet files are converted internally to JSON lines before processing. This means Parquet data flows through the same JSON processing pipeline as `--format json`, inheriting all features:

- Dot-notation nested paths
- Array index handling (`-a` flag)
- Path depth limiting (`-p` flag)
- Enhanced output with rules (`-e` / `-E`)
- All masking grains

## Supported Parquet Types

| Parquet/Arrow Type | JSON Representation |
|---|---|
| Int8, Int16, Int32, Int64 | Number |
| UInt8, UInt16, UInt32, UInt64 | Number |
| Float32, Float64 | Number (NaN/Inf become null) |
| Utf8, LargeUtf8 | String |
| Boolean | Boolean |
| Null | null |
| Struct | Nested JSON object |

### Note on timestamp handling

Parquet timestamps are converted to ISO8601 strings (e.g., `2024-01-15T10:30:00Z`) with full range support — any valid Unix epoch timestamp will be converted correctly.

However, when using enhanced output (`-e`), the **Rules/assertions engine** applies a separate Unix timestamp detection heuristic on numeric fields. This heuristic range-validates to **2000-01-01 through 2100-01-01** to avoid false positives on arbitrary integers. This means:

- Parquet timestamp columns (typed as Timestamp) are always converted correctly regardless of date range
- Numeric fields that happen to contain Unix epochs outside 2000-2100 will not be auto-detected as timestamps by the Rules engine
- Historical dates stored as typed Parquet Timestamps are unaffected — the range limit only applies to the heuristic detection of untyped numeric fields
| List, LargeList | JSON array |
| Timestamp (all units) | ISO8601 string |
| Date32, Date64 | Date string (YYYY-MM-DD) |
| Unsupported types | String: `<unsupported: TypeName>` |

## Building with Multiple Features

You can combine Parquet with other optional features:

```bash
# Parquet + Excel
cargo build --release --features parquet,excel

# Parquet only
cargo build --release --features parquet

# Everything
cargo build --release --features parquet,excel
```
