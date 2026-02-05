//! # IFC Inspector
//!
//! A terminal-based IFC file inspector for browsing BIM families and types.
//!
//! ## Features
//!
//! - Parse IFC files (IFC2x3 and IFC4 schemas)
//! - Browse element types organized by category
//! - Filter by building storey
//! - Export to CSV and JSON
//!
//! ## Example
//!
//! ```no_run
//! use ifc_inspector::parser::parse_ifc_file;
//!
//! let project = parse_ifc_file("model.ifc").expect("Failed to parse");
//! println!("Project: {}", project.name);
//! println!("Types: {}", project.total_types());
//! ```

pub mod error;
pub mod export;
pub mod model;
pub mod parser;
pub mod ui;
