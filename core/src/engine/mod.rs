//! Vietnamese IME Engine
//!
//! Core engine for Vietnamese input method processing.
//! Uses pattern-based transformation with validation-first approach.
//!
//! Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey
//!
//! ## Architecture
//!
//! 1. **Validation First**: Check if buffer is valid Vietnamese before transforming
//! 2. **Pattern-Based**: Scan entire buffer for patterns instead of case-by-case
//! 3. **Shortcut Support**: User-defined abbreviations with priority
//! 4. **Longest-Match-First**: For diacritic placement

pub mod buffer;
pub mod shortcut;
pub mod syllable;
pub mod transform;
pub mod validation;

mod types;
mod helpers;
mod key_handler;
mod tone_handler;
mod tone_placement;
mod mark_handler;
mod stroke_handler;
mod revert;
mod letter_handler;
mod auto_restore;
mod english_pattern;
mod raw_input;
#[cfg(test)]
mod tests;

use types::Transform;
pub use types::{Action, Result};
use helpers::WordHistory;

use crate::data::{
    chars,
    keys,
    vowel::Vowel,
};
use crate::input::ToneType;
use crate::utils;
use buffer::{Buffer, Char};
use shortcut::{InputMethod, ShortcutTable};

/// Main Vietnamese IME engine
pub struct Engine {
    pub(super) buf: Buffer,
    pub(super) method: u8,
    pub(super) enabled: bool,
    pub(super) last_transform: Option<Transform>,
    pub(super) shortcuts: ShortcutTable,
    /// Raw keystroke history for ESC restore (key, caps, shift)
    pub(super) raw_input: Vec<(u16, bool, bool)>,
    /// True if current word has non-letter characters before letters
    /// Used to prevent false shortcut matches (e.g., "149k" should not match "k")
    pub(super) has_non_letter_prefix: bool,
    /// Skip w→ư shortcut in Telex mode (user preference)
    /// When true, typing 'w' at word start stays as 'w' instead of converting to 'ư'
    pub(super) skip_w_shortcut: bool,
    /// Enable bracket shortcuts: ] → ư, [ → ơ (Issue #159)
    pub(super) bracket_shortcut: bool,
    /// Enable ESC key to restore raw ASCII (undo Vietnamese transforms)
    /// When false, ESC key is passed through without restoration
    pub(super) esc_restore_enabled: bool,
    /// Enable free tone placement (skip validation)
    /// When true, allows placing diacritics anywhere without spelling validation
    pub(super) free_tone_enabled: bool,
    /// Use modern orthography for tone placement (hoà vs hòa)
    /// When true: oà, uý (tone on second vowel)
    /// When false: òa, úy (tone on first vowel - traditional)
    pub(super) modern_tone: bool,
    /// Enable English auto-restore (experimental)
    /// When true, automatically restores English words that were transformed
    /// e.g., "tẽt" → "text", "ễpct" → "expect"
    pub(super) english_auto_restore: bool,
    /// Word history for backspace-after-space feature
    pub(super) word_history: WordHistory,
    /// Number of spaces typed after committing a word (for backspace tracking)
    /// When this reaches 0 on backspace, we restore the committed word
    pub(super) spaces_after_commit: u8,
    /// Pending breve position: position of 'a' that has deferred breve
    /// Breve on 'a' in open syllables (like "raw") is invalid Vietnamese
    /// We defer applying breve until a valid final consonant is typed
    pub(super) pending_breve_pos: Option<usize>,
    /// Issue #133: Pending horn position on 'u' in "uơ" pattern
    /// When "uo" + 'w' is typed at end of syllable, only 'o' gets horn initially.
    /// If a final consonant/vowel is added, also apply horn to 'u'.
    /// Examples: "huow" → "huơ" (stays), "duow" + "c" → "dược" (u gets horn)
    pub(super) pending_u_horn_pos: Option<usize>,
    /// Tracks if stroke was reverted in current word (ddd → dd)
    /// When true, subsequent 'd' keys are treated as normal letters, not stroke triggers
    /// This prevents "ddddd" from oscillating between đ and dd states
    pub(super) stroke_reverted: bool,
    /// Tracks if a mark was reverted in current word
    /// Used by auto-restore to detect words like "issue", "bass" that need restoration
    pub(super) had_mark_revert: bool,
    /// Pending pop from raw_input after mark revert
    /// When true, the NEXT consonant key will trigger a pop to remove the consumed modifier
    /// This differentiates: "tesst" → "test" (consonant after) vs "issue" → "issue" (vowel after)
    pub(super) pending_mark_revert_pop: bool,
    /// Tracks if ANY Vietnamese transform was ever applied during this word
    /// (marks, tones, or stroke). Used to prevent false auto-restore for words
    /// with numbers/symbols that never had Vietnamese transforms applied.
    /// Example: "nhatkha1407@gmail.com" has no transforms, so shouldn't restore.
    pub(super) had_any_transform: bool,
    /// Tracks if circumflex was applied from V+C+V pattern by vowel trigger (not mark key)
    /// Example: "toto" → "tôt" (second 'o' triggers circumflex on first 'o')
    /// Used for auto-restore: if no mark follows, restore on space (e.g., "toto " → "toto ")
    pub(super) had_vowel_triggered_circumflex: bool,
    /// Tracks if circumflex was REVERTED by third vowel (aa→â, aaa→aa)
    /// Example: "dataa" → "dât" (after 4th key), typing 5th 'a' reverts to "data"
    /// Used in build_raw_chars to collapse double vowel at end for restore
    pub(super) had_circumflex_revert: bool,
    /// Issue #211: Tracks which vowel key triggered circumflex revert (extended vowel mode)
    /// When set, subsequent same-key vowels append raw instead of re-transforming
    /// Example: aaa→aa (reverted_circumflex_key=A), aaaa→aaa (skip transform, append raw)
    pub(super) reverted_circumflex_key: Option<u16>,
    /// Tracks if ANY Telex transform was applied (tone, mark, or stroke)
    /// Used for whitelist-based auto-restore to English words
    pub(super) had_telex_transform: bool,
    /// Stores raw_input string when telex double pattern is detected (BEFORE modification)
    /// For stroke revert (ddd→dd), raw_input is modified to remove one 'd', but we need
    /// the original for whitelist lookup (e.g., "daddy" not "dady")
    pub(super) telex_double_raw: Option<String>,
    /// Stores length of raw_input at time telex_double_raw was stored
    /// Used to append subsequent chars typed after revert
    pub(super) telex_double_raw_len: usize,
    /// Issue #107: Special character prefix for shortcut matching
    /// When a shifted symbol (like #, @, $) is typed first, store it here
    /// so shortcuts like "#fne" can match even though # is normally a break char
    /// Extended: Now accumulates multiple break chars for shortcuts like "->" → "→"
    pub(super) shortcut_prefix: String,
    /// Buffer was just restored from DELETE - clear on next letter input
    /// This prevents typing after restore from appending to old buffer
    pub(super) restored_pending_clear: bool,
    /// Restored word was pure ASCII (no Vietnamese chars) - clear on ANY letter
    /// For Vietnamese restored words, only clear on consonant (allow mark/tone edits)
    pub(super) restored_is_ascii: bool,
    /// Auto-capitalize first letter after sentence-ending punctuation
    /// Triggers: . ! ? Enter → next letter becomes uppercase
    pub(super) auto_capitalize: bool,
    /// Pending capitalize state: set after sentence-ending punctuation + space
    pub(super) pending_capitalize: bool,
    /// Tracks if auto-capitalize was just used on the current word
    /// Used to restore pending_capitalize when user deletes the capitalized letter
    pub(super) auto_capitalize_used: bool,
    /// Tracks if we just saw sentence-ending punctuation (. ! ?)
    /// Only set pending_capitalize when space/Enter follows
    /// Issue #185: don't capitalize immediately after punctuation (e.g., google.com)
    pub(super) saw_sentence_ending: bool,
    /// Allow foreign consonants (z, w, j, f) as valid initial consonants
    /// When true, these letters are accepted as Vietnamese consonants for loanwords
    pub(super) allow_foreign_consonants: bool,
    /// Enable/disable shortcut expansion
    /// When false, shortcuts are not triggered
    pub(super) shortcuts_enabled: bool,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl Engine {
    pub fn new() -> Self {
        Self {
            buf: Buffer::new(),
            method: 0,
            enabled: true,
            last_transform: None,
            shortcuts: ShortcutTable::with_defaults(),
            raw_input: Vec::with_capacity(64),
            has_non_letter_prefix: false,
            skip_w_shortcut: false,
            bracket_shortcut: false,    // Default: OFF (Issue #159)
            esc_restore_enabled: false, // Default: OFF (user request)
            free_tone_enabled: false,
            modern_tone: true,           // Default: modern style (hoà, thuý)
            english_auto_restore: false, // Default: OFF (experimental feature)
            word_history: WordHistory::new(),
            spaces_after_commit: 0,
            pending_breve_pos: None,
            pending_u_horn_pos: None,
            stroke_reverted: false,
            had_mark_revert: false,
            pending_mark_revert_pop: false,
            had_any_transform: false,
            had_vowel_triggered_circumflex: false,
            had_circumflex_revert: false,
            reverted_circumflex_key: None,
            had_telex_transform: false,
            telex_double_raw: None,
            telex_double_raw_len: 0,
            shortcut_prefix: String::new(),
            restored_pending_clear: false,
            restored_is_ascii: false,
            auto_capitalize: false, // Default: OFF
            pending_capitalize: false,
            auto_capitalize_used: false,
            saw_sentence_ending: false,
            allow_foreign_consonants: false, // Default: OFF
            shortcuts_enabled: true, // Default: ON
        }
    }

    pub fn set_method(&mut self, method: u8) {
        self.method = method;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.buf.clear();
            self.word_history.clear();
            self.spaces_after_commit = 0;
        }
    }

    /// Set whether to skip w→ư shortcut in Telex mode
    pub fn set_skip_w_shortcut(&mut self, skip: bool) {
        self.skip_w_shortcut = skip;
    }

    /// Set whether bracket shortcuts are enabled: ] → ư, [ → ơ (Issue #159)
    pub fn set_bracket_shortcut(&mut self, enabled: bool) {
        self.bracket_shortcut = enabled;
    }

    /// Set whether ESC key restores raw ASCII
    pub fn set_esc_restore(&mut self, enabled: bool) {
        self.esc_restore_enabled = enabled;
    }

    /// Set whether to enable free tone placement (skip validation)
    pub fn set_free_tone(&mut self, enabled: bool) {
        self.free_tone_enabled = enabled;
    }

    /// Set whether to use modern orthography for tone placement
    pub fn set_modern_tone(&mut self, modern: bool) {
        self.modern_tone = modern;
    }

    /// Set whether to enable English auto-restore (experimental)
    pub fn set_english_auto_restore(&mut self, enabled: bool) {
        self.english_auto_restore = enabled;
    }

    /// Set whether to enable auto-capitalize after sentence-ending punctuation
    pub fn set_auto_capitalize(&mut self, enabled: bool) {
        self.auto_capitalize = enabled;
        if !enabled {
            self.pending_capitalize = false;
            self.saw_sentence_ending = false;
        }
    }

    /// Set whether to allow foreign consonants (z, w, j, f) as valid initials
    pub fn set_allow_foreign_consonants(&mut self, enabled: bool) {
        self.allow_foreign_consonants = enabled;
    }

    /// Get whether foreign consonants are allowed
    pub fn allow_foreign_consonants(&self) -> bool {
        self.allow_foreign_consonants
    }

    /// Set whether shortcuts are enabled
    pub fn set_shortcuts_enabled(&mut self, enabled: bool) {
        self.shortcuts_enabled = enabled;
    }

    /// Get whether shortcuts are enabled
    pub fn shortcuts_enabled(&self) -> bool {
        self.shortcuts_enabled
    }

    pub fn shortcuts(&self) -> &ShortcutTable {
        &self.shortcuts
    }

    pub fn shortcuts_mut(&mut self) -> &mut ShortcutTable {
        &mut self.shortcuts
    }

    /// Debug: get buffer length
    pub fn debug_buffer_len(&self) -> usize {
        self.buf.len()
    }

    /// Debug: get raw_input length (alias for raw_input_len)
    pub fn debug_raw_input_len(&self) -> usize {
        self.raw_input.len()
    }

    /// Debug: check had_any_transform flag
    pub fn debug_had_any_transform(&self) -> bool {
        self.had_any_transform
    }

    /// Debug: get buffer content as string
    pub fn debug_buffer_string(&self) -> String {
        self.buf.to_full_string()
    }

    /// Debug: dump full buffer state
    pub fn debug_buffer_state(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.buf.iter().enumerate() {
            result.push_str(&format!(
                "[{}] key={} tone={} mark={} stroke={}\n",
                i, c.key, c.tone, c.mark, c.stroke
            ));
        }
        result
    }

    /// Debug: check had_mark_revert flag
    pub fn debug_had_mark_revert(&self) -> bool {
        self.had_mark_revert
    }

    /// Debug: dump raw_input
    pub fn debug_raw_input(&self) -> String {
        self.raw_input
            .iter()
            .map(|(k, c, s)| format!("({},{},{})", k, c, s))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get current input method as InputMethod enum
    fn current_input_method(&self) -> InputMethod {
        match self.method {
            0 => InputMethod::Telex,
            1 => InputMethod::Vni,
            _ => InputMethod::All,
        }
    }

    /// Handle key event - main entry point
    ///
    /// # Arguments
    /// * `key` - macOS virtual keycode
    /// * `caps` - true if Caps Lock is active (for uppercase letters)
    /// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
    pub fn on_key(&mut self, key: u16, caps: bool, ctrl: bool) -> Result {
        self.on_key_ext(key, caps, ctrl, false)
    }

    /// Check if key+shift combo is a raw mode prefix character
    /// Raw prefixes: @ # : /
    #[allow(dead_code)] // TEMP DISABLED
    fn is_raw_prefix(key: u16, shift: bool) -> bool {
        // / doesn't need shift
        if key == keys::SLASH && !shift {
            return true;
        }
        // @ # : need shift
        if !shift {
            return false;
        }
        matches!(
            key,
            keys::N2              // @ = Shift+2
                | keys::N3        // # = Shift+3
                | keys::SEMICOLON // : = Shift+;
        )
    }

    /// Handle key event with extended parameters
    ///
    /// # Arguments
    /// * `key` - macOS virtual keycode
    /// * `caps` - true if Caps Lock is active (for uppercase letters)
    /// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
    /// * `shift` - true if Shift key is pressed (for symbols like @, #, $)
    pub fn on_key_ext(&mut self, key: u16, caps: bool, ctrl: bool, shift: bool) -> Result {
        key_handler::on_key_ext(self, key, caps, ctrl, shift)
    }

    /// Try "w" as vowel "ư" in Telex mode
    ///
    /// Rules:
    /// - "w" alone → "ư"
    /// - "nhw" → "như" (valid consonant + ư)
    /// - "kw" → "kw" (invalid, k cannot precede ư)
    /// - "ww" → revert to "w" (shortcut skipped)
    /// - "www" → "ww" (subsequent w just adds normally)
    fn try_w_as_vowel(&mut self, caps: bool) -> Option<Result> {
        stroke_handler::try_w_as_vowel(self, caps)
    }

    /// Try to apply stroke transformation (dd → đ, VNI d9 → đ)
    fn try_stroke(&mut self, key: u16, caps: bool) -> Option<Result> {
        stroke_handler::try_stroke(self, key, caps)
    }

    /// Try to apply tone transformation by scanning buffer for targets
    fn try_tone(
        &mut self,
        key: u16,
        caps: bool,
        tone_type: ToneType,
        targets: &[u16],
    ) -> Option<Result> {
        tone_handler::try_tone(self, key, caps, tone_type, targets)
    }

    /// Try to apply mark transformation
    fn try_mark(&mut self, key: u16, caps: bool, mark_val: u8) -> Option<Result> {
        mark_handler::try_mark(self, key, caps, mark_val)
    }

    /// Normalize ưo → ươ compound
    ///
    /// In Vietnamese, "ưo" (u with horn + plain o) is NEVER valid.
    /// It should always be "ươ" (both with horn).
    /// This function finds and fixes this pattern anywhere in the buffer.
    ///
    /// Returns Some(position) of the 'o' that was modified, None if no change.
    fn normalize_uo_compound(&mut self) -> Option<usize> {
        mark_handler::normalize_uo_compound(self)
    }

    /// Find positions of U+O or O+U compound (adjacent vowels)
    /// Returns Some((first_pos, second_pos)) if found, None otherwise
    fn find_uo_compound_positions(&self) -> Option<(usize, usize)> {
        mark_handler::find_uo_compound_positions(self)
    }

    /// Check for uo compound in buffer (any tone state)
    fn has_uo_compound(&self) -> bool {
        mark_handler::has_uo_compound(self)
    }

    /// Check for complete ươ compound (both u and o have horn)
    fn has_complete_uo_compound(&self) -> bool {
        mark_handler::has_complete_uo_compound(self)
    }

    /// Find target position for horn modifier with switching support
    /// Allows selecting vowels that have a different tone (for switching circumflex ↔ horn)
    fn find_horn_target_with_switch(&self, targets: &[u16], new_tone: u8) -> Vec<usize> {
        mark_handler::find_horn_target_with_switch(self, targets, new_tone)
    }

    /// Reposition tone (sắc/huyền/hỏi/ngã/nặng) after vowel pattern changes
    ///
    /// When user types out-of-order (e.g., "osa" instead of "oas"), the tone may be
    /// placed on wrong vowel. This function moves it to the correct position based
    /// on Vietnamese phonology rules.
    ///
    /// Returns Some((old_pos, new_pos)) if tone was moved, None otherwise.
    fn reposition_tone_if_needed(&mut self) -> Option<(usize, usize)> {
        tone_placement::reposition_tone_if_needed(self)
    }

    /// Check if there's a consonant between two positions
    fn has_consonant_between(&self, start: usize, end: usize) -> bool {
        tone_placement::has_consonant_between(self, start, end)
    }

    /// Check if vowels form a valid Vietnamese diphthong pattern
    ///
    /// This allows tone repositioning even when consonants are between vowels
    /// (typed out of order). Valid diphthongs: ia, ua, oa, ai, ao, oi, etc.
    fn vowels_form_valid_diphthong(&self, vowels: &[Vowel]) -> bool {
        tone_placement::vowels_form_valid_diphthong(vowels)
    }

    /// Reorder buffer when a vowel completes a diphthong with earlier vowel,
    /// and there are consonants between that should be final consonants.
    ///
    /// Example: "kisna" → buffer is k-í-n-a, but Vietnamese order is k-í-a-n
    /// because "ia" is a diphthong and 'n' is the final consonant.
    ///
    /// Returns Some(reorder_start_pos) if reordering happened, None otherwise.
    fn reorder_diphthong_with_final(&mut self) -> Option<usize> {
        tone_placement::reorder_diphthong_with_final(self)
    }

    /// Common revert logic: clear modifier, add key to buffer, rebuild output
    fn revert_and_rebuild(&mut self, pos: usize, key: u16, caps: bool) -> Result {
        revert::revert_and_rebuild(self, pos, key, caps)
    }

    /// Revert tone transformation
    fn revert_tone(&mut self, key: u16, caps: bool) -> Result {
        revert::revert_tone(self, key, caps)
    }

    /// Revert mark transformation
    fn revert_mark(&mut self, key: u16, caps: bool) -> Result {
        revert::revert_mark(self, key, caps)
    }

    /// Revert stroke transformation at specific position
    fn revert_stroke(&mut self, key: u16, pos: usize) -> Result {
        revert::revert_stroke(self, key, pos)
    }

    /// Try to apply remove modifier (None = nothing to remove, key falls through)
    fn try_remove(&mut self) -> Option<Result> {
        revert::try_remove(self)
    }

    /// Handle normal letter input
    fn handle_normal_letter(&mut self, key: u16, caps: bool) -> Result {
        letter_handler::handle_normal_letter(self, key, caps)
    }

    /// Check if buffer has w-as-vowel transform (standalone w→ư at start)
    /// This is different from w-as-tone which adds horn to existing vowels
    fn has_w_as_vowel_transform(&self) -> bool {
        stroke_handler::has_w_as_vowel_transform(self)
    }

    /// Revert w-as-vowel transforms and rebuild output
    fn revert_w_as_vowel_transforms(&mut self) -> Result {
        stroke_handler::revert_w_as_vowel_transforms(self)
    }

    /// Collect vowels from buffer
    fn collect_vowels(&self) -> Vec<Vowel> {
        utils::collect_vowels(&self.buf)
    }

    /// Check for final consonant after position
    fn has_final_consonant(&self, after_pos: usize) -> bool {
        utils::has_final_consonant(&self.buf, after_pos)
    }

    /// Check for qu initial
    fn has_qu_initial(&self) -> bool {
        utils::has_qu_initial(&self.buf)
    }

    /// Check for gi initial (gi + vowel)
    fn has_gi_initial(&self) -> bool {
        utils::has_gi_initial(&self.buf)
    }

    /// Rebuild output from position (delegates to helpers::rebuild_from)
    fn rebuild_from(&self, from: usize) -> Result {
        helpers::rebuild_from(&self.buf, from)
    }

    /// Rebuild output from position after a new character was inserted
    /// (delegates to helpers::rebuild_from_after_insert)
    fn rebuild_from_after_insert(&self, from: usize) -> Result {
        helpers::rebuild_from_after_insert(&self.buf, from)
    }

    /// Clear buffer and raw input history
    /// Note: Does NOT clear word_history to preserve backspace-after-space feature
    /// Also restores pending_capitalize if auto_capitalize was used (for selection-delete)
    pub fn clear(&mut self) {
        // Restore pending_capitalize if auto_capitalize was used
        // This handles selection-delete: user selects and deletes text,
        // we should restore pending state so next letter is capitalized
        if self.auto_capitalize_used {
            self.pending_capitalize = true;
            self.auto_capitalize_used = false;
        }
        self.buf.clear();
        self.raw_input.clear();
        self.last_transform = None;
        self.has_non_letter_prefix = false;
        self.pending_breve_pos = None;
        self.pending_u_horn_pos = None;
        self.stroke_reverted = false;
        self.had_mark_revert = false;
        self.pending_mark_revert_pop = false;
        self.had_any_transform = false;
        self.had_vowel_triggered_circumflex = false;
        self.had_circumflex_revert = false;
        self.reverted_circumflex_key = None;
        self.had_telex_transform = false;
        self.telex_double_raw = None;
        self.telex_double_raw_len = 0;
        self.restored_pending_clear = false;
        self.restored_is_ascii = false;
        self.shortcut_prefix.clear();
    }

    /// Clear everything including word history
    /// Used when cursor position changes (mouse click, arrow keys, etc.)
    /// to prevent accidental restore from stale history
    pub fn clear_all(&mut self) {
        self.clear();
        self.word_history.clear();
        self.spaces_after_commit = 0;
    }

    /// Get the full composed buffer as a Vietnamese string with diacritics.
    ///
    /// Used for "Select All + Replace" injection method.
    pub fn get_buffer_string(&self) -> String {
        self.buf.to_full_string()
    }

    /// Debug: Check if vowel-triggered circumflex flag is set
    pub fn had_vowel_circumflex(&self) -> bool {
        self.had_vowel_triggered_circumflex
    }

    /// Debug: Get raw_input length
    pub fn raw_input_len(&self) -> usize {
        self.raw_input.len()
    }

    /// Debug: Check if raw_input is valid English
    pub fn is_raw_english(&self) -> bool {
        self.is_raw_input_valid_english()
    }

    /// Restore buffer from a Vietnamese word string
    ///
    /// Used when native app detects cursor at word boundary and wants to edit.
    /// Parses Vietnamese characters back to buffer components.
    pub fn restore_word(&mut self, word: &str) {
        self.clear();
        let mut is_ascii = true;
        for c in word.chars() {
            if let Some(parsed) = chars::parse_char(c) {
                let mut ch = Char::new(parsed.key, parsed.caps);
                ch.tone = parsed.tone;
                ch.mark = parsed.mark;
                ch.stroke = parsed.stroke;
                self.buf.push(ch);
                self.raw_input.push((parsed.key, parsed.caps, false));
                // Check if this char has any Vietnamese diacritics
                if parsed.tone != 0 || parsed.mark != 0 || parsed.stroke {
                    is_ascii = false;
                }
            }
        }
        // Mark that buffer was restored from screen - if user types a regular consonant,
        // clear buffer first (they want fresh word, not append to restored word)
        // This allows: click on "shortcuts" → type "Nuw" → get "Nư" (not "shortcutsNuw")
        // But mark/tone keys like 's' will still work to modify the restored word
        if !self.buf.is_empty() {
            self.restored_pending_clear = true;
            self.restored_is_ascii = is_ascii;
        }
    }

    /// Check if buffer has transforms and is invalid Vietnamese
    /// Returns the raw chars if restore is needed, None otherwise
    ///
    /// `is_word_complete`: true when called on space/break (word is complete)
    ///                     false when called mid-word (during typing)
    fn should_auto_restore(&self, is_word_complete: bool) -> Option<Vec<char>> {
        auto_restore::should_auto_restore(self, is_word_complete)
    }

    /// Check if this is an intentional revert at end of word that should be kept.
    /// Returns true when double modifier is at end AND it's likely intentional (not English word).
    ///
    /// Heuristics:
    /// - Very short words (raw_input <= 3 chars): likely intentional revert → keep
    /// - Double vowel tone keys (a, e, o, w): always intentional → keep
    /// - Double 'x' or 'j': not common in English → keep
    /// - Double 's', 'f', 'r' in longer words (4+ chars): common English pattern → restore
    ///
    /// Examples:
    /// - "ass" (3 chars, ss) → keep "as"
    /// - "aaa" (3 chars, aa) → keep "aa" (circumflex revert)
    /// - "maxx" (4 chars, xx) → keep "max" (xx not common in English)
    /// - "bass" (4 chars, ss) → restore to "bass" (ss very common in English)
    fn ends_with_double_modifier(&self) -> bool {
        auto_restore::ends_with_double_modifier(self)
    }

    /// Get raw_input as lowercase ASCII string
    fn get_raw_input_string(&self) -> String {
        auto_restore::get_raw_input_string(self)
    }

    /// Get raw_input as ASCII string preserving original case
    fn get_raw_input_string_preserve_case(&self) -> String {
        auto_restore::get_raw_input_string_preserve_case(self)
    }

    /// Check if buffer is NOT valid Vietnamese (for unified auto-restore logic)
    ///
    /// Uses full validation including tone requirements (circumflex for êu, etc.)
    /// Also checks for patterns that are structurally valid but not real Vietnamese words.
    /// Returns true if buffer is structurally or phonetically invalid Vietnamese.
    fn is_buffer_invalid_vietnamese(&self) -> bool {
        auto_restore::is_buffer_invalid_vietnamese(self)
    }

    /// Check if raw_input is valid English (for unified auto-restore logic)
    ///
    /// Checks that raw_input contains only basic ASCII letters (A-Z, a-z)
    /// and doesn't have patterns that would indicate Vietnamese typing intent.
    /// Returns true if raw_input looks like an English word.
    fn is_raw_input_valid_english(&self) -> bool {
        auto_restore::is_raw_input_valid_english(self)
    }

    /// Build raw chars from raw_input EXACTLY as typed (no collapsing)
    /// Used for whitelist-based restore where we want the exact English word.
    fn build_raw_chars_exact(&self) -> Option<Vec<char>> {
        auto_restore::build_raw_chars_exact(self)
    }

    /// Build raw chars from raw_input for restore
    ///
    /// When a mark was reverted (e.g., "ss" → "s"), decide between buffer and raw_input:
    /// - If after revert there's vowel + consonant pattern → use buffer ("dissable" → "disable")
    /// - If after revert there's only vowels → use raw_input ("issue" → "issue")
    ///
    /// Also handles triple vowel collapse (e.g., "saaas" → "saas"):
    /// - Triple vowel (aaa, eee, ooo) is collapsed to double vowel
    /// - This handles circumflex revert in Telex (aa=â, aaa=aa)
    fn build_raw_chars(&self) -> Option<Vec<char>> {
        auto_restore::build_raw_chars(self)
    }

    /// Determine if buffer should be used for restore after a mark revert
    ///
    /// Heuristic: Use buffer when it forms a recognizable English word pattern,
    /// OR when raw_input looks like a typo (double letter + single vowel at end).
    ///
    /// Examples:
    /// - "dissable" → buffer "disable" has dis- prefix → use buffer
    /// - "soffa" → double ff + single vowel 'a' at end → use buffer "sofa"
    /// - "issue" → iss + ue pattern (double + multiple chars) → use raw_input "issue"
    /// - "error" → err + or pattern (double + multiple chars) → use raw_input "error"
    fn should_use_buffer_for_revert(&self) -> bool {
        auto_restore::should_use_buffer_for_revert(self)
    }

    /// Check for English patterns in raw_input that suggest non-Vietnamese
    ///
    /// Patterns detected:
    /// 1. Modifier (s/f/r/x/j in Telex) followed by consonant: "text" (x before t)
    /// 2. Modifier at end of long word (>2 chars): "their" (r at end)
    /// 3. Modifier after first vowel then another vowel: "use" (s between u and e)
    /// 4. Consonant + W + vowel without tone modifiers (only on word complete): "swim"
    fn has_english_modifier_pattern(&self, is_word_complete: bool) -> bool {
        auto_restore::has_english_modifier_pattern(self, is_word_complete)
    }

    /// Try to convert bracket key to vowel: ] → ư, [ → ơ (Issue #159)
    ///
    /// Returns Some(Result) if bracket was converted, None otherwise.
    /// Handles:
    /// - ] at word start or after consonant → ư
    /// - [ at word start or after consonant → ơ
    /// - Double bracket reverts: ]] → ], [[ → [, uppercase revert → } or {
    /// - Valid Vietnamese vowel combinations: ươ (from ][)
    fn try_bracket_as_vowel(&mut self, key: u16, caps: bool) -> Option<Result> {
        stroke_handler::try_bracket_as_vowel(self, key, caps)
    }

    /// Auto-restore invalid Vietnamese to raw English on space
    ///
    /// Called when SPACE is pressed. If buffer has transforms but result is not
    /// valid Vietnamese, restore to original English + space.
    /// Example: "tẽt" (from typing "text") → "text " (restored + space)
    /// Example: "ễpct" (from typing "expect") → "expect " (restored + space)
    fn try_auto_restore_on_space(&self) -> Result {
        auto_restore::try_auto_restore_on_space(self)
    }

    fn try_auto_restore_on_break(&self, break_char: Option<char>) -> Result {
        auto_restore::try_auto_restore_on_break(self, break_char)
    }

    fn restore_to_raw(&self) -> Result {
        auto_restore::restore_to_raw(self)
    }

    /// Restore raw_input from buffer (for ESC restore to work after backspace-restore)
    fn restore_raw_input_from_buffer(&mut self, buf: &Buffer) {
        self.raw_input.clear();
        for c in buf.iter() {
            self.raw_input.push((c.key, c.caps, false));
        }
    }
}

