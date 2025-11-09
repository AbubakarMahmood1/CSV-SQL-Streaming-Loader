//! CSV streaming parser

use crate::errors::{LoaderError, Result};
use crate::schema::{InferenceConfig, TableSchema};
use csv::{Reader, ReaderBuilder, StringRecord};
use std::fs::File;
use std::path::Path;

/// CSV parser with streaming capability
pub struct CsvParser {
    reader: Reader<File>,
    headers: StringRecord,
    delimiter: u8,
}

impl CsvParser {
    /// Create a new CSV parser from a file path
    pub fn from_path<P: AsRef<Path>>(path: P, delimiter: u8, has_headers: bool) -> Result<Self> {
        let file = File::open(&path).map_err(|_| {
            LoaderError::FileNotFound(path.as_ref().display().to_string())
        })?;

        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(has_headers)
            .flexible(false) // Enforce consistent column count
            .from_reader(file);

        let headers = if has_headers {
            reader.headers()?.clone()
        } else {
            // Generate default column names: col_0, col_1, etc.
            let first_record = reader.records().next()
                .ok_or(LoaderError::EmptyFile)??;

            let default_headers: Vec<String> = (0..first_record.len())
                .map(|i| format!("col_{}", i))
                .collect();

            StringRecord::from(default_headers)
        };

        Ok(Self {
            reader,
            headers,
            delimiter,
        })
    }

    /// Get column headers
    pub fn headers(&self) -> Vec<String> {
        self.headers.iter().map(String::from).collect()
    }

    /// Infer schema by sampling rows
    pub fn infer_schema(&mut self, table_name: String, config: &InferenceConfig) -> Result<TableSchema> {
        let mut schema = TableSchema::new(table_name, self.headers());

        let mut count = 0;
        for result in self.reader.records() {
            if count >= config.sample_size {
                break;
            }

            let record = result?;
            let row: Vec<String> = record.iter().map(String::from).collect();

            schema.update_row(&row)?;
            count += 1;
        }

        if count == 0 {
            return Err(LoaderError::EmptyFile);
        }

        schema.finalize();
        Ok(schema)
    }

    /// Get an iterator over records
    pub fn records(&mut self) -> CsvRecordIterator {
        CsvRecordIterator {
            reader: &mut self.reader,
        }
    }

    /// Reset reader to beginning (requires re-opening file)
    pub fn reset<P: AsRef<Path>>(&mut self, path: P, has_headers: bool) -> Result<()> {
        let file = File::open(&path).map_err(|_| {
            LoaderError::FileNotFound(path.as_ref().display().to_string())
        })?;

        self.reader = ReaderBuilder::new()
            .delimiter(self.delimiter)
            .has_headers(has_headers)
            .flexible(false)
            .from_reader(file);

        // Skip headers if present
        if has_headers {
            self.reader.headers()?;
        }

        Ok(())
    }
}

/// Iterator over CSV records
pub struct CsvRecordIterator<'a> {
    reader: &'a mut Reader<File>,
}

impl<'a> Iterator for CsvRecordIterator<'a> {
    type Item = Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.records().next() {
            Some(Ok(record)) => {
                let row: Vec<String> = record.iter().map(String::from).collect();
                Some(Ok(row))
            }
            Some(Err(e)) => Some(Err(e.into())),
            None => None,
        }
    }
}

/// Parse delimiter from string
pub fn parse_delimiter(s: &str) -> Result<u8> {
    match s {
        "," => Ok(b','),
        "\\t" | "tab" => Ok(b'\t'),
        "|" => Ok(b'|'),
        ";" => Ok(b';'),
        s if s.len() == 1 => Ok(s.as_bytes()[0]),
        _ => Err(LoaderError::ConfigError(format!("Invalid delimiter: {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_parse_csv_with_headers() {
        let file = create_test_csv("name,age,city\nAlice,25,NYC\nBob,30,LA\n");

        let parser = CsvParser::from_path(file.path(), b',', true).unwrap();
        let headers = parser.headers();

        assert_eq!(headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_parse_csv_no_headers() {
        let file = create_test_csv("Alice,25,NYC\nBob,30,LA\n");

        let parser = CsvParser::from_path(file.path(), b',', false).unwrap();
        let headers = parser.headers();

        assert_eq!(headers, vec!["col_0", "col_1", "col_2"]);
    }

    #[test]
    fn test_infer_schema() {
        let file = create_test_csv("name,age,salary\nAlice,25,50000.50\nBob,30,60000.75\n");

        let mut parser = CsvParser::from_path(file.path(), b',', true).unwrap();
        let config = InferenceConfig::new(100, true);
        let schema = parser.infer_schema("users".to_string(), &config).unwrap();

        assert_eq!(schema.columns.len(), 3);
        assert_eq!(schema.columns[0].name, "name");
        assert_eq!(schema.columns[1].name, "age");
        assert_eq!(schema.columns[2].name, "salary");
    }

    #[test]
    fn test_parse_delimiter() {
        assert_eq!(parse_delimiter(",").unwrap(), b',');
        assert_eq!(parse_delimiter("\\t").unwrap(), b'\t');
        assert_eq!(parse_delimiter("tab").unwrap(), b'\t');
        assert_eq!(parse_delimiter("|").unwrap(), b'|');
        assert_eq!(parse_delimiter(";").unwrap(), b';');
    }

    #[test]
    fn test_empty_file_error() {
        let file = create_test_csv("");

        let result = CsvParser::from_path(file.path(), b',', true);
        assert!(result.is_err());
    }
}
