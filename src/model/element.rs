use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Element {
    pub id: u64,
    pub global_id: String,
    pub name: String,
    pub tag: Option<String>,
    pub type_id: Option<u64>,
    pub storey_id: Option<u64>,
    pub properties: HashMap<String, String>,
}
