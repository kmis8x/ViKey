//! Vowel pattern validation and foreign word detection
//!
//! Contains complex validation rules for vowel patterns and foreign word detection,
//! split from validation.rs to keep file sizes under 200 lines.

use super::super::syllable::{parse, Syllable};
use super::{BufferSnapshot, Rule, ValidationResult};
use crate::data::chars::tone;
use crate::data::{constants, keys};

/// Rule 6: Vowel patterns must be valid Vietnamese (WHITELIST approach)
///
/// Validates against 40 valid patterns from docs 7.6.1:
/// - 29 diphthongs (nguyên âm đôi)
/// - 11 triphthongs (nguyên âm ba)
///
/// This catches common English patterns NOT in Vietnamese:
/// - "ea" (search, beach, teacher) - not valid in Vietnamese
/// - "ou" (you, our, house, about) - not valid in Vietnamese
/// - "yo" (yoke, York, your) - not valid in Vietnamese
///
/// Modifier requirements (circumflex checks) are ONLY enforced when tone info
/// is available (tones not all zeros). This allows legacy is_valid() to work
/// while is_valid_with_tones() can do full validation.
pub(super) fn rule_valid_vowel_pattern(
    snap: &BufferSnapshot,
    syllable: &Syllable,
) -> Option<ValidationResult> {
    if syllable.vowel.len() < 2 {
        return None; // Single vowel always valid
    }

    let vowel_indices: &[usize] = &syllable.vowel;
    let vowel_keys: Vec<u16> = vowel_indices.iter().map(|&i| snap.keys[i]).collect();
    let vowel_tones: Vec<u8> = vowel_indices.iter().map(|&i| snap.tones[i]).collect();

    match vowel_keys.len() {
        2 => {
            let pair = [vowel_keys[0], vowel_keys[1]];

            // Check if base pattern is in whitelist
            if !constants::VALID_DIPHTHONGS.contains(&pair) {
                return Some(ValidationResult::InvalidVowelPattern);
            }

            // Only check modifier requirements when tone info was explicitly provided
            // This is the key fix for "new" → "neư" bug
            // E+U requires circumflex on E (êu valid, eu/eư invalid)
            if snap.has_tone_info
                && constants::V1_CIRCUMFLEX_REQUIRED.contains(&pair)
                && vowel_tones[0] != tone::CIRCUMFLEX
            {
                return Some(ValidationResult::InvalidVowelPattern);
            }

            // V2 circumflex requirements (I+E → iê, U+E → uê, Y+E → yê)
            // Only check when tone info provided AND V2 has wrong modifier
            if snap.has_tone_info && constants::V2_CIRCUMFLEX_REQUIRED.contains(&pair) {
                // If V2 has horn modifier instead of circumflex, it's invalid
                // But if V2 has no modifier yet, allow it (modifier may come later)
                if vowel_tones[1] == tone::HORN {
                    return Some(ValidationResult::InvalidVowelPattern);
                }
            }

            // Breve (ă) restrictions: 'ă' cannot be followed by another vowel
            // Valid: ăm, ăn, ăng, ănh, ăp, ăt, ăc (consonant endings)
            // Valid: oă (in "xoăn" etc.)
            // Invalid: ăi, ăo, ău, ăy (breve + vowel)
            // In Vietnamese, horn tone on 'a' creates breve 'ă'
            if snap.has_tone_info && vowel_keys[0] == keys::A && vowel_tones[0] == tone::HORN {
                // A with breve followed by vowel is invalid
                // (V2 in diphthong is always a vowel, so this is always invalid)
                return Some(ValidationResult::InvalidVowelPattern);
            }
        }
        3 => {
            let triple = [vowel_keys[0], vowel_keys[1], vowel_keys[2]];

            // Check if base pattern is in whitelist
            if !constants::VALID_TRIPHTHONGS.contains(&triple) {
                return Some(ValidationResult::InvalidVowelPattern);
            }

            // Triphthong modifier checks only when tone info provided
            if snap.has_tone_info {
                // uyê requires circumflex on E (last vowel)
                if triple == [keys::U, keys::Y, keys::E] && vowel_tones[2] == tone::HORN {
                    return Some(ValidationResult::InvalidVowelPattern);
                }

                // iêu/yêu requires circumflex on E (middle vowel), U must NOT have horn
                // Issue #145: "view" → "vieư" is invalid (E has no circumflex, U has horn)
                // Valid: "iêu" (E has circumflex, U plain)
                // Invalid: "ieư" (E plain, U has horn)
                if (triple == [keys::I, keys::E, keys::U] || triple == [keys::Y, keys::E, keys::U])
                    && (vowel_tones[1] != tone::CIRCUMFLEX || vowel_tones[2] == tone::HORN)
                {
                    return Some(ValidationResult::InvalidVowelPattern);
                }
            }
        }
        _ => {
            // More than 3 vowels is always invalid
            return Some(ValidationResult::InvalidVowelPattern);
        }
    }

    None
}

/// Rules for pre-transformation validation (excludes vowel pattern check).
/// Used to validate buffer structure before applying tone/mark transformations.
/// Allows intermediate states like "aa" that become valid after transformation.
pub(super) const RULES_FOR_TRANSFORM: &[Rule] = &[
    super::rule_has_vowel,
    super::rule_valid_initial,
    super::rule_all_chars_parsed,
    super::rule_spelling,
    super::rule_valid_final,
    // NOTE: rule_valid_vowel_pattern is excluded - applied only to final results
];

/// Pre-transformation validation (allows intermediate vowel patterns).
///
/// Used by try_tone/try_stroke to validate buffer structure before transformation.
/// Does NOT check vowel patterns since intermediate states like "aa" → "â" are valid.
pub fn is_valid_for_transform(buffer_keys: &[u16]) -> bool {
    is_valid_for_transform_with_foreign(buffer_keys, false)
}

/// Pre-transformation validation with foreign consonants option.
pub fn is_valid_for_transform_with_foreign(
    buffer_keys: &[u16],
    allow_foreign_consonants: bool,
) -> bool {
    if buffer_keys.is_empty() {
        return false;
    }

    let snap =
        BufferSnapshot::from_keys_with_foreign(buffer_keys.to_vec(), allow_foreign_consonants);
    let syllable = parse(&snap.keys);

    for rule in RULES_FOR_TRANSFORM {
        if rule(&snap, &syllable).is_some() {
            return false;
        }
    }

    true
}

/// Check if the buffer shows patterns that suggest foreign word input.
///
/// This is a heuristic to detect when the user is likely typing a foreign word
/// rather than Vietnamese. It checks for:
/// 1. Invalid vowel patterns that don't exist in Vietnamese (using whitelist)
/// 2. Consonant clusters after finals that are common in English (T+R, P+R, C+R)
///
/// Returns true if the pattern suggests foreign word input.
pub fn is_foreign_word_pattern(
    buffer_keys: &[u16],
    _buffer_tones: &[u8],
    modifier_key: u16,
) -> bool {
    let syllable = parse(buffer_keys);

    // Check 1: Invalid vowel patterns (not in whitelist)
    if syllable.vowel.len() >= 2 {
        let vowels: Vec<u16> = syllable.vowel.iter().map(|&i| buffer_keys[i]).collect();

        // Check consecutive pairs for common foreign patterns
        for window in vowels.windows(2) {
            let pair = [window[0], window[1]];
            // "ou" and "yo" are common in English but never valid in Vietnamese
            if pair == [keys::O, keys::U] || pair == [keys::Y, keys::O] {
                return true;
            }
        }

        let is_valid_pattern = match vowels.len() {
            2 => {
                let pair = [vowels[0], vowels[1]];
                constants::VALID_DIPHTHONGS.contains(&pair)
            }
            3 => {
                let triple = [vowels[0], vowels[1], vowels[2]];
                constants::VALID_TRIPHTHONGS.contains(&triple)
            }
            _ => false,
        };

        if !is_valid_pattern {
            return true;
        }
    }

    // Check 2: Consonant clusters common in foreign words (T+R, P+R, C+R)
    if modifier_key == keys::R && syllable.final_c.len() == 1 && !syllable.initial.is_empty() {
        let final_key = buffer_keys[syllable.final_c[0]];
        if matches!(final_key, keys::T | keys::P | keys::C) {
            return true;
        }
    }

    // Check 5: Invalid final consonant + mark modifier → likely English
    // Valid Vietnamese finals: C, M, N, P, T (single) + CH, NG, NH (double)
    if syllable.initial.is_empty() && syllable.vowel.len() == 1 && !syllable.final_c.is_empty() {
        let finals: Vec<u16> = syllable.final_c.iter().map(|&i| buffer_keys[i]).collect();
        let is_invalid_final = match finals.len() {
            1 => {
                let f = finals[0];
                !matches!(f, keys::C | keys::M | keys::N | keys::P | keys::T)
            }
            2 => {
                let pair = [finals[0], finals[1]];
                !constants::VALID_FINALS_2.contains(&pair)
            }
            _ => true,
        };

        if is_invalid_final {
            let is_mark_modifier = matches!(
                modifier_key,
                keys::S | keys::F | keys::R | keys::X | keys::J
            );
            if is_mark_modifier {
                return true;
            }
        }
    }

    false
}
