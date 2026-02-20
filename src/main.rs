extern crate rayon;
//extern crate lazy_static;
use lazy_static::lazy_static;
use chrono::Local;
use clap::{App, Arg};
use rand::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use unic::ucd::GeneralCategory as Category;
use unicode_names2;
use serde_json::json;
use bytefreq::rules::enhancer::process_data;
use bytefreq::excel::ExcelReader;
use bytefreq::parquet::ParquetReader;
use rayon::prelude::*;
use csv::ReaderBuilder;

use std::sync::{Arc, Mutex};
use std::sync::RwLock;
//use flatten_json_object::Flattener;



lazy_static! {
    pub static ref HANDLE_COUNTRY_NAME_CACHE: RwLock<HashMap<String, Option<(String, String)>>> =
        RwLock::new(HashMap::new());
    pub static ref COUNTRY_NAME_TO_ISO3_CACHE: RwLock<HashMap<String, Option<String>>> =
        RwLock::new(HashMap::new());
}



pub fn identity_mask(value: &str) -> String {
    value.to_string()
}

// this is a highgrain Mask that works for unicode data!
fn high_grain_unicode_mask(c: char) -> char {
    match c {
        '0'..='9' => '9',
        'a'..='z' => 'a',
        'A'..='Z' => 'A',
        c if c.is_ascii_punctuation() && (c == '"' || c == '-' || c == '.' || c == ',') => c,
        c if c.is_whitespace() => ' ',
        _ => {
            let cat = Category::of(c);

            match cat {
                Category::UppercaseLetter => 'A',
                Category::LowercaseLetter => 'a',
                Category::TitlecaseLetter => 'A',
                Category::OtherLetter => 'a',
                Category::ModifierLetter => 'a',
                Category::DecimalNumber => '9',
                Category::LetterNumber => '9',
                Category::OtherNumber => '9',
                Category::SpaceSeparator => ' ',
                Category::LineSeparator => ' ',
                Category::ParagraphSeparator => ' ',
                _ => '_',
            }
        }
    }
}

fn high_grain_mask(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'a'..='z' => 'a',
            'A'..='Z' => 'A',
            '0'..='9' => '9',
            _ => c,
        })
        .collect()
}

fn low_grain_mask(value: &str) -> String {
    let high_grain = high_grain_mask(value);
    let mut output = String::new();
    let mut last_char = None;

    for c in high_grain.chars() {
        if last_char != Some(c) {
            output.push(c);
            last_char = Some(c);
        }
    }
    if output.is_empty() {
        "_".to_string()
    } else {
        output
    }
}

fn mask_value(value: &str, grain: &str, field_name: &str) -> String {
    if field_name.contains(".Rules.") {
        identity_mask(value)
    } else {
        match grain {
            "H" => high_grain_mask(value),
            "L" => low_grain_mask(value),
            "HU" => value.chars().map(|c| high_grain_unicode_mask(c)).collect(),
            "LU" => low_grain_mask(
                &value
                    .chars()
                    .map(|c| high_grain_unicode_mask(c))
                    .collect::<String>(),
            ),
            _u => value.chars().map(|c| high_grain_unicode_mask(c)).collect(),
        }
    }
}

fn process_json_value(
    value: &Value,
    frequency_maps: &mut Vec<HashMap<String, usize>>,
    example_maps: &mut Vec<HashMap<String, String>>,
    grain: &str,
    prefix: String,
    column_names: &mut HashMap<String, usize>,
    remove_array_numbers: bool,
    pathdepth: usize,
    current_depth: usize,
) {
    match value {
        Value::Object(map) => {
            if current_depth < pathdepth {
                for (key, value) in map.iter() {
                    let full_key = if prefix.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    process_json_value(
                        value,
                        frequency_maps,
                        example_maps,
                        grain,
                        full_key,
                        column_names,
                        remove_array_numbers,
                        pathdepth + 1,
                        current_depth,
                    );
                }
            }
        }
        Value::Array(values) => {
            for (idx, value) in values.iter().enumerate() {
                let full_key = if remove_array_numbers {
                    format!("{}[]", prefix)
                } else {
                    format!("{}[{}]", prefix, idx)
                };
                process_json_value(
                    value,
                    frequency_maps,
                    example_maps,
                    grain,
                    full_key,
                    column_names,
                    remove_array_numbers,
                    pathdepth + 1,
                    current_depth,
                );
            }
        }
        _ => {
            let value_str = value.to_string();
            let masked_value = mask_value(&value_str, grain, &prefix);
            let idx = column_names.entry(prefix.clone()).or_insert_with(|| {
                let new_idx = frequency_maps.len();
                frequency_maps.push(HashMap::new());
                example_maps.push(HashMap::new());
                new_idx
            });

            let count = frequency_maps[*idx]
                .entry(masked_value.clone())
                .or_insert(0);
            *count += 1;

            // Reservoir sampling
            let mut rng = thread_rng();
            if rng.gen::<f64>() < 1.0 / (*count as f64) {
                example_maps[*idx].insert(masked_value.clone(), value_str);
            }
        }
    }
}

// Enhanced for Performance using multithreading via rayon
// Function to process a tabular line and convert it into an enhanced JSON object
fn process_tabular_line_as_json(processed_fields: &Vec<(String, String)>) -> serde_json::Value {
    let json_line: std::collections::HashMap<String, serde_json::Value> = processed_fields
        .par_iter()
        .map(|(column_name, value)| {
            let hu_masked_value = mask_value(value, "HU", column_name);
            let lu_masked_value = mask_value(value, "LU", column_name);

            let data = json!({
                "raw": value,
                "LU": lu_masked_value,
                "HU": hu_masked_value
            });

            let assertions = process_data(&column_name, &data);

            let enhanced_value = json!({
                "raw": value,
                "HU": hu_masked_value,
                "LU": lu_masked_value,
                "Rules": assertions
            });

            (column_name.clone(), enhanced_value)
        })
        .collect::<std::collections::HashMap<String, serde_json::Value>>();

    serde_json::Value::Object(json_line.into_iter().collect())
}


/// Extract array from paginated JSON and convert to NDJSON
/// Returns Ok(ndjson_string) if extraction successful, Err(original_input) otherwise
fn extract_json_array(input: &str, field_name: Option<&str>) -> Result<String, String> {
    // Try to parse as single JSON object
    let parsed: Value = serde_json::from_str(input).map_err(|_| input.to_string())?;

    match parsed {
        // Top-level array: convert directly to NDJSON
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err(input.to_string());
            }
            let ndjson = arr.iter()
                .map(|item| serde_json::to_string(item).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\n");
            eprintln!("✓ Extracted {} records from top-level array", arr.len());
            Ok(ndjson)
        },
        // Object with array field: extract the array
        Value::Object(obj) => {
            let array_field = if let Some(field) = field_name {
                if field == "auto" {
                    // Auto-detect: prioritize common field names
                    let common_fields = ["items", "data", "results", "records", "rows"];
                    let array_fields: Vec<&String> = obj.keys()
                        .filter(|k| obj.get(*k).map(|v| v.is_array()).unwrap_or(false))
                        .collect();

                    if array_fields.is_empty() {
                        eprintln!("✗ No array fields found in JSON object");
                        return Err(input.to_string());
                    }

                    // Find common field name or use first array field
                    array_fields.iter()
                        .find(|f| common_fields.contains(&f.as_str()))
                        .or_else(|| array_fields.first())
                        .map(|s| s.as_str())
                } else {
                    Some(field)
                }
            } else {
                return Err(input.to_string());
            };

            if let Some(field) = array_field {
                if let Some(Value::Array(arr)) = obj.get(field) {
                    if arr.is_empty() {
                        eprintln!("✗ Array field '{}' is empty", field);
                        return Err(input.to_string());
                    }
                    let ndjson = arr.iter()
                        .map(|item| serde_json::to_string(item).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join("\n");
                    eprintln!("✓ Extracted {} records from field '{}'", arr.len(), field);
                    Ok(ndjson)
                } else {
                    eprintln!("✗ Field '{}' is not an array or doesn't exist", field);
                    Err(input.to_string())
                }
            } else {
                Err(input.to_string())
            }
        },
        _ => Err(input.to_string())
    }
}

fn process_json_line(
    line: &str,
    frequency_maps: &mut Vec<HashMap<String, usize>>,
    example_maps: &mut Vec<HashMap<String, String>>,
    grain: &str,
    column_names: &mut HashMap<String, usize>,
    pathdepth: usize,
    remove_array_numbers: bool,
) {
    if let Ok(json_value) = serde_json::from_str::<Value>(line) {
        process_json_value(
            &json_value,
            frequency_maps,
            example_maps,
            grain,
            String::new(),
            column_names,
            remove_array_numbers,
            pathdepth,
            0,
        );
    }
}

fn init_control_character_descriptions() -> HashMap<char, &'static str> {
    let mut ref_map = HashMap::new();
    ref_map.insert('\u{0000}', "NUL - Null char");
    ref_map.insert('\u{0001}', "SOH - Start of Heading");
    ref_map.insert('\u{0002}', "STX - Start of Text");
    ref_map.insert('\u{0003}', "ETX - End of Text");
    ref_map.insert('\u{0004}', "EOT - End of Transmission");
    ref_map.insert('\u{0005}', "ENQ - Enquiry");
    ref_map.insert('\u{0006}', "ACK - Acknowledgment");
    ref_map.insert('\u{0007}', "BEL - Bell");
    ref_map.insert('\u{0008}', "BS - Back Space");
    ref_map.insert('\u{0009}', "HT - Horizontal Tab");
    ref_map.insert('\u{000A}', "LF - Line Feed");
    ref_map.insert('\u{000B}', "VT - Vertical Tab");
    ref_map.insert('\u{000C}', "FF - Form Feed");
    ref_map.insert('\u{000D}', "CR - Carriage Return");
    ref_map.insert('\u{000E}', "SO - Shift Out / X-On");
    ref_map.insert('\u{000F}', "SI - Shift In / X-Off");
    ref_map.insert('\u{0010}', "DLE - Data Line Escape");
    ref_map.insert('\u{0011}', "DC1 - Device Control 1 (oft. XON)");
    ref_map.insert('\u{0012}', "DC2 - Device Control 2");
    ref_map.insert('\u{0013}', "DC3 - Device Control 3 (oft. XOFF)");
    ref_map.insert('\u{0014}', "DC4 - Device Control 4");
    ref_map.insert('\u{0015}', "NAK - Negative Acknowledgement");
    ref_map.insert('\u{0016}', "SYN - Synchronous Idle");
    ref_map.insert('\u{0017}', "ETB - End of Transmit Block");
    ref_map.insert('\u{0018}', "CAN - Cancel");
    ref_map.insert('\u{0019}', "EM - End of Medium");
    ref_map.insert('\u{001A}', "SUB - Substitute");
    ref_map.insert('\u{001B}', "ESC - Escape");
    ref_map.insert('\u{001C}', "FS - File Separator");
    ref_map.insert('\u{001D}', "GS - Group Separator");
    ref_map.insert('\u{001E}', "RS - Record Separator");
    ref_map.insert('\u{001F}', "US - Unit Separator");
    ref_map.insert(
        '\u{008A}',
        "LINE TABULATION SET * Deprecated from Unicode 3.2, 2002",
    );
    ref_map.insert('\u{0090}', "ERROR - Undefined CTRL Character.");
    ref_map.insert('\u{009A}', "LATIN CAPITAL S WITH CARON");
    ref_map.insert('\u{FDD0}', "Non-character code point");
    ref_map.insert('\u{FDD1}', "Non-character code point");
    ref_map.insert('\u{FDD2}', "Non-character code point");
    ref_map.insert('\u{FDD3}', "Non-character code point");
    ref_map.insert('\u{FDD4}', "Non-character code point");
    ref_map.insert('\u{FDD5}', "Non-character code point");
    ref_map.insert('\u{FDD6}', "Non-character code point");
    ref_map.insert('\u{FDD7}', "Non-character code point");
    ref_map.insert('\u{FDD8}', "Non-character code point");
    ref_map.insert('\u{FDD9}', "Non-character code point");
    ref_map.insert('\u{FDDA}', "Non-character code point");
    ref_map.insert('\u{FDDB}', "Non-character code point");
    ref_map.insert('\u{FDDC}', "Non-character code point");
    ref_map.insert('\u{FDDD}', "Non-character code point");
    ref_map.insert('\u{FDDE}', "Non-character code point");
    ref_map.insert('\u{FDDF}', "Non-character code point");
    ref_map.insert('\u{FDE0}', "Non-character code point");
    ref_map.insert('\u{FDE1}', "Non-character code point");
    ref_map.insert('\u{FDE2}', "Non-character code point");
    ref_map.insert('\u{FDE3}', "Non-character code point");
    ref_map.insert('\u{FDE4}', "Non-character code point");
    ref_map.insert('\u{FDE5}', "Non-character code point");
    ref_map.insert('\u{FDE6}', "Non-character code point");
    ref_map.insert('\u{FDE7}', "Non-character code point");
    ref_map.insert('\u{FDE8}', "Non-character code point");
    ref_map.insert('\u{FDE9}', "Non-character code point");
    ref_map.insert('\u{FDEA}', "Non-character code point");
    ref_map.insert('\u{FDEB}', "Non-character code point");
    ref_map.insert('\u{FDEC}', "Non-character code point");
    ref_map.insert('\u{FDED}', "Non-character code point");
    ref_map.insert('\u{FDEE}', "Non-character code point");
    ref_map.insert('\u{FDEF}', "Non-character code point");
    ref_map.insert('\u{FFFA}', "Undefined Control Character");
    ref_map.insert('\u{FFFB}', "Undefined Control Character");
    ref_map.insert('\u{FFFC}', "Undefined Control Character");
    ref_map.insert('\u{FFFD}', "Underfined Control Character: suggest remove");
    ref_map.insert('\u{1FFFE}', "Undefined Control Character");
    ref_map.insert('\u{1FFFF}', "Undefined Control Character");
    ref_map.insert('\u{2FFFE}', "Undefined Control Character");
    ref_map.insert('\u{2FFFF}', "Undefined Control Character");
    ref_map.insert('\u{3FFFE}', "Undefined Control Character");
    ref_map.insert('\u{3FFFF}', "Undefined Control Character");
    ref_map.insert('\u{4FFFE}', "Undefined Control Character");
    ref_map.insert('\u{4FFFF}', "Undefined Control Character");
    ref_map.insert('\u{5FFFE}', "Undefined Control Character");
    ref_map.insert('\u{5FFFF}', "Undefined Control Character");
    ref_map.insert('\u{6FFFE}', "Undefined Control Character");
    ref_map.insert('\u{6FFFF}', "Undefined Control Character");
    ref_map.insert('\u{7FFFE}', "Undefined Control Character");
    ref_map.insert('\u{7FFFF}', "Undefined Control Character");
    ref_map.insert('\u{8FFFE}', "Undefined Control Character");
    ref_map.insert('\u{8FFFF}', "Undefined Control Character");
    ref_map.insert('\u{9FFFE}', "Undefined Control Character");
    ref_map.insert('\u{9FFFF}', "Undefined Control Character");
    ref_map.insert('\u{AFFFE}', "Undefined Control Character");
    ref_map.insert('\u{AFFFF}', "Undefined Control Character");
    ref_map.insert('\u{BFFFE}', "Undefined Control Character");
    ref_map.insert('\u{BFFFF}', "Undefined Control Character");
    ref_map.insert('\u{CFFFE}', "Undefined Control Character");
    ref_map.insert('\u{CFFFF}', "Undefined Control Character");
    ref_map.insert('\u{DFFFE}', "Undefined Control Character");
    ref_map.insert('\u{DFFFF}', "Undefined Control Character");
    ref_map.insert('\u{EFFFE}', "Undefined Control Character");
    ref_map.insert('\u{EFFFF}', "Undefined Control Character");
    ref_map.insert('\u{FFFFE}', "Undefined Control Character");
    ref_map.insert('\u{FFFFF}', "Undefined Control Character");
    ref_map.insert('\u{10FFFE}', "Undefined Control Character");
    ref_map.insert('\u{10FFFF}', "Undefined Control Character");

    ref_map
}

struct LineReader<R: Read> {
    inner: R,
    buf: Vec<u8>,
}

impl<R: BufRead> LineReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            buf: Vec::new(),
        }
    }

    fn read_line_self(&mut self) -> io::Result<Option<String>> {
        let mut line = Vec::new();
        let bytes_read = self.inner.read_until(b'\n', &mut line)?;

        if bytes_read == 0 {
            if !self.buf.is_empty() {
                let cloned_buf = self.buf.clone();
                self.buf.clear();
                let cloned_string = String::from_utf8_lossy(&cloned_buf);
                return Ok(Some(cloned_string.into_owned()));
            }
            return Ok(None);
        }

        if line.last() == Some(&b'\r') {
            line.pop();
        }

        self.buf.extend(line.iter());

        let cloned_buf = self.buf.clone();
        self.buf.clear();
        let cloned_string = String::from_utf8_lossy(&cloned_buf);
        Ok(Some(cloned_string.into_owned()))
    }
}

impl<R: Read> Read for LineReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: BufRead> BufRead for LineReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&self.buf)
    }

    fn consume(&mut self, amt: usize) {
        self.buf.drain(..amt);
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        buf.clear();
        match self.read_line_self() {
            Ok(Some(line)) => {
                buf.push_str(&line);
                Ok(line.len())
            }
            Ok(None) => Ok(0),
            Err(e) => Err(e),
        }
    }
}

fn character_profiling() -> Result<(), std::io::Error> {
    let ascii_control_characters = init_control_character_descriptions();
    let stdin = io::stdin();
    let mut frequency_map: HashMap<char, usize> = HashMap::new();

    let file_reader: Box<dyn BufRead> = Box::new(stdin.lock());

    let mut reader = LineReader::new(file_reader);

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        for c in line.chars() {
            let count = frequency_map.entry(c).or_insert(0);
            *count += 1;
        }
        line.clear();
    }

    println!(
        "{:<6}\t{:<8}\t{:<8}\t{}\t{}",
        //"{:<8}\t{:<8}\t{}\t{}",
        "hex", "char", "count", "description", "name"
        //"char", "count", "description", "name"
    );
    //println!("{:-<8}\t{:-<8}\t{:-<15}\t{:-<15}", "", "", "", "");
    println!("{:-<6}\t{:-<8}\t{:-<8}\t{:-<15}\t{:-<15}", "", "", "", "", "");

    let mut sorted_chars: Vec<(char, usize)> = frequency_map.into_iter().collect();
    sorted_chars.sort_unstable_by_key(|&(c, _)| c as u32);

    for (c, count) in sorted_chars {
        let character_name = unicode_names2::name(c).map_or_else(
            || {
                ascii_control_characters
                    .get(&c)
                    .map_or("UNKNOWN".to_string(), |desc| desc.to_string())
            },
            |name| name.to_string(),
        );
        let hex_repr = format!("{:X}", c as u32);  // Convert char to its hexadecimal representation
        println!(
            "{:-<6}\t{:<10}\t{:<8}\t{:<8}\t{}",
            //"{:<8}\t{:<8}\t{}\t{}",
            hex_repr,
            c.escape_unicode(),
            count,
            c.escape_debug(),
            character_name
        );
    }
    Ok(())
}

// updated for parallel processing with rayon:
fn process_json_line_as_json(json_line: &str, grain: &str) -> serde_json::Value {
    let mut json_data: serde_json::Value = serde_json::from_str(json_line).unwrap();

    fn process_json_value(json_value: &mut serde_json::Value, grain: &str) {
        match json_value {
            serde_json::Value::Object(ref mut map) => {
                let mut new_entries: Vec<(String, serde_json::Value)> = Vec::new();
                for (key, value) in map.iter_mut() {
                    process_json_value(value, grain);

                    // Handle String values
                    if let serde_json::Value::String(s) = value {
                        let hu_masked_value = mask_value(s, "HU", key);
                        let lu_masked_value = mask_value(s, "LU", key);

                        let temp_data = json!({
                            "raw": s,
                            "HU": hu_masked_value,
                            "LU": lu_masked_value
                        });
                        let assertions = process_data(key, &temp_data).unwrap_or(serde_json::Value::Null);

                        let enhanced_value = json!({
                            "raw": s,
                            "HU": hu_masked_value,
                            "LU": lu_masked_value,
                            "Rules": assertions
                        });
                        new_entries.push((key.clone(), enhanced_value));
                    }
                    // Handle Number values (integers and floats)
                    else if let serde_json::Value::Number(n) = value {
                        let s = n.to_string();
                        let hu_masked_value = mask_value(&s, "HU", key);
                        let lu_masked_value = mask_value(&s, "LU", key);

                        let temp_data = json!({
                            "raw": s,
                            "HU": hu_masked_value,
                            "LU": lu_masked_value
                        });
                        let assertions = process_data(key, &temp_data).unwrap_or(serde_json::Value::Null);

                        let enhanced_value = json!({
                            "raw": n, // Keep original number type
                            "HU": hu_masked_value,
                            "LU": lu_masked_value,
                            "Rules": assertions
                        });
                        new_entries.push((key.clone(), enhanced_value));
                    }
                }
                for (key, value) in new_entries {
                    map.insert(key, value);
                }
            }
            serde_json::Value::Array(ref mut values) => {
                values.par_iter_mut().for_each(|value| process_json_value(value, grain));
            }
            _ => {}
        }
    }

    process_json_value(&mut json_data, grain);
    json_data
}

fn truncate_string(input: &str, max_length: usize) -> String {
    let mut result = String::new();
    for word in input.split_whitespace() {
        if result.len() + word.len() > max_length - 3 { // account for "..."
            break;
        } else {
            result += " ";
            result += word;
        }
    }
    if result.len() < input.len() {
        result += "...";
    }
    result
}

/// Parse a CSV line using proper CSV quoting rules
fn parse_csv_line(line: &str, delimiter: u8) -> Vec<String> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .from_reader(line.as_bytes());

    if let Some(result) = reader.records().next() {
        match result {
            Ok(record) => record.iter().map(|s| s.to_string()).collect(),
            Err(_) => {
                // Fallback to naive splitting if CSV parsing fails
                line.split(delimiter as char).map(|s| s.to_string()).collect()
            }
        }
    } else {
        vec![]
    }
}





fn main() {

    // Setup Rayon
    rayon::ThreadPoolBuilder::new()
        .num_threads(22) // Using 22 cores out of 24.
        .build_global()
        .unwrap();

    let matches = App::new("Bytefreq Data Profiler")
        .version("1.0")
        .author("Andrew Morgan <minkymorgan@gmail.com>\n")
        .about("A command-line tool to generate data profiling reports based on various masking strategies.")
        .arg(
            Arg::new("grain")
            .short('g')
            .long("grain")
            .value_name("GRAIN")
                .help("Sets the grain type for masking:\n\
                   'H' - High grain (A for uppercase letters, a for lowercase letters, 9 for digits)\n\
                   'L' - Low grain (repeated pattern characters will be compressed to one)\n\
                   'U' - Unicode (uses Unicode general categories for masking\n\
                   'LU'- Low grain Unicode (repeated pattern classes compressed to one\n)")
            .takes_value(true)
            .default_value("LU"),
        )
        .arg(
            Arg::new("delimiter")
                .short('d')
                .long("delimiter")
                .value_name("DELIMITER")
                .help("Sets the delimiter used to separate fields in input tabular data.\n\
                   Default: '|' (pipe character)")
                .takes_value(true)
                .default_value("|"),
        )
        .arg(
            Arg::new("maxlen")
                .short('l')
                .long("maxlen")
                .value_name("MAXLEN")
                .help("Sets the maximum string size for examples in the DQ reports.\n\
                   Default: 20 ")
                .takes_value(true)
                .default_value("20"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Sets the format of the input data:\n\
                   'json' - JSON data (each line should contain a JSON object)\n\
                   'tabular' - Tabular data (first line should be the header)\n\
                   'excel' - Excel file (.xlsx, .xls, .xlsb, .ods) - requires --excel-path\n\
                   'parquet' - Parquet file (.parquet) - requires --parquet-path")
                .takes_value(true)
                .default_value("tabular"),
        )
        .arg(
            Arg::new("excel_path")
                .long("excel-path")
                .value_name("EXCEL_PATH")
                .help("Path to Excel file (required when format is 'excel')")
                .takes_value(true),
        )
        .arg(
            Arg::new("parquet_path")
                .long("parquet-path")
                .value_name("PARQUET_PATH")
                .help("Path to Parquet file (required when format is 'parquet')")
                .takes_value(true),
        )
        .arg(
            Arg::new("sheet")
                .short('s')
                .long("sheet")
                .value_name("SHEET")
                .help("Sheet index to process (0-based, default: 0)")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::new("sheet_name")
                .long("sheet-name")
                .value_name("SHEET_NAME")
                .help("Sheet name to process (overrides --sheet if provided)")
                .takes_value(true),
        )
        .arg(
            Arg::new("header_row")
                .long("header-row")
                .value_name("HEADER_ROW")
                .help("Row index containing column headers (0-based, default: 0)")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
        Arg::new("report")
        .short('r')
        .long("report")
        .value_name("REPORT")
        .help("Sets the type of report to generate:\n\
               'DQ' - Data Quality (default)\n\
               'CP' - Character Profiling")
        .takes_value(true)
        .default_value("DQ"),
    )
        .arg(
            Arg::new("pathdepth")
                .short('p')
                .long("pathdepth")
                .value_name("PATHDEPTH")
                .help("Sets the depth for JSON paths (applicable for JSON data only).")
                .takes_value(true)
                .default_value("9"),
        )
        .arg(
            Arg::new("remove_array_numbers")
                .short('a')
                .long("remove-array-numbers")
                .value_name("REMOVE_ARRAY_NUMBERS")
                .help("Remove array numbers when set to true")
                .takes_value(false)
        )
        .arg(
            Arg::new("extract_array")
                .long("extract-array")
                .value_name("FIELD")
                .help("Extract array from paginated JSON and convert to NDJSON.\n\
                       Specify field name (e.g., 'items', 'data', 'results') or 'auto' for automatic detection.\n\
                       Converts {items: [...]} or [...] to newline-delimited JSON.")
                .takes_value(true)
        )
    .arg(
        Arg::new("enhanced_output")
        .short('e')
        .long("enhanced-output")
        .value_name("ENHANCED_OUTPUT")
        .help("Output the processed tabular data in JSON format when set to true.")
        .takes_value(false)
    )
    .arg(
         Arg::new("flat_enhanced")
         .short('E')
         .long("flat-enhanced")
         .value_name("FLAT_ENHANCED")
         .help("Formats the enhanced output in a flattened format")
         .takes_value(false)
    )
        .get_matches();


    let report = matches.value_of("report").unwrap();
    let enhanced_output = matches.is_present("enhanced_output");
    let flat_enhanced = matches.is_present("flat_enhanced");
    let remove_array_numbers = matches.is_present("remove_array_numbers");

    if report == "CP" {
        //character_profiling();
        match character_profiling() {
            Ok(_) => println!("--------END OF REPORT--------"),
            Err(e) => eprintln!("Error occurred during character profiling: {}", e),
        }
    } else {
        let grain = matches.value_of("grain").unwrap();
        let delimiter_str = matches.value_of("delimiter").unwrap();
        let delimiter = delimiter_str.as_bytes()[0]; // Convert first character to u8
        let format = matches.value_of("format").unwrap();
        let maxlen = matches.value_of("maxlen").unwrap().parse::<usize>().unwrap();
        // new code to process tabular or json data
        let stdin = io::stdin();

        // shared mutable state wrapped in Mutex and Arc to aid parallel processing in rayon
        let frequency_maps: Arc<Mutex<Vec<HashMap<String, usize>>>> = Arc::new(Mutex::new(Vec::new()));
        let example_maps: Arc<Mutex<Vec<HashMap<String, String>>>> = Arc::new(Mutex::new(Vec::new()));
        let column_names: Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(HashMap::new()));
        let field_count_map: Arc<Mutex<HashMap<usize, usize>>> = Arc::new(Mutex::new(HashMap::new()));
        let record_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

        let pathdepth = matches
            .value_of("pathdepth")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let header_row: usize = matches
            .value_of("header_row")
            .unwrap()
            .parse()
            .expect("header-row must be a valid number");

        let extract_array_field = matches.value_of("extract_array");

        // Handle Excel files differently
        let mut lines: Vec<String> = if format == "excel" {
            // Excel processing
            let excel_path = matches.value_of("excel_path")
                .expect("--excel-path is required when format is 'excel'");

            let sheet_name_opt = matches.value_of("sheet_name");
            let sheet_index: usize = matches.value_of("sheet")
                .unwrap()
                .parse()
                .expect("Sheet index must be a valid number");

            // Read Excel data
            let rows = if let Some(sheet_name) = sheet_name_opt {
                ExcelReader::read_sheet_by_name(excel_path, sheet_name)
                    .expect("Failed to read Excel sheet by name")
            } else {
                ExcelReader::read_sheet_by_index(excel_path, sheet_index)
                    .expect("Failed to read Excel sheet by index")
            };

            // Convert rows to delimited strings
            rows.into_iter()
                .map(|row| row.join(&(delimiter as char).to_string()))
                .collect()
        } else if format == "parquet" {
            let parquet_path = matches.value_of("parquet_path")
                .expect("--parquet-path is required when format is 'parquet'");
            ParquetReader::read_as_json_lines(parquet_path)
                .expect("Failed to read Parquet file")
        } else {
            stdin.lock().lines().filter_map(Result::ok).collect()
        };

        // Handle array extraction for JSON format
        if format == "json" && extract_array_field.is_some() {
            // Join all lines into single string (for paginated JSON detection)
            let input = lines.join("\n");

            // Attempt array extraction
            match extract_json_array(&input, extract_array_field) {
                Ok(ndjson) => {
                    // Replace lines with extracted NDJSON
                    lines = ndjson.lines().map(|s| s.to_string()).collect();
                },
                Err(_) => {
                    eprintln!("✗ Array extraction failed, processing as NDJSON");
                    // Keep original lines
                }
            }
        }

        // For tabular/Excel data, process the header first (sequentially)
        if format == "tabular" || format == "excel" {
            if lines.len() > header_row {
                let header_line = &lines[header_row];
                let mut local_column_names = column_names.lock().unwrap();
                let mut local_frequency_maps = frequency_maps.lock().unwrap();
                let mut local_example_maps = example_maps.lock().unwrap();

                let headers = parse_csv_line(header_line, delimiter);
                for (idx, name) in headers
                    .iter()
                    .map(|s| s.trim().replace(" ", "_"))
                    .enumerate()
                {
                    local_column_names.insert(name.to_string(), idx);
                    local_frequency_maps.push(HashMap::new());
                    local_example_maps.push(HashMap::new());
                }
            }
        }

        // Now we move the loop into a parallel iterator
        lines.par_iter().enumerate().for_each(|(line_idx, line)| {
            if !line.is_empty() {
                // Excel and tabular share the same processing logic
                let actual_format = match format {
                    "excel" => "tabular",
                    "parquet" => "json",
                    _ => format,
                };

                if actual_format == "json" {
                    if enhanced_output == true {
                        let json_line = process_json_line_as_json(&line, grain);
                        //let enhanced_json_line = process_data(&json_line);
                        println!("{}", serde_json::to_string(&json_line).unwrap());    // delivers very nested ehanced data
                    } else if flat_enhanced == true {
                        let json_line = process_json_line_as_json(&line, grain);
                        match flatten_json_object::Flattener::new().flatten(&json_line) {
                            Ok(flattened) => println!("{}",                           // significantly unnests data, line by line 
                            serde_json::to_string(&flattened).unwrap()),
                            Err(e) => eprintln!("Failed to flatten JSON: {}", e), 
                        } 
                    } else {
                        let mut local_frequency_maps = frequency_maps.lock().unwrap();
                        let mut local_example_maps = example_maps.lock().unwrap();
                        let mut local_column_names = column_names.lock().unwrap();
                        process_json_line(
                            &line,
                            &mut local_frequency_maps,
                            &mut local_example_maps,
                            grain,
                            &mut local_column_names,
                            pathdepth,
                            remove_array_numbers,
                        );
                    }
                } else {
                    // Tabular processing
                    // Skip lines up to and including the header row, then process data
                    if line_idx > header_row {
                        let mut local_column_names = column_names.lock().unwrap();
                        let mut local_record_count = record_count.lock().unwrap();
                        let mut local_frequency_maps = frequency_maps.lock().unwrap();
                        let mut local_example_maps = example_maps.lock().unwrap();
                        // Process tabular data
                        if !local_column_names.is_empty() {
                            let fields = parse_csv_line(line, delimiter);
                            let mut processed_fields = Vec::new();

                            for (i, field) in fields.iter().enumerate() {
                                let column_name = match local_column_names.iter().find(|(_, &v)| v == i) {
                                    Some((name, _)) => name.clone(),
                                    None => {
                                        // Handle ragged data - this field has no corresponding column
                                        let extra_column_index = if i + 1 > local_column_names.len() {
                                            i + 1 - local_column_names.len()
                                        } else {
                                            // This shouldn't happen but handle gracefully
                                            0
                                        };
                                        let new_name = format!("RaggedErr{}", extra_column_index);

                                        // Update column_names, frequency_maps, and example_maps for the new column
                                        //local_column_names.insert(new_name.clone(), local_column_names.len());
                                        let current_length = local_column_names.len();
                                        local_column_names.insert(new_name.clone(), current_length);

                                        local_frequency_maps.push(HashMap::new());
                                        local_example_maps.push(HashMap::new());

                                        new_name
                                    }
                                };
                                processed_fields.push((column_name, field.as_str()));
                            }

                            let field_count = processed_fields.len();
                            let mut field_count_map_guard = field_count_map.lock().unwrap();
                            *field_count_map_guard.entry(field_count).or_insert(0) += 1;

                            for (name, value) in &processed_fields {
                                let masked_value = mask_value(value, grain, &name);

                                if let Some(idx) = local_column_names.get(name) {
                                    let count = local_frequency_maps[*idx]
                                        .entry(masked_value.clone())
                                        .or_insert(0);
                                    *count += 1;

                                    // Reservoir sampling
                                    let mut rng = thread_rng();
                                    if rng.gen::<f64>() < 1.0 / (*count as f64) {
                                        local_example_maps[*idx]
                                            .insert(masked_value.clone(), value.to_string());
                                    }
                                } else {
                                    // Handle the case when the column name is not found in the HashMap
                                    println!(
                                        "Warning: Column name not found in the HashMap: {}",
                                        name
                                    );
                                }
                            }

                            // collect tabular data to enhance, enhance, print
                            if enhanced_output {
                                let processed_fields: Vec<(String, String)> = local_column_names.iter().map(|column_name| {
                                    let value = fields[*column_name.1].clone();
                                    (column_name.0.clone(), value)
                                }).collect();

                                let json_line = process_tabular_line_as_json(&processed_fields);
                                //let enhanced_json_line = process_data(&json_line);
                                println!("{}", serde_json::to_string(&json_line).unwrap());
                            } else if flat_enhanced {
                                let processed_fields: Vec<(String, String)> = local_column_names.iter().map(|column_name| {
                                    let value = fields[*column_name.1].clone();
                                    (column_name.0.clone(), value)
                                }).collect();

                                let json_line = process_tabular_line_as_json(&processed_fields);
                                match flatten_json_object::Flattener::new().flatten(&json_line) {
                                    Ok(flattened) => println!("{}", serde_json::to_string(&flattened).unwrap()),
                                    Err(e) => eprintln!("Failed to flatten tabular JSON: {}", e),
                                }
                            }
                        }
                        *local_record_count += 1;
                    }
                }
                
            }
        });

        // Output the processed tabular line in JSON format if the enhanced_output flag is set to true

        if enhanced_output == false {
            let now = Local::now();
            let now_string = now.format("%Y%m%d %H:%M:%S").to_string();
            println!();
            println!("Data Profiling Report: {}", now_string);
            let record_count_value = record_count.lock().unwrap();
            println!("Examined rows: {}", record_count_value);
            println!();
            println!("FieldsPerLine:");
            // Print the field count map
            let field_count_map_ref = field_count_map.lock().unwrap();
            for (field_count, frequency) in &*field_count_map_ref {
                println!("{} fields: {} rows", field_count, frequency);
            }

            println!();
            println!(
                "{:<32}\t{:<8}\t{:<8}\t{:<32}",
                "column", "count", "pattern", "example"
            );
            println!("{:-<32}\t{:-<8}\t{:-<8}\t{:-<32}", "", "", "", "");

            // sort the reporting lines
            let column_names_ref = column_names.lock().unwrap();
            let mut sorted_column_names: Vec<(&String, &usize)> = column_names_ref.iter().collect();

            sorted_column_names.sort_unstable_by_key(|(_, idx)| **idx);

            for (name, idx) in sorted_column_names { 
                let frequency_maps_ref = frequency_maps.lock().unwrap();
                if let Some(frequency_map) = frequency_maps_ref.get(*idx) {
                    let mut column_counts = frequency_map
                        .iter()
                        .map(|(value, count)| (value, count))
                        .collect::<Vec<(&String, &usize)>>();

                    column_counts.sort_unstable_by(|a, b| b.1.cmp(a.1));

                    for (value, count) in column_counts {
                        let empty_string = "".to_string();
                        let example_maps_ref = example_maps.lock().unwrap();
                        let example = example_maps_ref[*idx].get(value).unwrap_or(&empty_string);
                        let truncated_example = truncate_string(&example, maxlen); // adjust the maximum length as needed
                        println!(
                            "col_{:05}_{}\t{:<8}\t{:<8}\t{:<32}",
                            idx, name, count, value, truncated_example
                        );
                    }
                }
            }
        } //End not enhanced_output
    }
} // end of main
