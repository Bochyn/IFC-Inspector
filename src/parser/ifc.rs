use crate::error::ParseError;
use crate::model::{Category, ElementType, IfcProject, Storey};
use crate::parser::step::{StepFile, StepValue};
use std::collections::HashMap;
use std::path::Path;

const PRIORITY_CATEGORIES: &[(&str, &str)] = &[
    ("IFCWALL", "Walls"),
    ("IFCDOOR", "Doors"),
    ("IFCWINDOW", "Windows"),
    ("IFCFURNISHINGELEMENT", "Furniture"),
    ("IFCFLOWFIXTURE", "Fixtures"),
    ("IFCSANITARYTERMINAL", "Fixtures"),
    ("IFCFLOWTERMINAL", "Fixtures"), // NOWE - urządzenia przepływowe
];

// Maps element entity to its type entity (IFC4 and IFC2X3 compatible)
const ELEMENT_TYPES: &[(&str, &[&str])] = &[
    // Priority elements - IFC4 types first, then IFC2X3 styles
    ("IFCWALL", &["IFCWALLTYPE"]),
    ("IFCWALLSTANDARDCASE", &["IFCWALLTYPE"]),
    ("IFCDOOR", &["IFCDOORTYPE", "IFCDOORSTYLE"]),
    ("IFCWINDOW", &["IFCWINDOWTYPE", "IFCWINDOWSTYLE"]),
    ("IFCFURNISHINGELEMENT", &["IFCFURNITURETYPE"]),
    ("IFCFLOWFIXTURE", &["IFCFLOWTERMINALTYPE"]),
    ("IFCSANITARYTERMINAL", &["IFCSANITARYTERMINALTYPE"]),
    // Other elements
    ("IFCSLAB", &["IFCSLABTYPE"]),
    ("IFCCOLUMN", &["IFCCOLUMNTYPE"]),
    ("IFCBEAM", &["IFCBEAMTYPE"]),
    ("IFCSTAIR", &["IFCSTAIRTYPE", "IFCSTAIRFLIGHTTYPE"]),
    ("IFCRAILING", &["IFCRAILINGTYPE"]),
    ("IFCROOF", &["IFCROOFTYPE"]),
    ("IFCCOVERING", &["IFCCOVERINGTYPE"]),
    ("IFCCURTAINWALL", &["IFCCURTAINWALLTYPE"]),
];

/// Parses an IFC file and extracts project structure.
///
/// Supports both IFC2x3 and IFC4 schemas. Extracts:
/// - Project metadata (name, schema version)
/// - Building storeys with elevations
/// - Element types organized by category (Walls, Doors, Windows, etc.)
/// - Type-to-instance relationships
/// - Property sets
///
/// # Arguments
///
/// * `path` - Path to the IFC file
///
/// # Errors
///
/// Returns [`ParseError::FileRead`] if the file cannot be read.
/// Returns [`ParseError::InvalidStep`] if the STEP format is malformed.
///
/// # Example
///
/// ```no_run
/// use ifc_inspector::parser::parse_ifc_file;
///
/// let project = parse_ifc_file("model.ifc")?;
/// for category in &project.categories {
///     println!("{}: {} types", category.name, category.types.len());
/// }
/// # Ok::<(), ifc_inspector::error::ParseError>(())
/// ```
pub fn parse_ifc_file<P: AsRef<Path>>(path: P) -> Result<IfcProject, ParseError> {
    let content = std::fs::read_to_string(&path).map_err(|source| ParseError::FileRead {
        path: path.as_ref().to_path_buf(),
        source,
    })?;

    let step_file = StepFile::parse(&content)?;

    let project_name = extract_project_name(&step_file);
    let file_path = path.as_ref().to_string_lossy().to_string();

    let mut project = IfcProject::new(project_name, step_file.schema.clone(), file_path);

    // Extract storeys
    project.storeys = extract_storeys(&step_file);

    // Extract spatial containment (element → storey)
    let element_to_storey = extract_spatial_containment(&step_file);

    // Count elements per storey
    let mut storey_counts: HashMap<u64, usize> = HashMap::new();
    for storey_id in element_to_storey.values() {
        *storey_counts.entry(*storey_id).or_insert(0) += 1;
    }
    for storey in &mut project.storeys {
        storey.element_count = storey_counts.get(&storey.id).copied().unwrap_or(0);
    }

    // Store element_to_storey map in project for UI filtering
    project.element_to_storey = element_to_storey;

    // Sort storeys by elevation (descending - roof at top)
    project.storeys.sort_by(|a, b| {
        b.elevation
            .partial_cmp(&a.elevation)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Extract type-instance relationships
    let type_to_instances = extract_type_relationships(&step_file);

    // Extract property sets
    let element_properties = extract_property_sets(&step_file);

    // Build categories
    project.categories = build_categories(&step_file, &type_to_instances, &element_properties);

    // Store element properties for instance-level property lookup
    project.element_properties = element_properties;

    // Extract GlobalIds for all instances
    project.instance_global_ids = extract_instance_global_ids(&step_file, &type_to_instances);

    Ok(project)
}

fn extract_project_name(step_file: &StepFile) -> String {
    step_file
        .get_entities_by_type("IFCPROJECT")
        .first()
        .and_then(|e| e.values.get(2))
        .and_then(|v| match v {
            StepValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Unknown Project".to_string())
}

fn extract_storeys(step_file: &StepFile) -> Vec<Storey> {
    step_file
        .get_entities_by_type("IFCBUILDINGSTOREY")
        .iter()
        .map(|e| {
            let name = e
                .values
                .get(2)
                .and_then(|v| match v {
                    StepValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| format!("Storey #{}", e.id));

            let elevation = e
                .values
                .get(9)
                .and_then(|v| match v {
                    StepValue::Real(f) => Some(*f),
                    _ => None,
                })
                .unwrap_or(0.0);

            Storey {
                id: e.id,
                name,
                elevation,
                element_count: 0,
            }
        })
        .collect()
}

/// Extract element → storey relationships from IFCRELCONTAINEDINSPATIALSTRUCTURE
fn extract_spatial_containment(step_file: &StepFile) -> HashMap<u64, u64> {
    let mut element_to_storey: HashMap<u64, u64> = HashMap::new();

    for rel in step_file.get_entities_by_type("IFCRELCONTAINEDINSPATIALSTRUCTURE") {
        // Index 4 = RelatedElements (list of element refs)
        // Index 5 = RelatingStructure (spatial element ref - storey)
        let elements: Vec<u64> = rel
            .values
            .get(4)
            .and_then(|v| match v {
                StepValue::List(list) => Some(
                    list.iter()
                        .filter_map(|item| match item {
                            StepValue::Reference(id) => Some(*id),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();

        let storey_id = rel.values.get(5).and_then(|v| match v {
            StepValue::Reference(id) => Some(*id),
            _ => None,
        });

        if let Some(sid) = storey_id {
            for elem_id in elements {
                element_to_storey.insert(elem_id, sid);
            }
        }
    }

    element_to_storey
}

fn extract_instance_global_ids(
    step_file: &StepFile,
    type_to_instances: &HashMap<u64, Vec<u64>>,
) -> HashMap<u64, String> {
    let mut global_ids = HashMap::new();

    // Collect all instance IDs
    for instances in type_to_instances.values() {
        for &instance_id in instances {
            if let Some(entity) = step_file.get_entity(instance_id) {
                // GlobalId is always the first attribute (index 0) in IFC entities
                if let Some(StepValue::String(global_id)) = entity.values.first() {
                    global_ids.insert(instance_id, global_id.clone());
                }
            }
        }
    }

    global_ids
}

fn extract_type_relationships(step_file: &StepFile) -> HashMap<u64, Vec<u64>> {
    let mut type_to_instances: HashMap<u64, Vec<u64>> = HashMap::new();

    for rel in step_file.get_entities_by_type("IFCRELDEFINESBYTYPE") {
        // Index 4 = RelatedObjects (list of element refs)
        // Index 5 = RelatingType (type ref)
        let instances: Vec<u64> = rel
            .values
            .get(4)
            .and_then(|v| match v {
                StepValue::List(list) => Some(
                    list.iter()
                        .filter_map(|item| match item {
                            StepValue::Reference(id) => Some(*id),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();

        let type_id = rel.values.get(5).and_then(|v| match v {
            StepValue::Reference(id) => Some(*id),
            _ => None,
        });

        if let Some(tid) = type_id {
            type_to_instances.entry(tid).or_default().extend(instances);
        }
    }

    type_to_instances
}

fn extract_property_sets(step_file: &StepFile) -> HashMap<u64, HashMap<String, String>> {
    let mut element_properties: HashMap<u64, HashMap<String, String>> = HashMap::new();

    // Build property set id -> properties map
    let mut pset_props: HashMap<u64, HashMap<String, String>> = HashMap::new();

    for pset in step_file.get_entities_by_type("IFCPROPERTYSET") {
        let mut props = HashMap::new();

        if let Some(StepValue::List(prop_refs)) = pset.values.get(4) {
            for prop_ref in prop_refs {
                if let StepValue::Reference(prop_id) = prop_ref {
                    if let Some(prop) = step_file.get_entity(*prop_id) {
                        if prop.entity_type == "IFCPROPERTYSINGLEVALUE" {
                            let name = prop
                                .values
                                .first()
                                .and_then(|v| match v {
                                    StepValue::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .unwrap_or_default();

                            let value = prop
                                .values
                                .get(2)
                                .map(format_step_value)
                                .unwrap_or_default();

                            if !name.is_empty() {
                                props.insert(name, value);
                            }
                        }
                    }
                }
            }
        }

        pset_props.insert(pset.id, props);
    }

    // Link properties to elements via IFCRELDEFINESBYPROPERTIES
    for rel in step_file.get_entities_by_type("IFCRELDEFINESBYPROPERTIES") {
        let elements: Vec<u64> = rel
            .values
            .get(4)
            .and_then(|v| match v {
                StepValue::List(list) => Some(
                    list.iter()
                        .filter_map(|item| match item {
                            StepValue::Reference(id) => Some(*id),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();

        let pset_id = rel.values.get(5).and_then(|v| match v {
            StepValue::Reference(id) => Some(*id),
            _ => None,
        });

        if let Some(pid) = pset_id {
            if let Some(props) = pset_props.get(&pid) {
                for elem_id in elements {
                    element_properties
                        .entry(elem_id)
                        .or_default()
                        .extend(props.clone());
                }
            }
        }
    }

    element_properties
}

fn format_step_value(value: &StepValue) -> String {
    match value {
        StepValue::String(s) => s.clone(),
        StepValue::Real(f) => format!("{f:.2}"),
        StepValue::Integer(i) => i.to_string(),
        StepValue::Boolean(b) => if *b { "Yes" } else { "No" }.to_string(),
        StepValue::Enum(e) => e.clone(),
        StepValue::Reference(id) => format!("#{id}"),
        StepValue::List(list) => list
            .iter()
            .map(format_step_value)
            .collect::<Vec<_>>()
            .join(", "),
        StepValue::Null => "-".to_string(),
        StepValue::Derived => "*".to_string(),
    }
}

fn build_categories(
    step_file: &StepFile,
    type_to_instances: &HashMap<u64, Vec<u64>>,
    element_properties: &HashMap<u64, HashMap<String, String>>,
) -> Vec<Category> {
    let mut categories: HashMap<String, Category> = HashMap::new();
    let mut processed_type_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();

    // Process each element type mapping
    for (element_entity, type_entities) in ELEMENT_TYPES {
        let is_priority = PRIORITY_CATEGORIES.iter().any(|(e, _)| e == element_entity);

        let category_name = PRIORITY_CATEGORIES
            .iter()
            .find(|(e, _)| e == element_entity)
            .map_or_else(|| "Other".to_string(), |(_, name)| name.to_string());

        // Deduplicate types by name within category
        let mut types_by_name: HashMap<String, ElementType> = HashMap::new();

        // Try each type entity (IFC4 types, then IFC2X3 styles)
        for type_entity in *type_entities {
            for type_entity_instance in step_file.get_entities_by_type(type_entity) {
                // Skip if already processed (prevents duplicates in Other)
                if processed_type_ids.contains(&type_entity_instance.id) {
                    continue;
                }
                processed_type_ids.insert(type_entity_instance.id);

                let type_name = type_entity_instance
                    .values
                    .get(2)
                    .and_then(|v| match v {
                        StepValue::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| format!("Type #{}", type_entity_instance.id));

                let global_id = type_entity_instance
                    .values
                    .first()
                    .and_then(|v| match v {
                        StepValue::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let instance_ids = type_to_instances
                    .get(&type_entity_instance.id)
                    .cloned()
                    .unwrap_or_default();

                let instance_count = instance_ids.len();

                // Gather properties from type
                let mut properties = element_properties
                    .get(&type_entity_instance.id)
                    .cloned()
                    .unwrap_or_default();

                // Also gather properties from instances (merge first instance's properties)
                if let Some(&first_instance) = instance_ids.first() {
                    if let Some(instance_props) = element_properties.get(&first_instance) {
                        for (k, v) in instance_props {
                            properties.entry(k.clone()).or_insert_with(|| v.clone());
                        }
                    }
                }

                // Add dimension properties for doors/windows
                let is_door_or_window = *type_entity == "IFCDOORTYPE"
                    || *type_entity == "IFCDOORSTYLE"
                    || *type_entity == "IFCWINDOWTYPE"
                    || *type_entity == "IFCWINDOWSTYLE";

                if is_door_or_window {
                    // Get dimensions from first instance
                    if let Some(&first_instance) = instance_ids.first() {
                        if let Some(instance) = step_file.get_entity(first_instance) {
                            // For doors/windows: index 8 = height, index 9 = width
                            if let Some(StepValue::Real(h)) = instance.values.get(8) {
                                properties.insert("Height".to_string(), format!("{h:.0} mm"));
                            }
                            if let Some(StepValue::Real(w)) = instance.values.get(9) {
                                properties.insert("Width".to_string(), format!("{w:.0} mm"));
                            }
                        }
                    }
                }

                // Deduplicate: merge instances if type with same name already exists
                if let Some(existing) = types_by_name.get_mut(&type_name) {
                    existing.instance_count += instance_count;
                    existing.instance_ids.extend(instance_ids);
                    // Merge properties (keep existing, add new)
                    for (k, v) in properties {
                        existing.properties.entry(k).or_insert(v);
                    }
                } else {
                    let element_type = ElementType {
                        id: type_entity_instance.id,
                        global_id,
                        name: type_name.clone(),
                        category: category_name.clone(),
                        instance_count,
                        instance_ids,
                        properties,
                    };
                    types_by_name.insert(type_name, element_type);
                }
            }
        }

        // Add deduplicated types to category
        if !types_by_name.is_empty() {
            let category = categories
                .entry(category_name.clone())
                .or_insert_with(|| Category {
                    name: category_name.clone(),
                    is_priority,
                    types: Vec::new(),
                    total_count: 0,
                });

            for element_type in types_by_name.into_values() {
                category.total_count += element_type.instance_count;
                category.types.push(element_type);
            }
        }
    }

    // Sort: priority categories first, then alphabetically
    let mut result: Vec<Category> = categories.into_values().collect();
    result.sort_by(|a, b| match (a.is_priority, b.is_priority) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    // Sort types within each category by name
    for category in &mut result {
        category.types.sort_by(|a, b| a.name.cmp(&b.name));
    }

    result
}
