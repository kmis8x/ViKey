use super::{auto_restore, Engine, letter_handler, mark_handler, revert, stroke_handler, tone_handler};
use crate::data::keys;
use crate::engine::{buffer::Char, types::{Action, Result, Transform}};
use crate::engine::validation::is_valid;
use crate::{input, utils};
use super::helpers::{break_key_to_char, is_sentence_ending_punctuation, should_reset_pending_capitalize};

pub fn on_key_ext(e: &mut Engine, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result {
    // Issue #129: Process shortcuts even when IME is disabled
    // Only bypass completely for Ctrl/Cmd modifier keys
    if ctrl {
        e.clear();
        e.word_history.clear();
        e.spaces_after_commit = 0;
        return Result::none();
    }

    // When IME is disabled, process shortcuts but skip Vietnamese transforms
    // This allows both word shortcuts (btw → by the way) and symbol shortcuts (-> → →)
    if !e.enabled {
        // Clear Vietnamese state
        e.buf.clear();
        e.raw_input.clear();
        e.word_history.clear();
        e.spaces_after_commit = 0;

        // Word boundary keys (Space, Enter): check for word shortcuts
        if key == keys::SPACE || key == keys::RETURN || key == keys::ENTER {
            if e.shortcuts_enabled && !e.shortcut_prefix.is_empty() {
                let input_method = e.current_input_method();
                if let Some(m) = e.shortcuts.try_match_for_method(
                    &e.shortcut_prefix,
                    None,
                    true, // is_word_boundary = true for word shortcuts
                    input_method,
                ) {
                    let output: Vec<char> = m.output.chars().collect();
                    let backspace_count = m.backspace_count as u8;
                    e.shortcut_prefix.clear();
                    // For Space, include space in output; for Enter, don't
                    if key == keys::SPACE {
                        let mut output_with_space = output;
                        output_with_space.push(' ');
                        return Result::send(backspace_count, &output_with_space);
                    } else {
                        return Result::send(backspace_count, &output);
                    }
                }
            }
            e.shortcut_prefix.clear();
            return Result::none();
        }

        // Break keys (punctuation): check for immediate shortcuts like "->"
        if keys::is_break_ext(key, shift) {
            if let Some(ch) = break_key_to_char(key, shift) {
                e.shortcut_prefix.push(ch);

                if e.shortcuts_enabled {
                    let input_method = e.current_input_method();
                    if let Some(m) = e.shortcuts.try_match_for_method(
                        &e.shortcut_prefix,
                        None,
                        false,
                        input_method,
                    ) {
                        let output: Vec<char> = m.output.chars().collect();
                        let backspace_count = (m.backspace_count as u8).saturating_sub(1);
                        e.shortcut_prefix.clear();
                        return Result::send_consumed(backspace_count, &output);
                    }
                }
                return Result::none();
            }
            // Break key without char mapping (Tab, arrows, etc.) - clear and pass through
            e.shortcut_prefix.clear();
            return Result::none();
        }

        // Letter and number keys: accumulate for word shortcuts (e.g., "btw", "f1", "a1")
        if let Some(ch) = utils::key_to_char(key, caps) {
            e.shortcut_prefix.push(ch);
            return Result::none();
        }

        // Unknown keys: clear shortcut prefix and pass through
        e.shortcut_prefix.clear();
        return Result::none();
    }

    // Check for word boundary shortcuts ONLY on SPACE
    // Also auto-restore invalid Vietnamese to raw English
    if key == keys::SPACE {
        // Handle pending mark revert pop on space (end of word)
        // When telex_double_raw is set, we use it directly for restore, no pop needed.
        // The telex_double_raw contains the exact original input before any modification.
        // Examples:
        //   "nurses" → telex_double_raw="nurses", use directly for restore
        //   "simss" → telex_double_raw="simss", use directly for restore (ss→sims via whitelist)
        //   "taxxi" → telex_double_raw="taxx", buffer "taxi" kept (clean, no marks)
        if e.pending_mark_revert_pop {
            e.pending_mark_revert_pop = false;
            // telex_double_raw is always set when pending_mark_revert_pop is true
            // (both set in revert_mark). Don't modify raw_input here - use
            // telex_double_raw for restore which has the correct original chars.
        }

        // First check for shortcut
        let shortcut_result = try_word_boundary_shortcut(e);
        if shortcut_result.action != 0 {
            e.clear();
            return shortcut_result;
        }

        // Auto-restore: if buffer has transforms but is invalid Vietnamese,
        // restore to raw English (like ESC but triggered by space)
        let restore_result = auto_restore::try_auto_restore_on_space(e);

        // If auto-restore happened, repopulate buffer with plain chars from raw_input
        // This ensures word_history stores the correct restored word (not transformed)
        // Example: "restore" → buffer was "rếtore" (6 chars), raw_input has 7 keys
        // After this, buffer has "restore" (7 chars) for correct history
        if restore_result.action != 0 {
            e.buf.clear();
            for &(key, caps, _) in &e.raw_input {
                e.buf.push(Char::new(key, caps));
            }
        }

        // Push buffer to history before clearing (for backspace-after-space feature)
        if !e.buf.is_empty() {
            e.word_history.push(e.buf.clone());
            e.spaces_after_commit = 1; // First space after word
        } else if e.spaces_after_commit > 0 {
            // Additional space after commit - increment counter
            e.spaces_after_commit = e.spaces_after_commit.saturating_add(1);
        }
        e.auto_capitalize_used = false; // Reset on word commit

        // Issue #185: Set pending_capitalize on space AFTER sentence-ending punctuation
        // This ensures "google.com" doesn't capitalize, but "ok. ban" does
        if e.auto_capitalize && e.saw_sentence_ending {
            e.pending_capitalize = true;
            // Keep saw_sentence_ending for multiple spaces (e.g., "ok.  ban")
        }

        e.clear();
        return restore_result;
    }

    // ESC key: restore to raw ASCII (undo all Vietnamese transforms)
    // Only if esc_restore is enabled by user
    if key == keys::ESC {
        let result = if e.esc_restore_enabled {
            auto_restore::restore_to_raw(e)
        } else {
            Result::none()
        };
        e.clear();
        e.word_history.clear();
        e.spaces_after_commit = 0;
        return result;
    }

    // Issue #159: In Telex mode, `]` → ư and `[` → ơ
    // caps affects revert: ]] → ], uppercase (Shift/CapsLock) → }
    if e.method == 0 && (key == keys::RBRACKET || key == keys::LBRACKET) {
        if let Some(result) = stroke_handler::try_bracket_as_vowel(e, key, caps) {
            return result;
        }
    }

    // Other break keys (punctuation, arrows, etc.)
    // Also trigger auto-restore for invalid Vietnamese before clearing
    // Use is_break_ext to handle shifted symbols like @, !, #, etc.
    if keys::is_break_ext(key, shift) {
        // Issue #107 + Bug #11: When buffer is empty AND we're at true start of input
        // (no word history), accumulate break chars for shortcuts.
        // This allows shortcuts like "#fne", "->", "=>" to work.
        // BUT: if there's word history (user just typed "du "), break chars should
        // clear history as before, not accumulate.
        let at_true_start =
            e.buf.is_empty() && e.word_history.len == 0 && e.spaces_after_commit == 0;

        // Also continue accumulating if we already started a prefix
        let continuing_prefix = e.buf.is_empty() && !e.shortcut_prefix.is_empty();

        if at_true_start || continuing_prefix {
            // Reset has_non_letter_prefix when starting a new shortcut at true start
            // This ensures shortcuts like "->" work after DELETE cleared the buffer
            if at_true_start {
                e.has_non_letter_prefix = false;
            }

            // Try to get the character for this break key
            if let Some(ch) = break_key_to_char(key, shift) {
                e.shortcut_prefix.push(ch);

                // Check for immediate shortcut match
                if e.shortcuts_enabled {
                    let input_method = e.current_input_method();
                    if let Some(m) = e.shortcuts.try_match_for_method(
                        &e.shortcut_prefix,
                        None,
                        false,
                        input_method,
                    ) {
                        // Found a match! Send the replacement with key_consumed flag
                        // Note: backspace_count - 1 because current key hasn't been typed yet
                        // Example: "->" trigger has backspace_count=2, but only '-' is on screen
                        let output: Vec<char> = m.output.chars().collect();
                        let backspace_count = (m.backspace_count as u8).saturating_sub(1);
                        e.shortcut_prefix.clear();
                        return Result::send_consumed(backspace_count, &output);
                    }
                }

                // Issue #185: Only set saw_sentence_ending for punctuation (not Enter)
                // pending_capitalize will be set when space follows
                if e.auto_capitalize && is_sentence_ending_punctuation(key, shift) {
                    e.saw_sentence_ending = true;
                } else if e.auto_capitalize && (key == keys::RETURN || key == keys::ENTER) {
                    // Enter = newline = immediate capitalize (no space needed)
                    e.pending_capitalize = true;
                    e.saw_sentence_ending = false;
                }
                return Result::none(); // Let the char pass through, keep accumulating
            }
        }

        // Issue #185: Only set saw_sentence_ending for punctuation (not Enter)
        // pending_capitalize will be set when space follows
        if e.auto_capitalize && is_sentence_ending_punctuation(key, shift) {
            e.saw_sentence_ending = true;
        } else if e.auto_capitalize && (key == keys::RETURN || key == keys::ENTER) {
            // Enter = newline = immediate capitalize (no space needed)
            e.pending_capitalize = true;
            e.saw_sentence_ending = false;
        } else if e.auto_capitalize && should_reset_pending_capitalize(key, shift) {
            // Reset pending for word-breaking keys (comma, semicolon, etc.)
            // But preserve pending for neutral keys (quotes, parentheses, brackets)
            e.pending_capitalize = false;
            e.saw_sentence_ending = false;
        }
        e.auto_capitalize_used = false; // Reset on word boundary

        // Issue #167: Check for word boundary shortcuts on punctuation and ENTER
        // Example: "ko." → "không." or "ko<Enter>" → "không<Enter>"
        // ENTER doesn't have a printable char, so check it separately
        let trigger_char = if key == keys::RETURN || key == keys::ENTER {
            Some('\n') // ENTER: use newline as trigger (won't be appended)
        } else {
            break_key_to_char(key, shift)
        };
        if let Some(ch) = trigger_char {
            let shortcut_result = try_word_boundary_shortcut_with_char(e, ch);
            if shortcut_result.action != 0 {
                e.clear();
                e.word_history.clear();
                e.spaces_after_commit = 0;
                return shortcut_result;
            }
        }

        let restore_result = auto_restore::try_auto_restore_on_break(e);
        e.clear();
        e.word_history.clear();
        e.spaces_after_commit = 0;

        // Issue #130: After clearing buffer, store break char as potential shortcut prefix
        // This allows shortcuts like "->" to work after "abc->" (where "-" clears "abc")
        // Example: type "→abc->" should produce "→abc→"
        if let Some(ch) = break_key_to_char(key, shift) {
            e.shortcut_prefix.push(ch);
        }

        return restore_result;
    }

    if key == keys::DELETE {
        // Backspace-after-space feature: restore previous word when all spaces deleted
        // Track spaces typed after commit, restore word when counter reaches 0
        if e.spaces_after_commit > 0 && e.buf.is_empty() {
            e.spaces_after_commit -= 1;
            if e.spaces_after_commit == 0 {
                // All spaces deleted - restore the word buffer
                if let Some(restored_buf) = e.word_history.pop() {
                    // Restore raw_input from buffer (for ESC restore to work)
                    e.restore_raw_input_from_buffer(&restored_buf);
                    e.buf = restored_buf;
                    // Mark that buffer was restored - if user types new letter,
                    // clear buffer first (they want fresh word, not append)
                    e.restored_pending_clear = true;
                }
            }
            // Delete one space
            return Result::send(1, &[]);
        }
        // DON'T reset spaces_after_commit here!
        // User might delete all new input and want to restore previous word.
        // Reset only happens on: break keys, ESC, ctrl, or new commit.

        // If buffer is already empty, user is deleting content from previous word
        // that we don't track. Mark this to prevent false shortcut matches.
        // e.g., "đa" + SPACE + backspace×2 + "a" should NOT match shortcut "a"
        if e.buf.is_empty() {
            e.has_non_letter_prefix = true;
        }
        e.buf.pop();
        e.raw_input.pop();
        e.last_transform = None;
        // Reset stroke_reverted on backspace so user can re-trigger stroke
        // e.g., "ddddd" → "dddd", then backspace×3 → "d", then "d" → "đ"
        e.stroke_reverted = false;
        // Issue #217: Reset reverted_circumflex_key on backspace so user can re-trigger circumflex
        // e.g., "eee" → "ee", then backspace×2 → "", type "phe" → "phê" (not "phee")
        e.reverted_circumflex_key = None;
        // Only reset restored_pending_clear when buffer is empty
        // (user finished deleting restored word completely)
        // If buffer still has chars, user might think they cleared everything
        // but actually didn't - let them start fresh on next letter input
        if e.buf.is_empty() {
            e.restored_pending_clear = false;
            // Restore pending_capitalize if user deleted the auto-capitalized letter
            // This allows: ". B" → delete B → ". " → type again → auto-capitalizes
            if e.auto_capitalize_used {
                e.pending_capitalize = true;
                e.auto_capitalize_used = false;
            }
        }
        return Result::none();
    }

    // After DELETE restore, determine if user wants to:
    // 1. Continue editing restored word (add tone/mark) - mark keys, tone keys
    // 2. Start fresh word - regular letters (not mark/tone keys)
    // This allows "cha" + restore + "f" → "chà" (f is mark key)
    // But "cha" + restore + "m" → "m..." (m is consonant, start fresh)
    // For pure ASCII restored words (like "shortcuts"), also clear on vowels
    // unless they're mark/tone keys (allow "ban" + restore + "s" → "bán")
    if e.restored_pending_clear && keys::is_letter(key) {
        let m = input::get(e.method);
        let is_mark_or_tone = m.mark(key).is_some() || m.tone(key).is_some();
        // Clear buffer when letter is NOT a mark/tone modifier:
        // - Vietnamese restored: clear on consonant (vowels may add diacritics)
        // - ASCII restored: clear on any non-mark/tone letter (consonant OR vowel)
        let should_clear = if e.restored_is_ascii {
            // Pure ASCII: clear on any letter except mark/tone keys
            !is_mark_or_tone
        } else {
            // Vietnamese: clear only on consonant that's not mark/tone
            keys::is_consonant(key) && !is_mark_or_tone
        };
        if should_clear {
            e.clear();
        }
        // Reset flags regardless - user is now actively typing
        e.restored_pending_clear = false;
        e.restored_is_ascii = false;
    }

    // Issue #212: Reset has_non_letter_prefix when user starts typing letter into empty buffer
    // This allows shortcuts to work after: expand → delete all → retype
    // e.g., "ko" → "không " → backspace×6 → "ko" → should expand again
    if e.buf.is_empty() && keys::is_letter(key) && e.has_non_letter_prefix {
        e.has_non_letter_prefix = false;
    }

    // Auto-capitalize: force uppercase for first letter after sentence-ending punctuation
    let was_auto_capitalized = e.pending_capitalize && keys::is_letter(key) && !caps;
    let effective_caps = if e.pending_capitalize && keys::is_letter(key) {
        e.pending_capitalize = false;
        e.saw_sentence_ending = false; // Reset after capitalizing
        e.auto_capitalize_used = true; // Track that we used auto-capitalize
        true // Force uppercase
    } else {
        // Reset pending on number (e.g., "1.5" should not capitalize "5")
        if e.pending_capitalize && keys::is_number(key) {
            e.pending_capitalize = false;
            e.saw_sentence_ending = false;
            e.auto_capitalize_used = false; // Number after punctuation, reset
        }
        // Issue #185: Reset saw_sentence_ending when letter is typed without space
        // e.g., "google.com" - 'c' typed after '.' without space, don't capitalize
        if e.saw_sentence_ending && keys::is_letter(key) {
            e.saw_sentence_ending = false;
        }
        caps
    };

    // Record raw keystroke for ESC restore (letters and numbers only)
    if keys::is_letter(key) || keys::is_number(key) {
        e.raw_input.push((key, effective_caps, shift));
    }

    let result = process(e, key, effective_caps, shift);

    // If auto-capitalize triggered for first letter of a new word and process returned none,
    // we need to send the uppercase character since the original key was lowercase
    if was_auto_capitalized && result.action == Action::None as u8 && e.buf.len() == 1 {
        if let Some(ch) = crate::utils::key_to_char(key, true) {
            return Result::send(0, &[ch]);
        }
    }

    result
}

pub(super) fn process(e: &mut Engine, key: u16, caps: bool, shift: bool) -> Result {
    let m = input::get(e.method);

    // Handle pending mark revert pop: if previous key was a mark revert,
    // reset the flag. When telex_double_raw is set, we use it directly for
    // restore, so no need to modify raw_input here.
    // For vowel (issue) vs consonant (test) patterns, the whitelist and
    // restore logic will handle them correctly using telex_double_raw.
    if e.pending_mark_revert_pop && keys::is_letter(key) {
        e.pending_mark_revert_pop = false;
        // telex_double_raw is always set when pending_mark_revert_pop is true
        // (both set in revert_mark). Don't modify raw_input here.
    }

    // Revert short-pattern stroke when new letter creates invalid Vietnamese
    // This handles: "ded" → "đe" (stroke applied), then 'i' → "dedi" (invalid, revert)
    // IMPORTANT: This check must happen BEFORE any modifiers (tone, mark, etc.)
    // because the modifier key (like 'e' for circumflex) would transform the
    // buffer before we can check validity.
    //
    // We check validity using raw_input (not e.buf) because:
    // - e.buf = [đ, e] after stroke (2 chars)
    // - raw_input = [d, e, d, e] with new 'e' (4 chars - the actual full input)
    // Checking [D, E, D, E] correctly identifies "dede" as invalid.
    //
    // Skip revert for:
    // - Mark keys (s, f, r, x, j) - confirm Vietnamese intent
    // - Tone keys (a, e, o, w) that can apply to buffer - allows fast typing
    //   e.g., "dod" → "đo" + 'o' → "đô" (user typed d-o-d-o fast, intended "ddoo")
    // - Stroke keys ('d') - handled separately in try_stroke for proper revert behavior
    //   e.g., "dadd" → "dad" (d reverts stroke and adds itself, not "dadd")
    let is_mark_key = m.mark(key).is_some();
    let is_tone_key = m.tone(key).is_some();
    let is_stroke_key = m.stroke(key);

    if keys::is_letter(key)
        && !is_mark_key
        && !is_tone_key
        && !is_stroke_key
        && matches!(e.last_transform, Some(Transform::ShortPatternStroke))
    {
        // Build buffer_keys from raw_input (which already includes current key)
        let raw_keys: Vec<u16> = e.raw_input.iter().map(|&(k, _, _)| k).collect();

        // Also check if the buffer (with stroke) + new key would be valid Vietnamese
        // This handles delayed stroke patterns like "dadu" → "đau":
        // - raw_input = [d, a, d, u] (invalid as "dadu")
        // - But buffer + key = [đ, a] + [u] = "đau" (valid)
        // If buffer + key is valid, don't revert the stroke
        let mut buf_keys: Vec<u16> = e.buf.iter().map(|c| c.key).collect();
        buf_keys.push(key);

        if !is_valid(&raw_keys) && !is_valid(&buf_keys) {
            // Invalid pattern - revert stroke and rebuild from raw_input
            if let Some(raw_chars) = auto_restore::build_raw_chars(e) {
                // Calculate backspace: screen shows buffer content (e.g., "đe")
                let backspace = e.buf.len() as u8;

                // Rebuild buffer from raw_input (plain chars, no stroke)
                e.buf.clear();
                for &(k, c, _) in &e.raw_input {
                    e.buf.push(Char::new(k, c));
                }
                e.last_transform = None;

                return Result::send(backspace, &raw_chars);
            }
        }
    }

    // In VNI mode, if Shift is pressed with a number key, skip all modifiers
    // User wants the symbol (@ for Shift+2, # for Shift+3, etc.), not VNI marks
    let skip_vni_modifiers = e.method == 1 && shift && keys::is_number(key);

    // Check modifiers by scanning buffer for patterns

    // 1. Stroke modifier (d → đ)
    if !skip_vni_modifiers && m.stroke(key) {
        if let Some(result) = stroke_handler::try_stroke(e, key, caps) {
            return result;
        }
    }

    // 2. Tone modifier (circumflex, horn, breve)
    if !skip_vni_modifiers {
        if let Some(tone_type) = m.tone(key) {
            let targets = m.tone_targets(key);
            if let Some(result) = tone_handler::try_tone(e, key, caps, tone_type, targets) {
                return result;
            }
        }
    }

    // 3. Mark modifier
    if !skip_vni_modifiers {
        if let Some(mark_val) = m.mark(key) {
            if let Some(result) = mark_handler::try_mark(e, key, caps, mark_val) {
                return result;
            }
        }
    }

    // 4. Remove modifier
    // Only consume key if there's something to remove; otherwise fall through to normal letter
    // This allows shortcuts like "zz" to work when buffer has no marks/tones to remove
    if !skip_vni_modifiers && m.remove(key) {
        if let Some(result) = revert::try_remove(e) {
            return result;
        }
    }

    // 5. In Telex: "w" as vowel "ư" when valid Vietnamese context
    // Examples: "w" → "ư", "nhw" → "như", but "kw" → "kw" (invalid)
    if e.method == 0 && key == keys::W {
        if let Some(result) = stroke_handler::try_w_as_vowel(e, caps) {
            return result;
        }
    }

    // Not a modifier - normal letter
    letter_handler::handle_normal_letter(e, key, caps)
}

pub(super) fn try_word_boundary_shortcut_with_char(e: &mut Engine, _trigger_char: char) -> Result {
    // Skip if shortcuts are disabled
    if !e.shortcuts_enabled {
        return Result::none();
    }

    // Issue #107: Allow shortcuts with special char prefix (like "#fne")
    // If shortcut_prefix is set, we still try to match even with empty buffer
    if e.buf.is_empty() && e.shortcut_prefix.is_empty() {
        return Result::none();
    }

    // Don't trigger shortcut if word has non-letter prefix (like "149k")
    // But DO allow shortcut_prefix (like "#fne") - that's intentional
    if e.has_non_letter_prefix {
        return Result::none();
    }

    // Build full trigger string including shortcut_prefix if present
    let full_trigger = if e.shortcut_prefix.is_empty() {
        e.buf.to_full_string()
    } else {
        format!("{}{}", e.shortcut_prefix, e.buf.to_full_string())
    };

    let input_method = e.current_input_method();

    // Check for word boundary shortcut match
    // For SPACE: append to output so it's sent with the replacement text
    // For punctuation: pass None - let platform type it normally
    let key_char = if _trigger_char == ' ' {
        Some(' ')
    } else {
        None
    };
    if let Some(m) =
        e.shortcuts
            .try_match_for_method(&full_trigger, key_char, true, input_method)
    {
        let output: Vec<char> = m.output.chars().collect();
        // backspace_count = trigger.len() which already includes prefix (e.g., "#fne" = 4)
        return Result::send(m.backspace_count as u8, &output);
    }

    Result::none()
}

pub(super) fn try_word_boundary_shortcut(e: &mut Engine) -> Result {
    try_word_boundary_shortcut_with_char(e, ' ')
}
