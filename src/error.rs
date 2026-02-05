//! Error types for IFC Inspector.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when parsing IFC files.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Failed to read the IFC file from disk.
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    /// The STEP format is invalid or malformed.
    #[error("invalid STEP format: {message}")]
    InvalidStep { message: String },
}

/// Errors that can occur when exporting data.
#[derive(Debug, Error)]
pub enum ExportError {
    /// Failed to create the output file.
    #[error("failed to create file '{path}': {source}")]
    FileCreate {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to write data to the file.
    #[error("failed to write data: {message}")]
    WriteError { message: String },

    /// Failed to serialize data to JSON.
    #[error("JSON serialization failed: {source}")]
    JsonSerialize {
        #[from]
        source: serde_json::Error,
    },

    /// Failed to write CSV data.
    #[error("CSV write failed: {source}")]
    CsvWrite {
        #[from]
        source: csv::Error,
    },
}
