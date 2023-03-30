pub mod glob_utils;

pub enum DataFormat {
    CSV,
    TSV,
    PSV,
    JSON,
}

pub fn detect_delimiter(line: &str) -> char {
    // Implement the delimiter detection logic here
    // For now, let's return a comma as the default delimiter
    ','
}

pub fn detect_format(line: &str, delimiter: &char) -> DataFormat {
    // Implement the format detection logic here
    // For now, let's return DataFormat::CSV as the default format
    DataFormat::CSV
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

