use regex::Regex;
use serde_json::json;
use chrono::{NaiveDate, Utc};

// this is a library of assertion rules, that are matched to triples arriving (raw, HU, LU)

fn string_length(value: &str) -> i32 {
    let char_count = value.chars().count();
    return char_count as i32; 
}


fn parse_date(value: &str) -> Option<NaiveDate> {
    let formats = ["%d-%m-%Y", "%d/%m/%Y"];

    for format in &formats {
        if let Ok(parsed_date) = NaiveDate::parse_from_str(value, format) {
            return Some(parsed_date);
        }
    }

    None
}

fn is_sensible_dob(value: &str) -> bool {
    if let Some(parsed_date) = parse_date(value) {
        let now = Utc::now().naive_utc().date();
        let min_dob = now - chrono::Duration::weeks(127 * 52);

        return parsed_date >= min_dob && parsed_date <= now;
    }
    false
}

pub fn is_uk_postcode(value: &str) -> bool {
    let re = Regex::new(r"^(([A-Z][A-HJ-Y]?\d[A-Z\d]?|ASCN|STHL|TDCU|BBND|[BFS]IQQ|PCRN|TKCA) ?\d[A-Z]{2}|BFPO ?\d{1,4}|(KY\d|MSR|VG|AI)[ -]?\d{4}|[A-Z]{2} ?\d{2}|GE ?CX|GIR ?0A{2}|SAN ?TA1)$").unwrap();
    re.is_match(value)
}

pub fn is_numeric(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

pub fn poss_latitude(value: &str) -> bool {
    if let Ok(parsed_value) = value.parse::<f64>() {
        return parsed_value >= -90.0 && parsed_value <= 90.0;
    }
    false
}

pub fn poss_longitude(value: &str) -> bool {
    if let Ok(parsed_value) = value.parse::<f64>() {
        return parsed_value >= -180.0 && parsed_value <= 180.0;
    }
    false
}

pub fn execute_assertions(field_name: &str, raw: &str, lu: &str, hu: &str) -> serde_json::Value {
    let mut assertions: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    // Remove double quotes from the input strings
    let raw = raw.trim_matches('"');
    let lu = lu.trim_matches('"');
    let hu = hu.trim_matches('"');

    assertions.insert("string_length".to_string(), json!(string_length(raw)));

    if lu == "9" || lu == "9.9" {
        assertions.insert("is_numeric".to_string(), json!(is_numeric(raw)));
        //assertions.insert("poss_latitude".to_string(), json!(poss_latitude(raw)));
        //assertions.insert("poss_longitude".to_string(), json!(poss_longitude(raw)));
    }

    // Add more assertion checks based on different LU/HU patterns here

    if lu == "A9 9A" || hu == "A9A 9A" {
        assertions.insert("is_uk_postcode".to_string(), json!(is_uk_postcode(raw)));
    }

    if lu == "9_9_9" {
         assertions.insert("std_date".to_string(), json!(parse_date(raw)));
    }

    // check DOB
    if hu == "99_99_9999" && field_name.to_lowercase().contains("dob") {
        assertions.insert("is_sensible_dob".to_string(), json!(is_sensible_dob(raw)));
    }

    serde_json::Value::Object(assertions)
}

