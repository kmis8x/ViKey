//! Vietnamese Transformation
//!
//! Pattern-based transformation for Vietnamese diacritics.
//! Scans entire buffer instead of case-by-case processing.

use super::buffer::Buffer;
use crate::data::{
    chars::{mark, tone},
    keys,
    vowel::Phonology,
};
use crate::utils;

/// Modifier type detected from key
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierType {
    /// Tone diacritic: circumflex (^), horn, breve
    Tone(u8),
    /// Tone mark: sắc, huyền, hỏi, ngã, nặng
    Mark(u8),
    /// Stroke: d → đ
    Stroke,
    /// Remove last diacritic
    Remove,
}

/// Transformation result
#[derive(Debug)]
pub struct TransformResult {
    /// Positions that were modified
    pub modified_positions: Vec<usize>,
    /// Whether transformation was applied
    pub applied: bool,
}

impl TransformResult {
    pub fn none() -> Self {
        Self {
            modified_positions: vec![],
            applied: false,
        }
    }

    pub fn success(positions: Vec<usize>) -> Self {
        Self {
            modified_positions: positions,
            applied: true,
        }
    }

    pub fn earliest_position(&self) -> Option<usize> {
        self.modified_positions.iter().copied().min()
    }
}

// Child module: revert operations + reposition helper
#[path = "transform_revert.rs"]
mod transform_revert;
pub use transform_revert::{
    apply_remove, apply_stroke, revert_mark, revert_stroke, revert_tone,
};
pub(super) use transform_revert::reposition_mark_if_needed;

/// Apply tone diacritic transformation (^, ơ, ư, ă)
///
/// Pattern-based: scans buffer for matching vowels
pub fn apply_tone(buf: &mut Buffer, key: u16, tone_value: u8, method: u8) -> TransformResult {
    // Find target vowels based on key and method
    let targets = find_tone_targets(buf, key, tone_value, method);

    if targets.is_empty() {
        return TransformResult::none();
    }

    // Apply tone to targets
    let mut positions = vec![];
    for pos in &targets {
        if let Some(c) = buf.get_mut(*pos) {
            if c.tone == tone::NONE {
                c.tone = tone_value;
                positions.push(*pos);
            }
        }
    }

    if positions.is_empty() {
        TransformResult::none()
    } else {
        // After adding tone, reposition mark if needed
        reposition_mark_if_needed(buf);
        TransformResult::success(positions)
    }
}

/// Find which vowel positions should receive the tone modifier
fn find_tone_targets(buf: &Buffer, key: u16, tone_value: u8, method: u8) -> Vec<usize> {
    let mut targets = vec![];

    // Find all vowel positions
    let vowel_positions: Vec<usize> = buf
        .iter()
        .enumerate()
        .filter(|(_, c)| keys::is_vowel(c.key))
        .map(|(i, _)| i)
        .collect();

    if vowel_positions.is_empty() {
        return targets;
    }

    // Telex patterns
    if method == 0 {
        // aa, ee, oo → circumflex (immediate doubling only)
        // The target vowel must be at the LAST position in the buffer
        // This ensures "ee" doubling only works for consecutive presses,
        // not for words like "teacher" where 'e' appears twice non-adjacently
        if tone_value == tone::CIRCUMFLEX && matches!(key, keys::A | keys::E | keys::O) {
            // Issue #312: If ANY vowel in the buffer already has a tone (horn/circumflex/breve),
            // don't trigger same-vowel circumflex. The typed vowel should append as raw letter.
            // Example: "chưa" + "a" → "chưaa" (NOT "chưâ")
            // Issue #211: Also check for marks (sắc/huyền/hỏi/ngã/nặng) - if a vowel already
            // has a mark, the typed vowel should append raw for extended vowel patterns.
            // Example: "quá" + "a" → "quáa" (NOT "quấ")
            let any_vowel_has_tone_or_mark = buf
                .iter()
                .filter(|c| keys::is_vowel(c.key))
                .any(|c| c.has_tone() || c.has_mark());

            if any_vowel_has_tone_or_mark {
                // Skip circumflex, return empty targets to append raw vowel
                return targets;
            }

            // Find matching vowel (same key) - must be at last position
            let last_pos = buf.len().saturating_sub(1);
            for &pos in vowel_positions.iter().rev() {
                if buf.get(pos).map(|c| c.key) == Some(key) && pos == last_pos {
                    targets.push(pos);
                    break;
                }
            }
        }
        // w → horn/breve
        else if tone_value == tone::HORN && key == keys::W {
            let buffer_keys: Vec<u16> = buf.iter().map(|c| c.key).collect();
            targets = Phonology::find_horn_positions(&buffer_keys, &vowel_positions);
        }
    }
    // VNI patterns
    else {
        let buffer_keys: Vec<u16> = buf.iter().map(|c| c.key).collect();

        // 6 → circumflex for a, e, o
        if tone_value == tone::CIRCUMFLEX && key == keys::N6 {
            for &pos in vowel_positions.iter().rev() {
                if matches!(buffer_keys[pos], keys::A | keys::E | keys::O) {
                    targets.push(pos);
                    break;
                }
            }
        }
        // 7 → horn for o, u
        else if tone_value == tone::HORN && key == keys::N7 {
            targets = Phonology::find_horn_positions(&buffer_keys, &vowel_positions);
        }
        // 8 → breve for a only
        else if tone_value == tone::HORN && key == keys::N8 {
            for &pos in vowel_positions.iter().rev() {
                if buffer_keys[pos] == keys::A {
                    targets.push(pos);
                    break;
                }
            }
        }
    }

    targets
}

/// Apply mark transformation (sắc, huyền, hỏi, ngã, nặng)
pub fn apply_mark(buf: &mut Buffer, mark_value: u8, modern: bool) -> TransformResult {
    let vowels = utils::collect_vowels(buf);
    if vowels.is_empty() {
        return TransformResult::none();
    }

    // Find position using phonology rules
    let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
    let has_final = utils::has_final_consonant(buf, last_vowel_pos);
    let has_qu = utils::has_qu_initial(buf);
    let has_gi = utils::has_gi_initial(buf);
    let pos = Phonology::find_tone_position(&vowels, has_final, modern, has_qu, has_gi);

    // Clear any existing mark first
    for v in &vowels {
        if let Some(c) = buf.get_mut(v.pos) {
            c.mark = mark::NONE;
        }
    }

    // Apply new mark
    if let Some(c) = buf.get_mut(pos) {
        c.mark = mark_value;
        return TransformResult::success(vec![pos]);
    }

    TransformResult::none()
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
