use crate::model::{Category, ElementType, IfcProject};
use crate::parser::step::StepFile;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Dashboard,
    TypeDetail,
    InstanceBrowser,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub selected_level: usize, // 0 = "All", 1+ = storey index
    pub types_scroll_offset: usize,
    pub property_scroll_offset: usize,
    pub instances_scroll_offset: usize,
    pub should_quit: bool,
}

impl App {
    #[must_use]
    pub fn new(project: IfcProject) -> Self {
        Self {
            project,
            step_file: None,
            view: View::Dashboard,
            focus_panel: FocusPanel::Categories, // Start on Categories
            selected_category: 0,
            selected_type: 0,
            selected_instance: 0,
            selected_level: 0, // 0 = "All"
            types_scroll_offset: 0,
            property_scroll_offset: 0,
            instances_scroll_offset: 0,
            should_quit: false,
        }
    }

    #[must_use]
    pub fn with_step_file(mut self, step_file: StepFile) -> Self {
        self.step_file = Some(step_file);
        self
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.view {
            View::Dashboard => super::dashboard::draw_dashboard(frame, self),
            View::TypeDetail => super::dashboard::draw_type_detail(frame, self),
            View::InstanceBrowser => super::dashboard::draw_instance_browser(frame, self),
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match self.view {
                View::Dashboard => self.handle_dashboard_keys(key.code),
                View::TypeDetail => self.handle_detail_keys(key.code),
                View::InstanceBrowser => self.handle_instance_keys(key.code),
            }
        }
        Ok(())
    }

    fn handle_dashboard_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => self.navigate_up(),
            KeyCode::Down | KeyCode::Char('j') => self.navigate_down(),
            KeyCode::Left | KeyCode::Char('h') => self.navigate_left(),
            KeyCode::Right | KeyCode::Char('l') => self.navigate_right(),
            KeyCode::Enter => self.enter_type_detail(),
            _ => {}
        }
    }

    fn navigate_up(&mut self) {
        match self.focus_panel {
            FocusPanel::Levels => self.previous_level(),
            FocusPanel::Categories => self.previous_category(),
            FocusPanel::Types => self.previous_type(),
        }
    }

    fn navigate_down(&mut self) {
        match self.focus_panel {
            FocusPanel::Levels => self.next_level(),
            FocusPanel::Categories => self.next_category(),
            FocusPanel::Types => self.next_type(),
        }
    }

    fn navigate_left(&mut self) {
        match self.focus_panel {
            FocusPanel::Types => self.focus_panel = FocusPanel::Categories,
            FocusPanel::Categories => self.focus_panel = FocusPanel::Levels,
            FocusPanel::Levels => {}
        }
    }

    fn navigate_right(&mut self) {
        match self.focus_panel {
            FocusPanel::Levels => self.focus_panel = FocusPanel::Categories,
            FocusPanel::Categories => self.focus_panel = FocusPanel::Types,
            FocusPanel::Types => {}
        }
    }

    fn previous_level(&mut self) {
        if self.selected_level > 0 {
            self.selected_level -= 1;
            self.selected_type = 0;
            self.types_scroll_offset = 0;
        }
    }

    fn next_level(&mut self) {
        // 0 = "All", then storeys
        let max_level = self.project.storeys.len();
        if self.selected_level < max_level {
            self.selected_level += 1;
            self.selected_type = 0;
            self.types_scroll_offset = 0;
        }
    }

    fn handle_detail_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc | KeyCode::Backspace => {
                self.view = View::Dashboard;
                self.property_scroll_offset = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => self.scroll_properties_up(),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_properties_down(),
            KeyCode::Left | KeyCode::Char('h') => self.previous_instance_in_detail(),
            KeyCode::Right | KeyCode::Char('l') => self.next_instance_in_detail(),
            KeyCode::Enter => self.enter_instance_browser(),
            _ => {}
        }
    }

    fn handle_instance_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Enter => {
                // Return to Type Detail, keeping selected_instance
                self.view = View::TypeDetail;
                self.instances_scroll_offset = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => self.previous_instance(),
            KeyCode::Down | KeyCode::Char('j') => self.next_instance(),
            _ => {}
        }
    }

    fn previous_category(&mut self) {
        if self.selected_category > 0 {
            self.selected_category -= 1;
            self.selected_type = 0;
            self.types_scroll_offset = 0;
        }
    }

    fn next_category(&mut self) {
        if self.selected_category < self.project.categories.len().saturating_sub(1) {
            self.selected_category += 1;
            self.selected_type = 0;
            self.types_scroll_offset = 0;
        }
    }

    fn previous_type(&mut self) {
        if self.selected_type > 0 {
            self.selected_type -= 1;
            if self.selected_type < self.types_scroll_offset {
                self.types_scroll_offset = self.selected_type;
            }
        }
    }

    fn next_type(&mut self) {
        let filtered_count = self.get_filtered_types().len();
        if self.selected_type < filtered_count.saturating_sub(1) {
            self.selected_type += 1;
        }
    }

    fn enter_type_detail(&mut self) {
        // Only enter detail when focus is on Types panel
        if self.focus_panel == FocusPanel::Types && self.get_selected_type().is_some() {
            self.view = View::TypeDetail;
            self.property_scroll_offset = 0;
            self.selected_instance = 0;
        }
        // Enter on Levels or Categories does nothing (filtering happens via selected_level)
    }

    fn enter_instance_browser(&mut self) {
        // Extract data first to avoid borrow issues
        let instance_count = self.get_selected_type().map_or(0, |t| t.instance_ids.len());

        if instance_count > 0 {
            self.view = View::InstanceBrowser;
            // Keep selected_instance from Type Detail navigation
            // Just ensure it's within bounds
            if self.selected_instance >= instance_count {
                self.selected_instance = 0;
            }
            self.instances_scroll_offset = 0;
        }
    }

    fn scroll_properties_up(&mut self) {
        if self.property_scroll_offset > 0 {
            self.property_scroll_offset -= 1;
        }
    }

    fn scroll_properties_down(&mut self) {
        let max = self.get_all_properties().len().saturating_sub(1);
        if self.property_scroll_offset < max {
            self.property_scroll_offset += 1;
        }
    }

    fn previous_instance(&mut self) {
        if self.selected_instance > 0 {
            self.selected_instance -= 1;
            if self.selected_instance < self.instances_scroll_offset {
                self.instances_scroll_offset = self.selected_instance;
            }
        }
    }

    fn next_instance(&mut self) {
        if let Some(t) = self.get_selected_type() {
            if self.selected_instance < t.instance_ids.len().saturating_sub(1) {
                self.selected_instance += 1;
            }
        }
    }

    /// Navigate to previous instance in Type Detail view (wrap around)
    fn previous_instance_in_detail(&mut self) {
        if let Some(t) = self.get_selected_type() {
            let count = t.instance_ids.len();
            if count == 0 {
                return;
            }
            if self.selected_instance > 0 {
                self.selected_instance -= 1;
            } else {
                // Wrap around to last instance
                self.selected_instance = count - 1;
            }
        }
    }

    /// Navigate to next instance in Type Detail view (wrap around)
    fn next_instance_in_detail(&mut self) {
        if let Some(t) = self.get_selected_type() {
            let count = t.instance_ids.len();
            if count == 0 {
                return;
            }
            if self.selected_instance < count - 1 {
                self.selected_instance += 1;
            } else {
                // Wrap around to first instance
                self.selected_instance = 0;
            }
        }
    }

    /// Get types filtered by selected level
    #[must_use]
    pub fn get_filtered_types(&self) -> Vec<&crate::model::ElementType> {
        let category = match self.project.categories.get(self.selected_category) {
            Some(c) => c,
            None => return Vec::new(),
        };

        if self.selected_level == 0 {
            // "All" - no filtering
            return category.types.iter().collect();
        }

        // Get storey ID for selected level
        let storey_id = match self.project.storeys.get(self.selected_level - 1) {
            Some(s) => s.id,
            None => return category.types.iter().collect(),
        };

        // Filter types that have instances on this level
        category
            .types
            .iter()
            .filter(|t| {
                t.instance_ids
                    .iter()
                    .any(|id| self.project.element_to_storey.get(id) == Some(&storey_id))
            })
            .collect()
    }

    #[must_use]
    pub fn get_selected_type(&self) -> Option<&crate::model::ElementType> {
        let filtered = self.get_filtered_types();
        filtered.get(self.selected_type).copied()
    }

    /// Get selected storey ID (None if "All" is selected)
    fn get_selected_storey_id(&self) -> Option<u64> {
        if self.selected_level == 0 {
            None
        } else {
            self.project
                .storeys
                .get(self.selected_level - 1)
                .map(|s| s.id)
        }
    }

    /// Get filtered instance count for a category (respects `selected_level`)
    #[must_use]
    pub fn get_filtered_category_count(&self, category: &Category) -> usize {
        match self.get_selected_storey_id() {
            None => category.total_count,
            Some(storey_id) => category
                .types
                .iter()
                .flat_map(|t| &t.instance_ids)
                .filter(|id| self.project.element_to_storey.get(*id) == Some(&storey_id))
                .count(),
        }
    }

    /// Get filtered instance count for a type (respects `selected_level`)
    #[must_use]
    pub fn get_filtered_instance_count(&self, element_type: &ElementType) -> usize {
        match self.get_selected_storey_id() {
            None => element_type.instance_count,
            Some(storey_id) => element_type
                .instance_ids
                .iter()
                .filter(|id| self.project.element_to_storey.get(*id) == Some(&storey_id))
                .count(),
        }
    }

    /// Get the ID of the currently selected instance
    #[must_use]
    pub fn get_selected_instance_id(&self) -> Option<u64> {
        self.get_selected_type()
            .and_then(|t| t.instance_ids.get(self.selected_instance))
            .copied()
    }

    /// Get aggregated numeric properties for the selected type
    #[must_use]
    pub fn get_aggregated_properties(&self) -> Vec<AggregatedProperty> {
        let element_type = match self.get_selected_type() {
            Some(t) => t,
            None => return Vec::new(),
        };

        // Collect all numeric values for each property across instances
        let mut property_values: HashMap<String, Vec<f64>> = HashMap::new();

        // For now, use the properties we have from the type
        // In future, we could load all instance properties from step_file
        for (name, value) in &element_type.properties {
            if let Some(num) = parse_numeric_value(value) {
                property_values.entry(name.clone()).or_default().push(num);
            }
        }

        property_values
            .into_iter()
            .map(|(name, values)| {
                let sum: f64 = values.iter().sum();
                let count = values.len() as f64;
                let avg = sum / count;
                let min = values.iter().copied().fold(f64::INFINITY, f64::min);
                let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

                AggregatedProperty {
                    name,
                    sum,
                    avg,
                    min,
                    max,
                    count: values.len(),
                }
            })
            .collect()
    }

    /// Get text (non-numeric) properties
    #[must_use]
    pub fn get_text_properties(&self) -> Vec<(String, String)> {
        let element_type = match self.get_selected_type() {
            Some(t) => t,
            None => return Vec::new(),
        };

        element_type
            .properties
            .iter()
            .filter(|(_, v)| parse_numeric_value(v).is_none())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get all properties combined (numeric first, then text)
    /// Uses properties from the currently selected instance
    #[must_use]
    pub fn get_all_properties(&self) -> Vec<(String, String, bool)> {
        let element_type = match self.get_selected_type() {
            Some(t) => t,
            None => return Vec::new(),
        };

        // Start with type-level properties
        let mut all_props: HashMap<String, String> = element_type.properties.clone();

        // Override/merge with instance-level properties if available
        if let Some(instance_id) = self.get_selected_instance_id() {
            if let Some(instance_props) = self.project.element_properties.get(&instance_id) {
                for (k, v) in instance_props {
                    all_props.insert(k.clone(), v.clone());
                }
            }
        }

        let mut props: Vec<(String, String, bool)> = all_props
            .iter()
            .map(|(k, v)| {
                let is_numeric = parse_numeric_value(v).is_some();
                (k.clone(), v.clone(), is_numeric)
            })
            .collect();

        // Sort: numeric first, then text, alphabetically within each group
        props.sort_by(|a, b| match (a.2, b.2) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        });

        props
    }

    /// Get storey name for an instance ID
    #[must_use]
    pub fn get_storey_name_for_instance(&self, instance_id: u64) -> String {
        if let Some(&storey_id) = self.project.element_to_storey.get(&instance_id) {
            self.project
                .storeys
                .iter()
                .find(|s| s.id == storey_id)
                .map_or_else(|| "-".to_string(), |s| s.name.clone())
        } else {
            "-".to_string()
        }
    }

    /// Get `GlobalId` for an instance ID
    #[must_use]
    pub fn get_instance_global_id(&self, instance_id: u64) -> String {
        self.project
            .instance_global_ids
            .get(&instance_id)
            .cloned()
            .unwrap_or_else(|| "-".to_string())
    }

    /// Get selected level name (for display)
    #[must_use]
    pub fn get_selected_level_name(&self) -> String {
        if self.selected_level == 0 {
            "All".to_string()
        } else {
            self.project
                .storeys
                .get(self.selected_level - 1)
                .map_or_else(|| "-".to_string(), |s| s.name.clone())
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedProperty {
    pub name: String,
    pub sum: f64,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

fn parse_numeric_value(s: &str) -> Option<f64> {
    // Try to parse number, handling units like "0.88 m³" or "580 m²"
    let s = s.trim();

    // Remove common units
    let numeric_part = s
        .trim_end_matches(|c: char| c.is_alphabetic() || c == '³' || c == '²' || c == ' ')
        .trim();

    numeric_part.parse::<f64>().ok()
}
