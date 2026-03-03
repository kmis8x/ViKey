//! Vietnamese vowel phonology analyzer
//!
//! Implements tone position and horn placement rules.
//! Based on docs/vietnamese-language-system.md sections 7.2 and 7.3

use super::keys;
use super::vowel::{HornPlacement, TonePosition, Vowel, HORN_PATTERNS};
use super::vowel_tone_patterns::{
    TONE_FIRST_PATTERNS, TONE_SECOND_PATTERNS, TRIPHTHONG_PATTERNS,
};

/// Vietnamese vowel phonology analyzer
pub struct Phonology;

impl Phonology {
    /// Find the position where tone mark should be placed
    ///
    /// Uses data-driven pattern lookup from TONE_*_PATTERNS and TRIPHTHONG_PATTERNS.
    /// See docs/vietnamese-language-system.md section 7.3 for the complete matrix.
    pub fn find_tone_position(
        vowels: &[Vowel],
        has_final_consonant: bool,
        modern: bool,
        has_qu_initial: bool,
        has_gi_initial: bool,
    ) -> usize {
        // Handle gi-initial: first vowel 'i' is part of consonant, use remaining vowels
        if has_gi_initial && vowels.len() >= 2 && vowels[0].key == keys::I {
            let remaining = &vowels[1..];
            return match remaining.len() {
                0 => vowels[0].pos,
                1 => remaining[0].pos,
                2 => Self::find_diphthong_position(
                    remaining,
                    has_final_consonant,
                    modern,
                    false,
                    false,
                ),
                _ => Self::find_default_position(remaining),
            };
        }

        // Handle qu-initial: first vowel 'u' is part of consonant, use remaining vowels
        if has_qu_initial && vowels.len() >= 2 && vowels[0].key == keys::U {
            let remaining = &vowels[1..];
            return match remaining.len() {
                0 => vowels[0].pos,
                1 => remaining[0].pos,
                2 => Self::find_diphthong_position(
                    remaining,
                    has_final_consonant,
                    modern,
                    false,
                    false,
                ),
                _ => Self::find_default_position(remaining),
            };
        }

        match vowels.len() {
            0 => 0,
            1 => vowels[0].pos,
            2 => Self::find_diphthong_position(
                vowels,
                has_final_consonant,
                modern,
                has_qu_initial,
                has_gi_initial,
            ),
            3 => Self::find_triphthong_position(vowels),
            _ => {
                if vowels.len() >= 3 {
                    let first_three = &vowels[0..3];
                    let triphthong_pos = Self::find_triphthong_position(first_three);
                    if triphthong_pos >= vowels[0].pos && triphthong_pos <= vowels[2].pos {
                        return triphthong_pos;
                    }
                }
                Self::find_default_position(vowels)
            }
        }
    }

    /// Find tone position for diphthongs (2 vowels)
    fn find_diphthong_position(
        vowels: &[Vowel],
        has_final_consonant: bool,
        modern: bool,
        has_qu_initial: bool,
        has_gi_initial: bool,
    ) -> usize {
        let (v1, v2) = (&vowels[0], &vowels[1]);
        let pair = [v1.key, v2.key];

        // Rule 0: TONE_FIRST_PATTERNS (main+glide) always get mark on first vowel
        if TONE_FIRST_PATTERNS
            .iter()
            .any(|p| p[0] == pair[0] && p[1] == pair[1])
        {
            return v1.pos;
        }

        // Rule 1: With final consonant → 2nd vowel (for medial+main patterns)
        if has_final_consonant {
            return v2.pos;
        }

        // Rule 2: Diacritic priority (Section 7.2.5)
        if v1.has_diacritic() && !v2.has_diacritic() {
            return v1.pos;
        }
        if v2.has_diacritic() {
            return v2.pos;
        }

        // Rule 3: Context-dependent patterns
        // ia: 1st unless gi-initial (gia → a, kìa → i)
        if v1.key == keys::I && v2.key == keys::A {
            return if has_gi_initial { v2.pos } else { v1.pos };
        }

        // ua: context-dependent based on syllable structure
        if v1.key == keys::U && v2.key == keys::A {
            return if has_final_consonant || has_qu_initial {
                v2.pos
            } else {
                v1.pos
            };
        }

        // uy with qu-initial: always on y (quý)
        if v1.key == keys::U && v2.key == keys::Y && has_qu_initial {
            return v2.pos;
        }

        // Rule 4: TONE_SECOND_PATTERNS (medial + main, compound)
        if TONE_SECOND_PATTERNS
            .iter()
            .any(|p| p[0] == pair[0] && p[1] == pair[1])
        {
            let is_modern_pattern = matches!(
                (v1.key, v2.key),
                (keys::O, keys::A) | (keys::O, keys::E) | (keys::U, keys::Y)
            );
            if is_modern_pattern {
                return if modern { v2.pos } else { v1.pos };
            }
            return v2.pos;
        }

        // Default: 2nd vowel
        v2.pos
    }

    /// Find tone position for triphthongs (3 vowels)
    fn find_triphthong_position(vowels: &[Vowel]) -> usize {
        let (k0, k1, k2) = (vowels[0].key, vowels[1].key, vowels[2].key);

        // Rule 1: Pattern table lookup (takes priority)
        for pattern in TRIPHTHONG_PATTERNS {
            if k0 == pattern.v1 && k1 == pattern.v2 && k2 == pattern.v3 {
                return match pattern.position {
                    TonePosition::First => vowels[0].pos,
                    TonePosition::Second => vowels[1].pos,
                    TonePosition::Last => vowels[2].pos,
                };
            }
        }

        // Rule 2: Diacritic priority
        if vowels[0].has_diacritic() {
            return vowels[0].pos;
        }
        if vowels[1].has_diacritic() {
            return vowels[1].pos;
        }
        if vowels[2].has_diacritic() {
            return vowels[2].pos;
        }

        // Rule 3: Fall back to diphthong rules for first two vowels
        let pair = [k0, k1];
        if TONE_FIRST_PATTERNS.contains(&pair) {
            return vowels[0].pos;
        }
        if TONE_SECOND_PATTERNS.contains(&pair) {
            return vowels[1].pos;
        }

        // Default: middle vowel
        vowels[1].pos
    }

    /// Find tone position for 4+ vowels (rare cases)
    fn find_default_position(vowels: &[Vowel]) -> usize {
        let mid = vowels.len() / 2;

        if vowels[mid].has_diacritic() {
            return vowels[mid].pos;
        }

        for v in vowels {
            if v.has_diacritic() {
                return v.pos;
            }
        }

        if vowels.len() >= 2 {
            let pair = [vowels[0].key, vowels[1].key];
            if TONE_FIRST_PATTERNS.contains(&pair) {
                return vowels[0].pos;
            }
            if TONE_SECOND_PATTERNS.contains(&pair) {
                return vowels[1].pos;
            }
        }

        vowels[mid].pos
    }

    /// Find position(s) for horn modifier based on vowel patterns
    ///
    /// Uses HORN_PATTERNS array to match Vietnamese vowel pair patterns.
    /// Pattern matching is order-dependent (first match wins).
    pub fn find_horn_positions(buffer_keys: &[u16], vowel_positions: &[usize]) -> Vec<usize> {
        let mut result = Vec::new();
        let len = vowel_positions.len();

        if len == 0 {
            return result;
        }

        if len >= 2 {
            for i in 0..len - 1 {
                let pos1 = vowel_positions[i];
                let pos2 = vowel_positions[i + 1];

                if pos2 != pos1 + 1 {
                    continue;
                }

                let k1 = buffer_keys.get(pos1).copied().unwrap_or(0);
                let k2 = buffer_keys.get(pos2).copied().unwrap_or(0);

                // Special case: "ua" pattern
                if k1 == keys::U && k2 == keys::A {
                    let preceded_by_q =
                        pos1 > 0 && buffer_keys.get(pos1 - 1).copied() == Some(keys::Q);
                    result.push(if preceded_by_q { pos2 } else { pos1 });
                    return result;
                }

                // Match against pattern table
                for pattern in HORN_PATTERNS {
                    if k1 == pattern.v1 && k2 == pattern.v2 {
                        match pattern.placement {
                            HornPlacement::Both => {
                                let preceded_by_q =
                                    pos1 > 0 && buffer_keys.get(pos1 - 1).copied() == Some(keys::Q);
                                if preceded_by_q {
                                    result.push(pos2);
                                } else {
                                    let has_final = buffer_keys.get(pos2 + 1).is_some();
                                    if k1 == keys::U && k2 == keys::O && !has_final {
                                        result.push(pos2);
                                    } else {
                                        result.push(pos1);
                                        result.push(pos2);
                                    }
                                }
                            }
                            HornPlacement::First => {
                                result.push(pos1);
                            }
                            HornPlacement::Second => {
                                result.push(pos2);
                            }
                        }
                        return result;
                    }
                }
            }
        }

        // Default: find last u or o
        for &pos in vowel_positions.iter().rev() {
            let k = buffer_keys.get(pos).copied().unwrap_or(0);
            if k == keys::U || k == keys::O {
                if k == keys::O {
                    let next_key = buffer_keys.get(pos + 1).copied();
                    if next_key == Some(keys::E) {
                        continue;
                    }
                }
                result.push(pos);
                return result;
            }
        }

        // If no u/o, apply to last a (breve case in Telex)
        if let Some(&pos) = vowel_positions.last() {
            let k = buffer_keys.get(pos).copied().unwrap_or(0);
            if k == keys::A {
                result.push(pos);
            }
        }

        result
    }
}
