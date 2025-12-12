use bytefreq::excel::ExcelReader;

fn main() {
    let sheets = ExcelReader::get_sheet_names("testdata/Illegal_Dumping_Incidents.xls")
        .expect("Failed to read Excel file");

    println!("Sheets in workbook:");
    for (i, name) in sheets.iter().enumerate() {
        println!("  [{}] '{}'", i, name);
    }
}
