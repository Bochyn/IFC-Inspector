use super::{Element, ElementType};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub name: String,
    pub is_priority: bool,
    pub types: Vec<ElementType>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct IfcProject {
    pub name: String,
    pub schema: String,
    pub file_path: String,
    pub categories: Vec<Category>,
    pub storeys: Vec<Storey>,
    pub elements: HashMap<u64, Element>,
    pub element_to_storey: HashMap<u64, u64>, // element_id → storey_id
    pub element_properties: HashMap<u64, HashMap<String, String>>, // instance_id → properties
    pub instance_global_ids: HashMap<u64, String>, // instance_id → GlobalId
}

#[derive(Debug, Clone, Serialize)]
pub struct Storey {
    pub id: u64,
    pub name: String,
    pub elevation: f64,
    pub element_count: usize,
}

impl IfcProject {
    #[must_use]
    pub fn new(name: String, schema: String, file_path: String) -> Self {
        Self {
            name,
            schema,
            file_path,
            categories: Vec::new(),
            storeys: Vec::new(),
            elements: HashMap::new(),
            element_to_storey: HashMap::new(),
            element_properties: HashMap::new(),
            instance_global_ids: HashMap::new(),
        }
    }

    #[must_use]
    pub fn total_elements(&self) -> usize {
        self.categories.iter().map(|c| c.total_count).sum()
    }

    #[must_use]
    pub fn total_types(&self) -> usize {
        self.categories.iter().map(|c| c.types.len()).sum()
    }
}
