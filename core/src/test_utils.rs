//! Shared test utilities for Vietnamese IME engine
//!
//! Provides common helpers for testing: key mapping, typing simulation, test runners.
//! Used by `#[cfg(test)]` modules throughout the crate.

use crate::data::keys;
use crate::engine::{Action, Engine};

// ============================================================
// KEY MAPPING
// ============================================================

/// Convert character to key code
pub fn char_to_key(c: char) -> u16 {
    match c.to_ascii_lowercase() {
        'a' => keys::A,
        'b' => keys::B,
        'c' => keys::C,
        'd' => keys::D,
        'e' => keys::E,
        'f' => keys::F,
        'g' => keys::G,
        'h' => keys::H,
        'i' => keys::I,
        'j' => keys::J,
        'k' => keys::K,
        'l' => keys::L,
        'm' => keys::M,
        'n' => keys::N,
        'o' => keys::O,
        'p' => keys::P,
        'q' => keys::Q,
        'r' => keys::R,
        's' => keys::S,
        't' => keys::T,
        'u' => keys::U,
        'v' => keys::V,
        'w' => keys::W,
        'x' => keys::X,
        'y' => keys::Y,
        'z' => keys::Z,
        '0' => keys::N0,
        '1' => keys::N1,
        '2' => keys::N2,
        '3' => keys::N3,
        '4' => keys::N4,
        '5' => keys::N5,
        '6' => keys::N6,
        '7' => keys::N7,
        '8' => keys::N8,
        '9' => keys::N9,
        '.' => keys::DOT,
        ',' => keys::COMMA,
        ';' => keys::SEMICOLON,
        ':' => keys::SEMICOLON, // Approximate
        '\'' => keys::QUOTE,
        '"' => keys::QUOTE,
        '-' => keys::MINUS,
        '=' => keys::EQUAL,
        '[' => keys::LBRACKET,
        ']' => keys::RBRACKET,
        '\\' => keys::BACKSLASH,
        '/' => keys::SLASH,
        '`' => keys::BACKQUOTE,
        '<' => keys::DELETE,
        ' ' => keys::SPACE,
        '\x1b' => keys::ESC,
        // Common symbols - map to base key (handler checks shift state)
        '@' => keys::N2,    // Shift+2
        '!' => keys::N1,    // Shift+1
        '#' => keys::N3,    // Shift+3
        '$' => keys::N4,    // Shift+4
        '%' => keys::N5,    // Shift+5
        '^' => keys::N6,    // Shift+6
        '&' => keys::N7,    // Shift+7
        '*' => keys::N8,    // Shift+8
        '(' => keys::N9,    // Shift+9
        ')' => keys::N0,    // Shift+0
        '_' => keys::MINUS, // Shift+-
        '+' => keys::EQUAL, // Shift+=
        _ => 255,           // Unknown/Other
    }
}

/// Convert string to key codes
pub fn keys_from_str(s: &str) -> Vec<u16> {
    s.chars().map(char_to_key).filter(|&k| k != 255).collect()
}

// ============================================================
// TYPING SIMULATION
// ============================================================

/// Simulate typing, returns screen output
pub fn type_word(e: &mut Engine, input: &str) -> String {
    let mut screen = String::new();
    for c in input.chars() {
        // Detect shifted symbols and get proper (key, shift) pair
        // NOTE: '<' is NOT included here - it maps to DELETE in test utilities
        let (key, shift) = match c {
            '@' => (keys::N2, true),
            '!' => (keys::N1, true),
            '#' => (keys::N3, true),
            '$' => (keys::N4, true),
            '%' => (keys::N5, true),
            '^' => (keys::N6, true),
            '&' => (keys::N7, true),
            '*' => (keys::N8, true),
            '(' => (keys::N9, true),
            ')' => (keys::N0, true),
            '_' => (keys::MINUS, true),
            '+' => (keys::EQUAL, true),
            ':' => (keys::SEMICOLON, true),
            '"' => (keys::QUOTE, true),
            '>' => (keys::DOT, true),
            '?' => (keys::SLASH, true),
            '|' => (keys::BACKSLASH, true),
            '{' => (keys::LBRACKET, true),
            '}' => (keys::RBRACKET, true),
            '~' => (keys::BACKQUOTE, true),
            _ => (char_to_key(c), false),
        };
        let is_caps = c.is_uppercase();

        if key == keys::DELETE {
            let r = e.on_key_ext(key, false, false, false);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            } else {
                screen.pop();
            }
            continue;
        }

        // ESC key: restore to raw ASCII
        if key == keys::ESC {
            let r = e.on_key_ext(key, false, false, false);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            }
            continue;
        }

        if key == keys::SPACE {
            let r = e.on_key_ext(key, false, false, false);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            } else {
                screen.push(' ');
            }
            continue;
        }

        let r = e.on_key_ext(key, is_caps, false, shift);
        if r.action == Action::Send as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
            // For break keys (punctuation), add the character after auto-restore
            // BUT: if key_consumed flag is set (shortcut match), don't add the char
            if keys::is_break_ext(key, shift) && !r.key_consumed() {
                screen.push(c);
            }
        } else {
            screen.push(c);
        }
    }
    screen
}

// ============================================================
// TEST RUNNERS
// ============================================================

/// Run Telex test cases
pub fn telex(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        let result = type_word(&mut e, input);
        assert_eq!(result, *expected, "[Telex] '{}' → '{}'", input, result);
    }
}

/// Run Telex test cases with English auto-restore enabled
pub fn telex_auto_restore(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_english_auto_restore(true);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[Telex AutoRestore] '{}' → '{}'",
            input, result
        );
    }
}

/// Run Telex test cases with auto-capitalize enabled
pub fn telex_auto_capitalize(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_auto_capitalize(true);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[Telex AutoCapitalize] '{}' → '{}'",
            input, result
        );
    }
}

/// Run VNI test cases
pub fn vni(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_method(1);
        let result = type_word(&mut e, input);
        assert_eq!(result, *expected, "[VNI] '{}' → '{}'", input, result);
    }
}

/// Run Telex test cases with traditional tone placement (hòa, thúy style)
pub fn telex_traditional(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_modern_tone(false);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[Telex Traditional] '{}' → '{}'",
            input, result
        );
    }
}

/// Run VNI test cases with traditional tone placement (hòa, thúy style)
pub fn vni_traditional(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut e = Engine::new();
        e.set_method(1);
        e.set_modern_tone(false);
        let result = type_word(&mut e, input);
        assert_eq!(
            result, *expected,
            "[VNI Traditional] '{}' → '{}'",
            input, result
        );
    }
}

/// Simulate typing with extended parameters (supports raw mode prefix)
pub fn type_word_ext(e: &mut Engine, input: &str) -> String {
    let mut screen = String::new();
    for c in input.chars() {
        let (key, shift) = match c {
            '@' => (keys::N2, true),
            '#' => (keys::N3, true),
            ':' => (keys::SEMICOLON, true),
            '/' => (keys::SLASH, false),
            _ => (char_to_key(c), false),
        };

        let is_caps = c.is_uppercase();

        if key == keys::DELETE {
            let r = e.on_key_ext(key, false, false, false);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            } else {
                screen.pop();
            }
            continue;
        }

        if key == keys::ESC {
            let r = e.on_key_ext(key, false, false, false);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            }
            continue;
        }

        if key == keys::SPACE {
            let r = e.on_key_ext(key, false, false, false);
            if r.action == Action::Send as u8 {
                for _ in 0..r.backspace {
                    screen.pop();
                }
                for i in 0..r.count as usize {
                    if let Some(ch) = char::from_u32(r.chars[i]) {
                        screen.push(ch);
                    }
                }
            } else {
                screen.push(' ');
            }
            continue;
        }

        let r = e.on_key_ext(key, is_caps, false, shift);
        if r.action == Action::Send as u8 {
            for _ in 0..r.backspace {
                screen.pop();
            }
            for i in 0..r.count as usize {
                if let Some(ch) = char::from_u32(r.chars[i]) {
                    screen.push(ch);
                }
            }
            if keys::is_break(key) {
                screen.push(c);
            }
        } else {
            screen.push(c);
        }
    }
    screen
}
