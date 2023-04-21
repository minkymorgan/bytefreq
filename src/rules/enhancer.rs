use crate::rules::assertions::execute_assertions;
use std::collections::HashMap;

pub fn process_data(field_name: &str, data: &serde_json::Value, memo: &mut HashMap<String, (String, String)>) -> Option<serde_json::Value> {
    let lu = data["LU"].as_str().unwrap_or("");
    let hu = data["HU"].as_str().unwrap_or("");
    let raw = data["raw"].as_str().unwrap_or("");

    let assertions = execute_assertions(field_name, raw, lu, hu, memo);

    if assertions.as_object().unwrap().is_empty() {
        None
    } else {
        Some(assertions)
    }
}

