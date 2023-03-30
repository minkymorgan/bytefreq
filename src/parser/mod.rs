pub mod csv_parser;
pub mod json_parser;

pub enum Parser {
    Csv(csv_parser::CsvParser),
    Json(json_parser::JsonParser),
}

impl Parser {
    pub fn from_file_format(format: &str) -> Option<Self> {
        match format {
            "csv" => Some(Self::Csv(csv_parser::CsvParser::new())),
            "json" => Some(Self::Json(json_parser::JsonParser::new())),
            _ => None,
        }
    }
}

