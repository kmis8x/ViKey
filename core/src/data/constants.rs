//! Vietnamese Phonological Constants
//!
//! Centralized constants for valid initials, finals, vowel patterns, and spelling rules.
//! Vowel patterns based on docs/vietnamese-language-system.md Section 7.6.1
//!
//! Auto-restore specific constants live in `constants_auto_restore.rs`.
//!
//! Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey

use crate::data::keys;

// =============================================================================
// INITIAL CONSONANTS
// =============================================================================

/// Valid single initial consonants (16 consonants)
pub const VALID_INITIALS_1: &[u16] = &[
    keys::B, keys::C, keys::D, keys::G, keys::H, keys::K,
    keys::L, keys::M, keys::N, keys::P, keys::Q, keys::R,
    keys::S, keys::T, keys::V, keys::X,
];

/// Foreign consonants that can optionally be allowed as valid initials
pub const FOREIGN_INITIALS: &[u16] = &[keys::Z, keys::W, keys::J, keys::F];

/// Valid double initial consonants (11 digraphs)
/// Note: Kr is included for ethnic minority place names (Krông Búk)
pub const VALID_INITIALS_2: &[[u16; 2]] = &[
    [keys::C, keys::H], // ch
    [keys::G, keys::H], // gh
    [keys::G, keys::I], // gi
    [keys::K, keys::H], // kh
    [keys::K, keys::R], // kr - for ethnic minority words (Krông)
    [keys::N, keys::G], // ng
    [keys::N, keys::H], // nh
    [keys::P, keys::H], // ph
    [keys::Q, keys::U], // qu
    [keys::T, keys::H], // th
    [keys::T, keys::R], // tr
];

// =============================================================================
// FINAL CONSONANTS
// =============================================================================

/// Valid single final consonants
/// Note: K is included for ethnic minority language place names (e.g., Đắk Lắk)
pub const VALID_FINALS_1: &[u16] = &[
    keys::C, keys::K, keys::M, keys::N, keys::P, keys::T,
    keys::I, keys::Y, keys::O, keys::U, // semi-vowels
];

/// Valid double final consonants
pub const VALID_FINALS_2: &[[u16; 2]] = &[
    [keys::C, keys::H], // ch
    [keys::N, keys::G], // ng
    [keys::N, keys::H], // nh
];

// =============================================================================
// VALID VOWEL PATTERNS (Whitelist from docs 7.6.1)
// =============================================================================

/// Valid diphthong base key patterns (29 patterns from docs 7.6.1-A)
pub const VALID_DIPHTHONGS: &[[u16; 2]] = &[
    [keys::A, keys::I], // #1 ai
    [keys::A, keys::O], // #2 ao
    [keys::A, keys::U], // #3 au, #5 âu
    [keys::A, keys::Y], // #4 ay, #6 ây
    [keys::E, keys::O], // #7 eo
    [keys::E, keys::U], // #8 êu (REQUIRES circumflex on E)
    [keys::I, keys::A], // #9 ia
    [keys::I, keys::E], // #10 iê (requires circumflex on E)
    [keys::I, keys::U], // #11 iu
    [keys::O, keys::A], // #12 oa, #13 oă
    [keys::O, keys::E], // #14 oe
    [keys::O, keys::I], // #15 oi, #16 ôi, #17 ơi
    [keys::U, keys::A], // #18 ua, #20 uâ, #25 ưa
    [keys::U, keys::E], // #21 uê (requires circumflex on E)
    [keys::U, keys::I], // #22 ui, #26 ưi
    [keys::U, keys::O], // #23 uô, #27 ươ
    [keys::U, keys::Y], // #24 uy
    [keys::U, keys::U], // #28 ưu (requires horn on first U)
    [keys::Y, keys::E], // #29 yê (requires circumflex on E)
];

/// Valid triphthong base key patterns (13 patterns from docs 7.6.1-B)
pub const VALID_TRIPHTHONGS: &[[u16; 3]] = &[
    [keys::I, keys::E, keys::U], // #30 iêu
    [keys::Y, keys::E, keys::U], // #31 yêu
    [keys::O, keys::A, keys::I], // #32 oai
    [keys::O, keys::A, keys::Y], // #33 oay
    [keys::O, keys::E, keys::O], // #34 oeo
    [keys::U, keys::A, keys::Y], // #35 uây
    [keys::U, keys::O, keys::I], // #36 uôi, #38 ươi
    [keys::U, keys::Y, keys::A], // #37 uya (khuya)
    [keys::U, keys::O, keys::U], // #39 ươu
    [keys::U, keys::Y, keys::E], // #40 uyê
    [keys::U, keys::Y, keys::U], // #41 uyu (khuỷu - elbow)
    [keys::U, keys::E, keys::U], // #42 uêu (nguều)
    [keys::O, keys::A, keys::O], // #43 oao (ngoào)
];

// =============================================================================
// MODIFIER REQUIREMENTS FOR VOWEL PATTERNS
// =============================================================================

/// Patterns requiring CIRCUMFLEX on V1 (first vowel)
pub const V1_CIRCUMFLEX_REQUIRED: &[[u16; 2]] = &[
    [keys::E, keys::U], // êu: E (V1) must have circumflex
];

/// Patterns requiring CIRCUMFLEX on V2 (second vowel)
pub const V2_CIRCUMFLEX_REQUIRED: &[[u16; 2]] = &[
    [keys::I, keys::E], // iê: E (V2) must have circumflex
    [keys::U, keys::E], // uê: E (V2) must have circumflex
    [keys::Y, keys::E], // yê: E (V2) must have circumflex
];

// =============================================================================
// SPELLING RULES
// =============================================================================

/// Spelling rules: (consonant, invalid_vowels, description)
/// If consonant + vowel matches, it's INVALID
pub const SPELLING_RULES: &[(&[u16], &[u16], &str)] = &[
    (&[keys::C], &[keys::E, keys::I, keys::Y], "c before e/i/y"),
    (&[keys::K], &[keys::A, keys::O, keys::U], "k before a/o/u"),
    (&[keys::G], &[keys::E], "g before e"),
    (&[keys::N, keys::G], &[keys::E, keys::I], "ng before e/i"),
    (&[keys::G, keys::H], &[keys::A, keys::O, keys::U], "gh before a/o/u"),
    (&[keys::N, keys::G, keys::H], &[keys::A, keys::O, keys::U], "ngh before a/o/u"),
];

// Re-export auto-restore constants for backward compatibility
pub use super::constants_auto_restore::{
    COMMON_CIRCUMFLEX_NO_FINAL, COMMON_CIRCUMFLEX_VOWEL_WITH_MARK, COMMON_SINGLE_VOWEL_WORDS,
    INVALID_RHYME_ING, OPEN_DIPHTHONGS, UNCOMMON_CIRCUMFLEX_NO_FINAL,
};
