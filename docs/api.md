# API Reference

IFC Inspector can be used as a Rust library for parsing IFC files programmatically.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ifc-inspector = "1.0"
```

## Quick Start

```rust
use ifc_inspector::parser::parse_ifc_file;

fn main() -> color_eyre::Result<()> {
    let project = parse_ifc_file("model.ifc")?;

    println!("Project: {}", project.name);
    println!("Schema: {}", project.schema);
    println!("Total types: {}", project.total_types());
    println!("Total elements: {}", project.total_elements());

    Ok(())
}
```

## Core Types

### `IfcProject`

The root container for parsed IFC data.

```rust
pub struct IfcProject {
    pub name: String,
    pub schema: String,
    pub file_path: String,
    pub categories: Vec<Category>,
    pub storeys: Vec<Storey>,
    pub elements: HashMap<u64, Element>,
    pub element_to_storey: HashMap<u64, u64>,
    pub element_properties: HashMap<u64, HashMap<String, String>>,
    pub instance_global_ids: HashMap<u64, String>,
}
```

#### Methods

```rust
impl IfcProject {
    /// Total number of element instances
    pub fn total_elements(&self) -> usize;

    /// Total number of element types
    pub fn total_types(&self) -> usize;
}
```

#### Example

```rust
let project = parse_ifc_file("model.ifc")?;

println!("Name: {}", project.name);
println!("Schema: {}", project.schema);  // "IFC2X3" or "IFC4"
println!("Elements: {}", project.total_elements());
println!("Types: {}", project.total_types());
println!("Storeys: {}", project.storeys.len());
```

### `Category`

Groups element types by BIM category.

```rust
pub struct Category {
    pub name: String,
    pub is_priority: bool,
    pub types: Vec<ElementType>,
    pub total_count: usize,
}
```

#### Example

```rust
for category in &project.categories {
    println!("{}: {} types, {} instances",
        category.name,
        category.types.len(),
        category.total_count
    );
}
```

### `ElementType`

Represents a specific element type (e.g., "Basic Wall 200mm").

```rust
pub struct ElementType {
    pub id: u64,
    pub global_id: String,
    pub name: String,
    pub category: String,
    pub instance_count: usize,
    pub instance_ids: Vec<u64>,
    pub properties: HashMap<String, String>,
}
```

#### Example

```rust
for category in &project.categories {
    for element_type in &category.types {
        println!("{}: {} instances",
            element_type.name,
            element_type.instance_count
        );

        // Access properties
        if let Some(width) = element_type.properties.get("Width") {
            println!("  Width: {}", width);
        }

        // Access instance IDs
        for id in &element_type.instance_ids {
            println!("  Instance: #{}", id);
        }
    }
}
```

### `Element`

Represents an individual element instance (e.g., a specific wall segment).

```rust
pub struct Element {
    pub id: u64,
    pub global_id: String,
    pub name: String,
    pub tag: Option<String>,
    pub type_id: Option<u64>,
    pub storey_id: Option<u64>,
    pub properties: HashMap<String, String>,
}
```

#### Example

```rust
// Access elements directly by STEP entity ID
let element_id: u64 = 12345;

if let Some(element) = project.elements.get(&element_id) {
    println!("Element: {} ({})", element.name, element.global_id);
    if let Some(tag) = &element.tag {
        println!("Tag: {}", tag);
    }
    for (key, value) in &element.properties {
        println!("  {}: {}", key, value);
    }
}
```

### `Storey`

Represents a building storey.

```rust
pub struct Storey {
    pub id: u64,
    pub name: String,
    pub elevation: f64,
    pub element_count: usize,
}
```

#### Example

```rust
// Storeys are sorted by elevation (highest first)
for storey in &project.storeys {
    println!("{}: elevation {:.2}m, {} elements",
        storey.name,
        storey.elevation / 1000.0,  // mm to m
        storey.element_count
    );
}
```

## Parser Module

### `parse_ifc_file`

Main entry point for parsing IFC files.

```rust
pub fn parse_ifc_file<P: AsRef<Path>>(path: P) -> Result<IfcProject, ParseError>;
```

#### Errors

- `ParseError::FileRead` - File cannot be read
- `ParseError::InvalidStep` - Invalid STEP format

#### Example

```rust
use ifc_inspector::parser::parse_ifc_file;
use ifc_inspector::error::ParseError;

match parse_ifc_file("model.ifc") {
    Ok(project) => {
        println!("Parsed: {}", project.name);
    }
    Err(ParseError::FileRead { path, source }) => {
        eprintln!("Cannot read {}: {}", path.display(), source);
    }
    Err(ParseError::InvalidStep { message }) => {
        eprintln!("Invalid IFC: {}", message);
    }
}
```

## Export Module

### `export_csv`

Export type summary to CSV.

```rust
pub fn export_csv<P: AsRef<Path>>(
    project: &IfcProject,
    path: P
) -> Result<(), ExportError>;
```

#### Example

```rust
use ifc_inspector::export::export_csv;

let project = parse_ifc_file("model.ifc")?;
export_csv(&project, "output.csv")?;
```

### `export_json`

Export full project data to JSON.

```rust
pub fn export_json<P: AsRef<Path>>(
    project: &IfcProject,
    path: P
) -> Result<(), ExportError>;
```

#### Example

```rust
use ifc_inspector::export::export_json;

let project = parse_ifc_file("model.ifc")?;
export_json(&project, "output.json")?;
```

## Low-level STEP API

The parser module also exposes low-level STEP/ISO-10303 types for advanced use cases.

### `StepFile`

```rust
pub struct StepFile {
    pub entities: HashMap<u64, StepEntity>,
    pub schema: String,
}
```

#### Methods

```rust
impl StepFile {
    /// Parse a STEP file from string content
    pub fn parse(content: &str) -> Result<Self, ParseError>;

    /// Get a single entity by its STEP ID (#123)
    pub fn get_entity(&self, id: u64) -> Option<&StepEntity>;

    /// Get all entities of a given type (e.g., "IFCWALL")
    pub fn get_entities_by_type(&self, entity_type: &str) -> Vec<&StepEntity>;
}
```

### `StepEntity`

```rust
pub struct StepEntity {
    pub id: u64,
    pub entity_type: String,
    pub values: Vec<StepValue>,
}
```

### `StepValue`

```rust
pub enum StepValue {
    String(String),       // 'text'
    Real(f64),            // 3.14
    Integer(i64),         // 42
    Boolean(bool),        // .T. or .F.
    Enum(String),         // .ELEMENT.
    Reference(u64),       // #123
    List(Vec<StepValue>), // (item1,item2)
    Null,                 // $
    Derived,              // *
}
```

#### Example

```rust
use ifc_inspector::parser::{StepFile, StepValue};

let content = std::fs::read_to_string("model.ifc")?;
let step_file = StepFile::parse(&content)?;

// Find all wall entities
let walls = step_file.get_entities_by_type("IFCWALL");
for wall in walls {
    println!("Wall #{}: {:?}", wall.id, wall.values);
}
```

## Error Types

### `ParseError`

```rust
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("invalid STEP format: {message}")]
    InvalidStep { message: String },
}
```

### `ExportError`

```rust
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("failed to create file '{path}': {source}")]
    FileCreate {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to write data: {message}")]
    WriteError { message: String },

    #[error("JSON serialization failed: {source}")]
    JsonSerialize {
        #[from]
        source: serde_json::Error,
    },

    #[error("CSV write failed: {source}")]
    CsvWrite {
        #[from]
        source: csv::Error,
    },
}
```

## Advanced Usage

### Filtering by Storey

```rust
let project = parse_ifc_file("model.ifc")?;

// Find storey by name
let level_1 = project.storeys.iter()
    .find(|s| s.name.contains("Level 1"))
    .expect("Level 1 not found");

// Count elements on this storey
let count: usize = project.element_to_storey.values()
    .filter(|&&storey_id| storey_id == level_1.id)
    .count();

println!("Elements on {}: {}", level_1.name, count);
```

### Finding Types by Category

```rust
let walls = project.categories.iter()
    .find(|c| c.name == "Walls")
    .expect("No walls found");

println!("Wall types:");
for wall_type in &walls.types {
    println!("  {} ({} instances)", wall_type.name, wall_type.instance_count);
}
```

### Accessing Instance Properties

```rust
// Get properties for a specific instance
let instance_id: u64 = 12345;

if let Some(props) = project.element_properties.get(&instance_id) {
    for (key, value) in props {
        println!("{}: {}", key, value);
    }
}

// Get GlobalId for Revit lookup
if let Some(global_id) = project.instance_global_ids.get(&instance_id) {
    println!("GlobalId: {}", global_id);
}
```

### Custom Aggregation

```rust
// Sum all wall areas
let total_area: f64 = project.categories.iter()
    .find(|c| c.name == "Walls")
    .map(|walls| {
        walls.types.iter()
            .filter_map(|t| t.properties.get("Area"))
            .filter_map(|area| {
                area.trim_end_matches(" m²")
                    .parse::<f64>()
                    .ok()
            })
            .sum()
    })
    .unwrap_or(0.0);

println!("Total wall area: {:.2} m²", total_area);
```

## Feature Flags

Currently, IFC Inspector does not have optional feature flags. All functionality is included by default.

## Thread Safety

`IfcProject` and all its contained types are `Send` and can be shared between threads. The parsing operation is single-threaded.

## Memory Usage

Memory usage scales linearly with file size. Approximately:
- ~2.5-3x the IFC file size during parsing
- ~1.5x the IFC file size after parsing (entities + maps)

For very large files (>100MB), consider streaming approaches or processing in chunks.
