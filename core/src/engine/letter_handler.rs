use super::{auto_restore, Engine, helpers, mark_handler, stroke_handler, tone_placement};
use crate::data::{chars::{self, tone}, english_dict, keys};
use crate::engine::{buffer::Char, types::{Result, Transform}};
use crate::engine::validation::is_foreign_word_pattern;
use crate::{input, utils};

pub(super) fn handle_normal_letter(e: &mut Engine, key: u16, caps: bool) -> Result {
    // Special case: "o" after "w→ư" should form "ươ" compound
    // This only handles the WAsVowel case (typing "w" alone creates ư)
    // For "uw" pattern, the compound is normalized in try_mark via normalize_uo_compound
    if key == keys::O && matches!(e.last_transform, Some(Transform::WAsVowel)) {
        // Add O with horn to form ươ compound
        let mut c = Char::new(key, caps);
        c.tone = tone::HORN;
        e.buf.push(c);
        e.last_transform = None;

        // Return the ơ character (o with horn)
        let vowel_char = chars::to_char(keys::O, caps, tone::HORN, 0).unwrap();
        return Result::send(0, &[vowel_char]);
    }

    // Note: ShortPatternStroke revert is now handled at the beginning of process()
    // before any modifiers are applied, so we don't need to check it here.

    // Telex: Revert delayed circumflex when same vowel is typed again
    // Pattern: After "data" → "dât" (delayed circumflex), typing 'a' again should revert to "data"
    // Buffer ends with: vowel-with-circumflex + non-extending-final (t, m, p)
    // Typed key matches the base of the circumflex vowel (a→â, e→ê, o→ô)
    // IMPORTANT: Only apply this revert for DELAYED circumflex (V+C+V pattern), not for
    // immediate circumflex (VV pattern like "deep" → "dêp"). For immediate circumflex,
    // typing another vowel should NOT revert (allows words like "deeper").
    if e.method == 0
        && e.had_vowel_triggered_circumflex
        && matches!(key, keys::A | keys::E | keys::O)
        && e.buf.len() >= 2
    {
        let last_idx = e.buf.len() - 1;
        let vowel_idx = e.buf.len() - 2;

        // Check if last char is a non-extending final consonant
        let last_is_non_extending = e.buf
            .get(last_idx)
            .is_some_and(|c| matches!(c.key, keys::T | keys::M | keys::P));

        // Check if second-to-last has circumflex and matches typed vowel
        let should_revert = last_is_non_extending
            && e.buf.get(vowel_idx).is_some_and(|c| {
                c.tone == tone::CIRCUMFLEX
                    && c.key == key
                    && matches!(c.key, keys::A | keys::E | keys::O)
            });

        if should_revert {
            // Remove circumflex from the vowel
            if let Some(c) = e.buf.get_mut(vowel_idx) {
                c.tone = tone::NONE;
            }
            // Reset vowel-triggered circumflex flag since we're reverting
            e.had_vowel_triggered_circumflex = false;
            // Track circumflex revert for auto-restore (used to collapse double vowel at end)
            e.had_circumflex_revert = true;

            // Add the typed vowel to buffer (the one that triggered revert)
            // "dataa" flow: "dât" (3 chars) → revert â → "dat" → add 'a' → "data" (4 chars)
            e.buf.push(Char::new(key, caps));

            // Rebuild from vowel position using after_insert (new char not yet on screen)
            // Screen has: "dât" (3 chars), buffer now has: "data" (4 chars)
            // Need to delete "ât" (2 chars) and output "ata" (3 chars) → screen becomes "data"
            return helpers::rebuild_from_after_insert(&e.buf, vowel_idx);
        }
    }

    // Telex: Post-tone delayed circumflex (xepse → xếp)
    // Pattern: initial-consonant + vowel-with-mark + non-extending-final (t, m, p) + same vowel
    // When user types tone BEFORE circumflex modifier: "xeps" → "xép", then 'e' → "xếp"
    // The second vowel triggers circumflex on the first vowel (keeping existing mark)
    // IMPORTANT: Must have initial consonant to form valid Vietnamese syllable
    // "expect" (e-x-p-e) should NOT trigger because no initial consonant
    if e.method == 0 && matches!(key, keys::A | keys::E | keys::O) && e.buf.len() >= 3 {
        let last_idx = e.buf.len() - 1;
        let vowel_idx = e.buf.len() - 2;

        // Check if there's at least one initial consonant before the vowel
        let has_initial_consonant =
            vowel_idx > 0 && e.buf.get(0).is_some_and(|c| keys::is_consonant(c.key));

        // Check if last char is a non-extending final consonant
        let last_is_non_extending = e.buf
            .get(last_idx)
            .is_some_and(|c| matches!(c.key, keys::T | keys::M | keys::P));

        // Check if second-to-last has mark but NO circumflex, and matches typed vowel
        let should_add_circumflex = has_initial_consonant
            && last_is_non_extending
            && e.buf.get(vowel_idx).is_some_and(|c| {
                c.mark > 0 // has tone mark (sắc, huyền, etc.)
                    && c.tone == tone::NONE // but no circumflex yet
                    && c.key == key // matches typed vowel
                    && matches!(c.key, keys::A | keys::E | keys::O)
            });

        if should_add_circumflex {
            // Skip circumflex if raw_input is an English word
            // This prevents "pasta" → "pất", "costa" → "côt", etc.
            // raw_input includes the current key (pushed before process() is called)
            let raw_str: String = e.raw_input
                .iter()
                .filter_map(|&(k, caps, _)| utils::key_to_char(k, caps))
                .collect::<String>()
                .to_lowercase();
            if english_dict::is_english_word(&raw_str) {
                // Raw input is English - skip circumflex, add vowel normally
                // The auto-restore will handle restoring the English word
            } else {
                // Add circumflex to the vowel (keeping existing mark)
                if let Some(c) = e.buf.get_mut(vowel_idx) {
                    c.tone = tone::CIRCUMFLEX;
                    e.had_any_transform = true;
                }

                // Note: raw_input already has the key (pushed at on_key_ext before process)

                // Rebuild from vowel position (second vowel is NOT added to buffer - it's modifier)
                // Screen has: "xép" (3 chars), buffer stays: "xếp" (3 chars, vowel updated)
                // Need to delete "ép" (2 chars) and output "ếp" (2 chars)
                return helpers::rebuild_from(&e.buf, vowel_idx);
            }
        }
    }

    e.last_transform = None;
    // Add letters to buffer, and numbers in both Telex and VNI modes
    // This ensures buffer.len() stays in sync with screen chars for correct backspace count
    // Issue #162: Numbers must be added to buffer in Telex mode too, otherwise patterns
    // like "o2o" have buffer = [O] (missing '2') causing the second 'o' to incorrectly
    // trigger circumflex (thinking it's "oo" → "ô")
    if keys::is_letter(key) || keys::is_number(key) {
        // Add the letter/number to buffer
        e.buf.push(Char::new(key, caps));

        // Issue #44 (part 2): Apply deferred breve when valid final consonant is typed
        // "trawm" → after "traw" (pending breve on 'a'), typing 'm' applies breve → "trăm"
        if let Some(breve_pos) = e.pending_breve_pos {
            // Valid final consonants that make breve valid: c, k, m, n, p, t
            // Note: k is included for ethnic minority words (Đắk Lắk)
            if matches!(
                key,
                keys::C | keys::K | keys::M | keys::N | keys::P | keys::T
            ) {
                // Find and remove the breve modifier from buffer
                // Telex uses 'w', VNI uses '8' - it should be right after 'a' at breve_pos
                let modifier_pos = breve_pos + 1;
                if modifier_pos < e.buf.len() {
                    if let Some(c) = e.buf.get(modifier_pos) {
                        // Remove 'w' (Telex) or '8' (VNI)
                        if c.key == keys::W || c.key == keys::N8 {
                            e.buf.remove(modifier_pos);
                        }
                    }
                }

                // Apply breve to the 'a' at pending position
                let a_caps = e.buf.get(breve_pos).map(|c| c.caps).unwrap_or(false);
                if let Some(c) = e.buf.get_mut(breve_pos) {
                    if c.key == keys::A {
                        c.tone = tone::HORN; // HORN on A = breve (ă)
                        e.had_any_transform = true;
                    }
                }
                e.pending_breve_pos = None;

                // Rebuild from breve position: delete "aw" (or "awX"), output "ăX"
                // Buffer now has: ...ă (at breve_pos) + consonant (just added)
                // Screen has: ...aw (need to delete "aw", output "ă" + consonant)
                let vowel_char = chars::to_char(keys::A, a_caps, tone::HORN, 0).unwrap_or('ă');
                let cons_char = crate::utils::key_to_char(key, caps).unwrap_or('?');
                return Result::send(2, &[vowel_char, cons_char]); // backspace 2 ("aw"), output "ăm"
            } else if key == keys::W {
                // 'w' is the breve modifier - don't clear pending_breve_pos
                // It will be added as a regular letter and removed later
            } else if keys::is_vowel(key) {
                // Vowel after "aw" pattern - breve not valid, clear pending
                e.pending_breve_pos = None;
            }
            // For other consonants (not finals, not W), keep pending_breve_pos
            // They might be followed by more letters that complete the syllable
        }

        // Issue #133: Apply deferred horn to 'u' when final consonant/vowel is typed
        // "duow" → "duơ" (pending on u), then "c" → apply horn to u → "dược"
        if let Some(u_pos) = e.pending_u_horn_pos {
            // Apply horn to 'u' at pending position
            if let Some(c) = e.buf.get_mut(u_pos) {
                if c.key == keys::U && c.tone == tone::NONE {
                    c.tone = tone::HORN;
                    e.had_any_transform = true;
                }
            }
            e.pending_u_horn_pos = None;

            // Rebuild from u position: screen has "...uơ...", buffer has "...ươ...+new_char"
            // The new char was already pushed at line 1799 but not yet on screen
            // Use rebuild_from_after_insert which accounts for this
            return helpers::rebuild_from_after_insert(&e.buf, u_pos);
        }

        // Normalize ưo → ươ immediately when 'o' is typed after 'ư'
        // This ensures "dduwo" → "đươ" (Telex) and "u7o" → "ươ" (VNI)
        // Works for both methods since "ưo" alone is not valid Vietnamese
        if key == keys::O && mark_handler::normalize_uo_compound(e).is_some() {
            // ươ compound formed - reposition tone if needed (ư→ơ)
            if let Some((old_pos, _)) = tone_placement::reposition_tone_if_needed(e) {
                return helpers::rebuild_from_after_insert(&e.buf, old_pos);
            }

            // No tone to reposition - just output ơ
            let vowel_char = chars::to_char(keys::O, caps, tone::HORN, 0).unwrap();
            return Result::send(0, &[vowel_char]);
        }

        // Reorder buffer when a vowel completes a diphthong with earlier vowel
        // and there are consonants between that should be final consonants.
        // Example: "kisna" → buffer is k-í-n, adding 'a' should produce k-í-a-n (kían)
        // because "ia" is a diphthong and 'n' is a valid final consonant.
        if keys::is_vowel(key) {
            if let Some(reorder_pos) = tone_placement::reorder_diphthong_with_final(e) {
                // After reordering, also reposition tone if needed
                // Example: "musno" → buffer reordered to m-ú-o-n, but tone should be on 'o'
                // because "uo" diphthong has tone on second vowel.
                let tone_reposition = tone_placement::reposition_tone_if_needed(e);
                let rebuild_pos = tone_reposition.map(|(old, _)| old).unwrap_or(reorder_pos);
                return helpers::rebuild_from_after_insert(&e.buf, rebuild_pos);
            }
        }

        // Auto-correct tone position when new character changes the correct placement
        //
        // Two scenarios:
        // 1. New vowel changes diphthong pattern:
        //    "osa" → tone on 'o', then 'a' added → "oa" needs tone on 'a'
        // 2. New consonant creates final, which changes tone position:
        //    "muas" → tone on 'u' (ua open), then 'n' added → "uan" needs tone on 'a'
        //
        // Both cases need to reposition the tone mark based on Vietnamese phonology.
        if let Some((old_pos, _new_pos)) = tone_placement::reposition_tone_if_needed(e) {
            // Tone was moved - rebuild output from the old position
            // Note: the new char was just added to buffer but NOT yet displayed
            // So backspace = (chars from old_pos to BEFORE new char)
            // And output = (chars from old_pos to end INCLUDING new char)
            return helpers::rebuild_from_after_insert(&e.buf, old_pos);
        }

        // Check if adding this letter creates invalid vowel pattern (foreign word detection)
        // Only revert if the horn transforms are from w-as-vowel (standalone w→ư),
        // not from w-as-tone (adding horn to existing vowels like in "rượu")
        //
        // w-as-vowel: first horn is U at position 0 (was standalone 'w')
        // w-as-tone: horns are on vowels after initial consonant
        //
        // Exception: complete ươ compound + vowel = valid Vietnamese triphthong
        // (like "rượu" = ươu, "mười" = ươi) - don't revert in these cases
        // Only skip for vowels that form valid triphthongs (u, i), not for consonants
        // Only run foreign word detection if english_auto_restore is enabled
        if e.english_auto_restore {
            let is_valid_triphthong_ending =
                mark_handler::has_complete_uo_compound(e) && (key == keys::U || key == keys::I);
            if stroke_handler::has_w_as_vowel_transform(e) && !is_valid_triphthong_ending {
                let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
                let buffer_tones: Vec<u8> = e.buf.iter().map(|c| c.tone).collect();
                if is_foreign_word_pattern(&buffer_keys, &buffer_tones, key) {
                    return stroke_handler::revert_w_as_vowel_transforms(e);
                }
            }
        }

        // Auto-restore when consonant after mark creates clear English pattern
        // Example: "tex" → "tẽ", then 't' typed → "tẽt" has English modifier pattern → restore "text"
        //
        // IMPORTANT: Mid-word, only restore for clear English PATTERNS (modifier+consonant clusters),
        // NOT just structural invalidity. Words like "dọd" are invalid but user might still be typing.
        // Full structural validation happens at word boundary (space/break).
        //
        // This catches: "tex" + 't' where 'x' modifier before 't' creates English cluster
        // But preserves: "dọ" + 'd' where 'j' modifier before 'd' doesn't indicate English
        //
        // IMPORTANT: Skip mark keys (s, f, r, x, j in Telex) because they're tone modifiers,
        // not true consonants. User typing "đườ" + 's' wants to add sắc mark, not restore.
        //
        // Only run if english_auto_restore is enabled (experimental feature)
        let im = input::get(e.method);
        let is_mark_key = im.mark(key).is_some();
        if e.english_auto_restore
            && keys::is_consonant(key)
            && !is_mark_key
            && e.buf.len() >= 2
        {
            // Check if consonant immediately follows a marked character
            if let Some(prev_char) = e.buf.get(e.buf.len() - 2) {
                let prev_has_mark = prev_char.mark > 0 || prev_char.tone > 0;

                if prev_has_mark && auto_restore::has_english_modifier_pattern(e, false) {
                    // Clear English pattern detected - restore to raw
                    if let Some(raw_chars) = auto_restore::build_raw_chars(e) {
                        let backspace = (e.buf.len() - 1) as u8;

                        // Repopulate buffer with restored content (plain chars, no marks)
                        e.buf.clear();
                        for &(key, caps, _) in &e.raw_input {
                            e.buf.push(Char::new(key, caps));
                        }

                        e.last_transform = None;
                        return Result::send(backspace, &raw_chars);
                    }
                }
            }
        }
    } else {
        // Non-letter character (number, symbol, etc.)
        // Mark that this word has non-letter prefix to prevent false shortcut matches
        // e.g., "149k" should NOT trigger shortcut "k" → "không"
        // e.g., "@abc" should NOT trigger shortcut "abc"
        e.has_non_letter_prefix = true;
    }
    Result::none()
}
