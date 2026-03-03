//! Auto-restore specific constants for Vietnamese IME
//!
//! Patterns used by auto-restore to detect invalid Vietnamese and
//! decide whether to revert to raw ASCII input.

use crate::data::keys;

/// Invalid rhyme patterns: -ing + tone mark is NOT valid Vietnamese
/// Vietnamese uses -inh (tính, kính), not -ing with tone marks
pub const INVALID_RHYME_ING: &[[u16; 2]] = &[
    [keys::N, keys::G], // -ng final after 'i' with tone = invalid
];

/// Open diphthongs (vần mở) - CANNOT take consonant finals (C/K/M/N/P/T/CH/NG/NH)
/// These diphthongs end with semi-vowel I/O/U/Y that completes the rhyme.
/// Examples:
///   - "ai" (tài) ✓, "ain" ✗ (invalid)
///   - "ao" (cào) ✓, "aon" ✗ (invalid)  ← catches "mason" → "máon"
///   - "au" (đau) ✓, "aum" ✗ (invalid)
///   - "oi" (tôi) ✓, "oin" ✗ (invalid)
pub const OPEN_DIPHTHONGS: &[[u16; 2]] = &[
    [keys::A, keys::I], // ai - ends with semi-vowel I
    [keys::A, keys::O], // ao - ends with semi-vowel O
    [keys::A, keys::U], // au/âu - ends with semi-vowel U
    [keys::A, keys::Y], // ay/ây - ends with semi-vowel Y
    [keys::E, keys::O], // eo - ends with semi-vowel O
    [keys::I, keys::U], // iu - ends with semi-vowel U
    [keys::O, keys::I], // oi/ôi/ơi - ends with semi-vowel I
    [keys::U, keys::I], // ui/ưi - ends with semi-vowel I
    [keys::U, keys::U], // ưu - ends with semi-vowel U
];

/// Common Vietnamese single-vowel interjections (should NOT be restored)
/// These standalone vowels with tone marks are valid Vietnamese words
pub const COMMON_SINGLE_VOWEL_WORDS: &[(u16, u8)] = &[
    // Using mark values: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    (keys::A, 1), // á - very common interjection "huh?", "what?"
    (keys::A, 2), // à - common interjection "ah, I see"
    (keys::A, 4), // ã - interjection
    (keys::U, 2), // ù - exists (ù ù = buzzing sound)
    (keys::U, 4), // ũ - exists
    (keys::O, 2), // ò - skip "of" restore (keep current behavior)
    (keys::I, 2), // ì - skip "if" restore (keep current behavior)
    (keys::Y, 1), // ý - common word meaning "idea/opinion/intention"
];

/// Common Vietnamese single-vowel with CIRCUMFLEX + mark combinations
/// Format: (key, tone, mark) where tone=1 is circumflex
pub const COMMON_CIRCUMFLEX_VOWEL_WITH_MARK: &[(u16, u8, u8)] = &[
    (keys::O, 1, 2), // ồ - common exclamation "oh!" (o + circumflex + huyền)
    (keys::A, 1, 2), // ầ - exists (a + circumflex + huyền)
    (keys::E, 1, 2), // ề - exists (e + circumflex + huyền)
    (keys::O, 1, 1), // ố - exists (o + circumflex + sắc)
    (keys::A, 1, 1), // ấ - exists (a + circumflex + sắc)
    (keys::E, 1, 1), // ế - exists (e + circumflex + sắc)
];

/// Common Vietnamese words: C + circumflex vowel (from double vowel) + no final
/// These should NOT be restored to English
pub const COMMON_CIRCUMFLEX_NO_FINAL: &[u16] = &[
    keys::B, // bê - calf
    keys::M, // mê - obsessed
    keys::L, // lê - pear
    keys::D, // đê - dike (with stroke)
    keys::K, // kê - to list/declare
];

/// Initials that are UNCOMMON with circumflex vowel + no final
/// VIETNAMESE PRIORITY: Keep this list minimal to maximize Vietnamese pass rate
pub const UNCOMMON_CIRCUMFLEX_NO_FINAL: &[u16] = &[
    keys::F, // fê - F is invalid initial anyway
];
