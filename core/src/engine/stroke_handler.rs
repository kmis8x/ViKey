use super::{revert, helpers, Engine};
use crate::data::chars::{self, tone};
use crate::data::keys;
use crate::engine::buffer::Char;
use crate::engine::types::{Result, Transform};
use crate::utils;
use super::validation::{is_valid_for_transform_with_foreign, is_valid_with_foreign, is_valid_with_tones};
use crate::engine::syllable;

/// Try to convert 'w' as a vowel shortcut (w → ư)
pub(super) fn try_w_as_vowel(e: &mut Engine, caps: bool) -> Option<Result> {
    // Issue #44: If breve is pending (deferred due to open syllable),
    // don't convert w→ư. Let w be added as regular letter.
    // Example: "aw" → breve deferred → should stay "aw", not become "aư"
    if e.pending_breve_pos.is_some() {
        return None;
    }

    // If user disabled w→ư shortcut at word start, only skip when buffer is empty
    // This allows "hw" → "hư" even when shortcut is disabled
    if e.skip_w_shortcut && e.buf.is_empty() {
        return None;
    }

    // If shortcut was previously skipped, don't try again
    if matches!(e.last_transform, Some(Transform::WShortcutSkipped)) {
        return None;
    }

    // If we already have a complete ươ compound, swallow the second 'w'
    // This handles "dduwowcj" where the second 'w' should be no-op
    // Use send(0, []) to intercept and consume the key without output
    if e.has_complete_uo_compound() {
        return Some(Result::send(0, &[]));
    }

    // Check revert: ww → w (skip shortcut)
    // Preserve original case: Ww → W, wW → w
    if let Some(Transform::WAsVowel) = e.last_transform {
        e.last_transform = Some(Transform::WShortcutSkipped);
        // Track ww pattern for whitelist-based restore
        e.had_telex_transform = true;
        // Store raw_input BEFORE modification for whitelist lookup
        e.telex_double_raw = Some(get_raw_input_string_preserve_case(e));
        // Get original case from buffer before popping
        let original_caps = e.buf.last().map(|c| c.caps).unwrap_or(caps);
        e.buf.pop();
        e.buf.push(Char::new(keys::W, original_caps));
        // Fix raw_input: "ww" typed → raw has [w,w] but buffer is "w"
        // Remove the shortcut-triggering 'w' from raw_input so restore works correctly
        if e.raw_input.len() >= 2 {
            let current = e.raw_input.pop(); // current 'w' (just added)
            e.raw_input.pop(); // shortcut-trigger 'w' (consumed, discard)
            if let Some(c) = current {
                e.raw_input.push(c);
            }
        }
        // Store length AFTER modification
        e.telex_double_raw_len = e.raw_input.len();
        let w = if original_caps { 'W' } else { 'w' };
        return Some(Result::send(1, &[w]));
    }

    // Try adding U (ư base) to buffer and validate
    e.buf.push(Char::new(keys::U, caps));

    // Set horn tone to make it ư
    if let Some(c) = e.buf.get_mut(e.buf.len() - 1) {
        c.tone = tone::HORN;
    }

    // Validate: is this valid Vietnamese?
    // Use is_valid_with_tones to check modifier requirements (e.g., E+U needs circumflex)
    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
    let buffer_tones: Vec<u8> = e.buf.iter().map(|c| c.tone).collect();
    if is_valid_with_tones(&buffer_keys, &buffer_tones) {
        e.last_transform = Some(Transform::WAsVowel);
        e.had_any_transform = true;

        // W shortcut adds ư without replacing anything on screen
        // (the raw 'w' key was never output, so no backspace needed)
        let vowel_char = chars::to_char(keys::U, caps, tone::HORN, 0).unwrap();
        return Some(Result::send(0, &[vowel_char]));
    }

    // Invalid - remove the U we added
    e.buf.pop();
    None
}

/// Try to apply stroke transformation by scanning buffer
///
/// Issue #51: In Telex mode, only apply stroke when the new 'd' is ADJACENT to
/// an existing 'd'. According to Vietnamese Telex docs (Section 9.2.2), "dd" → "đ"
/// should only work when the two 'd's are consecutive.
///
/// In VNI mode, '9' is always an intentional stroke command (not a letter), so
/// delayed stroke is allowed (e.g., "duong9" → "đuong").
pub(super) fn try_stroke(e: &mut Engine, key: u16, caps: bool) -> Option<Result> {
    // If stroke was already reverted in this word (ddd → dd), skip further stroke attempts
    // This prevents "ddddd" from oscillating and ensures subsequent 'd's are just letters
    if e.stroke_reverted && key == keys::D {
        return None;
    }

    // Check for stroke revert first: ddd → dd
    // If last transform was stroke and same key pressed again, revert the stroke
    if let Some(Transform::Stroke(last_key)) = e.last_transform {
        if last_key == key {
            // Find the stroked 'd' to revert
            if let Some(pos) = e.buf.iter().position(|c| c.key == keys::D && c.stroke) {
                // Revert: un-stroke the 'd'
                if let Some(c) = e.buf.get_mut(pos) {
                    c.stroke = false;
                }
                // Add another 'd' as normal char (preserve caps state)
                e.buf.push(Char::new(key, caps));
                e.last_transform = None;
                // Mark that stroke was reverted - subsequent 'd' keys will be normal letters
                e.stroke_reverted = true;
                // Track dd pattern for whitelist-based restore
                e.had_telex_transform = true;
                // Store raw_input BEFORE modification for whitelist lookup
                e.telex_double_raw = Some(get_raw_input_string_preserve_case(e));
                // Fix raw_input: remove the stroke-triggering 'd'
                if e.raw_input.len() >= 2 {
                    let current = e.raw_input.pop(); // current 'd' (just added)
                    e.raw_input.pop(); // stroke-trigger 'd' (consumed, discard)
                    if let Some(c) = current {
                        e.raw_input.push(c);
                    }
                }
                // Store length AFTER modification
                e.telex_double_raw_len = e.raw_input.len();
                // Use rebuild_from_after_insert because the new 'd' was just pushed
                // and hasn't been displayed on screen yet
                return Some(helpers::rebuild_from_after_insert(&e.buf, pos));
            }
        }
    }

    // Check for short-pattern stroke revert: dadd → dad
    if let Some(Transform::ShortPatternStroke) = e.last_transform {
        if key == keys::D {
            if let Some(pos) = e.buf.iter().position(|c| c.key == keys::D && c.stroke) {
                if let Some(c) = e.buf.get_mut(pos) {
                    c.stroke = false;
                }
                e.buf.push(Char::new(key, caps));
                e.last_transform = None;
                e.stroke_reverted = true;
                e.had_telex_transform = true;
                e.telex_double_raw = Some(get_raw_input_string_preserve_case(e));
                if e.raw_input.len() >= 2 {
                    let current = e.raw_input.pop();
                    e.raw_input.pop();
                    if let Some(c) = current {
                        e.raw_input.push(c);
                    }
                }
                e.telex_double_raw_len = e.raw_input.len();
                return Some(helpers::rebuild_from_after_insert(&e.buf, pos));
            }
        }
    }

    // Collect buffer keys once for all validations
    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
    let has_vowel = buffer_keys.iter().any(|&k| keys::is_vowel(k));

    // Find position of un-stroked 'd' to apply stroke
    let (pos, is_short_pattern_stroke) = if e.method == 0 {
        // Telex: First try adjacent 'd' (last char is un-stroked d)
        let last_pos = e.buf.len().checked_sub(1)?;
        let last_char = e.buf.get(last_pos)?;

        if last_char.key == keys::D && !last_char.stroke {
            // Adjacent stroke: "dd" → "đ" (not a short pattern)
            (last_pos, false)
        } else {
            // Delayed stroke: check if initial 'd' can be stroked
            let first_char = e.buf.get(0)?;
            if first_char.key != keys::D || first_char.stroke {
                return None;
            }

            // Must have at least one vowel for delayed stroke
            if !has_vowel {
                return None;
            }

            // Must form valid Vietnamese for delayed stroke
            if !is_valid_with_foreign(&buffer_keys, e.allow_foreign_consonants) {
                return None;
            }

            // For open syllables (d + vowel only), defer stroke to try_mark
            let syllable = syllable::parse(&buffer_keys);
            let has_mark_applied = e.buf.iter().any(|c| c.mark > 0);
            // Allow 'd' to trigger immediate stroke on open syllables with d + vowels only
            let is_d_vowels_only_pattern = key == keys::D
                && e.buf.len() >= 2
                && e.buf.iter().skip(1).all(|c| keys::is_vowel(c.key));
            if syllable.final_c.is_empty() && !has_mark_applied && !is_d_vowels_only_pattern {
                return None;
            }

            (0, is_d_vowels_only_pattern && !has_mark_applied)
        }
    } else {
        // VNI: Allow delayed stroke - find first un-stroked 'd' anywhere in buffer
        let pos = e
            .buf
            .iter()
            .enumerate()
            .find(|(_, c)| c.key == keys::D && !c.stroke)
            .map(|(i, _)| i)?;
        (pos, false)
    };

    // Check revert: if last transform was stroke on same key at same position
    if let Some(Transform::Stroke(last_key)) = e.last_transform {
        if last_key == key {
            return Some(revert::revert_stroke(e, key, pos));
        }
    }

    // Validate buffer structure before applying stroke
    if !e.free_tone_enabled
        && has_vowel
        && !is_valid_for_transform_with_foreign(&buffer_keys, e.allow_foreign_consonants)
    {
        return None;
    }

    // Mark as stroked
    if let Some(c) = e.buf.get_mut(pos) {
        c.stroke = true;
    }

    // Track transform type for potential revert
    e.last_transform = if is_short_pattern_stroke {
        Some(Transform::ShortPatternStroke)
    } else {
        Some(Transform::Stroke(key))
    };
    e.had_any_transform = true;
    e.had_telex_transform = true;
    Some(helpers::rebuild_from(&e.buf, pos))
}

/// Check if buffer starts with w-as-vowel transform (U with horn at position 0)
pub(super) fn has_w_as_vowel_transform(e: &Engine) -> bool {
    e.buf
        .get(0)
        .map(|c| c.key == keys::U && c.tone == tone::HORN)
        .unwrap_or(false)
}

/// Revert w-as-vowel transforms and rebuild output
pub(super) fn revert_w_as_vowel_transforms(e: &mut Engine) -> Result {
    if !has_w_as_vowel_transform(e) {
        return Result::none();
    }

    let horn_positions: Vec<usize> = e
        .buf
        .iter()
        .enumerate()
        .filter(|(_, c)| c.tone == tone::HORN)
        .map(|(i, _)| i)
        .collect();

    if horn_positions.is_empty() {
        return Result::none();
    }

    let first_pos = horn_positions[0];

    // Clear horn tones and change U back to W (for w-as-vowel positions)
    for &pos in &horn_positions {
        if let Some(c) = e.buf.get_mut(pos) {
            // U with horn was from 'w' → change key to W
            if c.key == keys::U {
                c.key = keys::W;
            }
            c.tone = tone::NONE;
        }
    }

    helpers::rebuild_from(&e.buf, first_pos)
}

/// Try bracket as vowel shortcut (] → ư, [ → ơ)
pub(super) fn try_bracket_as_vowel(e: &mut Engine, key: u16, caps: bool) -> Option<Result> {
    // Check if bracket shortcut is enabled
    if !e.bracket_shortcut {
        return None;
    }

    // Check for revert: if last transform was BracketAsVowel with same bracket
    if e.last_transform == Some(Transform::BracketAsVowel) && !e.buf.is_empty() {
        if let Some(last_char) = e.buf.last() {
            let should_revert = match key {
                keys::RBRACKET => last_char.key == keys::U && last_char.tone == tone::HORN,
                keys::LBRACKET => last_char.key == keys::O && last_char.tone == tone::HORN,
                _ => false,
            };

            if should_revert {
                e.buf.pop();
                e.raw_input.pop();
                e.last_transform = None;

                let bracket_char = match (key, caps) {
                    (keys::RBRACKET, true) => '}',
                    (keys::RBRACKET, false) => ']',
                    (keys::LBRACKET, true) => '{',
                    (keys::LBRACKET, false) => '[',
                    (_, true) => '{',
                    (_, false) => '[',
                };
                return Some(Result::send_consumed(1, &[bracket_char]));
            }
        }
    }

    // Determine target vowel based on bracket key
    let base_key = if key == keys::RBRACKET {
        keys::U // ] → ư
    } else {
        keys::O // [ → ơ
    };

    // Add vowel to buffer
    e.buf.push(Char::new(base_key, caps));

    // Set horn tone
    if let Some(c) = e.buf.get_mut(e.buf.len() - 1) {
        c.tone = tone::HORN;
    }

    // Validate
    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
    let buffer_tones: Vec<u8> = e.buf.iter().map(|c| c.tone).collect();
    if !is_valid_with_tones(&buffer_keys, &buffer_tones) {
        e.buf.pop();
        return None;
    }

    // Track raw input for ESC restore
    e.raw_input.push((key, caps, false));

    // Mark transform
    e.last_transform = Some(Transform::BracketAsVowel);
    e.had_any_transform = true;

    let vowel_char = chars::to_char(base_key, caps, tone::HORN, 0).unwrap();
    Some(Result::send_consumed(0, &[vowel_char]))
}

/// Get raw_input as string preserving original case
fn get_raw_input_string_preserve_case(e: &Engine) -> String {
    e.raw_input
        .iter()
        .filter_map(|&(key, caps, _shift)| utils::key_to_char(key, caps))
        .collect()
}
