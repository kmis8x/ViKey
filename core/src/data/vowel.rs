//! Vietnamese Vowel System
//!
//! Implements phonological classification of Vietnamese vowels based on:
//! - docs/vietnamese-language-system.md
//! - https://vi.wikipedia.org/wiki/Quy_tắc_đặt_dấu_thanh_của_chữ_Quốc_ngữ
//!
//! Submodules:
//! - `vowel_tone_patterns`: Diphthong/triphthong tone placement tables
//! - `vowel_phonology`: Phonology struct with tone/horn placement logic

use super::keys;

/// Vowel modifier type (dấu phụ)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Modifier {
    None = 0,       // a, e, i, o, u, y
    Circumflex = 1, // â, ê, ô (^)
    Horn = 2,       // ơ, ư (móc) / ă (trăng)
}

/// Phonological role in syllable
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Main,   // Primary vowel (carries tone)
    Medial, // Glide before main vowel (o in oa, u in uy)
    Final,  // Glide at syllable end (i in ai, u in au)
}

/// Vowel information
#[derive(Clone, Copy, Debug)]
pub struct Vowel {
    pub key: u16,
    pub modifier: Modifier,
    pub pos: usize,
}

impl Vowel {
    pub fn new(key: u16, modifier: Modifier, pos: usize) -> Self {
        Self { key, modifier, pos }
    }

    /// Check if this vowel has a diacritic modifier (^, ơ, ư, ă)
    pub fn has_diacritic(&self) -> bool {
        self.modifier != Modifier::None
    }
}

// =============================================================================
// VOWEL PATTERN ENUMS - Shared across tone and horn placement
// =============================================================================

/// Position for tone mark placement
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TonePosition {
    First,
    Second,
    Last,
}

/// Horn placement rule for a vowel pair
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HornPlacement {
    Both,
    First,
    Second,
}

/// Vowel pair pattern with horn placement rule
pub struct VowelPairPattern {
    pub v1: u16,
    pub v2: u16,
    pub placement: HornPlacement,
    pub desc: &'static str,
}

/// All vowel pair patterns for horn/breve placement
///
/// Order matters: first match wins
pub const HORN_PATTERNS: &[VowelPairPattern] = &[
    VowelPairPattern {
        v1: keys::U,
        v2: keys::O,
        placement: HornPlacement::Both,
        desc: "ươ compound (được, ướt)",
    },
    VowelPairPattern {
        v1: keys::O,
        v2: keys::U,
        placement: HornPlacement::Both,
        desc: "ươ compound reversed",
    },
    VowelPairPattern {
        v1: keys::U,
        v2: keys::U,
        placement: HornPlacement::First,
        desc: "ưu cluster (lưu, hưu, ngưu)",
    },
    VowelPairPattern {
        v1: keys::O,
        v2: keys::A,
        placement: HornPlacement::Second,
        desc: "oă pattern (hoặc, xoắn)",
    },
];

// Re-export from submodules for backward compatibility
pub use super::vowel_phonology::Phonology;
pub use super::vowel_tone_patterns::{
    TONE_FIRST_PATTERNS, TONE_SECOND_PATTERNS, TRIPHTHONG_PATTERNS,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn v(key: u16, modifier: Modifier, pos: usize) -> Vowel {
        Vowel::new(key, modifier, pos)
    }

    #[test]
    fn test_single_vowel() {
        let vowels = vec![v(keys::A, Modifier::None, 0)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );
    }

    #[test]
    fn test_medial_pairs() {
        // oa → mark on a (pos 1)
        let vowels = vec![v(keys::O, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );

        // uy → mark on y (pos 1)
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::Y, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );
    }

    #[test]
    fn test_ua_patterns() {
        // ua open syllable (mùa) → mark on u (pos 0)
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );

        // ua with final consonant (chuẩn) → mark on a (pos 1)
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, true, true, false, false),
            1,
            "ua with final consonant should mark on a (chuẩn)"
        );

        // ua with q (quà) → mark on a (pos 1)
        let vowels = vec![v(keys::U, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, true, false),
            1
        );
    }

    #[test]
    fn test_ia_pattern() {
        let vowels = vec![v(keys::I, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );

        let vowels = vec![v(keys::I, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, true),
            1
        );
    }

    #[test]
    fn test_main_glide_pairs() {
        let vowels = vec![v(keys::A, Modifier::None, 0), v(keys::I, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );

        let vowels = vec![v(keys::A, Modifier::None, 0), v(keys::O, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );
    }

    #[test]
    fn test_with_final_consonant() {
        let vowels = vec![v(keys::O, Modifier::None, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, true, true, false, false),
            1
        );
    }

    #[test]
    fn test_compound_vowels() {
        let vowels = vec![v(keys::U, Modifier::Horn, 0), v(keys::O, Modifier::Horn, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );

        let vowels = vec![
            v(keys::I, Modifier::None, 0),
            v(keys::E, Modifier::Circumflex, 1),
        ];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );
    }

    #[test]
    fn test_three_vowels() {
        let vowels = vec![
            v(keys::U, Modifier::Horn, 0),
            v(keys::O, Modifier::Horn, 1),
            v(keys::I, Modifier::None, 2),
        ];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );

        let vowels = vec![
            v(keys::O, Modifier::None, 0),
            v(keys::A, Modifier::None, 1),
            v(keys::I, Modifier::None, 2),
        ];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            1
        );
    }

    #[test]
    fn test_diacritic_priority() {
        let vowels = vec![v(keys::U, Modifier::Horn, 0), v(keys::A, Modifier::None, 1)];
        assert_eq!(
            Phonology::find_tone_position(&vowels, false, true, false, false),
            0
        );
    }
}
