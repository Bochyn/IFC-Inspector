# IFC Inspector

**A blazing-fast terminal UI for exploring IFC files without heavy BIM software.**

<p align="center">
  <img src="docs/demo.gif" alt="IFC Inspector Demo" width="800">
</p>

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/Bochyn/ifc-inspector/ci.yml?branch=main)](https://github.com/Bochyn/ifc-inspector/actions)

## The Problem

Inspecting IFC files typically requires launching Revit, Solibri, or uploading to online viewers. This takes minutes when you just need a quick answer:

- How many wall types are in this model?
- Do all doors have proper parameters assigned?
- What's the instance count per type?

**IFC Inspector** opens any IFC file in under 2 seconds and displays everything you need for a quick audit.

## Features

- **Three-panel dashboard** - Levels, Categories, Types with live filtering
- **Level-based filtering** - Select a storey to see only elements on that level
- **Type details** - Property sets (Pset_) with numeric aggregation
- **Instance browser** - Navigate individual elements with GlobalId for Revit lookup
- **Export** - CSV for Excel, JSON for Power BI and custom workflows
- **Unicode support** - Handles Polish, German, French characters in type names
- **Custom STEP parser** - No external IFC libraries, pure Rust

## Who Is This For?

| Role | Use Case |
|------|----------|
| **BIM Coordinators** | Quick model validation before exchange |
| **Architects** | Verify IFC exports from Revit/ArchiCAD |
| **Quantity Surveyors** | Fast element counts and type summaries |
| **Developers** | Parse IFC data for custom tools |

## Installation

### From crates.io

```bash
cargo install ifc-inspector
```

### From source

```bash
git clone https://github.com/Bochyn/ifc-inspector
cd ifc-inspector
cargo build --release
# Binary at ./target/release/ifc-inspector
```

## Quick Start

```bash
# Interactive TUI mode
ifc-inspector model.ifc

# Export to CSV (type summary)
ifc-inspector model.ifc --csv types.csv

# Export to JSON (full data with properties)
ifc-inspector model.ifc --json data.json

# Both exports at once
ifc-inspector model.ifc --csv types.csv --json data.json
```

## Keyboard Navigation

### Dashboard View

| Key | Action |
|-----|--------|
| `←` `→` or `h` `l` | Switch panel (Levels → Categories → Types) |
| `↑` `↓` or `j` `k` | Navigate within active panel |
| `Enter` | Open type details (when on Types panel) |
| `q` | Quit |

### Type Detail View

| Key | Action |
|-----|--------|
| `↑` `↓` | Scroll properties |
| `←` `→` | Previous/next instance (with wrap-around) |
| `Enter` | Open instance browser |
| `Esc` | Back to dashboard |

### Instance Browser

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate instances |
| `Esc` or `Enter` | Back to type detail |
| `q` | Quit |

## Supported Elements

**Priority Categories:**
- Walls (`IfcWall`, `IfcWallStandardCase`)
- Doors (`IfcDoor`)
- Windows (`IfcWindow`)
- Furniture (`IfcFurnishingElement`)
- Fixtures (`IfcSanitaryTerminal`, `IfcFlowTerminal`, `IfcFlowFixture`)

**Other Categories:**
- Slabs, Columns, Beams, Stairs, Railings, Roofs, Coverings, Curtain Walls

**Schemas:**
- IFC2X3
- IFC4

## Architecture

```
ifc-inspector/
├── src/
│   ├── main.rs          # CLI entry point (clap)
│   ├── lib.rs           # Library exports
│   ├── error.rs         # Typed errors (thiserror)
│   ├── parser/
│   │   ├── step.rs      # STEP/ISO-10303 parser
│   │   └── ifc.rs       # IFC entity extraction
│   ├── model/
│   │   ├── project.rs   # IfcProject, Category, Storey
│   │   ├── element.rs   # Element instances
│   │   └── element_type.rs  # ElementType with properties
│   ├── export/
│   │   ├── csv.rs       # CSV export
│   │   └── json.rs      # JSON export
│   └── ui/
│       ├── app.rs       # Application state machine
│       └── dashboard.rs # Ratatui rendering
```

### Design Decisions

1. **Custom STEP Parser** - No dependency on `ifc-rs` or other IFC libraries. The parser handles only what's needed for inspection: entities, references, property sets.

2. **Memory-Mapped Relationships** - `element_to_storey`, `type_to_instances` maps enable O(1) lookups during UI interaction.

3. **Brandbook Colors** - UI uses a consistent color palette for professional appearance.

4. **Zero Unsafe Code** - `#![forbid(unsafe_code)]` ensures memory safety.

## Library Usage

IFC Inspector can be used as a Rust library:

```rust
use ifc_inspector::parser::parse_ifc_file;

fn main() -> color_eyre::Result<()> {
    let project = parse_ifc_file("model.ifc")?;

    println!("Project: {}", project.name);
    println!("Schema: {}", project.schema);
    println!("Total types: {}", project.total_types());
    println!("Total elements: {}", project.total_elements());

    for category in &project.categories {
        println!("\n{}:", category.name);
        for element_type in &category.types {
            println!("  {} - {} instances",
                element_type.name,
                element_type.instance_count
            );
        }
    }

    Ok(())
}
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2021 Edition |
| CLI | [clap](https://docs.rs/clap) with derive macros |
| TUI | [ratatui](https://ratatui.rs/) + crossterm |
| Error Handling | [color-eyre](https://docs.rs/color-eyre) + [thiserror](https://docs.rs/thiserror) |
| Serialization | [serde](https://serde.rs/) + serde_json |
| CSV | [csv](https://docs.rs/csv) crate |
| Linting | Clippy pedantic + custom rules |

## Performance

Tested on a 45MB IFC file (large school project):
- **Parse time**: ~1.8s
- **Memory usage**: ~120MB peak
- **UI responsiveness**: Instant (<16ms frame time)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests and lints (`cargo test && cargo clippy`)
4. Commit with clear messages
5. Open a Pull Request


## License

MIT License - see [LICENSE](LICENSE) for details.

## Author

**Mateusz Bochynski**
Architect | Developer | Creative Technologist

| [GitHub](https://github.com/Bochyn) | [LinkedIn](www.linkedin.com/in/mateusz-bochyński-711153234)

---

Built with Rust and passion for better BIM tools.
