use crate::rules::assertions::execute_assertions;


pub fn process_data(data: &serde_json::Value) -> serde_json::Value {
    // Here, you'll walk through the data, read the LU and HU fields, and execute the assertions

    let lu = data["LU"].as_str().unwrap_or("");
    let hu = data["HU"].as_str().unwrap_or("");
    let raw = data["raw"].as_str().unwrap_or("");

    let assertions = execute_assertions(raw, lu, hu);

    // Add assertions to the data, or do any other processing needed
    let mut enhanced_data = data.clone();
    let enhanced_data_object = enhanced_data.as_object_mut().unwrap();
    enhanced_data_object.insert("assertions".to_string(), assertions);

    serde_json::Value::Object(enhanced_data_object.clone())
}

