//! ViKey Linux Settings - JSON Helpers
//!
//! JSON parsing and serialization helpers for settings.
//! Hand-rolled parser to avoid serde dependency.

use std::collections::HashMap;
use super::Shortcut;

/// Simple JSON object parser (returns key-value pairs)
pub(super) fn parse_json_object(json: &str) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    let content = json.trim().trim_start_matches('{').trim_end_matches('}');

    // Split by commas, but handle nested objects/arrays
    let mut depth = 0;
    let mut current = String::new();
    let mut pairs: Vec<String> = Vec::new();

    for c in content.chars() {
        match c {
            '{' | '[' => {
                depth += 1;
                current.push(c);
            }
            '}' | ']' => {
                if depth > 0 { depth -= 1; }
                current.push(c);
            }
            ',' if depth == 0 => {
                pairs.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        pairs.push(current.trim().to_string());
    }

    for pair in pairs {
        if let Some(idx) = pair.find(':') {
            let key = pair[..idx].trim().trim_matches('"').to_string();
            let value = pair[idx + 1..].trim().trim_matches('"').to_string();
            map.insert(key, value);
        }
    }

    Some(map)
}

/// Parse shortcuts array from JSON
pub(super) fn parse_shortcuts_array(json: &str) -> Option<Vec<Shortcut>> {
    let mut shortcuts = Vec::new();
    let content = json.trim().trim_start_matches('[').trim_end_matches(']');

    if content.trim().is_empty() {
        return Some(shortcuts);
    }

    // Split by },{
    let parts: Vec<&str> = content.split("},{").collect();

    for (i, part) in parts.iter().enumerate() {
        let mut obj = part.to_string();
        if i == 0 {
            obj = obj.trim_start_matches('{').to_string();
        }
        if i == parts.len() - 1 {
            obj = obj.trim_end_matches('}').to_string();
        }

        if let Some(map) = parse_json_object(&format!("{{{}}}", obj)) {
            if let (Some(trigger), Some(replacement)) = (map.get("trigger"), map.get("replacement")) {
                shortcuts.push(Shortcut {
                    trigger: unescape_json(trigger),
                    replacement: unescape_json(replacement),
                });
            }
        }
    }

    Some(shortcuts)
}

/// Escape string for JSON
pub(super) fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Unescape JSON string
pub(super) fn unescape_json(s: &str) -> String {
    s.replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
}
