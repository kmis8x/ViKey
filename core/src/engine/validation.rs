//! Vietnamese Syllable Validation
//!
//! Whitelist-based validation for Vietnamese syllables.
//! Uses valid patterns from docs/vietnamese-language-system.md Section 7.6.1

use super::syllable::{parse, Syllable};
use crate::data::constants;
use crate::data::keys;

/// Validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    InvalidInitial,
    InvalidFinal,
    InvalidSpelling,
    InvalidVowelPattern,
    NoVowel,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}

// =============================================================================
// BUFFER SNAPSHOT - Keys + Modifiers for validation
// =============================================================================

/// Snapshot of buffer state for validation
/// Contains both keys and their modifiers (tones)
pub struct BufferSnapshot {
    pub keys: Vec<u16>,
    pub tones: Vec<u8>,
    /// True when tones were explicitly provided (validate modifier requirements)
    /// False when created from keys-only (legacy, skip modifier checks)
    pub has_tone_info: bool,
    /// True when foreign consonants (z, w, j, f) are allowed as valid initials
    pub allow_foreign_consonants: bool,
}

impl BufferSnapshot {
    /// Create from keys only (no modifier info - legacy compatibility)
    /// Modifier requirements will NOT be enforced
    pub fn from_keys(keys: Vec<u16>) -> Self {
        let len = keys.len();
        Self {
            keys,
            tones: vec![0; len],
            has_tone_info: false,
            allow_foreign_consonants: false,
        }
    }

    /// Create from keys with foreign consonants setting
    pub fn from_keys_with_foreign(keys: Vec<u16>, allow_foreign_consonants: bool) -> Self {
        let len = keys.len();
        Self {
            keys,
            tones: vec![0; len],
            has_tone_info: false,
            allow_foreign_consonants,
        }
    }
}

// =============================================================================
// VALIDATION RULES
// =============================================================================

/// Rule type: takes buffer snapshot and parsed syllable, returns error or None
pub(super) type Rule = fn(&BufferSnapshot, &Syllable) -> Option<ValidationResult>;

/// All validation rules in order of priority
const RULES: &[Rule] = &[
    rule_has_vowel,
    rule_valid_initial,
    rule_all_chars_parsed,
    rule_spelling,
    rule_valid_final,
    validation_vowel_pattern::rule_valid_vowel_pattern,
];

/// Rule 1: Must have at least one vowel
pub(super) fn rule_has_vowel(
    _snap: &BufferSnapshot,
    syllable: &Syllable,
) -> Option<ValidationResult> {
    if syllable.is_empty() {
        return Some(ValidationResult::NoVowel);
    }
    None
}

/// Rule 2: Initial consonant must be valid Vietnamese
pub(super) fn rule_valid_initial(
    snap: &BufferSnapshot,
    syllable: &Syllable,
) -> Option<ValidationResult> {
    if syllable.initial.is_empty() {
        return None;
    }

    let initial: Vec<u16> = syllable.initial.iter().map(|&i| snap.keys[i]).collect();

    let is_valid = match initial.len() {
        1 => {
            constants::VALID_INITIALS_1.contains(&initial[0])
                || (snap.allow_foreign_consonants
                    && constants::FOREIGN_INITIALS.contains(&initial[0]))
        }
        2 => constants::VALID_INITIALS_2
            .iter()
            .any(|p| p[0] == initial[0] && p[1] == initial[1]),
        3 => initial[0] == keys::N && initial[1] == keys::G && initial[2] == keys::H,
        _ => false,
    };

    if !is_valid {
        return Some(ValidationResult::InvalidInitial);
    }
    None
}

/// Rule 3: All characters must be parsed into syllable structure
pub(super) fn rule_all_chars_parsed(
    snap: &BufferSnapshot,
    syllable: &Syllable,
) -> Option<ValidationResult> {
    let parsed = syllable.initial.len()
        + syllable.glide.map_or(0, |_| 1)
        + syllable.vowel.len()
        + syllable.final_c.len();

    if parsed != snap.keys.len() {
        return Some(ValidationResult::InvalidFinal);
    }
    None
}

/// Rule 4: Vietnamese spelling rules (c/k, g/gh, ng/ngh)
pub(super) fn rule_spelling(
    snap: &BufferSnapshot,
    syllable: &Syllable,
) -> Option<ValidationResult> {
    if syllable.initial.is_empty() || syllable.vowel.is_empty() {
        return None;
    }

    let initial: Vec<u16> = syllable.initial.iter().map(|&i| snap.keys[i]).collect();
    let first_vowel = snap.keys[syllable.glide.unwrap_or(syllable.vowel[0])];

    for &(consonant, vowels, _msg) in constants::SPELLING_RULES {
        if initial == consonant && vowels.contains(&first_vowel) {
            return Some(ValidationResult::InvalidSpelling);
        }
    }

    None
}

/// Rule 5: Final consonant must be valid
pub(super) fn rule_valid_final(
    snap: &BufferSnapshot,
    syllable: &Syllable,
) -> Option<ValidationResult> {
    if syllable.final_c.is_empty() {
        return None;
    }

    let final_c: Vec<u16> = syllable.final_c.iter().map(|&i| snap.keys[i]).collect();

    let is_valid = match final_c.len() {
        1 => constants::VALID_FINALS_1.contains(&final_c[0]),
        2 => constants::VALID_FINALS_2
            .iter()
            .any(|p| p[0] == final_c[0] && p[1] == final_c[1]),
        _ => false,
    };

    if !is_valid {
        return Some(ValidationResult::InvalidFinal);
    }
    None
}

// Child module: vowel pattern rule + transform validation + foreign word detection
#[path = "validation_vowel_pattern.rs"]
mod validation_vowel_pattern;
pub use validation_vowel_pattern::{
    is_foreign_word_pattern, is_valid_for_transform, is_valid_for_transform_with_foreign,
};

// =============================================================================
// PUBLIC API
// =============================================================================

/// Validate buffer as Vietnamese syllable - runs all rules
pub fn validate(snap: &BufferSnapshot) -> ValidationResult {
    if snap.keys.is_empty() {
        return ValidationResult::NoVowel;
    }

    let syllable = parse(&snap.keys);

    for rule in RULES {
        if let Some(error) = rule(snap, &syllable) {
            return error;
        }
    }

    ValidationResult::Valid
}

/// Quick check if buffer could be valid Vietnamese (with modifier info)
/// This will fully validate modifier requirements (e.g., E+U requires circumflex)
pub fn is_valid_with_tones(keys: &[u16], tones: &[u8]) -> bool {
    let snap = BufferSnapshot {
        keys: keys.to_vec(),
        tones: tones.to_vec(),
        has_tone_info: true, // Enforce modifier requirements
        allow_foreign_consonants: false,
    };
    validate(&snap).is_valid()
}

/// Quick check if buffer could be valid Vietnamese (with modifier info and foreign consonants option)
pub fn is_valid_with_tones_and_foreign(
    keys: &[u16],
    tones: &[u8],
    allow_foreign_consonants: bool,
) -> bool {
    let snap = BufferSnapshot {
        keys: keys.to_vec(),
        tones: tones.to_vec(),
        has_tone_info: true,
        allow_foreign_consonants,
    };
    validate(&snap).is_valid()
}

/// Quick check if buffer could be valid Vietnamese (keys only - legacy)
///
/// NOTE: This cannot fully validate modifier requirements.
/// Use is_valid_with_tones() for complete validation.
pub fn is_valid(buffer_keys: &[u16]) -> bool {
    let snap = BufferSnapshot::from_keys(buffer_keys.to_vec());
    validate(&snap).is_valid()
}

/// Quick check if buffer could be valid Vietnamese with foreign consonants option
pub fn is_valid_with_foreign(buffer_keys: &[u16], allow_foreign_consonants: bool) -> bool {
    let snap =
        BufferSnapshot::from_keys_with_foreign(buffer_keys.to_vec(), allow_foreign_consonants);
    validate(&snap).is_valid()
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;
