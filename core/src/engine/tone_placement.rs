use super::Engine;
use crate::data::{chars::mark, english_dict, keys};
use crate::data::vowel::{Phonology, Vowel};
use crate::utils;

/// Reposition tone (sắc/huyền/hỏi/ngã/nặng) after vowel pattern changes
pub(super) fn reposition_tone_if_needed(e: &mut Engine) -> Option<(usize, usize)> {
    let raw_str: String = e
        .raw_input
        .iter()
        .filter_map(|&(k, caps, _)| utils::key_to_char(k, caps))
        .collect::<String>()
        .to_lowercase();
    let is_english_word = english_dict::is_english_word(&raw_str);

    let tone_info: Option<(usize, u8)> = e
        .buf
        .iter()
        .enumerate()
        .find(|(_, c)| c.mark > mark::NONE && keys::is_vowel(c.key))
        .map(|(i, c)| (i, c.mark));

    if let Some((old_pos, tone_value)) = tone_info {
        let vowels = utils::collect_vowels(&e.buf);
        if vowels.is_empty() {
            return None;
        }

        if is_english_word && !vowels_form_valid_diphthong(&vowels) {
            return None;
        }

        let has_consonant_after_tone = (old_pos + 1..e.buf.len()).any(|i| {
            e.buf
                .get(i)
                .is_some_and(|c| !keys::is_vowel(c.key) && c.key != keys::W)
        });
        let has_vowel_after_consonant = has_consonant_after_tone
            && vowels
                .iter()
                .any(|v| v.pos > old_pos && has_consonant_between(e, old_pos, v.pos));

        if has_vowel_after_consonant && !vowels_form_valid_diphthong(&vowels) {
            return None;
        }

        let has_qu = utils::has_qu_initial(&e.buf);
        let has_gi = utils::has_gi_initial(&e.buf);

        if has_gi && vowels.iter().all(|v| v.key == keys::I) {
            return None;
        }

        let effective_vowels: &[Vowel] = if vowels.len() >= 2
            && ((has_qu && vowels[0].key == keys::U) || (has_gi && vowels[0].key == keys::I))
        {
            &vowels[1..]
        } else {
            &vowels
        };

        if effective_vowels.len() >= 2
            && effective_vowels.iter().all(|v| v.key == effective_vowels[0].key)
        {
            return None;
        }

        let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
        let has_final = utils::has_final_consonant(&e.buf, last_vowel_pos);
        let new_pos =
            Phonology::find_tone_position(&vowels, has_final, e.modern_tone, has_qu, has_gi);

        if new_pos != old_pos {
            if let Some(c) = e.buf.get_mut(old_pos) {
                c.mark = mark::NONE;
            }
            if let Some(c) = e.buf.get_mut(new_pos) {
                c.mark = tone_value;
            }
            return Some((old_pos, new_pos));
        }
    }
    None
}

/// Check if there's a consonant between two positions
pub(super) fn has_consonant_between(e: &Engine, start: usize, end: usize) -> bool {
    (start + 1..end).any(|i| {
        e.buf
            .get(i)
            .is_some_and(|c| !keys::is_vowel(c.key) && c.key != keys::W)
    })
}

/// Check if vowels form a valid Vietnamese diphthong pattern
pub(super) fn vowels_form_valid_diphthong(vowels: &[Vowel]) -> bool {
    use crate::data::vowel::{TONE_FIRST_PATTERNS, TONE_SECOND_PATTERNS};

    if vowels.len() < 2 {
        return false;
    }

    let pair = [vowels[0].key, vowels[1].key];

    TONE_FIRST_PATTERNS
        .iter()
        .any(|p| p[0] == pair[0] && p[1] == pair[1])
        || TONE_SECOND_PATTERNS
            .iter()
            .any(|p| p[0] == pair[0] && p[1] == pair[1])
}

/// Reorder buffer when a vowel completes a diphthong with earlier vowel,
/// and there are consonants between that should be final consonants.
pub(super) fn reorder_diphthong_with_final(e: &mut Engine) -> Option<usize> {
    use crate::data::constants::VALID_FINALS_1;

    let len = e.buf.len();
    if len < 3 {
        return None;
    }

    let has_vn_transforms = e.buf.iter().any(|c| c.mark > 0 || c.tone > 0);
    if !has_vn_transforms {
        return None;
    }

    let raw_str: String = e
        .raw_input
        .iter()
        .filter_map(|&(key, caps, _)| utils::key_to_char(key, caps))
        .collect::<String>()
        .to_lowercase();
    if english_dict::is_english_word(&raw_str) {
        return None;
    }

    let new_vowel_pos = len - 1;
    let new_vowel_key = e.buf.get(new_vowel_pos)?.key;

    let mut prev_vowel_pos = None;
    let mut consonants_between = Vec::new();

    for i in (0..new_vowel_pos).rev() {
        let c = e.buf.get(i)?;
        if keys::is_vowel(c.key) {
            prev_vowel_pos = Some(i);
            break;
        } else if c.key != keys::W {
            consonants_between.push(i);
        }
    }

    let prev_vowel_pos = prev_vowel_pos?;
    if consonants_between.is_empty() {
        return None;
    }

    let has_earlier_vowels =
        (0..prev_vowel_pos).any(|i| e.buf.get(i).is_some_and(|c| keys::is_vowel(c.key)));
    if has_earlier_vowels {
        return None;
    }

    let prev_vowel = e.buf.get(prev_vowel_pos)?;
    let prev_vowel_key = prev_vowel.key;

    if prev_vowel.tone > 0 {
        return None;
    }

    let pair = [prev_vowel_key, new_vowel_key];
    let is_reorderable_diphthong = matches!(pair, [keys::I, keys::A] | [keys::U, keys::A]);

    if !is_reorderable_diphthong {
        return None;
    }

    if consonants_between.len() > 2 {
        return None;
    }

    let consonant_keys: Vec<u16> = consonants_between
        .iter()
        .rev()
        .filter_map(|&i| e.buf.get(i).map(|c| c.key))
        .collect();

    let is_valid_final = match consonant_keys.len() {
        1 => VALID_FINALS_1.contains(&consonant_keys[0]),
        2 => matches!(
            (consonant_keys[0], consonant_keys[1]),
            (keys::N, keys::G) | (keys::N, keys::H) | (keys::C, keys::H)
        ),
        _ => false,
    };

    if !is_valid_final {
        return None;
    }

    let new_vowel = *e.buf.get(new_vowel_pos)?;

    for &pos in &consonants_between {
        if let Some(c) = e.buf.get(pos) {
            let c_copy = *c;
            if let Some(next) = e.buf.get_mut(pos + 1) {
                *next = c_copy;
            }
        }
    }

    let insert_pos = prev_vowel_pos + 1;
    if let Some(slot) = e.buf.get_mut(insert_pos) {
        *slot = new_vowel;
    }

    Some(insert_pos)
}
