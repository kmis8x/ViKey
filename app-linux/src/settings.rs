//! ViKey Linux Settings
//!
//! Configuration management using XDG config directory.
//! Settings are stored in ~/.config/vikey/config.json

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Text shortcut (trigger -> replacement)
#[derive(Clone, Debug)]
pub struct Shortcut {
    pub trigger: String,
    pub replacement: String,
}

/// ViKey settings
#[derive(Clone, Debug)]
pub struct Settings {
    pub enabled: bool,
    pub method: u8,              // 0=Telex, 1=VNI
    pub modern_tone: bool,
    pub esc_restore: bool,
    pub english_auto_restore: bool,
    pub auto_capitalize: bool,
    pub free_tone: bool,
    pub skip_w_shortcut: bool,
    pub bracket_shortcut: bool,
    pub allow_foreign_consonants: bool,
    pub shortcuts_enabled: bool,
    pub shortcuts: Vec<Shortcut>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            method: 0, // Telex
            modern_tone: true,
            esc_restore: true,
            english_auto_restore: true,
            auto_capitalize: false,
            free_tone: false,
            skip_w_shortcut: false,
            bracket_shortcut: false,
            allow_foreign_consonants: false,
            shortcuts_enabled: true,
            shortcuts: vec![
                Shortcut { trigger: "vn".into(), replacement: "Việt Nam".into() },
                Shortcut { trigger: "hn".into(), replacement: "Hà Nội".into() },
                Shortcut { trigger: "hcm".into(), replacement: "Hồ Chí Minh".into() },
                Shortcut { trigger: "->".into(), replacement: "→".into() },
                Shortcut { trigger: "=>".into(), replacement: "⇒".into() },
                Shortcut { trigger: ":)".into(), replacement: "😊".into() },
            ],
        }
    }
}

impl Settings {
    /// Get config directory path (~/.config/vikey/)
    pub fn config_dir() -> PathBuf {
        let xdg_config = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
                PathBuf::from(home).join(".config")
            });
        xdg_config.join("vikey")
    }

    /// Get config file path (~/.config/vikey/config.json)
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    /// Load settings from config file
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => Self::parse_json(&content).unwrap_or_default(),
            Err(e) => {
                eprintln!("ViKey: Failed to read config: {}", e);
                Self::default()
            }
        }
    }

    /// Save settings to config file
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let json = self.to_json();
        fs::write(Self::config_path(), json)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    /// Apply all settings to Rust engine via FFI
    pub fn apply(&self) {
        unsafe {
            vikey_core::ime_method(self.method);
            vikey_core::ime_enabled(self.enabled);
            vikey_core::ime_modern(self.modern_tone);
            vikey_core::ime_esc_restore(self.esc_restore);
            vikey_core::ime_english_auto_restore(self.english_auto_restore);
            vikey_core::ime_auto_capitalize(self.auto_capitalize);
            vikey_core::ime_free_tone(self.free_tone);
            vikey_core::ime_skip_w_shortcut(self.skip_w_shortcut);
            vikey_core::ime_bracket_shortcut(self.bracket_shortcut);
            vikey_core::ime_allow_foreign_consonants(self.allow_foreign_consonants);
            vikey_core::ime_shortcuts_enabled(self.shortcuts_enabled);

            // Load shortcuts
            vikey_core::ime_clear_shortcuts();
            for shortcut in &self.shortcuts {
                let trigger = std::ffi::CString::new(shortcut.trigger.as_str()).unwrap();
                let replacement = std::ffi::CString::new(shortcut.replacement.as_str()).unwrap();
                vikey_core::ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
            }
        }
    }

    /// Parse JSON config (simple manual parsing to avoid serde dependency)
    fn parse_json(json: &str) -> Option<Self> {
        let mut settings = Self::default();

        // Simple JSON parsing using string operations
        let map = parse_json_object(json)?;

        if let Some(v) = map.get("enabled") {
            settings.enabled = v == "true";
        }
        if let Some(v) = map.get("method") {
            settings.method = v.parse().unwrap_or(0);
        }
        if let Some(v) = map.get("modern_tone") {
            settings.modern_tone = v == "true";
        }
        if let Some(v) = map.get("esc_restore") {
            settings.esc_restore = v == "true";
        }
        if let Some(v) = map.get("english_auto_restore") {
            settings.english_auto_restore = v == "true";
        }
        if let Some(v) = map.get("auto_capitalize") {
            settings.auto_capitalize = v == "true";
        }
        if let Some(v) = map.get("free_tone") {
            settings.free_tone = v == "true";
        }
        if let Some(v) = map.get("skip_w_shortcut") {
            settings.skip_w_shortcut = v == "true";
        }
        if let Some(v) = map.get("bracket_shortcut") {
            settings.bracket_shortcut = v == "true";
        }
        if let Some(v) = map.get("allow_foreign_consonants") {
            settings.allow_foreign_consonants = v == "true";
        }
        if let Some(v) = map.get("shortcuts_enabled") {
            settings.shortcuts_enabled = v == "true";
        }

        // Parse shortcuts array
        if let Some(shortcuts_str) = map.get("shortcuts") {
            if let Some(shortcuts) = parse_shortcuts_array(shortcuts_str) {
                settings.shortcuts = shortcuts;
            }
        }

        Some(settings)
    }

    /// Convert settings to JSON string
    fn to_json(&self) -> String {
        let shortcuts_json: Vec<String> = self.shortcuts.iter()
            .map(|s| format!(
                r#"{{"trigger":"{}","replacement":"{}"}}"#,
                escape_json(&s.trigger),
                escape_json(&s.replacement)
            ))
            .collect();

        format!(
            r#"{{
  "enabled": {},
  "method": {},
  "modern_tone": {},
  "esc_restore": {},
  "english_auto_restore": {},
  "auto_capitalize": {},
  "free_tone": {},
  "skip_w_shortcut": {},
  "bracket_shortcut": {},
  "allow_foreign_consonants": {},
  "shortcuts_enabled": {},
  "shortcuts": [
    {}
  ]
}}"#,
            self.enabled,
            self.method,
            self.modern_tone,
            self.esc_restore,
            self.english_auto_restore,
            self.auto_capitalize,
            self.free_tone,
            self.skip_w_shortcut,
            self.bracket_shortcut,
            self.allow_foreign_consonants,
            self.shortcuts_enabled,
            shortcuts_json.join(",\n    ")
        )
    }
}

/// Simple JSON object parser (returns key-value pairs)
fn parse_json_object(json: &str) -> Option<HashMap<String, String>> {
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
                depth -= 1;
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
fn parse_shortcuts_array(json: &str) -> Option<Vec<Shortcut>> {
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
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Unescape JSON string
fn unescape_json(s: &str) -> String {
    s.replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
}
