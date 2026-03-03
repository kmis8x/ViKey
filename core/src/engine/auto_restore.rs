use super::Engine;
use crate::data::{chars::tone, constants, english_dict, keys, telex_doubles};
use crate::engine::syllable;
use crate::engine::types::Result;
use crate::utils;
use super::validation::{self, is_valid_with_tones_and_foreign};

/// Get raw_input as lowercase ASCII string
pub(super) fn get_raw_input_string(e: &Engine) -> String {
    e.raw_input
        .iter()
        .filter_map(|&(key, caps, _)| utils::key_to_char(key, caps))
        .collect::<String>()
        .to_lowercase()
}

/// Get raw_input as ASCII string preserving original case
#[allow(dead_code)]
pub(super) fn get_raw_input_string_preserve_case(e: &Engine) -> String {
    e.raw_input
        .iter()
        .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
        .collect()
}

/// Check if buffer is NOT valid Vietnamese (for unified auto-restore logic)
pub(super) fn is_buffer_invalid_vietnamese(e: &Engine) -> bool {
    if e.buf.is_empty() {
        return false;
    }

    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
    let buffer_tones: Vec<u8> = e.buf.iter().map(|c| c.tone).collect();
    let buffer_marks: Vec<u8> = e.buf.iter().map(|c| c.mark).collect();

    if !is_valid_with_tones_and_foreign(&buffer_keys, &buffer_tones, e.allow_foreign_consonants) {
        return true;
    }

    if buffer_keys.len() >= 3 {
        let len = buffer_keys.len();
        if buffer_keys[len - 2] == keys::N
            && buffer_keys[len - 1] == keys::G
            && buffer_keys[len - 3] == keys::I
            && buffer_marks[len - 3] > 0
        {
            return true;
        }
    }

    if buffer_keys.len() == 1 && keys::is_vowel(buffer_keys[0]) && buffer_marks[0] > 0 {
        return false;
    }

    if buffer_keys.len() == 2 {
        let initial = buffer_keys[0];
        let vowel = buffer_keys[1];
        let vowel_tone = buffer_tones[1];
        let vowel_mark = buffer_marks[1];
        if keys::is_consonant(initial)
            && keys::is_vowel(vowel)
            && vowel_tone == tone::CIRCUMFLEX
            && vowel_mark == 0
        {
            if constants::UNCOMMON_CIRCUMFLEX_NO_FINAL.contains(&initial) {
                return true;
            }
        }
    }

    let syllable = syllable::parse(&buffer_keys);
    if syllable.vowel.len() == 2 && !syllable.final_c.is_empty() {
        let vowel_pair = [
            buffer_keys[syllable.vowel[0]],
            buffer_keys[syllable.vowel[1]],
        ];
        let final_key = buffer_keys[syllable.final_c[0]];
        let is_consonant_final = matches!(
            final_key,
            keys::C | keys::K | keys::M | keys::N | keys::P | keys::T
        ) || (syllable.final_c.len() == 2);

        if is_consonant_final && constants::OPEN_DIPHTHONGS.contains(&vowel_pair) {
            return true;
        }
    }

    for i in 0..buffer_keys.len().saturating_sub(1) {
        if buffer_keys[i] == keys::O
            && buffer_tones[i] == tone::HORN
            && buffer_keys[i + 1] == keys::E
        {
            return true;
        }
    }

    false
}

/// Check if raw_input is valid English
pub(super) fn is_raw_input_valid_english(e: &Engine) -> bool {
    if e.raw_input.is_empty() {
        return false;
    }

    let all_ascii_letters = e.raw_input.iter().all(|(k, _, _)| {
        keys::is_consonant(*k) || keys::is_vowel(*k)
    });

    if !all_ascii_letters {
        return false;
    }

    let has_vowel = e.raw_input.iter().any(|(k, _, _)| keys::is_vowel(*k));

    if e.raw_input.len() <= 2 {
        return true;
    }

    has_vowel
}

/// Check if this is an intentional revert at end of word that should be kept.
pub(super) fn ends_with_double_modifier(e: &Engine) -> bool {
    if e.raw_input.len() < 2 {
        return false;
    }

    let (last_key, _, _) = e.raw_input[e.raw_input.len() - 1];
    let (second_last_key, _, _) = e.raw_input[e.raw_input.len() - 2];

    if last_key != second_last_key {
        return false;
    }

    if e.method == 0 {
        if matches!(last_key, keys::A | keys::E | keys::O | keys::W) {
            return true;
        }
    } else {
        if matches!(last_key, keys::N6 | keys::N7 | keys::N8) {
            return true;
        }
    }

    let is_mark_key = if e.method == 0 {
        matches!(last_key, keys::S | keys::F | keys::R | keys::X | keys::J)
    } else {
        matches!(
            last_key,
            keys::N1 | keys::N2 | keys::N3 | keys::N4 | keys::N5
        )
    };

    if !is_mark_key {
        return false;
    }

    if e.raw_input.len() <= 3 && last_key != keys::F {
        return true;
    }

    if e.raw_input.len() == 4 && e.buf.len() == 3 {
        if last_key == keys::S {
            if let Some(last_char) = e.buf.last() {
                if last_char.key == keys::S {
                    return false;
                }
            }
        }
        return true;
    }

    if e.method == 0 {
        matches!(last_key, keys::X | keys::J)
    } else {
        true
    }
}

/// Build raw chars from raw_input EXACTLY as typed (no collapsing)
pub(super) fn build_raw_chars_exact(e: &Engine) -> Option<Vec<char>> {
    if let Some(ref raw_str) = e.telex_double_raw {
        if !raw_str.is_empty() && e.telex_double_raw_len > 0 {
            let mut result: Vec<char> = raw_str.chars().collect();
            let subsequent_start = if e.raw_input.len() < e.telex_double_raw_len {
                e.telex_double_raw_len.saturating_sub(1)
            } else {
                e.telex_double_raw_len
            };
            for i in subsequent_start..e.raw_input.len() {
                if let Some(&(key, caps, shift)) = e.raw_input.get(i) {
                    if let Some(ch) = utils::key_to_char_ext(key, caps, shift) {
                        result.push(ch);
                    }
                }
            }
            return Some(result);
        }
    }
    let chars: Vec<char> = e
        .raw_input
        .iter()
        .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
        .collect();
    if chars.is_empty() {
        None
    } else {
        Some(chars)
    }
}

/// Determine if buffer should be used for restore after a mark revert
pub(super) fn should_use_buffer_for_revert(e: &Engine) -> bool {
    let buf_str = e.buf.to_lowercase_string();

    const PREFIXES: &[&str] = &[
        "dis", "mis", "un", "re", "de", "pre", "anti", "non", "sub", "trans", "con",
    ];

    const SUFFIXES: &[&str] = &[
        "able", "ible", "tion", "sion", "ment", "ness", "less", "ful", "ing", "ive", "ified",
        "ous", "ory",
    ];

    const SHORT_SUFFIXES: &[&str] = &["er", "or"];

    for prefix in PREFIXES {
        if buf_str.starts_with(prefix) && buf_str.len() >= prefix.len() + 2 {
            return true;
        }
    }

    let raw_len = e.raw_input.len();
    for i in 0..raw_len.saturating_sub(1) {
        let (k1, _, _) = e.raw_input[i];
        let (k2, _, _) = e.raw_input[i + 1];
        if (k1 == keys::S && k2 == keys::S) || (k1 == keys::F && k2 == keys::F) {
            let chars_after_double = raw_len - (i + 2);

            let common_single_consonant_endings = ["son", "ton", "ron", "non", "mon"];
            let use_buffer_for_ending = common_single_consonant_endings
                .iter()
                .any(|ending| buf_str.ends_with(ending));

            if chars_after_double >= 2 && !use_buffer_for_ending {
                return false;
            }
            if use_buffer_for_ending {
                return true;
            }
            break;
        }
    }

    for suffix in SUFFIXES {
        if buf_str.ends_with(suffix) && buf_str.len() >= suffix.len() + 2 {
            if e.raw_input.len() <= buf_str.len() + 1 {
                return true;
            }
        }
    }

    if e.buf.len() == 4 && e.raw_input.len() == 5 {
        for suffix in SHORT_SUFFIXES {
            if buf_str.ends_with(suffix) {
                let (key_1, _, _) = e.raw_input[1];
                let (key_2, _, _) = e.raw_input[2];
                if key_1 == keys::S && key_2 == keys::S {
                    let s_count = e
                        .raw_input
                        .iter()
                        .filter(|(k, _, _)| *k == keys::S)
                        .count();
                    if s_count == 2 {
                        return true;
                    }
                }
            }
        }
    }

    if e.raw_input.len() >= 4 {
        let len = e.raw_input.len();
        let (last_key, _, _) = e.raw_input[len - 1];
        let (second_last_key, _, _) = e.raw_input[len - 2];
        let (third_last_key, _, _) = e.raw_input[len - 3];

        if keys::is_vowel(last_key) && second_last_key == keys::F && third_last_key == keys::F {
            return true;
        }

        let is_core_vowel = matches!(
            last_key,
            k if k == keys::A || k == keys::E || k == keys::I || k == keys::O || k == keys::U
        );
        if is_core_vowel && second_last_key == keys::S && third_last_key == keys::S {
            return true;
        }

        if last_key == keys::S && second_last_key == keys::S && len == buf_str.len() + 1 {
            let starts_with_digraph = buf_str.starts_with("th")
                || buf_str.starts_with("wh")
                || buf_str.starts_with("ch")
                || buf_str.starts_with("sh");
            if starts_with_digraph {
                return true;
            }

            if buf_str.len() >= 2 {
                let chars: Vec<char> = buf_str.chars().collect();
                let second_last_char = chars[chars.len() - 2];
                let last_char = chars[chars.len() - 1];
                let is_plural_pattern = last_char == 's'
                    && !matches!(second_last_char, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
                if is_plural_pattern {
                    return true;
                }
            }
        }
    }

    const RARE_DOUBLE_MODIFIERS: &[u16] = &[keys::R, keys::X, keys::J];

    if e.raw_input.len() >= 4 && e.raw_input.len() == buf_str.len() + 1 {
        let has_transforms = e
            .buf
            .iter()
            .any(|c| c.tone > 0 || c.mark > 0 || c.stroke);
        if has_transforms {
            return false;
        }

        let (last_key, _, _) = e.raw_input[e.raw_input.len() - 1];
        if !keys::is_consonant(last_key) {
            return false;
        }

        let (last_key_1, _, _) = e.raw_input[e.raw_input.len() - 1];
        let (last_key_2, _, _) = e.raw_input[e.raw_input.len() - 2];

        let is_common_suffix = matches!(
            (last_key_2, last_key_1),
            (keys::O, keys::W)
                | (keys::O, keys::R)
                | (keys::R, keys::Y)
                | (keys::E, keys::D)
                | (keys::L, keys::Y)
        );
        if is_common_suffix {
            return false;
        }

        for i in 0..e.raw_input.len().saturating_sub(2) {
            let (key_i, _, _) = e.raw_input[i];
            let (key_next, _, _) = e.raw_input[i + 1];

            if RARE_DOUBLE_MODIFIERS.contains(&key_i) && key_i == key_next {
                let chars_after_double = e.raw_input.len() - (i + 2);

                if chars_after_double == 2 {
                    let occurrence_count = e
                        .raw_input
                        .iter()
                        .filter(|(k, _, _)| *k == key_i)
                        .count();

                    if occurrence_count == 2 {
                        return true;
                    }
                }
            }
        }
    }

    if e.raw_input.len() >= 4 && buf_str.len() >= 4 && buf_str.len() <= 6 {
        let len = e.raw_input.len();
        let (last_key, _, _) = e.raw_input[len - 1];
        let (second_last_key, _, _) = e.raw_input[len - 2];

        let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];
        if tone_modifiers.contains(&last_key) && last_key == second_last_key {
            let occurrence_count = e
                .raw_input
                .iter()
                .filter(|(k, _, _)| *k == last_key)
                .count();
            if occurrence_count == 2 {
                let expected_char = match last_key {
                    k if k == keys::S => 's',
                    k if k == keys::F => 'f',
                    k if k == keys::R => 'r',
                    k if k == keys::X => 'x',
                    k if k == keys::J => 'j',
                    _ => '\0',
                };
                if expected_char != '\0' && buf_str.ends_with(expected_char) {
                    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
                    let buffer_tones: Vec<u8> = e.buf.iter().map(|c| c.tone).collect();
                    if validation::is_valid_with_tones(&buffer_keys, &buffer_tones) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Build raw chars from raw_input for restore
pub(super) fn build_raw_chars(e: &Engine) -> Option<Vec<char>> {
    let raw_chars: Vec<char> = if e.had_mark_revert && should_use_buffer_for_revert(e) {
        e.buf.to_string_preserve_case().chars().collect()
    } else {
        let mut chars: Vec<char> = e
            .raw_input
            .iter()
            .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
            .collect();

        let is_saas_pattern = chars.len() >= 3
            && chars.first().map(|c| c.to_ascii_lowercase())
                == chars.last().map(|c| c.to_ascii_lowercase())
            && chars
                .first()
                .map(|c| !matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
                .unwrap_or(false);

        let tone_modifiers = ['s', 'f', 'r', 'x', 'j'];
        let has_double_vowel_at_end = chars.len() >= 3 && chars.len() <= 4 && {
            let last = chars[chars.len() - 1].to_ascii_lowercase();
            let second_last = chars[chars.len() - 2].to_ascii_lowercase();
            let third_last = chars[chars.len() - 3].to_ascii_lowercase();
            matches!(second_last, 'a' | 'e' | 'o')
                && second_last == third_last
                && tone_modifiers.contains(&last)
        };

        let mut i = 0;
        while i + 2 < chars.len() {
            let c = chars[i].to_ascii_lowercase();
            if matches!(c, 'a' | 'e' | 'o')
                && chars[i].eq_ignore_ascii_case(&chars[i + 1])
                && chars[i + 1].eq_ignore_ascii_case(&chars[i + 2])
            {
                chars.remove(i + 1);
                continue;
            }
            i += 1;
        }

        let mut i = 0;
        while i + 2 < chars.len() {
            let c = chars[i].to_ascii_lowercase();
            if chars[i].eq_ignore_ascii_case(&chars[i + 1])
                && chars[i + 1].eq_ignore_ascii_case(&chars[i + 2])
                && !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
            {
                chars.remove(i + 1);
                continue;
            }
            i += 1;
        }

        if has_double_vowel_at_end && !is_saas_pattern {
            let pos = chars.len() - 3;
            chars.remove(pos);
        }

        if e.had_circumflex_revert && chars.len() >= 2 {
            let last = chars[chars.len() - 1].to_ascii_lowercase();
            let second_last = chars[chars.len() - 2].to_ascii_lowercase();
            if matches!(last, 'a' | 'e' | 'o') && last == second_last {
                chars.pop();
            }
        }

        if chars.len() > 2
            && chars[0].eq_ignore_ascii_case(&'w')
            && chars[1].eq_ignore_ascii_case(&'w')
        {
            chars.remove(0);
        }

        let tone_modifiers_char = ['s', 'r', 'x', 'j'];
        let starts_with_u_doubled_modifier = chars.len() >= 3
            && chars[0].eq_ignore_ascii_case(&'u')
            && tone_modifiers_char.contains(&chars[1].to_ascii_lowercase())
            && chars[1].eq_ignore_ascii_case(&chars[2]);

        if e.had_mark_revert && (e.buf.len() <= 3 || starts_with_u_doubled_modifier) {
            let tone_modifiers_full = ['s', 'f', 'r', 'x', 'j'];
            let mut i = 0;
            while i + 1 < chars.len() {
                let c = chars[i].to_ascii_lowercase();
                let next = chars[i + 1].to_ascii_lowercase();

                let is_at_end = i + 2 == chars.len();
                let is_ss_or_ff = (c == 's' && next == 's') || (c == 'f' && next == 'f');
                if is_at_end && is_ss_or_ff {
                    i += 1;
                    continue;
                }

                if tone_modifiers_full.contains(&c) && c == next {
                    chars.remove(i);
                    continue;
                }
                i += 1;
            }
        }

        if chars.len() == 5 && e.method == 0 {
            let c0 = chars[0].to_ascii_lowercase();
            let c1 = chars[1].to_ascii_lowercase();
            let c2 = chars[2].to_ascii_lowercase();
            let c3 = chars[3].to_ascii_lowercase();
            let c4 = chars[4].to_ascii_lowercase();

            let is_consonant_0 = !matches!(c0, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
            let is_vowel_1 = matches!(c1, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
            let is_tone_2 = matches!(c2, 's' | 'f' | 'r' | 'x' | 'j');
            let is_circumflex_vowel_34 = matches!(c3, 'a' | 'e' | 'o') && c3 == c4;

            if is_consonant_0 && is_vowel_1 && is_tone_2 && is_circumflex_vowel_34 {
                let toned_vowel = match (c1, c2) {
                    ('a', 's') => 'á', ('a', 'f') => 'à', ('a', 'r') => 'ả',
                    ('a', 'x') => 'ã', ('a', 'j') => 'ạ',
                    ('e', 's') => 'é', ('e', 'f') => 'è', ('e', 'r') => 'ẻ',
                    ('e', 'x') => 'ẽ', ('e', 'j') => 'ẹ',
                    ('i', 's') => 'í', ('i', 'f') => 'ì', ('i', 'r') => 'ỉ',
                    ('i', 'x') => 'ĩ', ('i', 'j') => 'ị',
                    ('o', 's') => 'ó', ('o', 'f') => 'ò', ('o', 'r') => 'ỏ',
                    ('o', 'x') => 'õ', ('o', 'j') => 'ọ',
                    ('u', 's') => 'ú', ('u', 'f') => 'ù', ('u', 'r') => 'ủ',
                    ('u', 'x') => 'ũ', ('u', 'j') => 'ụ',
                    ('y', 's') => 'ý', ('y', 'f') => 'ỳ', ('y', 'r') => 'ỷ',
                    ('y', 'x') => 'ỹ', ('y', 'j') => 'ỵ',
                    _ => c1,
                };
                let toned_vowel = if chars[1].is_uppercase() {
                    toned_vowel.to_uppercase().next().unwrap_or(toned_vowel)
                } else {
                    toned_vowel
                };
                return Some(vec![chars[0], toned_vowel, chars[3], chars[4]]);
            }
        }

        chars
    };

    if raw_chars.is_empty() {
        return None;
    }

    let buffer_str: String = e.buf.to_string_preserve_case();
    let raw_str: String = raw_chars.iter().collect();
    if buffer_str == raw_str {
        return None;
    }

    Some(raw_chars)
}

/// Check for English patterns in raw_input that suggest non-Vietnamese
pub(super) fn has_english_modifier_pattern(e: &Engine, is_word_complete: bool) -> bool {
    let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];

    if is_word_complete && !e.buf.is_empty() {
        let first_vowel_pos = e
            .raw_input
            .iter()
            .position(|(k, _, _)| keys::is_vowel(*k));

        if let Some(vowel_pos) = first_vowel_pos {
            if vowel_pos + 3 < e.raw_input.len() {
                let (k1, _, _) = e.raw_input[vowel_pos + 1];
                let (k2, _, _) = e.raw_input[vowel_pos + 2];
                let (k3, _, _) = e.raw_input[vowel_pos + 3];

                let has_tone_override = tone_modifiers.contains(&k1)
                    && tone_modifiers.contains(&k2)
                    && k1 != k2
                    && keys::is_vowel(k3);

                if has_tone_override {
                    let raw_str: String = e
                        .raw_input
                        .iter()
                        .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                        .collect();
                    let raw_in_dict = english_dict::is_english_word(&raw_str);

                    if !raw_in_dict && !is_buffer_invalid_vietnamese(e) {
                        return false;
                    }
                }
            }
        }
    }

    if e.raw_input.len() >= 2 {
        let (first, _, _) = e.raw_input[0];
        if keys::is_vowel(first) && first != keys::W {
            let all_after_are_modifiers = e.raw_input[1..]
                .iter()
                .all(|(k, _, _)| tone_modifiers.contains(k));
            if all_after_are_modifiers {
                return false;
            }
        }
    }

    if e.raw_input.len() >= 2 {
        let (first, _, _) = e.raw_input[0];
        if first == keys::W {
            let has_later_w = e.raw_input[2..].iter().any(|(k, _, _)| *k == keys::W);
            if has_later_w {
                return true;
            }

            let all_are_modifiers = e.raw_input[1..]
                .iter()
                .all(|(k, _, _)| tone_modifiers.contains(k));
            if all_are_modifiers && !e.raw_input[1..].is_empty() {
                return false;
            }

            if e.raw_input.len() >= 2 {
                let has_other_vowels = e.raw_input[1..]
                    .iter()
                    .any(|(k, _, _)| keys::is_vowel(*k) && *k != keys::W);

                if !has_other_vowels {
                    let non_modifier_consonants: Vec<u16> = e.raw_input[1..]
                        .iter()
                        .filter(|(k, _, _)| {
                            keys::is_consonant(*k) && !tone_modifiers.contains(k)
                        })
                        .map(|(k, _, _)| *k)
                        .collect();

                    let has_mark_modifier = e.raw_input[1..]
                        .iter()
                        .any(|(k, _, _)| tone_modifiers.contains(k));

                    if !non_modifier_consonants.is_empty() && has_mark_modifier {
                        let is_valid_final = match non_modifier_consonants.len() {
                            1 => constants::VALID_FINALS_1.contains(&non_modifier_consonants[0]),
                            2 => {
                                let pair = [non_modifier_consonants[0], non_modifier_consonants[1]];
                                constants::VALID_FINALS_2.contains(&pair)
                            }
                            _ => false,
                        };
                        if is_valid_final {
                            return false;
                        }
                    }
                }
            }

            let first_vowel_pos = e.raw_input[1..]
                .iter()
                .position(|(k, _, _)| keys::is_vowel(*k) && *k != keys::W);

            let vowels_after: Vec<u16> = e.raw_input[1..]
                .iter()
                .filter(|(k, _, _)| keys::is_vowel(*k) && *k != keys::W)
                .map(|(k, _, _)| *k)
                .collect();

            let consonants_after: Vec<u16> = e.raw_input[1..]
                .iter()
                .enumerate()
                .filter(|(i, (k, _, _))| {
                    if !keys::is_consonant(*k) || *k == keys::W {
                        return false;
                    }
                    if vowels_after.is_empty() && tone_modifiers.contains(k) {
                        return false;
                    }
                    if let Some(vowel_pos) = first_vowel_pos {
                        if *i > vowel_pos && tone_modifiers.contains(k) {
                            return false;
                        }
                    }
                    true
                })
                .map(|(_, (k, _, _))| *k)
                .collect();

            if !vowels_after.is_empty() && !consonants_after.is_empty() {
                let is_wo_final_pattern = vowels_after.len() == 1
                    && vowels_after[0] == keys::O
                    && match consonants_after.len() {
                        1 => matches!(
                            consonants_after[0],
                            keys::N | keys::M | keys::C | keys::T | keys::P
                        ),
                        2 => {
                            let pair = [consonants_after[0], consonants_after[1]];
                            pair == [keys::N, keys::G] || pair == [keys::N, keys::H]
                        }
                        _ => false,
                    };
                if is_wo_final_pattern {
                    return false;
                }
                return true;
            }

            if !vowels_after.is_empty() && consonants_after.is_empty() {
                let valid_vowels_after_w = [keys::A, keys::O, keys::U];
                let has_invalid_vowel = vowels_after
                    .iter()
                    .any(|v| !valid_vowels_after_w.contains(v));
                if has_invalid_vowel {
                    return true;
                }
            }

            if !consonants_after.is_empty() && vowels_after.is_empty() {
                let is_valid_final = match consonants_after.len() {
                    1 => constants::VALID_FINALS_1.contains(&consonants_after[0]),
                    2 => {
                        let pair = [consonants_after[0], consonants_after[1]];
                        constants::VALID_FINALS_2.contains(&pair)
                    }
                    _ => false,
                };

                if !is_valid_final {
                    return true;
                }
            }

            if is_word_complete {
                let (second, _, _) = e.raw_input[1];
                if second == keys::W && keys::is_consonant(first) && first != keys::Q {
                    if e.raw_input.len() >= 3 {
                        let (third, _, _) = e.raw_input[2];
                        if keys::is_vowel(third) {
                            if third == keys::O && e.raw_input.len() >= 5 {
                                let (fourth, _, _) = e.raw_input[3];
                                let (fifth, _, _) = e.raw_input[4];
                                if fourth == keys::N && fifth == keys::G {
                                    return false;
                                }
                            }

                            if third == keys::A && e.raw_input.len() == 3 {
                                return false;
                            }

                            if third == keys::U && e.raw_input.len() == 3 {
                                return false;
                            }

                            let has_tone_modifier = e.raw_input[2..]
                                .iter()
                                .any(|(k, _, _)| tone_modifiers.contains(k));

                            if !has_tone_modifier {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];

    for i in 0..e.raw_input.len().saturating_sub(2) {
        let (key, _, _) = e.raw_input[i];
        let (next_key, _, _) = e.raw_input[i + 1];
        let (after_key, _, _) = e.raw_input[i + 2];
        if tone_modifiers.contains(&key)
            && tone_modifiers.contains(&next_key)
            && key != next_key
            && keys::is_vowel(after_key)
        {
            return true;
        }
    }

    for i in 0..e.raw_input.len() {
        let (key, _, _) = e.raw_input[i];

        if !tone_modifiers.contains(&key) {
            continue;
        }

        if i + 1 < e.raw_input.len() {
            let (next_key, _, _) = e.raw_input[i + 1];
            let is_true_consonant = keys::is_consonant(next_key)
                && next_key != keys::W
                && !tone_modifiers.contains(&next_key);
            if is_true_consonant {
                let is_common_viet_mark = key == keys::J || key == keys::S;
                let is_rare_with_stop = key == keys::F || key == keys::R || key == keys::X;
                let is_sonorant_or_part_of_final = next_key == keys::M
                    || next_key == keys::N
                    || (next_key == keys::G && i >= 1 && e.raw_input[i - 1].0 == keys::N)
                    || (next_key == keys::H && i >= 1 && e.raw_input[i - 1].0 == keys::N);

                if is_common_viet_mark {
                    continue;
                }

                if is_rare_with_stop && is_sonorant_or_part_of_final {
                    continue;
                }

                if i + 2 < e.raw_input.len() {
                    return true;
                }

                let vowels_before: usize = (0..i)
                    .filter(|&j| keys::is_vowel(e.raw_input[j].0))
                    .count();
                if vowels_before == 1 {
                    return true;
                }
            }
        }

        if i + 1 == e.raw_input.len() && i >= 2 {
            let (v1, _, _) = e.raw_input[i - 2];
            let (v2, _, _) = e.raw_input[i - 1];
            if keys::is_vowel(v1) && keys::is_vowel(v2) && v1 != v2 {
                let total_vowels: usize = (0..i)
                    .filter(|&j| keys::is_vowel(e.raw_input[j].0))
                    .count();

                if v1 == keys::E && v2 == keys::I {
                    return true;
                }
                if v1 == keys::A && v2 == keys::I && total_vowels == 2 {
                    if !e.raw_input.is_empty() && e.raw_input[0].0 == keys::P {
                        let is_ph = e.raw_input.len() >= 2 && e.raw_input[1].0 == keys::H;
                        if !is_ph {
                            return true;
                        }
                    }
                }
                if v1 == keys::O && v2 == keys::E && total_vowels == 2 && key == keys::S {
                    let is_vietnamese_oe_initial = if e.raw_input.len() >= 2 {
                        let (c1, _, _) = e.raw_input[0];
                        let (c2, _, _) = e.raw_input[1];

                        let ends_with_h = c2 == keys::H
                            && matches!(
                                c1,
                                k if k == keys::K
                                    || k == keys::G
                                    || k == keys::P
                                    || k == keys::C
                                    || k == keys::T
                                    || k == keys::N
                            );
                        let is_tr = c1 == keys::T && c2 == keys::R;
                        let is_ng = c1 == keys::N && c2 == keys::G;

                        let is_single_initial_oe = c2 == keys::O
                            && matches!(
                                c1,
                                keys::H | keys::L | keys::T | keys::X | keys::B | keys::S
                            );

                        ends_with_h || is_tr || is_ng || is_single_initial_oe
                    } else {
                        false
                    };

                    if is_vietnamese_oe_initial {
                        continue;
                    }

                    let has_initial =
                        !e.raw_input.is_empty() && keys::is_consonant(e.raw_input[0].0);
                    if has_initial {
                        return true;
                    }
                }
            }

            if e.raw_input.len() >= 2 && e.raw_input[0].0 == keys::P {
                let is_ph = e.raw_input.len() >= 2 && e.raw_input[1].0 == keys::H;
                if !is_ph {
                    let vowels_before: usize = (0..i)
                        .filter(|&j| keys::is_vowel(e.raw_input[j].0))
                        .count();
                    if vowels_before == 1 && i + 1 == e.raw_input.len() {
                        return true;
                    }
                }
            }
        }

        let vowels_before: usize = (0..i)
            .filter(|&j| keys::is_vowel(e.raw_input[j].0))
            .count();

        if vowels_before == 1 && i + 1 < e.raw_input.len() {
            let (next_key, _, _) = e.raw_input[i + 1];
            if keys::is_vowel(next_key) {
                let first_vowel_pos = (0..i)
                    .find(|&j| keys::is_vowel(e.raw_input[j].0))
                    .unwrap_or(0);
                let has_initial_consonant = first_vowel_pos > 0
                    && keys::is_consonant(e.raw_input[first_vowel_pos - 1].0);
                let has_consonant_between = (first_vowel_pos + 1 < i)
                    && keys::is_consonant(e.raw_input[first_vowel_pos + 1].0);
                if !has_initial_consonant && !has_consonant_between {
                    let first_vowel = e.raw_input[first_vowel_pos].0;

                    if first_vowel == keys::O && next_key == keys::E {
                        let raw_str: String = e
                            .raw_input
                            .iter()
                            .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                            .collect();
                        if english_dict::is_english_word(&raw_str) {
                            return true;
                        }
                        continue;
                    }

                    let is_vietnamese_no_initial = first_vowel == next_key
                        || (first_vowel == keys::U && (next_key == keys::A || next_key == keys::I || next_key == keys::Y))
                        || (first_vowel == keys::A && (next_key == keys::O || next_key == keys::I || next_key == keys::Y))
                        || (first_vowel == keys::O && (next_key == keys::I || next_key == keys::E))
                        || (first_vowel == keys::E && next_key == keys::O);

                    if !is_vietnamese_no_initial {
                        return true;
                    }
                }

                if has_initial_consonant {
                    let (prev_char, _, _) = e.raw_input[i - 1];
                    if !keys::is_vowel(prev_char) {
                        continue;
                    }
                    let prev_vowel = prev_char;
                    if prev_vowel == next_key {
                        if i + 2 < e.raw_input.len() {
                            let (char_after, _, _) = e.raw_input[i + 2];
                            if keys::is_consonant(char_after) {
                                let is_circumflex_vowel =
                                    matches!(prev_vowel, keys::A | keys::E | keys::O);
                                let is_valid_final =
                                    constants::VALID_FINALS_1.contains(&char_after);

                                if is_circumflex_vowel && is_valid_final {
                                    let raw_str: String = e
                                        .raw_input
                                        .iter()
                                        .filter_map(|&(k, c, s)| {
                                            utils::key_to_char_ext(k, c, s)
                                        })
                                        .collect();
                                    if english_dict::is_english_word(&raw_str) {
                                        return true;
                                    }
                                } else {
                                    return true;
                                }
                            }
                        }
                        continue;
                    }
                    let is_vietnamese_pattern = match prev_vowel {
                        k if k == keys::U => {
                            next_key == keys::A
                                || next_key == keys::O
                                || next_key == keys::Y
                                || next_key == keys::I
                        }
                        k if k == keys::A => {
                            next_key == keys::I
                                || next_key == keys::Y
                                || next_key == keys::O
                                || next_key == keys::U
                        }
                        k if k == keys::O => next_key == keys::I || next_key == keys::A,
                        k if k == keys::E => next_key == keys::O || next_key == keys::U,
                        k if k == keys::I => next_key == keys::U || next_key == keys::A,
                        _ => false,
                    };
                    if !is_vietnamese_pattern {
                        if prev_vowel == keys::O && next_key == keys::E {
                            let has_vn_digraph = if e.raw_input.len() >= 2 {
                                let (c1, _, _) = e.raw_input[0];
                                let (c2, _, _) = e.raw_input[1];
                                let ends_with_h = c2 == keys::H
                                    && matches!(
                                        c1,
                                        keys::C | keys::K | keys::G | keys::T | keys::P | keys::N
                                    );
                                let is_ng = c1 == keys::N && c2 == keys::G;
                                let is_tr = c1 == keys::T && c2 == keys::R;
                                ends_with_h || is_ng || is_tr
                            } else {
                                false
                            };

                            if has_vn_digraph {
                                continue;
                            }

                            let raw_str: String = e
                                .raw_input
                                .iter()
                                .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                                .collect();
                            if !english_dict::is_english_word(&raw_str) {
                                continue;
                            }
                        }

                        if prev_vowel == keys::U && next_key == keys::E {
                            let has_vn_digraph = if e.raw_input.len() >= 2 {
                                let (c1, _, _) = e.raw_input[0];
                                let (c2, _, _) = e.raw_input[1];
                                let is_qu = c1 == keys::Q && c2 == keys::U;
                                let ends_with_h = c2 == keys::H
                                    && matches!(
                                        c1,
                                        keys::C | keys::K | keys::G | keys::T | keys::P | keys::N
                                    );
                                let is_ng = c1 == keys::N && c2 == keys::G;
                                let is_tr = c1 == keys::T && c2 == keys::R;
                                is_qu || ends_with_h || is_ng || is_tr
                            } else {
                                false
                            };

                            if has_vn_digraph {
                                continue;
                            }

                            let raw_str: String = e
                                .raw_input
                                .iter()
                                .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                                .collect();
                            if !english_dict::is_english_word(&raw_str) {
                                continue;
                            }
                        }
                        return true;
                    }
                }
            }
        }
    }

    if e.raw_input.len() >= 2 {
        let (last, _, _) = e.raw_input[e.raw_input.len() - 1];
        if last == keys::W {
            let (second_last, _, _) = e.raw_input[e.raw_input.len() - 2];
            if keys::is_vowel(second_last) && second_last != keys::U && second_last != keys::O {
                let w_was_absorbed = e.buf.len() < e.raw_input.len();

                let vowel_count = e.raw_input[..e.raw_input.len() - 1]
                    .iter()
                    .filter(|(k, _, _)| keys::is_vowel(*k))
                    .count();

                if !(w_was_absorbed && vowel_count >= 2) {
                    return true;
                }
            }
        }
    }

    if e.raw_input.len() >= 3 {
        for i in 0..e.raw_input.len() - 2 {
            let (v1, _, _) = e.raw_input[i];
            let (v2, _, _) = e.raw_input[i + 1];
            let (next, _, _) = e.raw_input[i + 2];

            if keys::is_vowel(v1) && v1 == v2 && next == keys::K {
                return true;
            }
        }
    }

    if is_word_complete && e.raw_input.len() >= 3 {
        let len = e.raw_input.len();
        let (last, _, _) = e.raw_input[len - 1];
        if last == keys::P {
            let (v1, _, _) = e.raw_input[len - 3];
            let (v2, _, _) = e.raw_input[len - 2];
            if v1 == keys::E && v2 == keys::E {
                if len >= 4 {
                    let (before_ee, _, _) = e.raw_input[len - 4];
                    if before_ee == keys::I || before_ee == keys::X {
                        // Vietnamese pattern, continue
                    } else {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
    }

    let tone_modifiers_final = [keys::S, keys::F, keys::R, keys::X, keys::J];
    if e.raw_input.len() >= 4 {
        let (first, _, _) = e.raw_input[0];
        let (last, _, _) = e.raw_input[e.raw_input.len() - 1];
        if (first == keys::S || first == keys::F) && tone_modifiers_final.contains(&last) {
            let (v1, _, _) = e.raw_input[e.raw_input.len() - 3];
            let (v2, _, _) = e.raw_input[e.raw_input.len() - 2];
            if keys::is_vowel(v1) && v1 == v2 {
                if v1 != keys::O && v1 != keys::E {
                    return true;
                }
            }
        }
    }

    if e.raw_input.len() == 3 {
        let (first, _, _) = e.raw_input[0];
        let (second, _, _) = e.raw_input[1];
        let (third, _, _) = e.raw_input[2];
        if first == keys::S && second == keys::A && third == keys::X {
            return true;
        }
    }

    if e.raw_input.len() == 5 {
        let (c0, _, _) = e.raw_input[0];
        let (c1, _, _) = e.raw_input[1];
        let (c2, _, _) = e.raw_input[2];
        let (c3, _, _) = e.raw_input[3];
        let (c4, _, _) = e.raw_input[4];

        let is_consonant_0 = keys::is_consonant(c0);
        let is_vowel_1 = keys::is_vowel(c1);
        let is_tone_2 = matches!(c2, keys::S | keys::F | keys::R | keys::X | keys::J);
        let is_circumflex_vowel_34 = matches!(c3, keys::A | keys::E | keys::O) && c3 == c4;

        if is_consonant_0 && is_vowel_1 && is_tone_2 && is_circumflex_vowel_34 {
            return true;
        }
    }

    if e.raw_input.len() >= 4 {
        let (last, _, _) = e.raw_input[e.raw_input.len() - 1];
        if last == keys::K {
            let (second_last, _, _) = e.raw_input[e.raw_input.len() - 2];
            let tone_modifiers_k = [keys::S, keys::F, keys::R, keys::X, keys::J];
            if tone_modifiers_k.contains(&second_last) {
                let has_breve_marker = e.raw_input[..e.raw_input.len() - 2]
                    .iter()
                    .any(|(k, _, _)| *k == keys::W);

                let (third_last, _, _) = e.raw_input[e.raw_input.len() - 3];
                let is_isk_ask_pattern = keys::is_vowel(third_last)
                    && second_last == keys::S
                    && !has_breve_marker
                    && e.raw_input.len() >= 4;

                if is_isk_ask_pattern {
                    let has_consonant_before_vowel =
                        e.raw_input.len() >= 4 && keys::is_consonant(e.raw_input[0].0);

                    if has_consonant_before_vowel {
                        let (first, _, _) = e.raw_input[0];
                        let is_ethnic_initial = first == keys::B || first == keys::L;

                        if !is_ethnic_initial {
                            return true;
                        }
                    }
                }
            }
        }
    }

    if e.raw_input.len() == 4 {
        let (c0, _, _) = e.raw_input[0];
        let (c1, _, _) = e.raw_input[1];
        let (c2, _, _) = e.raw_input[2];
        let (c3, _, _) = e.raw_input[3];

        if keys::is_consonant(c0)
            && (c1 == keys::I || c1 == keys::E)
            && c2 == keys::M
            && c3 == keys::S
        {
            let is_vietnamese_cim_word = (c1 == keys::I
                && matches!(c0, keys::B | keys::D | keys::M | keys::T))
                || (c1 == keys::E
                    && matches!(c0, keys::K | keys::L | keys::N | keys::S | keys::T));

            if !is_vietnamese_cim_word {
                return true;
            }
        }
    }

    false
}

/// Check if buffer has transforms and is invalid Vietnamese.
/// Returns the raw chars if restore is needed, None otherwise.
pub(super) fn should_auto_restore(e: &Engine, is_word_complete: bool) -> Option<Vec<char>> {
    if !e.english_auto_restore {
        return None;
    }

    if e.raw_input.is_empty() || e.buf.is_empty() {
        return None;
    }

    if !e.had_any_transform {
        return None;
    }

    if e.reverted_circumflex_key.is_some() {
        let vowels: Vec<u16> = e
            .buf
            .iter()
            .filter(|c| keys::is_vowel(c.key))
            .map(|c| c.key)
            .collect();
        if vowels.len() >= 2 && vowels.iter().all(|&k| k == vowels[0]) {
            return None;
        }
    }

    let has_w_in_raw = e.raw_input.iter().any(|(key, _, _)| *key == keys::W);
    let has_telex_double = e.raw_input.windows(2).any(|pair| {
        let (k1, _, _) = pair[0];
        let (k2, _, _) = pair[1];
        k1 == k2 && (k1 == keys::O || k1 == keys::E || k1 == keys::A || k1 == keys::D)
    }) || e.raw_input.windows(3).any(|triple| {
        let (k1, _, _) = triple[0];
        let (k2, _, _) = triple[1];
        let (k3, _, _) = triple[2];
        k1 == k3 && keys::is_vowel(k1) && !keys::is_vowel(k2)
    });
    let has_vn_specific_mark = e.buf.iter().any(|c| {
        c.tone == tone::CIRCUMFLEX || c.tone == tone::HORN || c.stroke
    });
    if has_vn_specific_mark
        && !has_w_in_raw
        && !has_telex_double
        && !is_buffer_invalid_vietnamese(e)
    {
        return None;
    }

    if e.had_telex_transform {
        let raw_str = if let Some(ref stored) = e.telex_double_raw {
            let subsequent_start = if e.raw_input.len() < e.telex_double_raw_len {
                e.telex_double_raw_len.saturating_sub(1)
            } else {
                e.telex_double_raw_len
            };
            let subsequent: String = e
                .raw_input
                .iter()
                .skip(subsequent_start)
                .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
                .collect();
            format!("{}{}", stored.to_lowercase(), subsequent.to_lowercase())
        } else {
            get_raw_input_string(e)
        };

        if telex_doubles::contains(&raw_str) {
            let has_stroke = e.buf.iter().any(|c| c.stroke);
            let buffer_invalid_vn = is_buffer_invalid_vietnamese(e);
            let raw_in_english_dict = english_dict::is_english_word(&raw_str);

            let w_at_end = e
                .raw_input
                .last()
                .map(|(k, _, _)| *k == keys::W)
                .unwrap_or(false);

            let is_standalone_stroke = e.buf.len() == 1 && has_stroke;
            if has_stroke && (!raw_in_english_dict || is_standalone_stroke) {
                // Skip restore
            } else if w_at_end && raw_in_english_dict {
                return build_raw_chars_exact(e);
            } else if buffer_invalid_vn && raw_in_english_dict {
                let buffer_str = e.buf.to_full_string().to_lowercase();
                if !english_dict::is_english_word(&buffer_str) {
                    return build_raw_chars_exact(e);
                }
            }
        }

        if let Some(ref stored) = e.telex_double_raw {
            let has_subsequent_chars = e.raw_input.len() > e.telex_double_raw_len;

            if !has_subsequent_chars {
                let chars: Vec<char> = stored.chars().collect();
                if chars.len() >= 2 {
                    let last = chars[chars.len() - 1].to_ascii_lowercase();
                    let second_last = chars[chars.len() - 2].to_ascii_lowercase();
                    let is_double_ss = last == 's' && second_last == 's';
                    let is_double_ff = last == 'f' && second_last == 'f';

                    if is_double_ss || is_double_ff {
                        let original_lower = stored.to_lowercase();
                        if english_dict::is_english_word(&original_lower) {
                            let is_exception = if chars.len() == 3 {
                                let first = chars[0].to_ascii_lowercase();
                                let is_off = first == 'o' && is_double_ff;
                                let is_iff = first == 'i' && is_double_ff;
                                let is_ass = first == 'a' && is_double_ss;
                                is_off || is_iff || is_ass
                            } else {
                                false
                            };

                            if !is_exception {
                                return build_raw_chars_exact(e);
                            }
                        }
                    }
                }
            }
        }

        if let Some(ref stored) = e.telex_double_raw {
            let has_marks = e.buf.iter().any(|c| c.tone > 0 || c.mark > 0);
            let has_stroke = e.buf.iter().any(|c| c.stroke);
            let buffer_str = e.buf.to_full_string();
            let has_repeated_consonant = buffer_str
                .as_bytes()
                .windows(2)
                .any(|w| w[0] == w[1] && matches!(w[0], b's' | b'f' | b'r' | b'x' | b'j'));
            let subsequent_len = e
                .raw_input
                .len()
                .saturating_sub(e.telex_double_raw_len);
            let full_restore_len = stored.len() + subsequent_len;
            let raw_much_longer = full_restore_len > e.buf.len() + 1;
            if !has_marks && !has_stroke && !has_repeated_consonant && !raw_much_longer {
                return None;
            }

            let raw_input_str = get_raw_input_string(e);
            let raw_is_english = english_dict::is_english_word(&raw_input_str);
            let chars: Vec<char> = raw_input_str.chars().collect();

            if !raw_is_english && chars.len() >= 4 {
                let len = chars.len();
                let is_mark_char = |c: char| matches!(c, 's' | 'f' | 'r' | 'x' | 'j');
                let is_vowel_char = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
                let last = chars[len - 1].to_ascii_lowercase();

                let is_vmvmv_pattern = len >= 5
                    && is_vowel_char(last)
                    && {
                        let v1 = last;
                        let m1 = chars[len - 2].to_ascii_lowercase();
                        let v2 = chars[len - 3].to_ascii_lowercase();
                        let m2 = chars[len - 4].to_ascii_lowercase();
                        let v3 = chars[len - 5].to_ascii_lowercase();

                        is_mark_char(m1)
                            && is_vowel_char(v2)
                            && is_mark_char(m2)
                            && is_vowel_char(v3)
                            && m1 == m2
                            && v1 == v2 && v2 == v3
                    };

                let is_vmvm_pattern = is_mark_char(last)
                    && {
                        let m1 = last;
                        let v1 = chars[len - 2].to_ascii_lowercase();
                        let m2 = chars[len - 3].to_ascii_lowercase();
                        let v2 = chars[len - 4].to_ascii_lowercase();

                        is_vowel_char(v1)
                            && is_mark_char(m2)
                            && is_vowel_char(v2)
                            && m1 == m2
                            && v1 == v2
                    };

                let has_diacritic_marks = e.buf.iter().any(|c| c.mark > 0);
                let has_stroke_buf = e.buf.iter().any(|c| c.stroke);

                if (is_vmvmv_pattern || is_vmvm_pattern) && !has_diacritic_marks && !has_stroke_buf {
                    return None;
                }
            }
        }
    }

    let has_marks_or_tones = e.buf.iter().any(|c| c.tone > 0 || c.mark > 0);
    let has_stroke = e.buf.iter().any(|c| c.stroke);

    if !has_marks_or_tones && !has_stroke && ends_with_double_modifier(e) {
        let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
        let buffer_tones: Vec<u8> = e.buf.iter().map(|c| c.tone).collect();
        if validation::is_valid_with_tones(&buffer_keys, &buffer_tones) {
            return None;
        }
    }

    let buffer_invalid_vn = is_buffer_invalid_vietnamese(e);

    if buffer_invalid_vn && has_stroke && !has_marks_or_tones && e.buf.len() < 4 {
        return None;
    }

    if e.had_mark_revert && e.raw_input.len() >= 2 && e.raw_input.len() <= 4 {
        let (last_key, _, _) = e.raw_input[e.raw_input.len() - 1];
        let (second_last_key, _, _) = e.raw_input[e.raw_input.len() - 2];
        if last_key == second_last_key && last_key == keys::R {
            return None;
        }
    }

    let raw_input_valid_en = is_raw_input_valid_english(e);

    if e.had_mark_revert && buffer_invalid_vn && raw_input_valid_en {
        let tone_mods = [keys::S, keys::F, keys::R, keys::X, keys::J];

        let mut doubled_pos = None;
        for i in 0..e.raw_input.len().saturating_sub(1) {
            let (k1, _, _) = e.raw_input[i];
            let (k2, _, _) = e.raw_input[i + 1];
            if tone_mods.contains(&k1) && k1 == k2 {
                doubled_pos = Some(i);
                break;
            }
        }

        if let Some(pos) = doubled_pos {
            let is_at_end = pos + 2 >= e.raw_input.len();

            let is_after_initial_vowel = pos == 1 && {
                let (first_key, _, _) = e.raw_input[0];
                keys::is_vowel(first_key)
            };

            let chars_after = e.raw_input.len() - pos - 2;

            let ends_with_e = e
                .raw_input
                .last()
                .map(|(k, _, _)| *k == keys::E)
                .unwrap_or(false);
            let is_telex_pattern = chars_after == 1 && ends_with_e;

            let w_converted_to_horn = !e.raw_input.is_empty() && {
                let (first_key, _, _) = e.raw_input[0];
                first_key == keys::W && e.buf.get(0).map(|c| c.key) != Some(keys::W)
            };

            if !is_at_end && !is_after_initial_vowel && is_telex_pattern && !w_converted_to_horn {
                return None;
            }
        }
    }

    if buffer_invalid_vn && raw_input_valid_en {
        return build_raw_chars(e);
    }

    if is_word_complete && buffer_invalid_vn && raw_input_valid_en {
        let has_ow_in_raw = e.raw_input.windows(2).any(|w| {
            let (k1, _, _) = w[0];
            let (k2, _, _) = w[1];
            k1 == keys::O && k2 == keys::W
        });
        let has_horn_o_in_buffer = e
            .buf
            .iter()
            .any(|c| c.key == keys::O && c.tone == tone::HORN);
        if has_ow_in_raw && has_horn_o_in_buffer {
            return build_raw_chars(e);
        }
    }

    if is_word_complete && !e.raw_input.is_empty() {
        let (first_key, _, _) = e.raw_input[0];
        if first_key == keys::W {
            if e.raw_input.len() >= 2 {
                let (second_key, _, _) = e.raw_input[1];
                if second_key == keys::R || second_key == keys::H {
                    return build_raw_chars(e);
                }
            }
            if buffer_invalid_vn {
                return build_raw_chars(e);
            }
        }
    }

    if is_word_complete && has_english_modifier_pattern(e, true) && raw_input_valid_en {
        if !has_stroke {
            return build_raw_chars(e);
        }
    }

    if is_word_complete
        && e.raw_input.len() >= e.buf.len() + 2
        && !has_stroke
        && raw_input_valid_en
    {
        let has_circumflex = e.buf.iter().any(|c| c.tone == tone::CIRCUMFLEX);
        let has_marks = e.buf.iter().any(|c| c.mark > 0);
        if has_circumflex && !has_marks {
            return build_raw_chars(e);
        }
    }

    if is_word_complete
        && e.had_vowel_triggered_circumflex
        && !has_stroke
        && raw_input_valid_en
    {
        let has_marks = e.buf.iter().any(|c| c.mark > 0);
        if !has_marks {
            let buf_str = e.buf.to_full_string().to_lowercase();
            if buf_str.ends_with("ât")
                || buf_str.ends_with("êt")
                || buf_str.ends_with("ôt")
                || buf_str.ends_with("âc")
                || buf_str.ends_with("êc")
                || buf_str.ends_with("ôc")
                || buf_str.ends_with("âp")
                || buf_str.ends_with("êp")
                || buf_str.ends_with("ôp")
            {
                return build_raw_chars(e);
            }
        }
    }

    if is_word_complete
        && e.had_mark_revert
        && e.buf.len() <= 3
        && raw_input_valid_en
        && !has_stroke
    {
        let tone_modifiers = [keys::S, keys::F, keys::R, keys::X, keys::J];
        let has_same_modifier_doubled_vowel =
            (0..e.raw_input.len().saturating_sub(2)).any(|i| {
                let (key, _, _) = e.raw_input[i];
                let (next_key, _, _) = e.raw_input[i + 1];
                let (after_key, _, _) = e.raw_input[i + 2];
                tone_modifiers.contains(&key)
                    && key == next_key
                    && keys::is_vowel(after_key)
            });
        if has_same_modifier_doubled_vowel {
            return build_raw_chars(e);
        }
    }

    if is_word_complete && !has_stroke && raw_input_valid_en {
        let raw_vowels: Vec<u16> = e
            .raw_input
            .iter()
            .map(|(k, _, _)| *k)
            .filter(|k| keys::is_vowel(*k))
            .collect();

        if raw_vowels.len() >= 3 {
            let last_three = &raw_vowels[raw_vowels.len() - 3..];
            let v1 = last_three[0];
            let v2 = last_three[1];
            let v3 = last_three[2];

            if v1 == v3 && v1 != v2 {
                let first_three = [raw_vowels[0], raw_vowels[1], raw_vowels[2]];
                if constants::VALID_TRIPHTHONGS.contains(&first_three) {
                    let buf_vowels: Vec<(u16, u8)> = e
                        .buf
                        .iter()
                        .filter(|c| keys::is_vowel(c.key))
                        .map(|c| (c.key, c.tone))
                        .collect();
                    if buf_vowels.len() == 3 {
                        let (bv0, _) = buf_vowels[0];
                        let (bv1, bv1_tone) = buf_vowels[1];
                        let (bv2, _) = buf_vowels[2];
                        if bv0 == first_three[0]
                            && bv1 == first_three[1]
                            && bv2 == first_three[2]
                            && bv1_tone == tone::CIRCUMFLEX
                        {
                            return None;
                        }
                    }
                }

                let buf_vowels: Vec<(u16, u8)> = e
                    .buf
                    .iter()
                    .filter(|c| keys::is_vowel(c.key))
                    .map(|c| (c.key, c.tone))
                    .collect();

                if buf_vowels.len() >= 2 {
                    let buf_last_two = &buf_vowels[buf_vowels.len() - 2..];
                    let (buf_v1, buf_v1_tone) = buf_last_two[0];
                    let (buf_v2, _) = buf_last_two[1];

                    if buf_v1 == v1
                        && buf_v1_tone == tone::CIRCUMFLEX
                        && buf_v2 == v2
                        && !e.buf.iter().any(|c| c.mark > 0)
                    {
                        return build_raw_chars(e);
                    }
                }
            }
        }
    }

    None
}

/// Auto-restore invalid Vietnamese to raw English on space
pub(super) fn try_auto_restore_on_space(e: &Engine) -> Result {
    if let Some(mut raw_chars) = should_auto_restore(e, true) {
        raw_chars.push(' ');
        let backspace = e.buf.len() as u8;
        Result::send(backspace, &raw_chars)
    } else {
        Result::none()
    }
}

/// Auto-restore invalid Vietnamese to raw English on break key
pub(super) fn try_auto_restore_on_break(e: &Engine) -> Result {
    if let Some(raw_chars) = should_auto_restore(e, true) {
        let backspace = e.buf.len() as u8;
        Result::send(backspace, &raw_chars)
    } else {
        Result::none()
    }
}

/// Restore buffer to raw ASCII (undo all Vietnamese transforms)
pub(super) fn restore_to_raw(e: &Engine) -> Result {
    if e.raw_input.is_empty() || e.buf.is_empty() {
        return Result::none();
    }

    let raw_chars: Vec<char> = if let Some(ref base_raw) = e.telex_double_raw {
        let mut chars: Vec<char> = base_raw.chars().collect();
        for &(key, caps, shift) in e.raw_input.iter().skip(e.telex_double_raw_len) {
            if let Some(ch) = utils::key_to_char_ext(key, caps, shift) {
                chars.push(ch);
            }
        }
        chars
    } else {
        e.raw_input
            .iter()
            .filter_map(|&(key, caps, shift)| utils::key_to_char_ext(key, caps, shift))
            .collect()
    };

    if raw_chars.is_empty() {
        return Result::none();
    }

    let buffer_str = e.buf.to_full_string();
    let raw_str: String = raw_chars.iter().collect();

    if !e.had_any_transform && buffer_str == raw_str {
        return Result::none();
    }

    let backspace = e.buf.len() as u8;
    Result::send(backspace, &raw_chars)
}
