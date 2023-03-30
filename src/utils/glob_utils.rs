use glob::glob;
use std::io;
use std::path::PathBuf;

pub fn get_matching_files(pattern: &str) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in glob(pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => paths.push(path),
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_matching_files() {
        let pattern = "src/parser/*.rs";
        let paths = get_matching_files(pattern).unwrap();

        assert!(paths.iter().any(|path| path.ends_with("csv_parser.rs")));
        assert!(paths.iter().any(|path| path.ends_with("json_parser.rs")));
    }
}

