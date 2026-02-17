# Usage Guide

Detailed guide for using IFC Inspector effectively.

## Command Line Interface

### Basic Usage

```bash
ifc-inspector <FILE> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<FILE>` | Path to IFC file (required) |

### Options

| Option | Description |
|--------|-------------|
| `--csv <FILE>` | Export type summary to CSV |
| `--json <FILE>` | Export full data to JSON |
| `--help` | Show help message |
| `--version` | Show version |

### Examples

```bash
# Open in interactive mode
ifc-inspector model.ifc

# Handle paths with spaces
ifc-inspector "My Project/model.ifc"

# Export only (no TUI)
ifc-inspector model.ifc --csv output.csv

# Export both formats
ifc-inspector model.ifc --csv types.csv --json full.json
```

## Interactive Mode

### Dashboard Layout

```
┌──────────────────────────────────────────────────────────┐
│ IFC Inspector | Project Name | 42 types | 1234 elements  │
├─────────┬───────────┬────────────────────────────────────┤
│ Levels  │ Categories│ Type Name                 Instances│
│         │           │                                    │
│ ► All   │ ► Walls   │ Basic Wall 200mm              45│
│   +3.0m │   Doors   │ Basic Wall 300mm              23│
│   +0.0m │   Windows │ Curtain Wall                  12│
│   -3.0m │   Other   │                                    │
├─────────┴───────────┴────────────────────────────────────┤
│ ←→ Category | ↑↓ Type | Enter Details | q Quit           │
└──────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

#### Dashboard View

| Key | Action |
|-----|--------|
| `←` `→` or `h` `l` | Switch panel (Levels → Categories → Types) |
| `↑` `↓` or `j` `k` | Navigate within active panel |
| `Enter` | Open type details (when on Types panel) |
| `q` | Quit |

#### Type Detail View

| Key | Action |
|-----|--------|
| `↑` `↓` | Scroll properties |
| `←` `→` | Previous/next instance (with wrap-around) |
| `Enter` | Open instance browser |
| `Esc` | Back to dashboard |

#### Instance Browser

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate instances |
| `Esc` or `Enter` | Back to type detail |
| `q` | Quit |

### Panel Navigation

The dashboard has three panels. Use arrow keys or `h`/`l` to switch:

1. **Levels** (left) - Filter by building storey
   - "All" shows everything
   - Select a level to filter types and counts

2. **Categories** (center) - Element categories
   - Priority categories (Walls, Doors, Windows) shown first
   - Count in parentheses reflects level filter

3. **Types** (right) - Element types in selected category
   - Instance count shown per type
   - Press Enter to see details

### Level Filtering

When you select a specific level:
- Categories show only elements on that level
- Type list filters to types with instances on that level
- Instance counts update accordingly

This helps answer questions like "How many wall types on Level 1?"

### Type Detail View

```
┌──────────────────────────────────────────────────────────┐
│ Type: Basic Wall 200mm                                   │
├──────────────────────────────────────────────────────────┤
│ Walls  |  Level: Level 1  |  Instance: 1/45 (#234)      │
├──────────────────────────────────────────────────────────┤
│ Property                    Value                        │
│ ── Numeric ──                                            │
│ Width                       200.00 mm                    │
│ Height                      2800.00 mm                   │
│ Area                        5.60 m²                      │
│ ── Text ──                                               │
│ Material                    Concrete                     │
│ Fire Rating                 REI 60                       │
├──────────────────────────────────────────────────────────┤
│ Esc Back | ↑↓ Scroll | ←→ Instance | Enter Browse       │
└──────────────────────────────────────────────────────────┘
```

**Features:**
- Navigate between instances with ←→ (wraps around)
- Properties update for each instance
- GlobalId shown for Revit Schedule lookup
- Numeric properties listed first, then text

### Instance Browser

Press Enter in Type Detail to see all instances:

```
┌──────────────────────────────────────────────────────────┐
│ Instances of: Basic Wall 200mm (45)                      │
├──────────────────────────────────────────────────────────┤
│ #   Level      ID      GlobalId          Length   Area   │
│ 1   Level 0    #234    2Xk9jP...        4500mm   12.6m² │
│ 2   Level 0    #456    3Yk8kQ...        3200mm   8.96m² │
│ 3   Level 1    #789    4Zl7lR...        2800mm   7.84m² │
├──────────────────────────────────────────────────────────┤
│ Esc Back to Type | ↑↓ Navigate | q Quit                  │
└──────────────────────────────────────────────────────────┘
```

**Features:**
- Sorted by elevation (lowest level first)
- Dynamic columns based on available properties
- GlobalId for Revit lookup

## Export Formats

### CSV Export

Simple tabular format for Excel:

```csv
Category,Type Name,Instance Count,Global ID
Walls,Basic Wall 200mm,45,2Xk9jPqR5E9QhKlMnOpQr
Walls,Basic Wall 300mm,23,3Yk8kQrS6F0RiLmNoQpRs
Doors,Single Swing Door,12,4Zl7lRsT7G1SjMnOpRqSt
```

**Use cases:**
- Quick import to Excel for pivot tables
- Summary reports
- BIM Execution Plan compliance checks

### JSON Export

Full hierarchical data with all properties:

```json
{
  "name": "School Project",
  "schema": "IFC4",
  "file_path": "model.ifc",
  "categories": [
    {
      "name": "Walls",
      "is_priority": true,
      "types": [
        {
          "id": 1234,
          "global_id": "2Xk9jPqR5E9QhKlMnOpQr",
          "name": "Basic Wall 200mm",
          "category": "Walls",
          "instance_count": 45,
          "instance_ids": [234, 456, 789],
          "properties": {
            "Width": "200.00 mm",
            "Fire Rating": "REI 60"
          }
        }
      ],
      "total_count": 68
    }
  ],
  "storeys": [
    {
      "id": 100,
      "name": "Level 0",
      "elevation": 0.0,
      "element_count": 234
    }
  ]
}
```

**Use cases:**
- Power BI dashboards
- Custom scripts and tools
- Data analysis with Python/pandas
- Integration with other BIM workflows

## Supported IFC Schemas

- **IFC2X3** — widely used for Revit/ArchiCAD exports
- **IFC4** — newer standard with extended property support

The parser auto-detects the schema from the file header.

## Tips and Tricks

### Quick Audit Workflow

1. Open file: `ifc-inspector model.ifc`
2. Check total counts in header
3. Navigate to each priority category
4. Look for unusual type counts
5. Drill into suspicious types for details

### Finding Specific Elements

Use GlobalId from Instance Browser:
1. Navigate to the type
2. Press Enter for details
3. Use ←→ to find the instance
4. Copy GlobalId
5. In Revit: Schedule → Filter by GlobalId

### Comparing Models

Export both models to JSON, then use diff tools:

```bash
ifc-inspector model_v1.ifc --json v1.json
ifc-inspector model_v2.ifc --json v2.json
diff v1.json v2.json
```

### Batch Processing

Process multiple files with a shell loop:

```bash
for f in *.ifc; do
    ifc-inspector "$f" --csv "${f%.ifc}.csv"
done
```

## Troubleshooting

### File Won't Open

1. Check file exists and is readable
2. Verify it's a valid IFC file (starts with "ISO-10303-21")
3. Try with a smaller test file

### Unicode Characters Display Wrong

- Ensure terminal supports UTF-8
- On Windows, try Windows Terminal instead of cmd.exe

### Slow Performance

- Files >100MB may take several seconds to parse
- Close other memory-intensive applications
- Consider SSD storage for faster I/O

### Missing Elements

- Check if element type is in supported list
- Some proprietary BIM authoring tools use non-standard entities
- Open an issue with sample file
