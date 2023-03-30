use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::collections::HashMap;

mod parser;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: bytefreq-rs <file_path>");
        return;
    }

    let file_path = &args[1];
    let path = Path::new(file_path);

    if !path.exists() {
        eprintln!("File not found: {}", file_path);
        return;
    }

    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut frequency_map: HashMap<String, u64> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let delimiter = utils::detect_delimiter(&line);
        let format = utils::detect_format(&line, &delimiter);

        match format {
            utils::DataFormat::CSV | utils::DataFormat::TSV | utils::DataFormat::PSV => {
                let fields = parser::parse_delimited_line(&line, &delimiter);

                for field in fields {
                    *frequency_map.entry(field).or_insert(0) += 1;
                }
            }
            utils::DataFormat::JSON => {
                let json_data = parser::parse_json_line(&line);
                for (key, value) in json_data {
                    *frequency_map.entry(key).or_insert(0) += value;
                }
            }
            _ => {
                eprintln!("Unsupported file format. Please provide a CSV, TSV, PSV, or JSON file.");
                return;
            }
        }
    }

    // Output the results
    println!("Field, Frequency");
    for (field, count) in &frequency_map {
        println!("{}, {}", field, count);
    }
}

