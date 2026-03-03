use crate::data::keys;
use crate::engine::buffer::Buffer;
use crate::engine::types::Result;
use crate::{data::chars, utils};

/// Word history ring buffer capacity (stores last N committed words)
pub(super) const HISTORY_CAPACITY: usize = 10;

/// Ring buffer for word history (stack-allocated, O(1) push/pop)
///
/// Used for backspace-after-space feature: when user presses backspace
/// immediately after committing a word with space, restore the previous
/// buffer state to allow editing.
pub(crate) struct WordHistory {
    pub(super) data: [Buffer; HISTORY_CAPACITY],
    pub(super) head: usize,
    pub(super) len: usize,
}

impl WordHistory {
    pub(super) fn new() -> Self {
        Self {
            data: std::array::from_fn(|_| Buffer::new()),
            head: 0,
            len: 0,
        }
    }

    /// Push buffer to history (overwrites oldest if full)
    pub(super) fn push(&mut self, buf: Buffer) {
        self.data[self.head] = buf;
        self.head = (self.head + 1) % HISTORY_CAPACITY;
        if self.len < HISTORY_CAPACITY {
            self.len += 1;
        }
    }

    /// Pop most recent buffer from history
    pub(super) fn pop(&mut self) -> Option<Buffer> {
        if self.len == 0 {
            return None;
        }
        self.head = (self.head + HISTORY_CAPACITY - 1) % HISTORY_CAPACITY;
        self.len -= 1;
        Some(self.data[self.head].clone())
    }

    pub(super) fn clear(&mut self) {
        self.len = 0;
        self.head = 0;
    }
}

/// Check if key is sentence-ending punctuation (. ! ?) but NOT Enter
/// Issue #185: Only set pending_capitalize after punctuation + space
#[inline]
pub(super) fn is_sentence_ending_punctuation(key: u16, shift: bool) -> bool {
    key == keys::DOT
        || (shift && key == keys::N1) // !
        || (shift && key == keys::SLASH) // ?
}

/// Check if a break key should reset pending_capitalize
/// Neutral keys like quotes, parentheses, arrows should NOT reset (preserve pending)
/// Word-breaking keys like comma should reset
#[inline]
pub(super) fn should_reset_pending_capitalize(key: u16, shift: bool) -> bool {
    let is_neutral = key == keys::QUOTE
        || key == keys::LBRACKET
        || key == keys::RBRACKET
        || (shift && key == keys::N9)  // (
        || (shift && key == keys::N0)  // )
        || key == keys::LEFT
        || key == keys::RIGHT
        || key == keys::UP
        || key == keys::DOWN
        || key == keys::TAB
        || key == keys::ESC;

    !is_neutral
}

/// Convert break key to its character representation
/// Handles both shifted and unshifted break characters for shortcut matching.
/// Examples: MINUS → '-', Shift+DOT → '>', Shift+MINUS → '_'
pub(super) fn break_key_to_char(key: u16, shift: bool) -> Option<char> {
    if shift {
        match key {
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
            _ => None,
        }
    } else {
        match key {
            keys::MINUS => Some('-'),
            keys::EQUAL => Some('='),
            keys::SEMICOLON => Some(';'),
            keys::QUOTE => Some('\''),
            keys::COMMA => Some(','),
            keys::DOT => Some('.'),
            keys::SLASH => Some('/'),
            keys::BACKSLASH => Some('\\'),
            keys::LBRACKET => Some('['),
            keys::RBRACKET => Some(']'),
            keys::BACKQUOTE => Some('`'),
            _ => None,
        }
    }
}

/// Rebuild output from position `from` to end of buffer.
/// Backspace count = number of chars from `from` to end.
pub(super) fn rebuild_from(buf: &Buffer, from: usize) -> Result {
    let mut output = Vec::with_capacity(buf.len() - from);
    let mut backspace = 0u8;

    for i in from..buf.len() {
        if let Some(c) = buf.get(i) {
            backspace += 1;

            if c.key == keys::D && c.stroke {
                output.push(chars::get_d(c.caps));
            } else if let Some(ch) = chars::to_char(c.key, c.caps, c.tone, c.mark) {
                output.push(ch);
            } else if let Some(ch) = utils::key_to_char(c.key, c.caps) {
                output.push(ch);
            }
        }
    }

    if output.is_empty() {
        Result::none()
    } else {
        Result::send(backspace, &output)
    }
}

/// Rebuild output from position after a new character was inserted.
///
/// Unlike rebuild_from, this accounts for the fact that the last character
/// in the buffer was just added but NOT yet displayed on screen.
/// So backspace count = (chars from `from` to end - 1) because last char isn't on screen.
pub(super) fn rebuild_from_after_insert(buf: &Buffer, from: usize) -> Result {
    if buf.is_empty() {
        return Result::none();
    }

    let mut output = Vec::with_capacity(buf.len() - from);
    let backspace = (buf.len().saturating_sub(1).saturating_sub(from)) as u8;

    for i in from..buf.len() {
        if let Some(c) = buf.get(i) {
            if c.key == keys::D && c.stroke {
                output.push(chars::get_d(c.caps));
            } else if let Some(ch) = chars::to_char(c.key, c.caps, c.tone, c.mark) {
                output.push(ch);
            } else if let Some(ch) = utils::key_to_char(c.key, c.caps) {
                output.push(ch);
            }
        }
    }

    if output.is_empty() {
        Result::none()
    } else {
        Result::send(backspace, &output)
    }
}
