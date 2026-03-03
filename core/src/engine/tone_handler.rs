use super::{Engine, helpers, mark_handler, revert, tone_placement};
use crate::data::{chars::tone, constants, keys};
use crate::engine::{syllable, types::{Result, Transform}};
use crate::input::ToneType;
use crate::utils;
use super::validation::is_valid_for_transform_with_foreign;

pub(super) fn try_tone(e: &mut Engine, key: u16, caps: bool, tone_type: ToneType, targets: &[u16]) -> Option<Result> {
    if e.buf.is_empty() {
        return None;
    }

    // Issue #44: Cancel pending breve if same modifier pressed again ("aww" → "aw")
    // When breve was deferred and user presses 'w' again, cancel without adding another 'w'
    if e.pending_breve_pos.is_some()
        && (tone_type == ToneType::Horn || tone_type == ToneType::Breve)
    {
        // Cancel the pending breve - user doesn't want Vietnamese
        e.pending_breve_pos = None;
        // Return "consumed but no change" to prevent 'w' from being typed
        // action=Send with 0 backspace and 0 chars effectively consumes the key
        return Some(Result::send(0, &[]));
    }

    // Check revert first (same key pressed twice)
    if let Some(Transform::Tone(last_key, _)) = e.last_transform {
        if last_key == key {
            return Some(revert::revert_tone(e, key, caps));
        }
    }

    // Issue #211: Extended vowel mode - skip circumflex transform after revert
    // After aaa→aa revert, aaaa should become aaa (append raw), not aâ (re-transform)
    if e.reverted_circumflex_key == Some(key) && tone_type == ToneType::Circumflex {
        return None; // Let normal letter handling append raw vowel
    }

    // Validate buffer structure (not vowel patterns - those are checked after transform)
    // Skip validation if free_tone mode is enabled
    let buffer_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();

    if !e.free_tone_enabled
        && !is_valid_for_transform_with_foreign(&buffer_keys, e.allow_foreign_consonants)
    {
        return None;
    }

    // Check for invalid "-ing" rhyme: Vietnamese uses "-inh", NOT "-ing" with tone
    // Examples: "thíng" is invalid (things), but "tính" is valid
    // If vowel is 'i' and final is 'ng', reject tone marks
    if !e.free_tone_enabled {
        let syllable = syllable::parse(&buffer_keys);
        if syllable.vowel.len() == 1 && syllable.final_c.len() == 2 {
            let vowel_key = buffer_keys[syllable.vowel[0]];
            let final_keys = [
                buffer_keys[syllable.final_c[0]],
                buffer_keys[syllable.final_c[1]],
            ];
            // i + ng = invalid Vietnamese rhyme for tone marks
            if vowel_key == keys::I && final_keys == [keys::N, keys::G] {
                return None;
            }
        }
    }

    let tone_val = tone_type.value();

    // Check if we're switching from one tone to another (e.g., ô → ơ)
    // Find vowels that have a DIFFERENT tone (to switch) or NO tone (to add)
    let is_switching = e.buf
        .iter()
        .any(|c| targets.contains(&c.key) && c.tone != tone::NONE && c.tone != tone_val);

    // Scan buffer for eligible target vowels
    let mut target_positions = Vec::new();

    // Special case: uo/ou compound for horn - find adjacent pair only
    // But ONLY apply compound logic when BOTH vowels are plain (not when switching)
    if tone_type == ToneType::Horn && !is_switching {
        if let Some((pos1, pos2)) = mark_handler::find_uo_compound_positions(e) {
            if let (Some(c1), Some(c2)) = (e.buf.get(pos1), e.buf.get(pos2)) {
                // Only apply compound when BOTH vowels have no tone
                if c1.tone == tone::NONE && c2.tone == tone::NONE {
                    // Issue #133: Check if "uo" pattern is at end of syllable (no final)
                    // If no final consonant/vowel after "uo", only apply horn to 'o'
                    // Examples: "huow" → "huơ", "khuow" → "khuơ"
                    // But: "duowc" → "dược", "muowif" → "mười" (both get horn)
                    let is_uo_pattern = c1.key == keys::U && c2.key == keys::O;
                    let has_final = e.buf.get(pos2 + 1).is_some();

                    // Check if 'u' is preceded by 'Q' (qu-initial consonant cluster)
                    // In "Qu-", the 'u' is part of the initial and should not get horn
                    // Examples: "Quoiws" → "Quới" (not "Qưới"), "quốc" (not "qước")
                    let preceded_by_q =
                        pos1 > 0 && e.buf.get(pos1 - 1).map(|c| c.key) == Some(keys::Q);

                    if preceded_by_q {
                        // "Qu-" pattern - only second vowel gets horn
                        target_positions.push(pos2);
                        e.pending_u_horn_pos = None;
                    } else if is_uo_pattern && !has_final {
                        // "uơ" pattern - only 'o' gets horn initially
                        // Set pending so 'u' gets horn if final consonant/vowel is added
                        target_positions.push(pos2);
                        e.pending_u_horn_pos = Some(pos1);
                    } else {
                        // "ươ" pattern (or has final) - both get horn
                        target_positions.push(pos1);
                        target_positions.push(pos2);
                        e.pending_u_horn_pos = None;
                    }
                }
            }
        }
    }

    // Normal case: find last matching target
    if target_positions.is_empty() {
        if is_switching {
            // When switching, ONLY target vowels that already have a diacritic
            // (don't add diacritics to plain vowels during switch)
            for (i, c) in e.buf.iter().enumerate().rev() {
                if targets.contains(&c.key) && c.tone != tone::NONE && c.tone != tone_val {
                    target_positions.push(i);
                    break;
                }
            }
        } else if tone_type == ToneType::Horn {
            // For horn modifier, apply smart vowel selection based on Vietnamese phonology
            target_positions = mark_handler::find_horn_target_with_switch(e, targets, tone_val);
        } else {
            // Non-horn modifiers (circumflex): use standard target matching
            // For Telex circumflex (aa, ee, oo pattern), require either:
            // 1. Target at LAST position (immediate doubling: "oo" → "ô")
            // 2. No consonants between target and end (delayed diphthong: "oio" → "ôi")
            // This prevents transformation in words like "teacher" where consonants
            // (c, h) appear between the two 'e's
            let is_telex_circumflex = e.method == 0
                && tone_type == ToneType::Circumflex
                && matches!(key, keys::A | keys::E | keys::O);

            // Issue #312: If any vowel already has a tone (horn/circumflex/breve),
            // don't trigger same-vowel circumflex. The typed vowel should append raw.
            // Example: "chưa" + "a" → "chưaa" (NOT "chưâ")
            if is_telex_circumflex {
                let any_vowel_has_tone = e.buf
                    .iter()
                    .filter(|c| keys::is_vowel(c.key))
                    .any(|c| c.has_tone());

                if any_vowel_has_tone {
                    // Skip circumflex, let the vowel append as raw letter
                    return None;
                }

                // Check if buffer has multiple vowel types and any has a mark
                // Skip circumflex if it would create invalid diphthong (like ôà, âo)
                // But allow if circumflex creates valid pattern (like uê, iê, yê)
                // Examples:
                // - "toà" + "a" → [O,A], âo invalid → skip → "toàa"
                // - "ué" + "e" → [U,E], uê valid → allow → "uế"
                let vowel_chars: Vec<_> =
                    e.buf.iter().filter(|c| keys::is_vowel(c.key)).collect();

                let has_any_mark = vowel_chars.iter().any(|c| c.has_mark());
                let unique_vowel_types: std::collections::HashSet<u16> =
                    vowel_chars.iter().map(|c| c.key).collect();
                let has_multiple_vowel_types = unique_vowel_types.len() > 1;

                if has_any_mark && has_multiple_vowel_types {
                    // Check if circumflex on V2 (the key) creates a valid pattern
                    // Valid V2 circumflex patterns: iê, uê, yê, uô
                    // Invalid: oa→oâ, ao→âo, ae→âe, etc.
                    let other_vowel = unique_vowel_types.iter().find(|&&v| v != key).copied();

                    // Check if this is a same-vowel trigger for V1 circumflex
                    // Example: "dausa" (d-á-u + a) → circumflex on first 'a' → "dấu"
                    // The trigger 'a' matches existing 'a' in buffer
                    let is_same_vowel_trigger = unique_vowel_types.contains(&key);

                    // V1 circumflex patterns: circumflex on FIRST vowel of diphthong
                    // These patterns have the trigger vowel + another vowel forming valid diphthong
                    // âu, ây (A with circumflex + U/Y)
                    // êu (E with circumflex + U) - already in V1_CIRCUMFLEX_REQUIRED
                    // ôi (O with circumflex + I)
                    let v1_circumflex_diphthongs: &[[u16; 2]] = &[
                        [keys::A, keys::U], // âu - "dấu"
                        [keys::A, keys::Y], // ây - "dây"
                        [keys::E, keys::U], // êu - "nếu"
                        [keys::O, keys::I], // ôi - "tối"
                    ];

                    let is_valid_v1_circumflex = is_same_vowel_trigger
                        && other_vowel
                            .is_some_and(|v| v1_circumflex_diphthongs.contains(&[key, v]));

                    // Patterns where circumflex on V2 is valid
                    let v2_circumflex_valid: &[[u16; 2]] = &[
                        [keys::I, keys::E], // iê
                        [keys::U, keys::E], // uê
                        [keys::Y, keys::E], // yê
                        [keys::U, keys::O], // uô
                    ];

                    let is_valid_v2_circumflex =
                        other_vowel.is_some_and(|v| v2_circumflex_valid.contains(&[v, key]));

                    if !is_valid_v2_circumflex && !is_valid_v1_circumflex {
                        // Invalid pattern → skip circumflex
                        return None;
                    }
                }

                // Check if adding this vowel would create a valid triphthong
                // If so, skip circumflex and let the vowel append raw
                // Example: "oe" + "o" → [O, E, O] = "oeo" triphthong → skip circumflex
                // BUT: Only check this if the last char in buffer is a vowel
                // If there's a consonant at the end (e.g., "boem"), then same-vowel
                // trigger applies instead of triphthong building
                let last_is_vowel = e.buf.last().is_some_and(|c| keys::is_vowel(c.key));

                if last_is_vowel {
                    let vowels: Vec<u16> = e.buf
                        .iter()
                        .filter(|c| keys::is_vowel(c.key))
                        .map(|c| c.key)
                        .collect();

                    if vowels.len() == 2 {
                        let potential_triphthong = [vowels[0], vowels[1], key];
                        if constants::VALID_TRIPHTHONGS.contains(&potential_triphthong) {
                            // This would create a valid triphthong, skip circumflex
                            return None;
                        }
                    }

                    // Check for V1-V2-V1 pattern in last 2 vowels + new key
                    // Example: "queue" has buffer vowels [U, E, U], new key = E
                    // Last 2 vowels = [E, U], new key = E → pattern is E-U-E (V1-V2-V1)
                    // ONLY block when:
                    // 1. NO Vietnamese indicators present (mark/stroke)
                    // 2. There's a consonant initial (foreign word pattern)
                    // 3. NOT a valid Vietnamese triphthong pattern
                    // This allows: "oio" → "ôi" (no initial, valid VN interjection)
                    // This allows: "hieu" + e → "hiêu" (iêu is valid VN triphthong)
                    // But blocks: "queue" → "quêu" (has "qu" initial, foreign word)
                    let has_vn_indicator = e.buf.iter().any(|c| c.mark > 0 || c.stroke);
                    let has_initial =
                        e.buf.get(0).is_some_and(|c| keys::is_consonant(c.key));

                    if !has_vn_indicator && has_initial && vowels.len() >= 2 {
                        let last_two = &vowels[vowels.len() - 2..];
                        let v1 = last_two[0]; // second-to-last vowel
                        let v2 = last_two[1]; // last vowel
                                              // V1-V2-V1 pattern: new key matches v1 but not v2
                        if key == v1 && key != v2 {
                            // Exception: Allow circumflex for valid Vietnamese triphthongs
                            // e.g., [i, e, u] = iêu (hiểu), [y, e, u] = yêu, [u, e, u] = uêu (nguều)
                            // These require circumflex on E (middle vowel)
                            // The trigger 'e' is the same as v1, which triggers circumflex
                            //
                            // BUT: Exclude Q + U pattern (like "queue")
                            // In Vietnamese, Q only appears as part of "qu" initial cluster
                            // If initial is Q and first vowel is U, it's English (queue, quest)
                            let initial_q = e.buf.get(0).is_some_and(|c| c.key == keys::Q);
                            let first_vowel_u = vowels.first().is_some_and(|&v| v == keys::U);
                            let is_english_qu_pattern = initial_q && first_vowel_u;

                            let is_valid_vn_triphthong = vowels.len() == 3
                                && !is_english_qu_pattern
                                && constants::VALID_TRIPHTHONGS
                                    .contains(&[vowels[0], vowels[1], vowels[2]]);

                            // Issue #183: Also allow V1-V2 diphthongs requiring circumflex on V1
                            // e.g., "neue" → [e, u] = êu (nếu), "xaua" → [a, u] = âu (xấu)
                            // When typing the second V1, it should trigger circumflex on first V1
                            // BUT: Exclude English "qu" patterns (like "queue")
                            let v1_circumflex_diphthongs: &[[u16; 2]] = &[
                                [keys::A, keys::U], // âu - "dấu", "xấu"
                                [keys::A, keys::Y], // ây - "dây"
                                [keys::E, keys::U], // êu - "nếu", "kêu"
                                [keys::O, keys::I], // ôi - "tối"
                            ];
                            let is_valid_v1_circumflex_diphthong = !is_english_qu_pattern
                                && v1_circumflex_diphthongs.contains(&[v1, v2]);

                            if !is_valid_vn_triphthong && !is_valid_v1_circumflex_diphthong {
                                return None;
                            }
                        }
                    }
                }
            }

            for (i, c) in e.buf.iter().enumerate().rev() {
                if targets.contains(&c.key) && c.tone == tone::NONE {
                    // For Telex circumflex, check if there are consonants after target
                    if is_telex_circumflex && i != e.buf.len() - 1 {
                        // Check for consonants between target position and end of buffer
                        let consonants_after: Vec<u16> = (i + 1..e.buf.len())
                            .filter_map(|j| {
                                e.buf.get(j).and_then(|ch| {
                                    if !keys::is_vowel(ch.key) {
                                        Some(ch.key)
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect();

                        if !consonants_after.is_empty() {
                            // Check if there's a NON-ADJACENT vowel between target and final
                            // "teacher": e-a-ch has 'a' between first 'e' and 'ch' → block
                            // "hongo": o-ng has no vowel between 'o' and 'ng' → allow
                            // "dau": a-u is a diphthong (adjacent vowels) → allow
                            // Adjacent vowels (position i+1) form diphthongs, not separate syllables
                            let has_non_adjacent_vowel = (i + 2..e.buf.len()).any(|j| {
                                e.buf.get(j).is_some_and(|ch| keys::is_vowel(ch.key))
                            });

                            if has_non_adjacent_vowel {
                                // A vowel exists after the adjacent position → different syllable
                                // Skip this target (e.g., "teacher" → don't make "têacher")
                                continue;
                            }

                            // Check if consonants form valid Vietnamese finals
                            // Valid finals: single (c,m,n,p,t) or pairs (ch,ng,nh)
                            // Double consonant finals (ng,nh,ch) are distinctly Vietnamese
                            // - "hongo" → "hông" (ng final, allow circumflex)
                            // - "khongo" → "không" (ng final, allow circumflex)
                            // Single consonant finals need additional context
                            // - "data" → should NOT become "dât" (t final, but English)
                            // - "nhana" → "nhân" (n final, but has nh initial)
                            let (all_are_valid_finals, is_double_final) = match consonants_after
                                .len()
                            {
                                1 => (
                                    constants::VALID_FINALS_1.contains(&consonants_after[0]),
                                    false,
                                ),
                                2 => {
                                    let pair = [consonants_after[0], consonants_after[1]];
                                    (constants::VALID_FINALS_2.contains(&pair), true)
                                }
                                _ => (false, false), // More than 2 consonants is invalid
                            };

                            // Double consonant finals (ng,nh,ch) are distinctly Vietnamese
                            // But still need to check: if there's an adjacent vowel, it must
                            // form a valid diphthong with the target. Otherwise skip.
                            // Example: "teacher" has 'e' at i=1 with adjacent 'a' at i+1,
                            // but "ea" is NOT a valid Vietnamese diphthong → skip
                            if is_double_final && all_are_valid_finals {
                                // Check for adjacent vowel that doesn't form valid diphthong
                                let adjacent_vowel_key = (i + 1 < e.buf.len())
                                    .then(|| e.buf.get(i + 1))
                                    .flatten()
                                    .filter(|ch| keys::is_vowel(ch.key))
                                    .map(|ch| ch.key);

                                if let Some(adj_key) = adjacent_vowel_key {
                                    // Check if [target, adjacent] forms valid diphthong
                                    let diphthong =
                                        [e.buf.get(i).map(|c| c.key).unwrap_or(0), adj_key];
                                    if !constants::VALID_DIPHTHONGS.contains(&diphthong) {
                                        // Invalid diphthong like "ea" → skip this target
                                        continue;
                                    }
                                }
                                // Valid double final with valid diphthong (or no adjacent vowel)
                                // This handles "hongo" → "hông", "khongo" → "không"
                            } else if !all_are_valid_finals {
                                // Invalid final consonants → skip
                                continue;
                            } else {
                                // Single consonant final - need VALID diphthong or double initial
                                // Check if there's another vowel adjacent to target that forms
                                // a VALID Vietnamese diphthong (in correct order)
                                // Example: "coup" + "o" → "ou" is NOT valid diphthong → block
                                // Example: "daup" + "a" → "au" IS valid diphthong → allow
                                // Note: diphthong order matters: [V1, V2] not [V2, V1]
                                let target_key = e.buf.get(i).map(|c| c.key).unwrap_or(0);
                                // Adjacent BEFORE: [adjacent, target] order
                                let adjacent_before = i > 0
                                    && e.buf.get(i - 1).is_some_and(|ch| {
                                        keys::is_vowel(ch.key)
                                            && constants::VALID_DIPHTHONGS
                                                .contains(&[ch.key, target_key])
                                    });
                                // Adjacent AFTER: [target, adjacent] order
                                let adjacent_after = i + 1 < e.buf.len()
                                    && e.buf.get(i + 1).is_some_and(|ch| {
                                        keys::is_vowel(ch.key)
                                            && constants::VALID_DIPHTHONGS
                                                .contains(&[target_key, ch.key])
                                    });
                                let has_valid_adjacent_diphthong =
                                    adjacent_before || adjacent_after;

                                // Check for Vietnamese-specific double initial (nh, ch, th, ph, etc.)
                                // This allows "nhana" → "nhân" (nh + a + n + a)
                                // but still blocks "data" → "dât" (d is not a Vietnamese digraph)
                                let has_vietnamese_double_initial = if i >= 2 {
                                    // Get first two consonants before the target vowel
                                    let initial_keys: Vec<u16> = (0..i)
                                        .filter_map(|j| e.buf.get(j).map(|ch| ch.key))
                                        .take_while(|k| !keys::is_vowel(*k))
                                        .collect();
                                    if initial_keys.len() >= 2 {
                                        let pair = [initial_keys[0], initial_keys[1]];
                                        constants::VALID_INITIALS_2.contains(&pair)
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };

                                // Same-vowel trigger: typing the same vowel after consonant
                                // Example: "nanag" → second 'a' triggers circumflex on first 'a'
                                // Pattern: initial + vowel + consonant + SAME_VOWEL
                                // Only allow immediate circumflex for middle consonants that
                                // can form double finals (n→ng/nh, c→ch). These are clearly
                                // Vietnamese patterns.
                                // For other single finals (t,m,p), delay circumflex until
                                // a mark key is typed to avoid false positives like "data"→"dât"
                                let is_same_vowel_trigger =
                                    e.buf.get(i).is_some_and(|c| c.key == key);
                                // Consonants that can form double finals: n→ng/nh, c→ch
                                let middle_can_extend = consonants_after.len() == 1
                                    && matches!(consonants_after[0], keys::N | keys::C);

                                // Check if initial consonant already has stroke (đ/Đ)
                                // If so, it's clearly Vietnamese (from delayed stroke pattern)
                                let initial_has_stroke = (0..i)
                                    .filter_map(|j| e.buf.get(j))
                                    .take_while(|c| !keys::is_vowel(c.key))
                                    .any(|c| c.stroke);

                                // Check for non-extending middle consonant (t, m, p)
                                // These require special handling for delayed circumflex
                                let is_non_extending_final = consonants_after.len() == 1
                                    && matches!(
                                        consonants_after[0],
                                        keys::T | keys::M | keys::P
                                    );

                                // Allow circumflex if any of these conditions are true:
                                // 1. Has adjacent vowel forming VALID diphthong (au, oi, etc.)
                                //    BUT NOT if final is non-extending (t,m,p) - diphthong+t/m/p rarely valid
                                //    EXCEPTION: V2_CIRCUMFLEX_REQUIRED diphthongs (iê, uê, yê, uô) ARE
                                //    valid with non-extending finals (viết, thiết, miếng, etc.)
                                // 2. Has Vietnamese double initial (nh, th, ph, etc.)
                                // 3. Same-vowel trigger with middle consonant that can extend (n,c)
                                // 4. Initial has stroke (đ) - clearly Vietnamese
                                let is_v2_circumflex_diphthong = adjacent_before && {
                                    let v1 = e.buf.get(i - 1).map(|c| c.key).unwrap_or(0);
                                    constants::V2_CIRCUMFLEX_REQUIRED
                                        .contains(&[v1, target_key])
                                };
                                let diphthong_allows = has_valid_adjacent_diphthong
                                    && (!is_non_extending_final || is_v2_circumflex_diphthong);
                                let allow_circumflex = diphthong_allows
                                    || has_vietnamese_double_initial
                                    || (is_same_vowel_trigger && middle_can_extend)
                                    || initial_has_stroke;

                                // Special case: same-vowel trigger with non-extending middle consonant
                                // Apply circumflex immediately when typing second matching vowel
                                // Example: "toto" → "tôt" (second 'o' triggers circumflex on first 'o')
                                // Auto-restore on space will revert if invalid (e.g., "data " → "data ")
                                // Only apply if target has NO mark - if it has a mark (like ngã from 'x'),
                                // the user is building a different pattern (like "expect" → ẽ-p-e-c-t)
                                // Also block if adjacent vowel forms INVALID diphthong
                                // Example: "coupo" → [O, U] invalid → don't apply circumflex
                                let target_has_no_mark =
                                    e.buf.get(i).is_some_and(|c| c.mark == 0);
                                // Check if target has ANY adjacent vowel
                                // Diphthong + non-extending final (t,m,p) is rarely valid Vietnamese
                                // Examples: "âup", "oem", "aum" are all invalid syllables
                                let has_adjacent_vowel_before = i > 0
                                    && e.buf
                                        .get(i - 1)
                                        .is_some_and(|ch| keys::is_vowel(ch.key));
                                let has_adjacent_vowel_after = i + 1 < e.buf.len()
                                    && e.buf
                                        .get(i + 1)
                                        .is_some_and(|ch| keys::is_vowel(ch.key));
                                let has_any_adjacent_vowel =
                                    has_adjacent_vowel_before || has_adjacent_vowel_after;
                                // Block if: has adjacent vowel (diphthong pattern) with non-extending final
                                if is_same_vowel_trigger
                                    && is_non_extending_final
                                    && target_has_no_mark
                                    && !has_any_adjacent_vowel
                                {
                                    // Apply circumflex to first vowel
                                    if let Some(c) = e.buf.get_mut(i) {
                                        c.tone = tone::CIRCUMFLEX;
                                        e.had_any_transform = true;
                                        e.had_vowel_triggered_circumflex = true;
                                    }
                                    // Don't add the trigger vowel - return result immediately
                                    // Need extra backspace because we're replacing displayed char
                                    let result = helpers::rebuild_from(&e.buf, i);
                                    let chars: Vec<char> = result.chars
                                        [..result.count as usize]
                                        .iter()
                                        .filter_map(|&c| char::from_u32(c))
                                        .collect();
                                    return Some(Result::send(result.backspace, &chars));
                                }

                                if !allow_circumflex {
                                    // Single final, no diphthong, no double initial, not valid same-vowel → likely English
                                    continue;
                                }
                            }
                        }
                    }
                    target_positions.push(i);
                    break;
                }
            }
        }
    }

    if target_positions.is_empty() {
        // Check if any target vowels already have the requested tone
        // This handles redundant tone keys like "u7o7" → "ươ" (second 7 absorbed)
        //
        // EXCEPTION: Don't absorb 'w' if last_transform was WAsVowel
        // because try_w_as_vowel needs to handle the revert (ww → w)
        let is_w_revert_pending =
            key == keys::W && matches!(e.last_transform, Some(Transform::WAsVowel));

        let has_tone_already = e.buf
            .iter()
            .any(|c| targets.contains(&c.key) && c.tone == tone_val);
        if has_tone_already && !is_w_revert_pending {
            // Absorb the key (no-op)
            return Some(Result::send(0, &[]));
        }
        return None;
    }

    // Track earliest position modified for rebuild
    let mut earliest_pos = usize::MAX;

    // If switching, clear old tones first for proper rebuild
    if is_switching {
        for &pos in &target_positions {
            if let Some(c) = e.buf.get_mut(pos) {
                c.tone = tone::NONE;
                earliest_pos = earliest_pos.min(pos);
            }
        }

        // Special case: switching from horn compound (ươ) to circumflex (uô)
        // When switching to circumflex on 'o', also clear horn from adjacent 'u'
        if tone_type == ToneType::Circumflex {
            for &pos in &target_positions {
                if let Some(c) = e.buf.get(pos) {
                    if c.key == keys::O {
                        // Check for adjacent 'u' with horn and clear it
                        if pos > 0 {
                            if let Some(prev) = e.buf.get_mut(pos - 1) {
                                if prev.key == keys::U && prev.tone == tone::HORN {
                                    prev.tone = tone::NONE;
                                    earliest_pos = earliest_pos.min(pos - 1);
                                }
                            }
                        }
                        if pos + 1 < e.buf.len() {
                            if let Some(next) = e.buf.get_mut(pos + 1) {
                                if next.key == keys::U && next.tone == tone::HORN {
                                    next.tone = tone::NONE;
                                    earliest_pos = earliest_pos.min(pos + 1);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Special case: switching from circumflex (uô) to horn compound (ươ)
        // For standalone uo compound (no final consonant), add horn to adjacent 'u'
        if tone_type == ToneType::Horn && mark_handler::has_uo_compound(e) {
            // Check if this is a standalone compound (o is last vowel, no final consonant)
            let has_final = target_positions.iter().any(|&pos| {
                pos + 1 < e.buf.len()
                    && e.buf
                        .get(pos + 1)
                        .is_some_and(|c| !keys::is_vowel(c.key))
            });

            if !has_final {
                for &pos in &target_positions {
                    if let Some(c) = e.buf.get(pos) {
                        if c.key == keys::O {
                            // Add horn to adjacent 'u' for compound
                            if pos > 0 {
                                if let Some(prev) = e.buf.get_mut(pos - 1) {
                                    if prev.key == keys::U && prev.tone == tone::NONE {
                                        prev.tone = tone::HORN;
                                        earliest_pos = earliest_pos.min(pos - 1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply new tone
    for &pos in &target_positions {
        if let Some(c) = e.buf.get_mut(pos) {
            c.tone = tone_val;
            earliest_pos = earliest_pos.min(pos);
        }
    }

    // Validate result: check for breve (ă) followed by vowel - NEVER valid in Vietnamese
    // Issue #44: "tai" + 'w' → "tăi" is INVALID (ăi, ăo, ău, ăy don't exist)
    // Only check this specific pattern, not all vowel patterns, to allow Telex shortcuts
    // like "eie" → "êi" which may not be standard but are expected Telex behavior
    // Note: ToneType::Horn (Telex 'w') and ToneType::Breve (VNI '8') both create breve on 'a'
    if tone_type == ToneType::Horn || tone_type == ToneType::Breve {
        // Early check: "W at end after vowel (not U)" with earlier Vietnamese transforms
        // suggests English word like "seesaw" where:
        // - Earlier chars were transformed (sê, sế)
        // - But "aw" ending makes it look like English
        // Only restore if buffer has EARLIER transforms (tone or mark)
        // Don't restore for simple "aw" or "raw" - let breve deferral handle those
        // Only run if english_auto_restore is enabled (experimental feature)
        if e.english_auto_restore && key == keys::W && e.raw_input.len() >= 2 {
            let (prev_key, _, _) = e.raw_input[e.raw_input.len() - 2];
            if prev_key == keys::A {
                // Check if there are earlier Vietnamese transforms in buffer
                // (tone marks on OTHER vowels, or circumflex/horn on non-A vowels)
                // IMPORTANT: Exclude positions we just modified in this call
                let has_earlier_transforms = e.buf.iter().enumerate().any(|(i, c)| {
                    // Skip positions we just applied horn to - those aren't "earlier" transforms
                    if target_positions.contains(&i) {
                        return false;
                    }
                    // Check for any tone (circumflex, horn) or mark on NON-A vowels
                    // A itself might just be plain "a" waiting for breve
                    c.key != keys::A && (c.tone > 0 || c.mark > 0)
                });

                if has_earlier_transforms {
                    // "aw" ending is English (like "seesaw") - restore immediately
                    let raw_chars: Vec<char> = e.raw_input
                        .iter()
                        .filter_map(|&(k, c, s)| utils::key_to_char_ext(k, c, s))
                        .collect();
                    let backspace = e.buf.len() as u8;
                    e.buf.clear();
                    e.raw_input.clear();
                    e.last_transform = None;
                    return Some(Result::send(backspace, &raw_chars));
                }
            }
        }
        let has_breve_vowel_pattern = target_positions.iter().any(|&pos| {
            if let Some(c) = e.buf.get(pos) {
                // Check if this is 'a' with horn (breve) followed by another vowel
                if c.key == keys::A {
                    // Look for any vowel after this position
                    return (pos + 1..e.buf.len()).any(|i| {
                        e.buf
                            .get(i)
                            .map(|next| keys::is_vowel(next.key))
                            .unwrap_or(false)
                    });
                }
            }
            false
        });

        if has_breve_vowel_pattern {
            // Revert: clear applied tones
            for &pos in &target_positions {
                if let Some(c) = e.buf.get_mut(pos) {
                    c.tone = tone::NONE;
                }
            }
            return None;
        }

        // Issue #44 (part 2): Always apply breve for "aw" pattern immediately
        // "aw" → "ă", "taw" → "tă", "raw" → "ră"
        // The breve is always applied - English auto-restore handles English words separately
        let has_breve_open_syllable = false;

        if has_breve_open_syllable {
            // Revert: clear applied tones, defer breve until final consonant
            for &pos in &target_positions {
                if let Some(c) = e.buf.get_mut(pos) {
                    if c.key == keys::A {
                        c.tone = tone::NONE;
                        // Store position for deferred breve
                        e.pending_breve_pos = Some(pos);
                    }
                }
            }
            // Return None to let 'w' fall through:
            // - try_w_as_vowel will fail (invalid vowel pattern)
            // - handle_normal_letter will add 'w' as regular letter
            // - When final consonant is typed, breve is applied
            return None;
        }
    }

    // Normalize ưo → ươ compound if horn was applied to 'u'
    if let Some(compound_pos) = mark_handler::normalize_uo_compound(e) {
        earliest_pos = earliest_pos.min(compound_pos);
    }

    e.last_transform = Some(Transform::Tone(key, tone_val));
    e.had_any_transform = true;
    e.had_telex_transform = true; // Track for whitelist-based auto-restore

    // Reposition tone mark if vowel pattern changed
    let mut rebuild_pos = earliest_pos;
    if let Some((old_pos, _)) = tone_placement::reposition_tone_if_needed(e) {
        rebuild_pos = rebuild_pos.min(old_pos);
    }

    Some(helpers::rebuild_from(&e.buf, rebuild_pos))
}
