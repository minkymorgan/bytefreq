use regex::Regex;
use serde_json::json;

// this is a library of assertion rules, that are matched to triples arriving (raw, HU, LU)

pub fn poss_valid_uk_postcode(value: &str) -> bool {
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

pub fn execute_assertions(raw: &str, lu: &str, hu: &str) -> serde_json::Value {
    let mut assertions: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    // Remove double quotes from the input strings
    let raw = raw.trim_matches('"');
    let lu = lu.trim_matches('"');
    let hu = hu.trim_matches('"');


    if lu == "9" || lu == "9.9" {
        assertions.insert("is_numeric".to_string(), json!(is_numeric(raw)));
        //assertions.insert("poss_latitude".to_string(), json!(poss_latitude(raw)));
        //assertions.insert("poss_longitude".to_string(), json!(poss_longitude(raw)));
    }

    // Add more assertion checks based on different LU/HU patterns here

    if hu == "A9 9A" || hu == "A9A 9A " {
        assertions.insert("poss_valid_UK_Postcode".to_string(), json!(poss_valid_uk_postcode(raw)));
    }



    serde_json::Value::Object(assertions)
}

