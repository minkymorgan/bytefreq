use serde_json::{Map, Value};
use std::collections::HashMap;
use std::io::{self, BufRead};

pub struct JsonParser {}

impl JsonParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_file<F: BufRead>(&self, file: &mut F) -> io::Result<HashMap<String, u64>> {
        let mut frequency_map = HashMap::new();
        let mut buffer = String::new();

        file.read_to_string(&mut buffer)?;

        let json_value: Value = serde_json::from_str(&buffer)?;
        self.process_value("", &json_value, &mut frequency_map);

        Ok(frequency_map)
    }

    fn process_value(
        &self,
        current_path: &str,
        value: &Value,
        frequency_map: &mut HashMap<String, u64>,
    ) {
        match value {
            Value::Object(map) => self.process_object(current_path, map, frequency_map),
            Value::Array(array) => self.process_array(current_path, array, frequency_map),
            _ => *frequency_map.entry(current_path.to_string()).or_insert(0) += 1,
        }
    }

    fn process_object(
        &self,
        current_path: &str,
        object: &Map<String, Value>,
        frequency_map: &mut HashMap<String, u64>,
    ) {
        for (key, value) in object {
            let new_path = if current_path.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", current_path, key)
            };
            self.process_value(&new_path, value, frequency_map);
        }
    }

    fn process_array(
        &self,
        current_path: &str,
        array: &[Value],
        frequency_map: &mut HashMap<String, u64>,
    ) {
        for value in array {
            self.process_value(current_path, value, frequency_map);
        }
    }
}
   
