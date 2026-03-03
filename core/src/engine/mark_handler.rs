use super::{Engine, helpers, revert};
use crate::data::{chars::tone, constants, english_dict, keys};
use crate::data::vowel::Phonology;
use crate::engine::{syllable, types::{Result, Transform}};
use crate::utils;
use super::validation::{is_foreign_word_pattern, is_valid, is_valid_for_transform_with_foreign};

pub(super) fn try_mark(e: &mut Engine, key: u16, caps: bool, mark_val: u8) -> Option<Result> {
    if e.buf.is_empty() {
        return None;
    }

    // Check revert first
    if let Some(Transform::Mark(last_key, _)) = e.last_transform {
        if last_key == key {
            return Some(revert::revert_mark(e, key, caps));
        }
    }

    // Telex: Check for delayed stroke pattern (d + vowels + d)
    // When buffer is "dod" and mark key is typed, apply stroke to initial 'd'
    // This enables "dods" → "đó" while preventing "de" + "d" → "đe"
    let had_delayed_stroke = e.method == 0
        && e.buf.len() >= 2
        && e.buf
            .get(0)
            .is_some_and(|c| c.key == keys::D && !c.stroke)
        && e.buf.last().is_some_and(|c| c.key == keys::D)
        && {
            // Check vowels and validity in one pass
            let buf_len = e.buf.len();
            let has_vowel = e.buf
                .iter()
                .take(buf_len - 1)
                .any(|c| keys::is_vowel(c.key));
            has_vowel && {
                let buffer_without_last: Vec<u16> =
                    e.buf.iter().take(buf_len - 1).map(|c| c.key).collect();
                is_valid(&buffer_without_last) && {
                    // Apply delayed stroke: stroke initial 'd', remove trigger 'd'
                    if let Some(c) = e.buf.get_mut(0) {
                        c.stroke = true;
                    }
                    e.buf.pop();
                    true
                }
            }
        };

    // Issue #44: Apply pending breve before adding mark
    // When user types "aws" (Telex) or "a81" (VNI), they want "ắ" (breve + sắc)
    // Breve was deferred due to open syllable, but adding mark confirms Vietnamese input
    let mut had_pending_breve = false;
    if let Some(breve_pos) = e.pending_breve_pos {
        had_pending_breve = true;
        // Try to find and remove the breve modifier from buffer
        // Both Telex 'w' and VNI '8' are stored in buffer (handle_normal_letter adds them)
        let modifier_pos = breve_pos + 1;
        if modifier_pos < e.buf.len() {
            if let Some(c) = e.buf.get(modifier_pos) {
                // Remove 'w' (Telex) or '8' (VNI) breve modifier from buffer
                if c.key == keys::W || c.key == keys::N8 {
                    e.buf.remove(modifier_pos);
                }
            }
        }
        // Apply breve to 'a'
        if let Some(c) = e.buf.get_mut(breve_pos) {
            if c.key == keys::A {
                c.tone = tone::HORN; // HORN on A = breve (ă)
                e.had_any_transform = true;
            }
        }
        e.pending_breve_pos = None;
    }

    // Telex: Check for delayed circumflex pattern (V + C + V where both V are same)
    // When buffer is "toto" (t-o-t-o) and mark key is typed, apply circumflex + remove trigger
    // This enables "totos" → "tốt" while preventing "data" → "dât"
    // Pattern: C₁ + V + C₂ + V where V is same vowel (a, e, o)
    let mut had_delayed_circumflex = false;
    if e.method == 0 && e.buf.len() >= 3 {
        // Get vowel positions
        let vowel_positions: Vec<(usize, u16)> = e.buf
            .iter()
            .enumerate()
            .filter(|(_, c)| keys::is_vowel(c.key))
            .map(|(i, c)| (i, c.key))
            .collect();

        // Check for exactly 2 vowels that are the same (a, e, or o for circumflex)
        if vowel_positions.len() == 2 {
            let (pos1, key1) = vowel_positions[0];
            let (pos2, key2) = vowel_positions[1];
            let is_circumflex_vowel = matches!(key1, keys::A | keys::E | keys::O);

            // Check if first vowel already has circumflex - skip delayed circumflex if so
            // This prevents "deeper" from being corrupted: after "dee" → "dê", then "deepe"
            // should NOT trigger delayed circumflex since first 'e' already has circumflex
            let first_vowel_already_has_circumflex = e.buf
                .get(pos1)
                .is_some_and(|c| c.tone == tone::CIRCUMFLEX);

            // Must be same vowel, must have consonant(s) between them, first vowel must not already have circumflex
            if key1 == key2
                && is_circumflex_vowel
                && pos2 > pos1 + 1
                && !first_vowel_already_has_circumflex
            {
                // Check for consonants between the two vowels
                let consonants_between: Vec<u16> = (pos1 + 1..pos2)
                    .filter_map(|j| {
                        e.buf.get(j).and_then(|c| {
                            if !keys::is_vowel(c.key) {
                                Some(c.key)
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                // Must have exactly one consonant between and it must be a non-extending
                // final (t, m, p). Consonants that can extend (n→ng/nh, c→ch) are
                // handled immediately in try_tone.
                let is_non_extending_final = consonants_between.len() == 1
                    && matches!(consonants_between[0], keys::T | keys::M | keys::P);

                // Check if second vowel is at end of buffer (typical trigger position)
                let second_vowel_at_end = pos2 == e.buf.len() - 1;

                // Check initial consonants for Vietnamese validity
                // Skip delayed circumflex if initial looks English (e.g., "pr" in "proposal")
                let initial_keys: Vec<u16> = (0..pos1)
                    .filter_map(|j| e.buf.get(j).map(|ch| ch.key))
                    .take_while(|k| !keys::is_vowel(*k))
                    .collect();

                // Validate initial consonants:
                // - 0 initials: valid (vowel-only start)
                // - 1 initial: valid (single consonant)
                // - 2 initials: must be in VALID_INITIALS_2 (nh, th, ph, etc.)
                // - 3+ initials: skip for delayed circumflex
                //   (words like "proposal" with "pr" will be rejected here)
                let has_valid_vietnamese_initial = match initial_keys.len() {
                    0 | 1 => true,
                    2 => {
                        let pair = [initial_keys[0], initial_keys[1]];
                        constants::VALID_INITIALS_2.contains(&pair)
                    }
                    _ => false,
                };

                // Check for double initial specifically (for immediate vs delayed handling)
                let has_vietnamese_double_initial =
                    initial_keys.len() >= 2 && has_valid_vietnamese_initial;

                // Only apply delayed circumflex if:
                // - Has non-extending middle consonant (t, m, p)
                // - Second vowel is at end (trigger position)
                // - Has valid Vietnamese initial (skip English like "proposal")
                // - No double initial (those work immediately without delay)
                // - User didn't just revert a circumflex (typing 3rd vowel to cancel)
                if is_non_extending_final
                    && second_vowel_at_end
                    && has_valid_vietnamese_initial
                    && !has_vietnamese_double_initial
                    && !e.had_circumflex_revert
                {
                    // Skip delayed circumflex if raw_input is an English word
                    // This prevents "pasta" → "pất", "costa" → "côt", etc.
                    // The raw_input check works because English words like "pasta"
                    // are in our dictionary, while Vietnamese typing patterns are not.
                    let raw_str: String = e.raw_input
                        .iter()
                        .filter_map(|&(k, caps, _)| utils::key_to_char(k, caps))
                        .collect::<String>()
                        .to_lowercase();
                    if english_dict::is_english_word(&raw_str) {
                        // Raw input is English - don't apply delayed circumflex
                        // Let the letter be added normally, auto-restore will handle it
                    } else {
                        // IMPORTANT: Check foreign word pattern BEFORE modifying buffer
                        // to avoid leaving buffer in inconsistent state if we need to return None.
                        // Example: "cete" + 'r' → "cêt" (delayed circumflex) + T+R check → foreign
                        // Without this check, buffer would be left as "cêt" even though we return None.
                        let temp_buffer_keys: Vec<u16> =
                            e.buf.iter().map(|c| c.key).collect();
                        let temp_buffer_tones: Vec<u8> =
                            e.buf.iter().map(|c| c.tone).collect();
                        // Check what buffer would look like after circumflex (keys without trigger)
                        let mut post_circumflex_keys = temp_buffer_keys.clone();
                        post_circumflex_keys.remove(pos2); // simulate removing trigger vowel
                        let post_circumflex_tones: Vec<u8> = post_circumflex_keys
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                if i == pos1 {
                                    tone::CIRCUMFLEX
                                } else {
                                    temp_buffer_tones.get(i).copied().unwrap_or(0)
                                }
                            })
                            .collect();

                        // Skip delayed circumflex if the resulting buffer would trigger foreign pattern
                        if is_foreign_word_pattern(
                            &post_circumflex_keys,
                            &post_circumflex_tones,
                            key,
                        ) {
                            // Don't apply delayed circumflex - let the letter be added normally
                        } else {
                            had_delayed_circumflex = true;
                            // Apply circumflex to first vowel
                            if let Some(c) = e.buf.get_mut(pos1) {
                                c.tone = tone::CIRCUMFLEX;
                                e.had_any_transform = true;
                            }
                            // Remove second vowel (it was just a trigger)
                            e.buf.remove(pos2);
                        }
                    }
                }
            }
        }
    }

    // Check if buffer has horn transforms - indicates intentional Vietnamese typing
    // (e.g., "rượu" has base keys [R,U,O,U] which looks like "ou" pattern,
    // but with horns applied it's valid "ươu")
    let has_horn_transforms = e.buf.iter().any(|c| c.tone == tone::HORN);

    // Check if buffer has stroke transforms (đ) - indicates intentional Vietnamese typing
    // Issue #48: "ddeso" → "đéo" (d was stroked to đ, so this is Vietnamese, not English)
    let has_stroke_transforms = e.buf.iter().any(|c| c.stroke);

    // Validate buffer structure (skip if has horn/stroke transforms - already intentional Vietnamese)
    // Also skip validation if free_tone mode is enabled
    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
    let buffer_tones: Vec<u8> = e.buf.iter().map(|c| c.tone).collect();
    if !e.free_tone_enabled
        && !has_horn_transforms
        && !has_stroke_transforms
        && !is_valid_for_transform_with_foreign(&buffer_keys, e.allow_foreign_consonants)
    {
        return None;
    }

    // Check for invalid "-ing" rhyme: Vietnamese uses "-inh", NOT "-ing" with tone marks
    // Examples: "thíng" is invalid (things), but "tính" is valid
    // If vowel is 'i' and final is 'ng', reject marks
    if !e.free_tone_enabled && !has_horn_transforms && !has_stroke_transforms {
        let syllable = syllable::parse(&buffer_keys);
        if syllable.vowel.len() == 1 && syllable.final_c.len() == 2 {
            let vowel_key = buffer_keys[syllable.vowel[0]];
            let final_keys = [
                buffer_keys[syllable.final_c[0]],
                buffer_keys[syllable.final_c[1]],
            ];
            // i + ng = invalid Vietnamese rhyme for tone/mark
            if vowel_key == keys::I && final_keys == [keys::N, keys::G] {
                return None;
            }
        }
    }

    // Skip modifier if buffer shows foreign word patterns.
    // Only check when NO horn/stroke transforms exist.
    //
    // Detected patterns:
    // - Invalid vowel combinations (ou, yo) that don't exist in Vietnamese
    // - Consonant clusters after finals common in English (T+R, P+R, C+R)
    //
    // Examples:
    // - "met" + 'r' → T+R cluster common in English → skip modifier
    // - "you" + 'r' → "ou" vowel pattern invalid → skip modifier
    // - "rươu" + 'j' → has horn transforms → DON'T skip, apply mark normally
    // - "đe" + 's' → has stroke transform → DON'T skip, apply mark normally (Issue #48)
    // Skip foreign word detection if free_tone mode is enabled
    if !e.free_tone_enabled
        && !has_horn_transforms
        && !has_stroke_transforms
        && is_foreign_word_pattern(&buffer_keys, &buffer_tones, key)
    {
        return None;
    }

    // Issue #29: Normalize ưo → ươ compound before placing mark
    // In Vietnamese, "ưo" is never valid - it's always "ươ"
    let rebuild_from_compound = normalize_uo_compound(e);

    let vowels = utils::collect_vowels(&e.buf);
    if vowels.is_empty() {
        return None;
    }

    // Find mark position using phonology rules
    let last_vowel_pos = vowels.last().map(|v| v.pos).unwrap_or(0);
    let has_final = utils::has_final_consonant(&e.buf, last_vowel_pos);
    let has_qu = utils::has_qu_initial(&e.buf);
    let has_gi = utils::has_gi_initial(&e.buf);
    let pos =
        Phonology::find_tone_position(&vowels, has_final, e.modern_tone, has_qu, has_gi);

    // Check if target vowel already has the same mark
    // This handles two cases:
    //
    // 1. "lists" pattern: After mark, user typed CONSONANT then same mark key
    //    - Buffer: [L, í, T] → has consonant after marked vowel
    //    - User wants to REVERT the mark → "lits"
    //
    // 2. "roofif" pattern: After mark, user typed VOWEL then same mark key
    //    - Buffer: [R, ồ, I] → only vowels after marked vowel (diphthong)
    //    - User is still in same syllable, second 'f' is likely accidental
    //    - Absorb → "rồi"
    //
    // EXCEPTION: Words starting with W are English - don't apply revert logic.
    // W is not a valid Vietnamese initial, so "writer", "wrong", "wrap" etc.
    // should NOT trigger delayed revert even if same mark key is pressed.
    // Auto-restore will handle these at word boundary.
    let starts_with_w = e.raw_input
        .first()
        .map(|(k, _, _)| *k == keys::W)
        .unwrap_or(false);

    if let Some(c) = e.buf.get(pos) {
        if c.mark == mark_val && !starts_with_w {
            // Check if there's a consonant after the marked vowel position
            let has_consonant_after = e.buf
                .iter()
                .skip(pos + 1)
                .any(|ch| !keys::is_vowel(ch.key));

            // Check if vowel is at the END of buffer (no chars after at all)
            // Issue #197: After backspace, vowel may be at end - pressing same
            // mark key should REVERT, not absorb
            // Example: "serv" → "sẻv" → backspace → "sẻ" → 'r' should → "ser"
            let is_vowel_at_end = pos + 1 >= e.buf.len();

            if has_consonant_after || is_vowel_at_end {
                // Consonant after OR vowel at end: REVERT the mark (remove dấu)
                // "lists" → "lits", user typed s twice to undo the mark
                // "sẻ" → "se", user typed r after backspace to undo the mark
                return Some(revert::revert_mark(e, key, caps));
            } else {
                // Vowels after (not at end): absorb (user double-tapped in same syllable)
                // "roofif" → "rồi"
                return Some(Result::send(0, &[]));
            }
        }
    }

    if let Some(c) = e.buf.get_mut(pos) {
        c.mark = mark_val;
        e.last_transform = Some(Transform::Mark(key, mark_val));
        e.had_any_transform = true;
        e.had_telex_transform = true; // Track for whitelist-based auto-restore
                                         // Rebuild from the earlier position if compound was formed
        let mut rebuild_pos = rebuild_from_compound.map_or(pos, |cp| cp.min(pos));

        // If delayed stroke was applied, rebuild from position 0
        // and add extra backspace for the trigger 'd' that was on screen
        if had_delayed_stroke {
            rebuild_pos = 0;
            let result = helpers::rebuild_from(&e.buf, rebuild_pos);
            let chars: Vec<char> = result.chars[..result.count as usize]
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect();
            // Add 1 to backspace for the trigger 'd' that was on screen but removed from buffer
            return Some(Result::send(result.backspace + 1, &chars));
        }

        // If there was pending breve, we need extra backspace
        // Screen has 'w' (Telex) or '8' (VNI) that needs to be deleted
        // Note: Telex 'w' was in buffer and removed, VNI '8' was never in buffer
        if had_pending_breve {
            let result = helpers::rebuild_from(&e.buf, rebuild_pos);
            // Convert u32 chars to char vec
            let chars: Vec<char> = result.chars[..result.count as usize]
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect();
            // Add 1 to backspace to account for modifier on screen
            return Some(Result::send(result.backspace + 1, &chars));
        }

        // If delayed circumflex was applied, rebuild from earliest vowel position
        // and add extra backspace for the trigger vowel that was on screen but removed
        if had_delayed_circumflex {
            rebuild_pos = rebuild_pos.min(1); // Start from first vowel position
            let result = helpers::rebuild_from(&e.buf, rebuild_pos);
            let chars: Vec<char> = result.chars[..result.count as usize]
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect();
            // Add 1 to backspace for the removed trigger vowel still on screen
            return Some(Result::send(result.backspace + 1, &chars));
        }

        return Some(helpers::rebuild_from(&e.buf, rebuild_pos));
    }

    None
}

pub(super) fn normalize_uo_compound(e: &mut Engine) -> Option<usize> {
    // Look for pattern: U with horn + O without horn (anywhere in buffer)
    for i in 0..e.buf.len().saturating_sub(1) {
        let c1 = e.buf.get(i)?;
        let c2 = e.buf.get(i + 1)?;

        // Check: U with horn + O plain → always normalize to ươ
        let is_u_with_horn = c1.key == keys::U && c1.tone == tone::HORN;
        let is_o_plain = c2.key == keys::O && c2.tone == tone::NONE;

        if is_u_with_horn && is_o_plain {
            // Apply horn to O to form the ươ compound
            if let Some(c) = e.buf.get_mut(i + 1) {
                c.tone = tone::HORN;
                return Some(i + 1);
            }
        }
    }
    None
}

pub(super) fn find_uo_compound_positions(e: &Engine) -> Option<(usize, usize)> {
    for i in 0..e.buf.len().saturating_sub(1) {
        if let (Some(c1), Some(c2)) = (e.buf.get(i), e.buf.get(i + 1)) {
            let is_uo = c1.key == keys::U && c2.key == keys::O;
            let is_ou = c1.key == keys::O && c2.key == keys::U;
            if is_uo || is_ou {
                return Some((i, i + 1));
            }
        }
    }
    None
}

pub(super) fn has_uo_compound(e: &Engine) -> bool {
    find_uo_compound_positions(e).is_some()
}

pub(super) fn has_complete_uo_compound(e: &Engine) -> bool {
    if let Some((pos1, pos2)) = find_uo_compound_positions(e) {
        if let (Some(c1), Some(c2)) = (e.buf.get(pos1), e.buf.get(pos2)) {
            // Check ư + ơ pattern (both with horn)
            let is_u_horn = c1.key == keys::U && c1.tone == tone::HORN;
            let is_o_horn = c2.key == keys::O && c2.tone == tone::HORN;
            return is_u_horn && is_o_horn;
        }
    }
    false
}

pub(super) fn find_horn_target_with_switch(e: &Engine, targets: &[u16], new_tone: u8) -> Vec<usize> {
    // Find vowel positions that match targets and either:
    // - have no tone (normal case)
    // - have a different tone (switching case)
    let vowels: Vec<usize> = e.buf
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            targets.contains(&c.key) && (c.tone == tone::NONE || c.tone != new_tone)
        })
        .map(|(i, _)| i)
        .collect();

    if vowels.is_empty() {
        return vec![];
    }

    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();

    // Use centralized phonology rules (context inferred from buffer)
    let mut result = Phonology::find_horn_positions(&buffer_keys, &vowels);

    // Special case: standalone "ua" pattern where U already has a mark
    // If user typed "uaf" → "ùa", then 'w' should go to U (making "ừa"), not A
    // This ensures consistent behavior: mark placement indicates user's intent
    if result.len() == 1 {
        if let Some(&pos) = result.first() {
            if let Some(c) = e.buf.get(pos) {
                // If horn target is A, check if U exists before it with a mark
                if c.key == keys::A && pos > 0 {
                    if let Some(prev) = e.buf.get(pos - 1) {
                        // Adjacent U with a mark → user wants horn on U, not breve on A
                        if prev.key == keys::U && prev.mark > 0 {
                            result = vec![pos - 1]; // Return U position instead
                        }
                    }
                }
            }
        }
    }

    result
        .into_iter()
        .filter(|&pos| {
            e.buf
                .get(pos)
                .map(|c| {
                    targets.contains(&c.key) && (c.tone == tone::NONE || c.tone != new_tone)
                })
                .unwrap_or(false)
        })
        .collect()
}
