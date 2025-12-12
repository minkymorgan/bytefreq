# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

**bytefreq** is a Rust-based data profiling tool that uses mask-based pattern analysis to quickly assess data quality on unknown datasets. It generalizes string data into patterns (masks) to reduce cardinality, making it efficient to identify outliers and data quality issues at scale. The tool supports both JSON and tabular (delimited) data formats and provides two main report types: Data Quality profiling (DQ) and Character/Byte Frequency profiling (CP).

## Build and Development Commands

### Building the project
```bash
cargo clean
cargo build --release

# Build with Excel support (adds ~500KB to binary)
cargo build --release --features excel
```

### Installing locally
```bash
cargo install --path .

# Install with Excel support
cargo install --path . --features excel
```

### Running the tool
The binary is named `bytefreq` and reads from stdin:
```bash
cat testdata/test.pip | ./target/release/bytefreq
cat testdata/example.json | ./target/release/bytefreq -f json
```

### Testing with example data
Test data is located in the `testdata/` directory. Examples from README:
```bash
# Tabular data with default options
cat testdata/test.pip | ./target/release/bytefreq

# JSON with low grain masking
cat testdata/example.json | ./target/release/bytefreq -f json -g L

# Character profiling report
cat testdata/example.json | ./target/release/bytefreq -f json -r CP

# Excel file processing (requires --features excel build)
./target/release/bytefreq -f excel --excel-path testdata/Illegal_Dumping_Incidents.xls --sheet 1
```

### Running tests
```bash
cargo test
```

### Building documentation
```bash
cargo doc --open
```

## Code Architecture

### Core Masking System

The heart of bytefreq is its masking engine, which transforms raw data into patterns:

- **High Grain (H/HU)**: Maps characters to pattern types - 'A' for uppercase, 'a' for lowercase, '9' for digits. The 'HU' variant handles Unicode using `unic::ucd::GeneralCategory`.
- **Low Grain (L/LU)**: Like high grain but compresses repeated pattern characters to one. "AAA123" becomes "A9" in low grain.
- **Identity**: Used internally for Rules fields to preserve actual values.

The masking functions are in `src/main.rs`:
- `high_grain_mask()` - ASCII-only high grain
- `high_grain_unicode_mask()` - Unicode-aware high grain using character categories
- `low_grain_mask()` - Compresses repeated pattern characters
- `mask_value()` - Dispatcher that selects the appropriate masking strategy

### Data Processing Pipeline

**JSON Processing (`process_json_value`, `process_json_line`):**
- Recursively traverses JSON structures (objects and arrays)
- Respects `pathdepth` parameter to limit nesting depth
- Can optionally remove array index numbers from paths (`remove_array_numbers`)
- Builds frequency maps and uses reservoir sampling for examples
- Paths are dot-separated (e.g., `geometry.coordinates[0]`)

**Tabular Processing:**
- First line is always treated as header
- Fields split by delimiter (default: pipe `|`)
- Handles ragged data by creating `RaggedErr{N}` columns dynamically
- Tracks field count per line in `field_count_map`

**Enhanced Output Mode (`-e` and `-E` flags):**
- Transforms each field into a JSON object containing: `raw`, `HU`, `LU`, and `Rules`
- `Rules` contains data quality assertions from `src/rules/assertions.rs`
- `-E` additionally flattens the nested JSON structure for easier parsing
- Uses Rayon for parallel processing of fields

**Excel File Processing (with `excel` feature flag):**
- Native support for .xlsx, .xls, .xlsb, and .ods formats via the `calamine` crate
- Located in `src/excel.rs` with conditional compilation (`#[cfg(feature = "excel")]`)
- Reads entire workbook into memory, converts selected sheet to `Vec<Vec<String>>`
- Cell type conversion handles: Int, Float, String, Bool, DateTime, Error, Empty
- Floats with no fractional part displayed as integers (e.g., 42.0 → "42")
- Excel rows converted to pipe-delimited strings before entering main processing loop
- Sheet selection via `--sheet` (index, 0-based) or `--sheet-name` parameters
- Excel processing shares the same tabular data pipeline after conversion

### Excel Module Architecture

The Excel reader (`src/excel.rs`) provides:

**Key Functions:**
- `ExcelReader::read_sheet_by_index()` - Read by zero-based sheet index
- `ExcelReader::read_sheet_by_name()` - Read by sheet name string
- `ExcelReader::get_sheet_names()` - List all sheet names in workbook
- `ExcelReader::read_first_sheet()` - Convenience method for sheet 0

**Implementation Details:**
- Uses `calamine::open_workbook_auto()` to auto-detect Excel format
- `cell_to_string()` converts all calamine Data types to String
- Dummy implementation when feature not enabled returns helpful error message
- Unit tests verify cell type conversion logic (integers, floats, strings, booleans, empty)

**Integration with Main Loop (`src/main.rs`):**
- Lines 706-732: Excel file reading before parallel processing
- Lines 734-752: Sequential header processing to avoid race conditions
- Lines 789-872: Parallel processing of data rows (skips header at index 0)
- Overflow protection at lines 804-809 handles ragged data gracefully

### Rules and Assertions System

Located in `src/rules/`:

- **`assertions.rs`**: Contains assertion functions that detect data patterns (dates, countries, postal codes, numeric checks, etc.). The main entry point is `execute_assertions()` which returns a JSON object of detected properties.
- **`enhancer.rs`**: Thin wrapper that calls `execute_assertions()` with field name and mask data.

Assertions include:
- String length tracking
- Numeric type detection (`is_numeric`)
- Date parsing and standardization (`std_date`, `is_sensible_dob`)
- Country name to ISO3 code conversion (with UK subdivision support)
- Postal code pattern matching for European countries

### Caching

`src/cache.rs` and lazy_static globals in `main.rs`:
- `HANDLE_COUNTRY_NAME_CACHE`: Caches country name variations (England -> GBR/GB-ENG)
- `COUNTRY_NAME_TO_ISO3_CACHE`: Caches country name to ISO3 code lookups
- Uses `RwLock<HashMap>` for thread-safe caching

### Parallel Processing

The codebase uses Rayon extensively:
- Main thread pool configured to use 22 cores (line 549 in `main.rs`)
- All input lines are read into memory then processed with `par_iter()`
- Field processing in enhanced mode uses `par_iter_mut()`
- Shared state protected by `Arc<Mutex<>>` wrappers

**Important**: The current architecture reads all input into memory before processing. This is fast for medium datasets but could be a constraint for very large files.

### Report Generation

**Data Quality Report (DQ - default):**
- Groups values by their masked pattern
- Shows frequency counts for each pattern
- Provides one random example per pattern (via reservoir sampling)
- Output sorted by column index, then by pattern frequency
- Column names prefixed with `col_{index:05}_` in output

**Character Profiling Report (CP):**
- Counts individual character/byte frequencies
- Includes Unicode character names via `unicode_names2` crate
- Special handling for ASCII control characters with descriptive names
- Output shows hex representation, escaped form, count, and character name

### Key Data Structures

- `frequency_maps`: `Vec<HashMap<String, usize>>` - One HashMap per column mapping masked patterns to counts
- `example_maps`: `Vec<HashMap<String, String>>` - One HashMap per column mapping masked patterns to example values
- `column_names`: `HashMap<String, usize>` - Maps column names to their index in the frequency/example maps
- `field_count_map`: `HashMap<usize, usize>` - Tracks how many rows have each field count (for ragged data detection)

## Important Implementation Details

### Reservoir Sampling
The tool uses reservoir sampling (lines 183-186, 773-777) to maintain random examples without storing all values. Each pattern keeps exactly one example, updated probabilistically as new values arrive.

### String Truncation
The `truncate_string()` function (line 525) truncates examples by word boundaries, controlled by the `-l/--maxlen` parameter (default: 20 characters).

### Thread Safety
When modifying code that touches shared state (`frequency_maps`, `example_maps`, `column_names`, etc.), remember these are wrapped in `Arc<Mutex<>>`. Always acquire locks in consistent order to avoid deadlocks.

### Unicode Handling
The tool properly handles Unicode throughout:
- Uses `.chars()` instead of byte indexing
- Leverages `unic` crate for character categorization
- String length assertions count characters, not bytes

### Path Depth Control
For deeply nested JSON, the `pathdepth` parameter prevents excessive memory usage by limiting how deep the recursion goes. Default is 9 levels.

## Configuration Notes

The tool expects:
- Tabular data to have a header row
- JSON data to be newline-delimited (one JSON object per line)
- Input via stdin (pipe data in)

The delimiter for tabular data defaults to `|` but can be changed with `-d`.

## Common Development Patterns

When adding new assertion rules:
1. Add the detection function to `src/rules/assertions.rs`
2. Call it from `execute_assertions()`
3. Return results as JSON fields that will appear under "Rules" in enhanced output

When modifying masking behavior:
1. Core masking logic is in the `mask_value()` function and its helpers
2. Grain types are: "H" (high), "L" (low), "HU" (high unicode), "LU" (low unicode)
3. Identity masking is used for paths containing ".Rules." to preserve actual values
