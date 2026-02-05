use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ElementType {
    pub id: u64,
    pub global_id: String,
    pub name: String,
    pub category: String,
    pub instance_count: usize,
    pub instance_ids: Vec<u64>,
    pub properties: HashMap<String, String>,
}
