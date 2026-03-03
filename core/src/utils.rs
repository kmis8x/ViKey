//! Shared utilities for Vietnamese IME processing
//!
//! Contains common functions used across engine modules to avoid duplication.
//! Test utilities live in `test_utils.rs` and are re-exported under `#[cfg(test)]`.

use crate::data::{
    chars::tone,
    keys,
    vowel::{Modifier, Vowel},
};
use crate::engine::buffer::Buffer;

/// Convert key code to character
pub fn key_to_char(key: u16, caps: bool) -> Option<char> {
    let ch = match key {
        keys::A => 'a',
        keys::B => 'b',
        keys::C => 'c',
        keys::D => 'd',
        keys::E => 'e',
        keys::F => 'f',
        keys::G => 'g',
        keys::H => 'h',
        keys::I => 'i',
        keys::J => 'j',
        keys::K => 'k',
        keys::L => 'l',
        keys::M => 'm',
        keys::N => 'n',
        keys::O => 'o',
        keys::P => 'p',
        keys::Q => 'q',
        keys::R => 'r',
        keys::S => 's',
        keys::T => 't',
        keys::U => 'u',
        keys::V => 'v',
        keys::W => 'w',
        keys::X => 'x',
        keys::Y => 'y',
        keys::Z => 'z',
        keys::N0 => return Some('0'),
        keys::N1 => return Some('1'),
        keys::N2 => return Some('2'),
        keys::N3 => return Some('3'),
        keys::N4 => return Some('4'),
        keys::N5 => return Some('5'),
        keys::N6 => return Some('6'),
        keys::N7 => return Some('7'),
        keys::N8 => return Some('8'),
        keys::N9 => return Some('9'),
        _ => return None,
    };
    Some(if caps { ch.to_ascii_uppercase() } else { ch })
}

/// Convert key code to character with shift state support
/// Handles shifted symbols like @ (Shift+2), # (Shift+3), etc.
pub fn key_to_char_ext(key: u16, caps: bool, shift: bool) -> Option<char> {
    if shift {
        return match key {
            keys::N1 => Some('!'),
            keys::N2 => Some('@'),
            keys::N3 => Some('#'),
            keys::N4 => Some('$'),
            keys::N5 => Some('%'),
            keys::N6 => Some('^'),
            keys::N7 => Some('&'),
            keys::N8 => Some('*'),
            keys::N9 => Some('('),
            keys::N0 => Some(')'),
            keys::MINUS => Some('_'),
            keys::EQUAL => Some('+'),
            keys::SEMICOLON => Some(':'),
            keys::QUOTE => Some('"'),
            keys::COMMA => Some('<'),
            keys::DOT => Some('>'),
            keys::SLASH => Some('?'),
            keys::BACKSLASH => Some('|'),
            keys::LBRACKET => Some('{'),
            keys::RBRACKET => Some('}'),
            keys::BACKQUOTE => Some('~'),
            _ => key_to_char(key, caps),
        };
    }
    key_to_char(key, caps)
}

/// Collect vowels from buffer with phonological info
pub fn collect_vowels(buf: &Buffer) -> Vec<Vowel> {
    buf.iter()
        .enumerate()
        .filter(|(_, c)| keys::is_vowel(c.key))
        .map(|(pos, c)| {
            let modifier = match c.tone {
                tone::CIRCUMFLEX => Modifier::Circumflex,
                tone::HORN => Modifier::Horn,
                _ => Modifier::None,
            };
            Vowel::new(c.key, modifier, pos)
        })
        .collect()
}

/// Check if there's a consonant after position
pub fn has_final_consonant(buf: &Buffer, after_pos: usize) -> bool {
    (after_pos + 1..buf.len()).any(|i| {
        buf.get(i)
            .map(|c| keys::is_consonant(c.key))
            .unwrap_or(false)
    })
}

/// Check if 'q' precedes 'u' in buffer
pub fn has_qu_initial(buf: &Buffer) -> bool {
    for (i, c) in buf.iter().enumerate() {
        if c.key == keys::U && i > 0 {
            if let Some(prev) = buf.get(i - 1) {
                return prev.key == keys::Q;
            }
        }
    }
    false
}

/// Check if 'gi' is initial followed by another vowel
/// e.g., "gia", "giau" → gi is initial, 'i' is NOT a vowel
pub fn has_gi_initial(buf: &Buffer) -> bool {
    if buf.len() < 3 {
        return false;
    }
    let first = buf.get(0).map(|c| c.key);
    let second = buf.get(1).map(|c| c.key);
    let third = buf.get(2).map(|c| c.key);

    matches!((first, second), (Some(keys::G), Some(keys::I)))
        && third.map(keys::is_vowel).unwrap_or(false)
}

// Re-export test utilities for use in other test modules
#[cfg(test)]
#[path = "test_utils.rs"]
pub mod test_utils;

#[cfg(test)]
pub use test_utils::*;
