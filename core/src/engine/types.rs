use crate::engine::buffer::MAX;

/// Engine action result
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    None = 0,
    Send = 1,
    Restore = 2,
}

/// Result for FFI
#[repr(C)]
pub struct Result {
    pub chars: [u32; MAX],
    pub action: u8,
    pub backspace: u8,
    pub count: u8,
    /// Flags byte:
    /// - bit 0 (0x01): key_consumed - if set, the trigger key should NOT be passed through
    ///   Used for shortcuts where the trigger key is part of the replacement
    pub flags: u8,
}

/// Flag: key was consumed by shortcut, don't pass through
pub const FLAG_KEY_CONSUMED: u8 = 0x01;

impl Result {
    pub fn none() -> Self {
        Self {
            chars: [0; MAX],
            action: Action::None as u8,
            backspace: 0,
            count: 0,
            flags: 0,
        }
    }

    pub fn send(backspace: u8, chars: &[char]) -> Self {
        let mut result = Self {
            chars: [0; MAX],
            action: Action::Send as u8,
            backspace,
            count: chars.len().min(MAX) as u8,
            flags: 0,
        };
        for (i, &c) in chars.iter().take(MAX).enumerate() {
            result.chars[i] = c as u32;
        }
        result
    }

    /// Send with key_consumed flag set (shortcut consumed the trigger key)
    pub fn send_consumed(backspace: u8, chars: &[char]) -> Self {
        let mut result = Self::send(backspace, chars);
        result.flags = FLAG_KEY_CONSUMED;
        result
    }

    /// Check if key was consumed (should not be passed through)
    pub fn key_consumed(&self) -> bool {
        self.flags & FLAG_KEY_CONSUMED != 0
    }
}

/// Transform type for revert tracking
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Transform {
    Mark(u16, u8),
    Tone(u16, u8),
    Stroke(u16),
    /// Short-pattern stroke (d + vowel + d → đ + vowel)
    /// This is revertible if next character creates invalid Vietnamese
    ShortPatternStroke,
    /// W as vowel ư (for revert: ww → w)
    WAsVowel,
    /// W shortcut was explicitly skipped (prevent re-transformation)
    WShortcutSkipped,
    /// Bracket as vowel: ] → ư, [ → ơ (Issue #159)
    BracketAsVowel,
}
