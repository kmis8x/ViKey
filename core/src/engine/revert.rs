use super::{Engine, helpers};
use crate::data::{chars, keys};
use crate::data::chars::{mark, tone};
use crate::engine::buffer::Char;
use crate::engine::types::Result;
use crate::utils;

/// Revert the most recent transform and rebuild output from `pos`
pub(super) fn revert_and_rebuild(e: &mut Engine, pos: usize, key: u16, caps: bool) -> Result {
    // Calculate backspace BEFORE adding key (based on old buffer state)
    let backspace = (e.buf.len() - pos) as u8;

    // Add the reverted key to buffer so validation sees the full sequence
    e.buf.push(Char::new(key, caps));

    // Build output from position (includes new key)
    // Use chars::to_char to preserve mark (sắc/huyền/etc) on reverted vowels
    let mut output = Vec::with_capacity(e.buf.len() - pos);
    for i in pos..e.buf.len() {
        if let Some(c) = e.buf.get(i) {
            if c.key == keys::D && c.stroke {
                output.push(chars::get_d(c.caps));
            } else if let Some(ch) = chars::to_char(c.key, c.caps, c.tone, c.mark) {
                output.push(ch);
            } else if let Some(ch) = utils::key_to_char(c.key, c.caps) {
                output.push(ch);
            }
        }
    }

    Result::send(backspace, &output)
}

/// Revert tone transformation
pub(super) fn revert_tone(e: &mut Engine, key: u16, caps: bool) -> Result {
    e.last_transform = None;
    // Issue #211: Track which vowel triggered revert for extended vowel mode
    // After revert, subsequent same-key vowels append raw instead of re-transforming
    e.reverted_circumflex_key = Some(key);

    for pos in e.buf.find_vowels().into_iter().rev() {
        if let Some(c) = e.buf.get_mut(pos) {
            if c.tone > tone::NONE {
                c.tone = tone::NONE;
                // Track for auto-restore logic (double ss/ff detection)
                e.had_mark_revert = true;
                // Track ww pattern for whitelist-based restore
                e.had_telex_transform = true;
                // Store raw_input BEFORE modification for whitelist lookup
                e.telex_double_raw = Some(get_raw_input_string_preserve_case(e));
                // Fix raw_input: "ww" typed → raw has [w,w] but buffer is "w"
                // Remove the tone-triggering key from raw_input so restore works correctly
                // raw_input: [a, w, w] → [a, w] (remove first 'w' that triggered tone)
                // This ensures "awwait" → "await" not "awwait" on auto-restore
                if e.raw_input.len() >= 2 {
                    let current = e.raw_input.pop(); // current key (just added)
                    e.raw_input.pop(); // tone-trigger key (consumed, discard)
                    if let Some(c) = current {
                        e.raw_input.push(c);
                    }
                }
                // Store length AFTER modification
                e.telex_double_raw_len = e.raw_input.len();
                return revert_and_rebuild(e, pos, key, caps);
            }
        }
    }
    Result::none()
}

/// Revert mark transformation
/// When mark is reverted, only the reverting key appears as a letter.
/// Standard behavior: "ass" → "as" (first 's' was modifier, second 's' reverts + outputs one 's')
/// This matches standard Vietnamese IME behavior (UniKey, ibus-unikey, etc.)
pub(super) fn revert_mark(e: &mut Engine, key: u16, caps: bool) -> Result {
    e.last_transform = None;
    e.had_mark_revert = true; // Track for auto-restore
                               // Set had_telex_transform for whitelist-based auto-restore
                               // This allows "taxxi" → "taxi" (not in whitelist → keep buffer)
    e.had_telex_transform = true;
    // Store raw_input for whitelist lookup
    e.telex_double_raw = Some(get_raw_input_string_preserve_case(e));
    e.telex_double_raw_len = e.raw_input.len();

    for pos in e.buf.find_vowels().into_iter().rev() {
        if let Some(c) = e.buf.get_mut(pos) {
            if c.mark > mark::NONE {
                c.mark = mark::NONE;

                // Set flag to defer raw_input pop until next key
                // If next key is CONSONANT: pop the mark key (user intended revert)
                //   Example: "tesst" → next is 't' (consonant) → pop → "test"
                // If next key is VOWEL: don't pop (user typing English word like "issue")
                //   Example: "issue" → next is 'u' (vowel) → keep → "issue"
                e.pending_mark_revert_pop = true;

                // Add only the reverting key (current key being pressed)
                // The original mark key was consumed as a modifier and doesn't produce output
                e.buf.push(Char::new(key, caps));

                // Calculate backspace and output
                let backspace = (e.buf.len() - pos - 1) as u8; // -1 because we added 1 char
                let output: Vec<char> = (pos..e.buf.len())
                    .filter_map(|i| e.buf.get(i))
                    .filter_map(|c| utils::key_to_char(c.key, c.caps))
                    .collect();

                return Result::send(backspace, &output);
            }
        }
    }
    Result::none()
}

/// Revert stroke transformation at specific position
pub(super) fn revert_stroke(e: &mut Engine, key: u16, pos: usize) -> Result {
    e.last_transform = None;

    if let Some(c) = e.buf.get_mut(pos) {
        if c.key == keys::D && !c.stroke {
            // Un-stroked d found at pos - this means we need to add another d
            let caps = c.caps;
            e.buf.push(Char::new(key, caps));
            return helpers::rebuild_from(&e.buf, pos);
        }
    }
    Result::none()
}

/// Try to apply remove modifier
/// Returns Some(Result) if a mark/tone was removed, None if nothing to remove
/// When None is returned, the key falls through to handle_normal_letter()
pub(super) fn try_remove(e: &mut Engine) -> Option<Result> {
    e.last_transform = None;
    for pos in e.buf.find_vowels().into_iter().rev() {
        if let Some(c) = e.buf.get_mut(pos) {
            if c.mark > mark::NONE {
                c.mark = mark::NONE;
                return Some(helpers::rebuild_from(&e.buf, pos));
            }
            if c.tone > tone::NONE {
                c.tone = tone::NONE;
                return Some(helpers::rebuild_from(&e.buf, pos));
            }
        }
    }
    // Nothing to remove - return None so key can be processed as normal letter
    // This allows shortcuts like "zz" to work
    None
}

/// Get raw_input as string preserving original case (used for whitelist lookup)
fn get_raw_input_string_preserve_case(e: &Engine) -> String {
    e.raw_input
        .iter()
        .filter_map(|&(key, caps, _shift)| utils::key_to_char(key, caps))
        .collect()
}
