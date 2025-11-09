//! Error types for CSV-SQL Loader

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid table name: {0}")]
    InvalidTableName(String),

    #[error("Schema inference failed: {0}")]
    SchemaInferenceError(String),

    #[error("Type conversion error: {0}")]
    TypeConversionError(String),

    #[error("Batch processing failed after {retries} retries: {message}")]
    BatchError { retries: usize, message: String },

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Empty CSV file")]
    EmptyFile,
}

pub type Result<T> = std::result::Result<T, LoaderError>;
