use std::collections::HashMap;
use std::io::{self, BufRead};
use rand::prelude::*;
use chrono::{Local, Datelike, Timelike};
use clap::{App, Arg};

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
        _ => low_grain_mask(value)
    }
}


fn main() {

    let matches = App::new("Bytefreq Data Profiler")
        .version("1.0")
        .author("Your Name <minkymorganl@gmail.com>")
        .help("Mask based commandline data profiler")
        .arg(
            Arg::new("grain")
                .short('g')
                .long("grain")
                .value_name("GRAIN")
                .help("Sets the grain type for masking ('H' for highgrain, 'L' for lowgrain)")
                .takes_value(true)
                .default_value("L"),
        )
        .get_matches();

    let grain = matches.value_of("grain").unwrap();




    let stdin = io::stdin();
    let mut frequency_maps: Option<Vec<HashMap<String, usize>>> = None;
    let mut example_maps: Option<Vec<HashMap<String, String>>> = None;
    let mut header: Option<Vec<String>> = None;
    let total_records = stdin
        .lock()
        .lines()
        .filter_map(Result::ok)
        .enumerate()
        .inspect(|(i, line)| {
            if *i == 0 {
                header = Some(line.split('|').map(String::from).collect());
                return;
            }

            let fields = line.split('|').map(String::from).collect::<Vec<String>>();

            if frequency_maps.is_none() {
                frequency_maps = Some(vec![HashMap::new(); fields.len()]);
                example_maps = Some(vec![HashMap::new(); fields.len()]);
            }

            let frequency_maps = frequency_maps.as_mut().unwrap();
            let example_maps = example_maps.as_mut().unwrap();
            let mut rng = thread_rng();

            for (idx, field) in fields.iter().enumerate() {
                let masked_value = mask_value(field, grain);
                let count = frequency_maps[idx].entry(masked_value.clone()).or_insert(0);
                *count += 1;

                // Reservoir sampling
                if rng.gen::<f64>() < 1.0 / (*count as f64) {
                    example_maps[idx].insert(masked_value.clone(), field.clone());
                }
            }
        })
        .count();

    let now = Local::now();
    let now_string = now.format("%Y%m%d %H:%M:%S").to_string();
    println!();
    println!("Data Profiling Report: {}", now_string);
    println!("Examined rows: {}", total_records);
    println!();
    println!(
        "{:<32}\t{:<8}\t{:<8}\t{:<32}",
        "column", "count", "pattern", "example"
    );
    println!("{:-<32}\t{:-<8}\t{:-<8}\t{:-<32}", "", "", "", "");

    if let Some(header) = header {
        let frequency_maps = frequency_maps.unwrap();
        let example_maps = example_maps.unwrap();
        for (idx, name) in header.iter().enumerate() {
            let mut column_counts = frequency_maps[idx]
                .iter()
                .map(|(value, count)| (value, count))
                .collect::<Vec<(&String, &usize)>>();

            column_counts.sort_unstable_by(|a, b| b.1.cmp(a.1));

            for (value, count) in column_counts {
                let empty_string = "".to_string();
                let example = example_maps[idx].get(value).unwrap_or(&empty_string);
                //let example = example_maps[idx].get(value).unwrap_or(&"".to_string());

                println!(
                    "col_{:05}_{}\t{:<8}\t{:<8}\t{:<32}",
                    idx, name, count, value, example
                );
            }
        }
    }
}

