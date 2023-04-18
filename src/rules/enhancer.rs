use crate::rules::assertions::execute_assertions;

pub fn process_data(data: &serde_json::Value) -> serde_json::Value {
    let lu = data["LU"].as_str().unwrap_or("");
    let hu = data["HU"].as_str().unwrap_or("");
    let raw = data["raw"].as_str().unwrap_or("");

    let assertions = execute_assertions(raw, lu, hu);

    let mut enhanced_data = assertions.clone();
    let enhanced_data_obj = enhanced_data.as_object_mut().unwrap();
    enhanced_data_obj.insert("assertions".to_string(), assertions);

    enhanced_data
}


