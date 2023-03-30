use std::collections::HashMap;
use std::io::{self, BufRead};

pub struct CsvParser {
    delimiter: char,
}

impl CsvParser {
    pub fn new() -> Self {
        Self { delimiter: ',' }
    }

    pub fn with_delimiter(delimiter: char) -> Self {
        Self { delimiter }
    }

    pub fn process_file<F: BufRead>(
        &self,
        file: &mut F,
    ) -> io::Result<HashMap<String, u64>> {
        let mut frequency_map = HashMap::new();

        for line in file.lines() {
            let line = line?;
            let fields = line.split(self.delimiter);

            for field in fields {
                let field = field.trim();
                *frequency_map.entry(field.to_string()).or_insert(0) += 1;
            }
        }

        Ok(frequency_map)
    }
}

