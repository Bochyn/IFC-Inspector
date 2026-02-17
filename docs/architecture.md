# Architecture

This document explains the internal architecture of IFC Inspector.

## Overview

IFC Inspector follows a layered architecture separating concerns:

```
┌─────────────────────────────────────────────────┐
│                     CLI (main.rs)               │
│              clap argument parsing              │
└─────────────────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Parser    │  │   Export    │  │     UI      │
│ STEP + IFC  │  │ CSV + JSON  │  │  Ratatui    │
└─────────────┘  └─────────────┘  └─────────────┘
        │               │               │
        └───────────────┼───────────────┘
                        ▼
              ┌─────────────────┐
              │     Model       │
              │ IfcProject, etc │
              └─────────────────┘
```

## Module Breakdown

### Parser Layer (`src/parser/`)

#### `step.rs` - STEP/ISO-10303 Parser

The STEP file format is a text-based representation defined by ISO-10303. IFC files use this format.

**Key structures:**

```rust
pub enum StepValue {
    String(String),      // 'text'
    Real(f64),           // 3.14
    Integer(i64),        // 42
    Boolean(bool),       // .T. or .F.
    Enum(String),        // .ELEMENT.
    Reference(u64),      // #123
    List(Vec<StepValue>), // (item1,item2)
    Null,                // $
    Derived,             // *
}

pub struct StepEntity {
    pub id: u64,
    pub entity_type: String,  // e.g., "IFCWALL"
    pub values: Vec<StepValue>,
}

pub struct StepFile {
    pub entities: HashMap<u64, StepEntity>,
    pub schema: String,  // "IFC2X3" or "IFC4"
}
```

**Unicode handling:**

The parser decodes STEP-encoded Unicode sequences:
- `\X2\00D3\X0\` → Unicode BMP character (Polish Ó)
- `\X\E9` → ISO 8859-1 (accented e)
- `''` → Escaped apostrophe

#### `ifc.rs` - IFC Entity Extraction

Extracts BIM-specific entities from the generic STEP structure:

1. **Project metadata** - Name, schema version from `IFCPROJECT`
2. **Spatial structure** - Building storeys from `IFCBUILDINGSTOREY`
3. **Element types** - Wall types, door styles from `IFCWALLTYPE`, `IFCDOORSTYLE`, etc.
4. **Type-instance relationships** - Via `IFCRELDEFINESBYTYPE`
5. **Spatial containment** - Element to storey via `IFCRELCONTAINEDINSPATIALSTRUCTURE`
6. **Property sets** - Via `IFCPROPERTYSET` and `IFCRELDEFINESBYPROPERTIES`

**Category mapping:**

```rust
const PRIORITY_CATEGORIES: &[(&str, &str)] = &[
    ("IFCWALL", "Walls"),
    ("IFCDOOR", "Doors"),
    ("IFCWINDOW", "Windows"),
    ("IFCFURNISHINGELEMENT", "Furniture"),
    ("IFCFLOWFIXTURE", "Fixtures"),
    ("IFCSANITARYTERMINAL", "Fixtures"),
    ("IFCFLOWTERMINAL", "Fixtures"),
];
```

### Model Layer (`src/model/`)

Domain objects representing parsed IFC data:

#### `IfcProject`

Root container for all extracted data:

```rust
pub struct IfcProject {
    pub name: String,
    pub schema: String,
    pub file_path: String,
    pub categories: Vec<Category>,
    pub storeys: Vec<Storey>,
    pub elements: HashMap<u64, Element>,           // all parsed elements
    pub element_to_storey: HashMap<u64, u64>,      // O(1) lookups
    pub element_properties: HashMap<u64, HashMap<String, String>>,
    pub instance_global_ids: HashMap<u64, String>,
}
```

#### `Category`

Groups element types by BIM category:

```rust
pub struct Category {
    pub name: String,           // "Walls", "Doors", etc.
    pub is_priority: bool,      // Shown first in UI
    pub types: Vec<ElementType>,
    pub total_count: usize,     // Total instances
}
```

#### `ElementType`

Represents a specific type (e.g., "Basic Wall 200mm"):

```rust
pub struct ElementType {
    pub id: u64,
    pub global_id: String,      // For Revit lookup
    pub name: String,
    pub category: String,
    pub instance_count: usize,
    pub instance_ids: Vec<u64>, // For instance browser
    pub properties: HashMap<String, String>,
}
```

### UI Layer (`src/ui/`)

#### `app.rs` - Application State

State machine managing views and navigation:

```rust
pub enum View {
    Dashboard,
    TypeDetail,
    InstanceBrowser,
}

pub enum FocusPanel {
    Levels,
    Categories,
    Types,
}

pub struct App {
    pub project: IfcProject,
    pub step_file: Option<StepFile>,
    pub view: View,
    pub focus_panel: FocusPanel,
    pub selected_category: usize,
    pub selected_type: usize,
    pub selected_instance: usize,
    pub selected_level: usize,      // 0 = "All", 1+ = storey index
    pub types_scroll_offset: usize,
    pub property_scroll_offset: usize,
    pub instances_scroll_offset: usize,
    pub should_quit: bool,
}
```

#### `dashboard.rs` - Rendering

Ratatui-based rendering with brandbook colors:

```rust
const BRAND_DARK: Color = Color::Rgb(0x1F, 0x2F, 0x3C);
const BRAND_SELECT_BG: Color = Color::Rgb(0xC3, 0xD3, 0xE0);
const BRAND_GREEN: Color = Color::Rgb(0x82, 0x9A, 0x68);
const BRAND_ORANGE: Color = Color::Rgb(0x9E, 0x68, 0x3C);
```

### Export Layer (`src/export/`)

#### `csv.rs`

Exports type summary:

```csv
Category,Type Name,Instance Count,Global ID
Walls,Basic Wall 200mm,45,2Xk9j...
```

#### `json.rs`

Exports full project structure with all properties (uses serde serialization).

### Error Handling (`src/error.rs`)

Typed errors using `thiserror`:

```rust
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to read file '{path}': {source}")]
    FileRead { path: PathBuf, source: std::io::Error },

    #[error("invalid STEP format: {message}")]
    InvalidStep { message: String },
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("failed to create file '{path}': {source}")]
    FileCreate { path: PathBuf, source: std::io::Error },
    // ...
}
```

## Data Flow

```
IFC File
    │
    ▼ (std::fs::read_to_string)
Raw String
    │
    ▼ (StepFile::parse)
StepFile { entities: HashMap<u64, StepEntity> }
    │
    ▼ (parse_ifc_file)
IfcProject {
    categories: [Category { types: [ElementType] }],
    storeys: [Storey],
    element_to_storey: HashMap,
    ...
}
    │
    ├──▶ App (TUI) ──▶ Terminal
    │
    ├──▶ export_csv() ──▶ CSV File
    │
    └──▶ export_json() ──▶ JSON File
```

## Performance Considerations

1. **Single-pass parsing** - STEP file is parsed line-by-line, entities stored in HashMap

2. **Pre-computed relationships** - Type-instance and element-storey maps built during parse, not queried on demand

3. **Lazy property loading** - Instance properties fetched only when viewing Type Detail

4. **Efficient filtering** - Level filtering uses pre-built `element_to_storey` map

## Testing Strategy

- **Unit tests** in each module
- **Integration tests** with sample IFC files
- **Clippy pedantic** enforced in CI

## Future Architecture

Planned improvements:

1. **Streaming parser** - For very large files (>100MB)
2. **Parallel entity extraction** - Using rayon
3. **Memory-mapped files** - Reduce memory footprint
4. **Plugin system** - Custom exporters via WASM
