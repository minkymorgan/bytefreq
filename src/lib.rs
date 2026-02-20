// Library module for WASM and other uses

pub mod cache;
pub mod rules;

#[cfg(feature = "excel")]
pub mod excel;

#[cfg(feature = "parquet")]
pub mod parquet;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn profile_csv(csv_data: &str, delimiter: char) -> String {
    // Set panic hook for better error messages in browser console
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    // Simple CSV profiling - just count rows and show first few
    let lines: Vec<&str> = csv_data.lines().collect();
    let row_count = lines.len();

    if row_count == 0 {
        return "No data found".to_string();
    }

    // Parse header
    let header = lines[0];
    let columns: Vec<&str> = header.split(delimiter).collect();
    let col_count = columns.len();

    // Build simple report
    let mut report = String::new();
    report.push_str("Data Quality Report\n");
    report.push_str("===================\n\n");
    report.push_str(&format!("Rows: {}\n", row_count - 1)); // minus header
    report.push_str(&format!("Columns: {}\n\n", col_count));
    report.push_str("Column Names:\n");

    for (i, col) in columns.iter().enumerate() {
        report.push_str(&format!("  {}. {}\n", i + 1, col));
    }

    report.push_str(&format!("\nFirst 3 data rows:\n"));
    for (i, line) in lines.iter().skip(1).take(3).enumerate() {
        report.push_str(&format!("  Row {}: {}\n", i + 1, line));
    }

    report
}
