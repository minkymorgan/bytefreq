use regex::Regex;
use serde_json::json;
use chrono::{NaiveDate, Utc};
use geonamescache::mappers::country;
use crate::cache::{COUNTRY_NAME_TO_ISO3_CACHE};

// this is a library of assertion rules, that are matched to triples arriving (raw, HU, LU)

fn handle_country_name_variations(country_name: &str) -> Option<(String, String)> {
    match country_name.to_lowercase().as_str() {
        "england" => Some(("GBR".to_string(), "GB-ENG".to_string())),
        "scotland" => Some(("GBR".to_string(), "GB-SCT".to_string())),
        "northern ireland" => Some(("GBR".to_string(), "GB-NIR".to_string())),
        "wales" | "cymru" => Some(("GBR".to_string(), "GB-WLS".to_string())),
        // Add more variations here if needed
        _ => None,
    }
}

fn country_name_to_iso3(value: &str) -> Option<String> {
    let cache = COUNTRY_NAME_TO_ISO3_CACHE.read().unwrap();
    if let Some(cached_value) = cache.get(value) {
        return cached_value.clone();
    }
    drop(cache);

    let name_to_iso3 = country(|c| (c.name.to_lowercase(), c.iso3));
    let result = name_to_iso3.get(&value.to_lowercase()).map(|s| s.to_string());

    let mut cache = COUNTRY_NAME_TO_ISO3_CACHE.write().unwrap();
    if let Some(ref res) = result {
        cache.insert(value.to_string(), Some(res.clone()));
    }

    result
}

fn get_possible_countries(_column_name: &str, raw: &str, hu: &str, lu: &str) -> Vec<String> {
    let mut possible_countries: Vec<String> = Vec::new();

    match hu {
        "9999" => {
            possible_countries.extend(vec!["AT", "BE", "BG", "CH", "CY", "CZ", "DK", "EE", "FI", "GR", "HU", "IE", "LT", "LU", "LV", "MT", "NL", "NO", "PL", "PT", "RO", "SE", "SI", "SK"].into_iter().map(|s| s.to_string()));
        }
        "99999" => {
            possible_countries.extend(vec!["DE", "ES", "FR", "HR", "IT"].into_iter().map(|s| s.to_string()));
        }
        "999-99" => {
            possible_countries.extend(vec!["SE"].into_iter().map(|s| s.to_string()));
        }
        "AAA-9999" => {
            possible_countries.extend(vec!["IE"].into_iter().map(|s| s.to_string()));
        }
        _ => {}
    }

    // Additional country-specific checks
    if lu == "9-9999" && raw.starts_with("1") {
        possible_countries.retain(|country| country == "DE");
    } else if hu == "9999" && raw.starts_with("0") {
        possible_countries.retain(|country| country == "NL");
    } else if hu == "99999" && raw.starts_with("9") {
        possible_countries.retain(|country| country == "FR");
    }

    // UK postal code patterns
    let uk_patterns = vec!["A9 9A", "A9A 9A", "A9A"];
    if uk_patterns.contains(&lu) {
        possible_countries.push("UK".to_string());
    }

    possible_countries
}


fn string_length(value: &str) -> i32 {
    let char_count = value.chars().count();
    return char_count as i32; 
}


fn parse_date(value: &str) -> Option<NaiveDate> {
    let formats = [
        "%d-%b-%Y",   // 31-Dec-2015
        "%d-%m-%Y",   // 31-12-2015
        "%d/%m/%Y",   // 31/12/2015
        "%Y-%m-%d",   // 2015-12-31 (ISO 8601)
        "%m/%d/%Y",   // 12/31/2015 (US format)
        "%Y%m%d",     // 20151231 (compact)
    ];

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

    // sniff potential postal country
    let target_substring = "post"; 
    if field_name.to_lowercase().contains(&target_substring.to_lowercase()) {
        let possible_countries = get_possible_countries(field_name, raw, hu, lu);
        if !possible_countries.is_empty() {
            assertions.insert("poss_postal_country".to_string(), json!(possible_countries));
        }
    }

    // Check country name
    if field_name.to_lowercase().contains("country") && !lu.chars().any(|c| c.is_numeric()) {
        if let Some((iso3, region_code)) = country_name_to_iso3(raw).map(|iso3| (iso3.clone(), format!("{}-{}", iso3, raw)))
            .or_else(|| handle_country_name_variations(raw)) {
            assertions.insert("std_country_iso3".to_string(), json!(iso3));
        assertions.insert("std_region_code".to_string(), json!(region_code));
        }
    }

    if lu == "9" || lu == "9.9" {
        assertions.insert("is_numeric".to_string(), json!(is_numeric(raw)));
        //assertions.insert("poss_latitude".to_string(), json!(poss_latitude(raw)));
        //assertions.insert("poss_longitude".to_string(), json!(poss_longitude(raw)));
    }

    // Add more assertion checks based on different LU/HU patterns here

    if lu == "A9 9A" || hu == "A9A 9A" {
        assertions.insert("is_uk_postcode".to_string(), json!(is_uk_postcode(raw)));
    }

    // Check for date patterns - either common LU patterns or field name contains "date"
    if lu == "9_9_9" || lu == "9-9-9" || lu == "9/9/9" || lu == "9-Aa-9" || field_name.to_lowercase().contains("date") {
        if let Some(parsed_date) = parse_date(raw) {
            assertions.insert("std_date".to_string(), json!(parsed_date.format("%Y-%m-%d").to_string()));
        }
    }

    // check DOB
    if hu == "99_99_9999" && field_name.to_lowercase().contains("dob") {
        assertions.insert("is_sensible_dob".to_string(), json!(is_sensible_dob(raw)));
    }

    

    serde_json::Value::Object(assertions)
}

