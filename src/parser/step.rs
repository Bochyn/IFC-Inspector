use std::collections::HashMap;

use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum StepValue {
    String(String),
    Real(f64),
    Integer(i64),
    Boolean(bool),
    Enum(String),
    Reference(u64),
    List(Vec<StepValue>),
    Null,
    Derived,
}

#[derive(Debug, Clone)]
pub struct StepEntity {
    pub id: u64,
    pub entity_type: String,
    pub values: Vec<StepValue>,
}

#[derive(Debug)]
pub struct StepFile {
    pub entities: HashMap<u64, StepEntity>,
    pub schema: String,
}

impl StepFile {
    pub fn parse(content: &str) -> Result<Self, ParseError> {
        let mut entities = HashMap::new();
        let mut schema = String::new();
        let mut in_data = false;

        for line in content.lines() {
            let line = line.trim();

            // Parse schema
            if line.starts_with("FILE_SCHEMA") {
                if let Some(start) = line.find("('") {
                    if let Some(end) = line[start + 2..].find('\'') {
                        schema = line[start + 2..start + 2 + end].to_string();
                    }
                }
                continue;
            }

            if line == "DATA;" {
                in_data = true;
                continue;
            }
            if line == "ENDSEC;" {
                in_data = false;
                continue;
            }

            if in_data && line.starts_with('#') {
                if let Some(entity) = Self::parse_entity_line(line) {
                    entities.insert(entity.id, entity);
                }
            }
        }

        Ok(StepFile { entities, schema })
    }

    fn parse_entity_line(line: &str) -> Option<StepEntity> {
        // Format: #123=IFCWALL('guid',#ref,'name',...);
        let line = line.trim_end_matches(';');

        let eq_pos = line.find('=')?;
        let id: u64 = line[1..eq_pos].parse().ok()?;

        let rest = &line[eq_pos + 1..];
        let paren_pos = rest.find('(')?;
        let entity_type = rest[..paren_pos].to_string();

        let values_str = &rest[paren_pos + 1..rest.len() - 1];
        let values = Self::parse_values(values_str);

        Some(StepEntity {
            id,
            entity_type,
            values,
        })
    }

    fn parse_values(s: &str) -> Vec<StepValue> {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut paren_depth = 0;

        for ch in s.chars() {
            match ch {
                '\'' if paren_depth == 0 => {
                    in_string = !in_string;
                    current.push(ch);
                }
                '(' if !in_string => {
                    paren_depth += 1;
                    current.push(ch);
                }
                ')' if !in_string => {
                    paren_depth -= 1;
                    current.push(ch);
                }
                ',' if !in_string && paren_depth == 0 => {
                    values.push(Self::parse_single_value(current.trim()));
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            values.push(Self::parse_single_value(current.trim()));
        }

        values
    }

    fn parse_single_value(s: &str) -> StepValue {
        let s = s.trim();

        if s == "$" {
            return StepValue::Null;
        }
        if s == "*" {
            return StepValue::Derived;
        }
        if let Some(stripped) = s.strip_prefix('#') {
            if let Ok(id) = stripped.parse::<u64>() {
                return StepValue::Reference(id);
            }
        }
        if s.starts_with('\'') && s.ends_with('\'') {
            let raw = &s[1..s.len() - 1];
            return StepValue::String(decode_step_string(raw));
        }
        if s.starts_with('.') && s.ends_with('.') {
            let inner = &s[1..s.len() - 1];
            if inner == "T" {
                return StepValue::Boolean(true);
            }
            if inner == "F" {
                return StepValue::Boolean(false);
            }
            return StepValue::Enum(inner.to_string());
        }
        if s.starts_with('(') && s.ends_with(')') {
            let inner = &s[1..s.len() - 1];
            return StepValue::List(Self::parse_values(inner));
        }
        if let Ok(i) = s.parse::<i64>() {
            return StepValue::Integer(i);
        }
        if let Ok(f) = s.parse::<f64>() {
            return StepValue::Real(f);
        }
        // Typed value like IFCBOOLEAN(.T.)
        if let Some(paren_pos) = s.find('(') {
            let inner = &s[paren_pos + 1..s.len() - 1];
            return Self::parse_single_value(inner);
        }

        StepValue::String(s.to_string())
    }

    #[must_use]
    pub fn get_entity(&self, id: u64) -> Option<&StepEntity> {
        self.entities.get(&id)
    }

    #[must_use]
    pub fn get_entities_by_type(&self, entity_type: &str) -> Vec<&StepEntity> {
        self.entities
            .values()
            .filter(|e| e.entity_type == entity_type)
            .collect()
    }
}

/// Decode STEP/IFC encoded strings with Unicode escape sequences.
/// Supports:
/// - `\X2\XXXX\X0\` - 2-byte Unicode (BMP), can have multiple 4-char hex codes
/// - `\X\XX` - 1-byte ISO 8859-1
/// - `\\` - escaped backslash
/// - `''` - escaped apostrophe
fn decode_step_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('X') => {
                    chars.next(); // consume 'X'
                    match chars.peek() {
                        Some('2') => {
                            // \X2\XXXX...\X0\ - 2-byte Unicode encoding
                            chars.next(); // consume '2'
                            chars.next(); // consume '\'

                            let mut hex = String::new();
                            while let Some(&c) = chars.peek() {
                                if c == '\\' {
                                    break;
                                }
                                hex.push(c);
                                chars.next();
                            }
                            // Skip \X0\
                            if chars.peek() == Some(&'\\') {
                                chars.next(); // '\'
                                chars.next(); // 'X'
                                chars.next(); // '0'
                                chars.next(); // '\'
                            }
                            // Decode hex pairs (each 4 chars = one Unicode char)
                            for chunk in hex.as_bytes().chunks(4) {
                                if chunk.len() == 4 {
                                    if let Ok(s) = std::str::from_utf8(chunk) {
                                        if let Ok(code) = u32::from_str_radix(s, 16) {
                                            if let Some(c) = char::from_u32(code) {
                                                result.push(c);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some('\\') => {
                            // \X\ followed by 2 hex digits - ISO 8859-1
                            chars.next(); // consume '\'
                            let mut hex = String::new();
                            for _ in 0..2 {
                                if let Some(&c) = chars.peek() {
                                    hex.push(c);
                                    chars.next();
                                }
                            }
                            if let Ok(code) = u8::from_str_radix(&hex, 16) {
                                result.push(code as char);
                            }
                        }
                        _ => {
                            result.push('\\');
                            result.push('X');
                        }
                    }
                }
                Some('\\') => {
                    chars.next();
                    result.push('\\');
                }
                Some('S') => {
                    // \S\X - single char shift (ISO 8859-1 high bit)
                    chars.next(); // 'S'
                    chars.next(); // '\'
                    if let Some(c) = chars.next() {
                        result.push(((c as u8) + 128) as char);
                    }
                }
                _ => result.push('\\'),
            }
        } else if ch == '\'' {
            // '' is escaped apostrophe in STEP
            if chars.peek() == Some(&'\'') {
                chars.next();
            }
            result.push('\'');
        } else {
            result.push(ch);
        }
    }

    result
}
