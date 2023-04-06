use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use rand::prelude::*;
use chrono::{Local};
use clap::{App, Arg};
use serde_json::{Value};
use unic::ucd::GeneralCategory as Category;
use unicode_names2; 

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

fn mask_value(value: &str, grain: &str) -> String {
    match grain {
        "H" => high_grain_mask(value),
        "L" => low_grain_mask(value),
        "LU" => low_grain_mask(&value.chars().map(|c| high_grain_unicode_mask(c)).collect::<String>()),
        _u => value.chars().map(|c| high_grain_unicode_mask(c)).collect(),
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
                    //process_json_value(value, frequency_maps, example_maps, grain, full_key, column_names, pathdepth + 1, current_depth);
                    process_json_value(value, frequency_maps, example_maps, grain, full_key, column_names, remove_array_numbers, pathdepth + 1, current_depth);
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
                process_json_value(value, frequency_maps, example_maps, grain, full_key, column_names, remove_array_numbers, pathdepth + 1, current_depth);
            }
        }
        _ => {
            let value_str = value.to_string();
            let masked_value = mask_value(&value_str, grain);
            let idx = column_names
                .entry(prefix.clone())
                .or_insert_with(|| {
                    let new_idx = frequency_maps.len();
                    frequency_maps.push(HashMap::new());
                    example_maps.push(HashMap::new());
                    new_idx
                });

            let count = frequency_maps[*idx].entry(masked_value.clone()).or_insert(0);
            *count += 1;

            // Reservoir sampling
            let mut rng = thread_rng();
            if rng.gen::<f64>() < 1.0 / (*count as f64) {
                example_maps[*idx].insert(masked_value.clone(), value_str);
            }
        }
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
        //process_json_value(&json_value, frequency_maps, example_maps, grain, String::new(), column_names, pathdepth, 0);
        //process_json_value(&json_value, frequency_maps, example_maps, grain, String::new(), column_names, remove_array_numbers, pathdepth, 0);
        //process_json_value(&json_value, frequency_maps, example_maps, grain, String::new(), column_names, || remove_array_numbers, pathdepth, 0);
        //process_json_value(&json_value, frequency_maps, example_maps, grain, String::new(), column_names, || remove_array_numbers.clone(), pathdepth, 0);
        //process_json_value(&json_value, frequency_maps, example_maps, grain, String::new(), column_names, remove_array_numbers, pathdepth, 0);
        process_json_value(&json_value, frequency_maps, example_maps, grain, String::new(), column_names, remove_array_numbers, pathdepth, 0);

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
    ref_map.insert('\u{008A}', "LINE TABULATION SET * Deprecated from Unicode 3.2, 2002");
    ref_map
}

struct LineReader<R: Read> {
    inner: R,
    buf: Vec<u8>,
}

/// <<<<<<
impl<R: BufRead> LineReader<R> {
    fn new(inner: R) -> Self {
        Self { inner, buf: Vec::new() }
    }

    fn read_line_self(&mut self) -> io::Result<Option<String>> {
        let mut line = Vec::new();
        let bytes_read = self.inner.read_until(b'\n', &mut line)?;

        if bytes_read == 0 {
            if !self.buf.is_empty() {
                let leftover = String::from_utf8_lossy(&self.buf);
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

        let result = String::from_utf8_lossy(&self.buf);
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

    println!("{:<8}\t{:<8}\t{}\t{}", "char", "count", "description", "name");
    println!("{:-<8}\t{:-<8}\t{:-<15}\t{:-<15}", "", "", "", "");

    let mut sorted_chars: Vec<(char, usize)> = frequency_map.into_iter().collect();
    sorted_chars.sort_unstable_by_key(|&(c, _)| c as u32);

    for (c, count) in sorted_chars {
        let character_name = unicode_names2::name(c).map_or_else(
             || {
                 ascii_control_characters.get(&c).map_or("UNKNOWN".to_string(), |desc| desc.to_string())
             },
             |name| name.to_string(),
         );
        println!("{:<8}\t{:<8}\t{}\t{}", c.escape_unicode(), count, c.escape_debug(), character_name);
    }
    Ok(())
}

fn main() {

    let matches = App::new("Bytefreq Data Profiler")
        .version("1.0")
        .author("Andrew Morgan <minkymorgan@gmail.com>\nhttps://www.linkedin.com/in/andrew-morgan-8590b22/\n")
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
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Sets the format of the input data:\n\
                   'json' - JSON data (each line should contain a JSON object)\n\
                   'tabular' - Tabular data (first line should be the header)")
                .takes_value(true)
                .default_value("tabular"),
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
                .default_value("2"),
        )
        .arg(
            Arg::new("remove_array_numbers")
                .short('a')
                .long("remove-array-numbers")
                .value_name("REMOVE_ARRAY_NUMBERS")
                .help("Remove array numbers when set to true")
                .takes_value(true)
                .default_value("false"),
        )
        .get_matches();


    let report = matches.value_of("report").unwrap();

    if report == "CP" {
        //character_profiling();
        match character_profiling() {
            Ok(_) => println!("--------END OF REPORT--------"),
            Err(e) => eprintln!("Error occurred during character profiling: {}", e),
        }
    } else {

	    let grain = matches.value_of("grain").unwrap();
	    let delimiter = matches.value_of("delimiter").unwrap();
	    let format = matches.value_of("format").unwrap();

	    // new code to process tabular or json data
	    let stdin = io::stdin();
	    let mut frequency_maps: Vec<HashMap<String, usize>> = Vec::new();
	    let mut example_maps: Vec<HashMap<String, String>> = Vec::new();
	    let mut column_names: HashMap<String, usize> = HashMap::new();
            let mut field_count_map: HashMap<usize, usize> = HashMap::new();
	    let mut record_count: usize = 0;
	    let mut input_processed = false;
            let pathdepth = matches.value_of("pathdepth").unwrap().parse::<usize>().unwrap();
            let remove_array_numbers = matches.value_of("remove_array_numbers").unwrap() != "false";

	    for line in stdin.lock().lines().filter_map(Result::ok) {
		if !line.is_empty() {
		    input_processed = true;      
		    if format == "json" {
                        //process_json_line(&line, &mut frequency_maps, &mut example_maps, grain, &mut column_names, pathdepth);
                        //process_json_line(&line, &mut frequency_maps, &mut example_maps, grain, &mut column_names, remove_array_numbers);
                        process_json_line(&line, &mut frequency_maps, &mut example_maps, grain, &mut column_names, pathdepth, remove_array_numbers);

		    } else {
			if record_count == 0 {
                            let header = line; //+ delimiter + "Err1" + delimiter + "Err2";
			    // Process header for tabular data
			    for (idx, name) in header 
				.split(delimiter)
				.map(|s| s.trim().replace(" ", "_"))
				.enumerate()
			    {
				column_names.insert(name.to_string(), idx);
				frequency_maps.push(HashMap::new());
				example_maps.push(HashMap::new());
			    }
			} else {
                            // Process tabular data
                            if !column_names.is_empty() {
                                let fields = line.split(delimiter).collect::<Vec<&str>>();
                                let mut processed_fields = Vec::new();

                                for (i, field) in fields.iter().enumerate() {
                                    let column_name = match column_names.iter().find(|(_, &v)| v == i) {
                                        Some((name, _)) => name.clone(),
                                        None => {
                                            let extra_column_index = i + 1 - column_names.len();
                                            let new_name = format!("RaggedErr{}", extra_column_index);

                                            // Update column_names, frequency_maps, and example_maps for the new column
                                            column_names.insert(new_name.clone(), column_names.len());
                                            frequency_maps.push(HashMap::new());
                                            example_maps.push(HashMap::new());

                                            new_name
                                        }
                                    };
                                    processed_fields.push((column_name, field));
                                }

                                let field_count = processed_fields.len();
                                *field_count_map.entry(field_count).or_insert(0) += 1;

                                for (name, value) in &processed_fields {
                                    let masked_value = mask_value(value, grain);

                                    if let Some(idx) = column_names.get(name) {
                                        let count = frequency_maps[*idx].entry(masked_value.clone()).or_insert(0);
                                        *count += 1;

                                        // Reservoir sampling
                                        let mut rng = thread_rng();
                                        if rng.gen::<f64>() < 1.0 / (*count as f64) {
                                            example_maps[*idx].insert(masked_value.clone(), value.to_string());
                                        }
                                    } else {
                                        // Handle the case when the column name is not found in the HashMap
                                        println!("Warning: Column name not found in the HashMap: {}", name);
                                    }
                                }
                            }
			}   //end of else
		    }
		    record_count += 1;
		}
	    }

	    let now = Local::now();
	    let now_string = now.format("%Y%m%d %H:%M:%S").to_string();
	    println!();
	    println!("Data Profiling Report: {}", now_string);
	    println!("Examined rows: {}", record_count);
	    println!();
            println!("FieldsPerLine:"); 
            // Print the field count map
                for (field_count, frequency) in &field_count_map {
                    println!("{} fields: {} rows", field_count, frequency);
                }
 
            println!();
	    println!(
		"{:<32}\t{:<8}\t{:<8}\t{:<32}",
		"column", "count", "pattern", "example"
	    );
	    println!("{:-<32}\t{:-<8}\t{:-<8}\t{:-<32}", "", "", "", "");

	    for (name, idx) in column_names.iter() {
		if let Some(frequency_map) = frequency_maps.get(*idx) {
		    let mut column_counts = frequency_map
			.iter()
			.map(|(value, count)| (value, count))
			.collect::<Vec<(&String, &usize)>>();
	    
		    column_counts.sort_unstable_by(|a, b| b.1.cmp(a.1));
	    
		    for (value, count) in column_counts {
			let empty_string = "".to_string();
			let example = example_maps[*idx].get(value).unwrap_or(&empty_string);
	    
			println!(
			    "col_{:05}_{}\t{:<8}\t{:<8}\t{:<32}",
			    idx, name, count, value, example
			);
		    }
		}
	    }
    }    
} // end of main

