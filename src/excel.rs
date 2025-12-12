#[cfg(feature = "excel")]
use calamine::{open_workbook_auto, Reader, Range, Data};
use std::path::Path;

#[cfg(feature = "excel")]
type CellValue = Data;

#[cfg(feature = "excel")]
/// Represents an Excel workbook reader
pub struct ExcelReader;

#[cfg(feature = "excel")]
impl ExcelReader {
    /// Read data from a specific sheet by index (0-based)
    /// Returns a vector of rows, where each row is a vector of strings
    /// The first row is treated as the header
    pub fn read_sheet_by_index<P: AsRef<Path>>(
        path: P,
        sheet_index: usize,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        let mut workbook = open_workbook_auto(&path)?;
        let sheet_names = workbook.sheet_names().to_vec();

        if sheet_index >= sheet_names.len() {
            return Err(format!(
                "Sheet index {} out of range. Workbook has {} sheets.",
                sheet_index,
                sheet_names.len()
            )
            .into());
        }

        let sheet_name = &sheet_names[sheet_index];
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| format!("Error reading sheet '{}': {}", sheet_name, e))?;

        Self::range_to_rows(range)
    }

    /// Read data from a specific sheet by name
    /// Returns a vector of rows, where each row is a vector of strings
    /// The first row is treated as the header
    pub fn read_sheet_by_name<P: AsRef<Path>>(
        path: P,
        sheet_name: &str,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        let mut workbook = open_workbook_auto(&path)?;
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| format!("Error reading sheet '{}': {}", sheet_name, e))?;

        Self::range_to_rows(range)
    }

    /// Get list of sheet names from an Excel file
    pub fn get_sheet_names<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let workbook = open_workbook_auto(path)?;
        Ok(workbook.sheet_names().to_vec())
    }

    /// Convert a Range to a vector of rows
    fn range_to_rows(range: Range<CellValue>) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        let mut rows = Vec::new();

        for row in range.rows() {
            let string_row: Vec<String> = row
                .iter()
                .map(|cell| Self::cell_to_string(cell))
                .collect();
            rows.push(string_row);
        }

        if rows.is_empty() {
            return Err("Sheet is empty".into());
        }

        Ok(rows)
    }

    /// Convert a CellValue to a String
    fn cell_to_string(cell: &CellValue) -> String {
        match cell {
            CellValue::Int(i) => i.to_string(),
            CellValue::Float(f) => {
                // Check if float is actually an integer value
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            }
            CellValue::String(s) => s.clone(),
            CellValue::Bool(b) => b.to_string(),
            CellValue::DateTime(dt) => {
                // Excel dates are stored as days since 1900-01-01
                // For now, just convert to string representation
                dt.to_string()
            }
            CellValue::DateTimeIso(s) => s.clone(),
            CellValue::DurationIso(s) => s.clone(),
            CellValue::Error(e) => format!("#ERROR:{:?}", e),
            CellValue::Empty => String::new(),
        }
    }

    /// Read the first sheet from an Excel file
    pub fn read_first_sheet<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        Self::read_sheet_by_index(path, 0)
    }
}

#[cfg(not(feature = "excel"))]
/// Dummy implementation when Excel feature is not enabled
pub struct ExcelReader;

#[cfg(not(feature = "excel"))]
impl ExcelReader {
    pub fn read_sheet_by_index<P: AsRef<Path>>(
        _path: P,
        _sheet_index: usize,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        Err("Excel support not enabled. Rebuild with --features excel".into())
    }

    pub fn read_sheet_by_name<P: AsRef<Path>>(
        _path: P,
        _sheet_name: &str,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        Err("Excel support not enabled. Rebuild with --features excel".into())
    }

    pub fn read_first_sheet<P: AsRef<Path>>(
        _path: P,
    ) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        Err("Excel support not enabled. Rebuild with --features excel".into())
    }

    pub fn get_sheet_names<P: AsRef<Path>>(
        _path: P,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Err("Excel support not enabled. Rebuild with --features excel".into())
    }
}

#[cfg(test)]
#[cfg(feature = "excel")]
mod tests {
    use super::*;

    #[test]
    fn test_cell_to_string() {
        assert_eq!(ExcelReader::cell_to_string(&CellValue::Int(42)), "42");
        assert_eq!(ExcelReader::cell_to_string(&CellValue::Float(3.14)), "3.14");
        assert_eq!(ExcelReader::cell_to_string(&CellValue::Float(42.0)), "42");
        assert_eq!(
            ExcelReader::cell_to_string(&CellValue::String("Hello".to_string())),
            "Hello"
        );
        assert_eq!(ExcelReader::cell_to_string(&CellValue::Bool(true)), "true");
        assert_eq!(ExcelReader::cell_to_string(&CellValue::Empty), "");
    }
}
