use std::collections::HashMap;
use std::io::{self, BufRead};
use rand::prelude::*;
use chrono::{Local};
use clap::{App, Arg};
use serde_json::{Value, Map};
use unic::ucd::GeneralCategory as Category;


// this is a highgrain Mask that works for unicode data!
fn get_generalized_char(c: char) -> char {
    match c {
        '0'..='9' => '9',
        'a'..='z' => 'a',
        'A'..='Z' => 'A',
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
    output
}

fn mask_value(value: &str, grain: &str) -> String {
    match grain {
        "H" => high_grain_mask(value),
        "L" => low_grain_mask(value),
        _u => value.chars().map(|c| get_generalized_char(c)).collect(),
        //_ => value.chars().map(|c| get_generalized_char(c)).collect(),
    }
}

fn process_json_map(
    map: &Map<String, Value>,
    frequency_maps: &mut Vec<HashMap<String, usize>>,
    example_maps: &mut Vec<HashMap<String, String>>,
    grain: &str,
    prefix: String,
    column_names: &mut HashMap<String, usize>,
) {
    for (key, value) in map.iter() {
        let full_key = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            Value::Object(obj) => process_json_map(obj, frequency_maps, example_maps, grain, full_key.clone(), column_names),
            _ => {
                let value_str = value.to_string();
                let masked_value = mask_value(&value_str, grain);
                let idx = column_names
                    .entry(full_key.clone())
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
}

fn process_json_line(
    line: &str,
    frequency_maps: &mut Vec<HashMap<String, usize>>,
    example_maps: &mut Vec<HashMap<String, String>>,
    grain: &str,
    column_names: &mut HashMap<String, usize>,
) {
    if let Ok(json_value) = serde_json::from_str::<Value>(line) {
        if let Value::Object(json_map) = json_value {
            process_json_map(&json_map, frequency_maps, example_maps, grain, String::new(), column_names);
        }
    }
}


fn main() {

    let matches = App::new("Bytefreq Data Profiler")
        .version("1.0")
        .author("Andrew Morgan <minkymorganl@gmail.com>")
        .help("Mask based commandline data profiler")
        .arg(
            Arg::new("grain")
	        .short('g')
  	        .long("grain")
	        .value_name("GRAIN")
	        .help("Sets the grain type for masking ('H' for highgrain, 'L' for lowgrain, 'U' for Unicode)")
	        .takes_value(true)
	        .default_value("U"),
        )
        .arg(
            Arg::new("delimiter")
                .short('d')
                .long("delimiter")
                .value_name("DELIMITER")
                .help("Sets the delimiter used to separate fields in input data")
                .takes_value(true)
                .default_value("|"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Sets the format of the input data ('json' for JSON data, 'tabular' for tabular data)")
                .takes_value(true)
                .default_value("tabular"),
        )
        .get_matches();

    let grain = matches.value_of("grain").unwrap();
    let delimiter = matches.value_of("delimiter").unwrap();
    let format = matches.value_of("format").unwrap();


    // new code to process tabular or json data
    let stdin = io::stdin();
    let mut frequency_maps: Vec<HashMap<String, usize>> = Vec::new();
    let mut example_maps: Vec<HashMap<String, String>> = Vec::new();
    let mut column_names: HashMap<String, usize> = HashMap::new();
    let mut record_count: usize = 0;

    for line in stdin.lock().lines().filter_map(Result::ok) {
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
                }
            } else {
                // Process tabular data
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
        record_count += 1;
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

} // end of main

