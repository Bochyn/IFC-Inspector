use crate::ui::app::{App, FocusPanel};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
    Frame,
};

// Brandbook colors
#[allow(dead_code)]
const BRAND_BG: Color = Color::Rgb(0xED, 0xED, 0xED); // #ededed - tło
const BRAND_DARK: Color = Color::Rgb(0x1F, 0x2F, 0x3C); // #1f2f3c - główny ciemny
#[allow(dead_code)]
const BRAND_ACCENT: Color = Color::Rgb(0x58, 0x6B, 0x71); // #586b71 - akcent niebieski (reserved)
const BRAND_SELECT_BG: Color = Color::Rgb(0xC3, 0xD3, 0xE0); // #c3d3e0 - tło zaznaczenia
const BRAND_GREEN: Color = Color::Rgb(0x82, 0x9A, 0x68); // #829a68 - zielony (count)
const BRAND_ORANGE: Color = Color::Rgb(0x9E, 0x68, 0x3C); // #9e683c - pomarańczowy (priority)
const BRAND_MUTED: Color = Color::Rgb(0x71, 0x65, 0x65); // #716565 - przygaszony (footer)

// Styles
const HEADER_STYLE: Style = Style::new().fg(BRAND_DARK).add_modifier(Modifier::BOLD);
const SELECTED_STYLE: Style = Style::new()
    .bg(BRAND_SELECT_BG)
    .fg(BRAND_DARK)
    .add_modifier(Modifier::BOLD);
const PRIORITY_COLOR: Color = BRAND_ORANGE;
const COUNT_COLOR: Color = BRAND_GREEN;

pub fn draw_dashboard(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(10),   // Main content
        Constraint::Length(3), // Footer
    ])
    .split(frame.area());

    draw_header(frame, chunks[0], app);
    draw_main_content(frame, chunks[1], app);
    draw_footer(
        frame,
        chunks[2],
        " ←→ Category | ↑↓ Type | Enter Details | q Quit ",
    );
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        " IFC Inspector | {} | {} types | {} elements ",
        app.project.name,
        app.project.total_types(),
        app.project.total_elements()
    );

    let header = Paragraph::new(title)
        .style(HEADER_STYLE)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(15), // Levels
        Constraint::Percentage(25), // Categories
        Constraint::Percentage(60), // Types
    ])
    .split(area);

    draw_levels(frame, chunks[0], app);
    draw_categories(frame, chunks[1], app);
    draw_types(frame, chunks[2], app);
}

fn draw_levels(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus_panel == FocusPanel::Levels;

    // Build items: "All" first, then storeys
    let mut items: Vec<ListItem> = Vec::new();

    // "All" option (index 0)
    let all_selected = app.selected_level == 0;
    let all_style = if all_selected && is_focused {
        SELECTED_STYLE
    } else if all_selected {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let all_marker = if all_selected && is_focused {
        " ◄"
    } else {
        ""
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled("All", all_style),
        Span::styled(all_marker, Style::default().fg(BRAND_ORANGE)),
    ])));

    // Storeys (index 1+)
    for (i, storey) in app.project.storeys.iter().enumerate() {
        let is_selected = (i + 1) == app.selected_level;

        let elev_str = if storey.elevation >= 0.0 {
            format!("+{:.1}m", storey.elevation / 1000.0)
        } else {
            format!("{:.1}m", storey.elevation / 1000.0)
        };

        let style = if is_selected && is_focused {
            SELECTED_STYLE
        } else if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let marker = if is_selected && is_focused {
            " ◄"
        } else {
            ""
        };

        let content = Line::from(vec![
            Span::styled(&storey.name, style),
            Span::styled(format!(" {elev_str}"), Style::default().fg(BRAND_MUTED)),
            Span::styled(marker, Style::default().fg(BRAND_ORANGE)),
        ]);

        items.push(ListItem::new(content));
    }

    let border_style = if is_focused {
        Style::default().fg(BRAND_ORANGE)
    } else {
        Style::default()
    };

    let title = format!(" Levels ({}) ", app.project.storeys.len() + 1); // +1 for "All"
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn draw_categories(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus_panel == FocusPanel::Categories;

    let items: Vec<ListItem> = app
        .project
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let is_selected = i == app.selected_category;
            let style = if is_selected && is_focused {
                SELECTED_STYLE
            } else if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else if cat.is_priority {
                Style::default().fg(PRIORITY_COLOR)
            } else {
                Style::default()
            };

            let marker = if is_selected && is_focused {
                " ◄"
            } else {
                ""
            };

            // Get filtered count (respects selected_level)
            let filtered_count = app.get_filtered_category_count(cat);

            let content = Line::from(vec![
                Span::styled(&cat.name, style),
                Span::raw(" "),
                Span::styled(
                    format!("({filtered_count})"),
                    Style::default().fg(COUNT_COLOR),
                ),
                Span::styled(marker, Style::default().fg(BRAND_ORANGE)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let border_style = if is_focused {
        Style::default().fg(BRAND_ORANGE)
    } else {
        Style::default()
    };

    let list = List::new(items).block(
        Block::default()
            .title(" Categories ")
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

fn draw_types(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus_panel == FocusPanel::Types;

    // Get filtered types (respects selected_level)
    let filtered_types = app.get_filtered_types();

    let category_name = app
        .project
        .categories
        .get(app.selected_category)
        .map(|c| c.name.clone())
        .unwrap_or_default();

    // Calculate visible area (subtract 3 for borders and header)
    let visible_rows = (area.height as usize).saturating_sub(3);

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if app.selected_type >= visible_rows {
        app.selected_type - visible_rows + 1
    } else {
        0
    };

    let header = Row::new(vec!["Type Name", "Instances"])
        .style(HEADER_STYLE)
        .height(1);

    let rows: Vec<Row> = filtered_types
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_rows)
        .map(|(i, t)| {
            let is_selected = i == app.selected_type;
            let style = if is_selected && is_focused {
                SELECTED_STYLE
            } else if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Get filtered instance count (respects selected_level)
            let filtered_count = app.get_filtered_instance_count(t);

            Row::new(vec![t.name.clone(), format!("{} szt.", filtered_count)]).style(style)
        })
        .collect();

    let widths = [Constraint::Percentage(70), Constraint::Percentage(30)];

    let border_style = if is_focused {
        Style::default().fg(BRAND_ORANGE)
    } else {
        Style::default()
    };

    let title = format!(" {} ({} types) ", category_name, filtered_types.len());
    let table = Table::new(rows, widths).header(header).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(table, area);

    // Draw scrollbar if needed
    if filtered_types.len() > visible_rows {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(filtered_types.len()).position(app.selected_type);

        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y + 2,
            width: 1,
            height: area.height - 3,
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, help: &str) {
    let footer = Paragraph::new(help)
        .style(Style::default().fg(BRAND_MUTED))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

pub fn draw_type_detail(frame: &mut Frame, app: &App) {
    let element_type = match app.get_selected_type() {
        Some(t) => t,
        None => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(3), // Header: Type name
        Constraint::Length(3), // Info: Family | Level | Instance X/Y
        Constraint::Min(6),    // Combined Properties (scrollable)
        Constraint::Length(3), // Footer
    ])
    .split(frame.area());

    // Header - Type name
    let header = Paragraph::new(format!(" Type: {} ", element_type.name))
        .style(HEADER_STYLE)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Info line: Family | Level (of current instance) | Instance X/Y | GlobalId
    let family = element_type.category.clone();

    // Get level of the currently selected instance (not the filter level)
    let instance_level = app.get_selected_instance_id().map_or_else(
        || "-".to_string(),
        |id| app.get_storey_name_for_instance(id),
    );

    let instance_id_str = app
        .get_selected_instance_id()
        .map_or_else(|| "-".to_string(), |id| format!("#{id}"));

    let instance_info = format!(
        "Instance: {}/{} ({})",
        app.selected_instance + 1,
        element_type.instance_count,
        instance_id_str
    );

    // GlobalId for Revit lookup (use in Schedule filter)
    let global_id = if element_type.global_id.is_empty() {
        "-".to_string()
    } else {
        element_type.global_id.clone()
    };

    let info_text = format!(
        "{family}  |  Level: {instance_level}  |  {instance_info}  |  GlobalId: {global_id}"
    );
    let info_widget = Paragraph::new(info_text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(info_widget, chunks[1]);

    // Combined Properties (Numeric + Text in one scrollable area)
    let all_props = app.get_all_properties();
    let visible_props = (chunks[2].height as usize).saturating_sub(3);

    // Build rows with section headers
    let mut rows: Vec<Row> = Vec::new();
    let mut last_was_numeric = None;

    for (name, value, is_numeric) in all_props
        .iter()
        .skip(app.property_scroll_offset)
        .take(visible_props)
    {
        // Add section header if type changes
        if last_was_numeric != Some(*is_numeric) {
            let section_title = if *is_numeric {
                "── Numeric ──"
            } else {
                "── Text ──"
            };
            rows.push(
                Row::new(vec![section_title.to_string(), String::new()]).style(
                    Style::default()
                        .fg(BRAND_MUTED)
                        .add_modifier(Modifier::ITALIC),
                ),
            );
            last_was_numeric = Some(*is_numeric);
        }

        rows.push(Row::new(vec![name.clone(), value.clone()]));
    }

    let prop_widths = [Constraint::Percentage(40), Constraint::Percentage(60)];
    let prop_header = Row::new(vec!["Property", "Value"]).style(HEADER_STYLE);

    let prop_table = Table::new(rows, prop_widths).header(prop_header).block(
        Block::default()
            .title(format!(" Properties ({}) ", all_props.len()))
            .borders(Borders::ALL),
    );
    frame.render_widget(prop_table, chunks[2]);

    // Scrollbar if needed
    if all_props.len() > visible_props {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(all_props.len()).position(app.property_scroll_offset);

        let scrollbar_area = Rect {
            x: chunks[2].x + chunks[2].width - 1,
            y: chunks[2].y + 2,
            width: 1,
            height: chunks[2].height - 3,
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Footer
    draw_footer(
        frame,
        chunks[3],
        " Esc Back | ↑↓ Scroll | ←→ Instance | Enter Browse | q Quit ",
    );
}

pub fn draw_instance_browser(frame: &mut Frame, app: &App) {
    let element_type = match app.get_selected_type() {
        Some(t) => t,
        None => return,
    };

    // Sort instances by elevation (lowest first)
    let mut sorted_instances: Vec<(usize, u64, f64)> = element_type
        .instance_ids
        .iter()
        .enumerate()
        .map(|(original_idx, id)| {
            let elevation = app
                .project
                .element_to_storey
                .get(id)
                .and_then(|storey_id| app.project.storeys.iter().find(|s| s.id == *storey_id))
                .map_or(f64::MAX, |s| s.elevation);
            (original_idx, *id, elevation)
        })
        .collect();

    sorted_instances.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(10),   // Instance list
        Constraint::Length(3), // Footer
    ])
    .split(frame.area());

    // Header
    let header = Paragraph::new(format!(
        " Instances of: {} ({} szt.) ",
        element_type.name,
        element_type.instance_ids.len()
    ))
    .style(HEADER_STYLE)
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Instance list
    let visible_rows = (chunks[1].height as usize).saturating_sub(3);
    let scroll_offset = if app.selected_instance >= visible_rows {
        app.selected_instance - visible_rows + 1
    } else {
        0
    };

    // Check which dimension properties are available for this type
    let has_length = element_type.properties.contains_key("Length");
    let has_area = element_type.properties.contains_key("Area");
    let has_volume = element_type.properties.contains_key("Volume");

    // Build dynamic header
    let mut header_cells = vec!["#", "Level", "ID", "GlobalId"];
    if has_length {
        header_cells.push("Length");
    }
    if has_area {
        header_cells.push("Area");
    }
    if has_volume {
        header_cells.push("Volume");
    }

    let instance_header = Row::new(header_cells).style(HEADER_STYLE).height(1);

    let instance_rows: Vec<Row> = sorted_instances
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_rows)
        .map(|(display_idx, (_original_idx, id, _elev))| {
            let style = if display_idx == app.selected_instance {
                SELECTED_STYLE
            } else {
                Style::default()
            };

            let level_name = app.get_storey_name_for_instance(*id);
            let global_id = app.get_instance_global_id(*id);

            // Get instance properties for dimensions
            let instance_props = app.project.element_properties.get(id);

            let mut cells = vec![
                format!("{}", display_idx + 1),
                level_name,
                format!("#{}", id),
                global_id,
            ];

            if has_length {
                let val = instance_props
                    .and_then(|p| p.get("Length"))
                    .cloned()
                    .unwrap_or_else(|| "-".to_string());
                cells.push(val);
            }
            if has_area {
                let val = instance_props
                    .and_then(|p| p.get("Area"))
                    .cloned()
                    .unwrap_or_else(|| "-".to_string());
                cells.push(val);
            }
            if has_volume {
                let val = instance_props
                    .and_then(|p| p.get("Volume"))
                    .cloned()
                    .unwrap_or_else(|| "-".to_string());
                cells.push(val);
            }

            Row::new(cells).style(style)
        })
        .collect();

    // Build dynamic widths
    let mut widths: Vec<Constraint> = vec![
        Constraint::Length(4),      // #
        Constraint::Percentage(18), // Level
        Constraint::Percentage(12), // ID
        Constraint::Percentage(28), // GlobalId
    ];
    let dim_count = [has_length, has_area, has_volume]
        .iter()
        .filter(|&&x| x)
        .count();
    if dim_count > 0 {
        let dim_width = 38 / dim_count as u16; // remaining ~38% split among dimensions
        for _ in 0..dim_count {
            widths.push(Constraint::Percentage(dim_width));
        }
    }

    let instance_table = Table::new(instance_rows, widths)
        .header(instance_header)
        .block(Block::default().title(" Instances ").borders(Borders::ALL));
    frame.render_widget(instance_table, chunks[1]);

    // Scrollbar
    if sorted_instances.len() > visible_rows {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(sorted_instances.len()).position(app.selected_instance);

        let scrollbar_area = Rect {
            x: chunks[1].x + chunks[1].width - 1,
            y: chunks[1].y + 2,
            width: 1,
            height: chunks[1].height - 3,
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Footer
    draw_footer(
        frame,
        chunks[2],
        " Esc Back to Type | ↑↓ Navigate | q Quit ",
    );
}
