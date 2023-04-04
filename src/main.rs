use std::collections::HashMap;
use std::io::{self, BufRead};
use std::io::Read;
use rand::prelude::*;
use chrono::{Local};
use clap::{App, Arg};
use serde_json::{Value, Map};
use unic::ucd::GeneralCategory as Category;
use unicode_names2; 

// this is a highgrain Mask that works for unicode data!
fn get_generalized_char(c: char) -> char {
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
        "LU" => low_grain_mask(&value.chars().map(|c| get_generalized_char(c)).collect::<String>()),
        _u => value.chars().map(|c| get_generalized_char(c)).collect(),
    }
}

fn process_json_value(
    value: &Value,
    frequency_maps: &mut Vec<HashMap<String, usize>>,
    example_maps: &mut Vec<HashMap<String, String>>,
    grain: &str,
    prefix: String,
    column_names: &mut HashMap<String, usize>,
) {
    match value {
        Value::Object(map) => {
            for (key, value) in map.iter() {
                let full_key = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{}.{}", prefix, key)
                };
                process_json_value(value, frequency_maps, example_maps, grain, full_key, column_names);
            }
        }
        Value::Array(values) => {
            for (idx, value) in values.iter().enumerate() {
                let full_key = format!("{}[{}]", prefix, idx);
                process_json_value(value, frequency_maps, example_maps, grain, full_key, column_names);
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
) {
    if let Ok(json_value) = serde_json::from_str::<Value>(line) {
        process_json_value(&json_value, frequency_maps, example_maps, grain, String::new(), column_names);
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

    ref_map
}


fn character_profiling() {

    let stdin = io::stdin();
    let mut frequency_map: HashMap<char, usize> = HashMap::new();
    let control_character_descriptions = init_control_character_descriptions();

    let mut buf = [0; 1];
    let mut stdin_lock = stdin.lock();
    while let Ok(bytes_read) = stdin_lock.read(&mut buf) {
        if bytes_read == 0 {
            break;
        }
        if let Some(c) = std::str::from_utf8(&buf).ok().and_then(|s| s.chars().next()) {
            let count = frequency_map.entry(c).or_insert(0);
            *count += 1;
        }
    }
 
    println!("{:<8}\t{:<8}\t{}\t{}", "char", "count", "description", "name");
    println!("{:-<8}\t{:-<8}\t{:-<15}\t{:-<15}", "", "", "", "");

    let mut sorted_chars: Vec<(char, usize)> = frequency_map.into_iter().collect();
    sorted_chars.sort_unstable_by_key(|&(c, _)| c as u32);

    for (c, count) in sorted_chars {
        let character_name = match unicode_names2::name(c) {
             Some(name) if name != "UNKNOWN" => name,
             _ => control_character_descriptions.get(&c).unwrap_or(&"UNKNOWN"),
         };
        println!("{:<8}\t{:<8}\t{}\t{}", c.escape_unicode(), count, c.escape_debug(), character_name);
    }
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
        .get_matches();


    let report = matches.value_of("report").unwrap();

    if report == "CP" {
        character_profiling();
    } else {

	    let grain = matches.value_of("grain").unwrap();
	    let delimiter = matches.value_of("delimiter").unwrap();
	    let format = matches.value_of("format").unwrap();

	    

	    // new code to process tabular or json data
	    let stdin = io::stdin();
	    let mut frequency_maps: Vec<HashMap<String, usize>> = Vec::new();
	    let mut example_maps: Vec<HashMap<String, String>> = Vec::new();
	    let mut column_names: HashMap<String, usize> = HashMap::new();
	    let mut record_count: usize = 0;
	    let mut input_processed = false;

	    for line in stdin.lock().lines().filter_map(Result::ok) {
		if !line.is_empty() {
		    input_processed = true;      
		    if format == "json" {
			process_json_line(&line, &mut frequency_maps, &mut example_maps, grain, &mut column_names);
		    } else {
			if record_count == 0 {
			    // Process header for tabular data
			    for (idx, name) in line
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
			    if !column_names.is_empty() { // <-- check if column_names is not empty
				let fields = line
				    .split(delimiter)
				    .enumerate()
				    .map(|(i, s)| (column_names.iter().find(|(_, &v)| v == i).unwrap().0.clone(), s))
				    .collect::<Vec<(String, &str)>>();

				for (name, value) in fields {
				    let masked_value = mask_value(value, grain);
				    let idx = column_names[&name];
	    
				    let count = frequency_maps[idx].entry(masked_value.clone()).or_insert(0);
				    *count += 1;

				    // Reservoir sampling
				    let mut rng = thread_rng();
				    if rng.gen::<f64>() < 1.0 / (*count as f64) {
					example_maps[idx].insert(masked_value.clone(), value.to_string());
				    }
				}
			    }
			}
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

