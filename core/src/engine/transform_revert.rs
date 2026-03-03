//! Transform revert operations
//!
//! Contains apply_stroke, apply_remove, revert_tone/mark/stroke,
//! and reposition_mark_if_needed - split from transform.rs to keep
//! file sizes under 200 lines.

use super::super::buffer::Buffer;
use crate::data::{
    chars::{mark, tone},
    keys,
    vowel::Phonology,
};
use crate::utils;

use super::TransformResult;

/// Apply stroke transformation (d → đ)
///
/// Scans buffer for 'd' at any position
pub fn apply_stroke(buf: &mut Buffer) -> TransformResult {
    // Find first 'd' that hasn't been stroked
    for i in 0..buf.len() {
        if let Some(c) = buf.get_mut(i) {
            if c.key == keys::D && !c.stroke {
                c.stroke = true;
                return TransformResult::success(vec![i]);
            }
        }
    }
    TransformResult::none()
}

/// Remove last diacritic (mark first, then tone)
pub fn apply_remove(buf: &mut Buffer) -> TransformResult {
    let vowel_positions = buf.find_vowels();

    // Try to remove mark first
    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.mark > mark::NONE {
                c.mark = mark::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    // Then try to remove tone
    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.tone > tone::NONE {
                c.tone = tone::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    TransformResult::none()
}

/// Revert tone transformation
pub fn revert_tone(buf: &mut Buffer, target_key: u16) -> TransformResult {
    let vowel_positions = buf.find_vowels();

    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.key == target_key && c.tone > tone::NONE {
                c.tone = tone::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    TransformResult::none()
}

/// Revert mark transformation
pub fn revert_mark(buf: &mut Buffer) -> TransformResult {
    let vowel_positions = buf.find_vowels();

    for pos in vowel_positions.iter().rev() {
        if let Some(c) = buf.get_mut(*pos) {
            if c.mark > mark::NONE {
                c.mark = mark::NONE;
                return TransformResult::success(vec![*pos]);
            }
        }
    }

    TransformResult::none()
}

/// Revert stroke transformation
pub fn revert_stroke(buf: &mut Buffer) -> TransformResult {
    // Find stroked 'd' and un-stroke it
    for i in 0..buf.len() {
        if let Some(c) = buf.get_mut(i) {
            if c.key == keys::D && c.stroke {
                c.stroke = false;
                return TransformResult::success(vec![i]);
            }
        }
    }
    TransformResult::none()
}

/// Reposition mark after tone change if needed
pub fn reposition_mark_if_needed(buf: &mut Buffer) {
    // Find current mark
    let mark_info: Option<(usize, u8)> = buf
        .iter()
        .enumerate()
        .find(|(_, c)| c.mark > 0)
        .map(|(i, c)| (i, c.mark));

    if let Some((old_pos, mark_value)) = mark_info {
        let vowels = utils::collect_vowels(buf);
        if vowels.is_empty() {
            return;
        }

        let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
        let has_final = utils::has_final_consonant(buf, last_vowel_pos);
        let has_qu = utils::has_qu_initial(buf);
        let has_gi = utils::has_gi_initial(buf);
        let new_pos = Phonology::find_tone_position(&vowels, has_final, true, has_qu, has_gi);

        if new_pos != old_pos {
            // Clear old mark
            if let Some(c) = buf.get_mut(old_pos) {
                c.mark = 0;
            }
            // Set new mark
            if let Some(c) = buf.get_mut(new_pos) {
                c.mark = mark_value;
            }
        }
    }
}
