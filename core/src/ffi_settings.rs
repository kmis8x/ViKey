//! FFI setting functions for Vietnamese IME
//!
//! All ime_method, ime_enabled, and other setter functions.

use crate::lock_engine;

/// Set the input method.
///
/// # Arguments
/// * `method` - 0 for Telex, 1 for VNI
///
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_method(method: u8) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_method(method);
    }
}

/// Enable or disable the engine.
///
/// When disabled, `ime_key` returns action=0 (pass through).
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_enabled(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_enabled(enabled);
    }
}

/// Set whether to skip w→ư shortcut in Telex mode.
///
/// When `skip` is true, typing 'w' at word start stays as 'w'
/// instead of converting to 'ư'.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_skip_w_shortcut(skip: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_skip_w_shortcut(skip);
    }
}

/// Set whether bracket shortcuts are enabled: ] → ư, [ → ơ (Issue #159)
///
/// When `enabled` is true (default), ] types ư and [ types ơ in Telex mode.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_bracket_shortcut(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_bracket_shortcut(enabled);
    }
}

/// Set whether ESC key restores raw ASCII input.
///
/// When `enabled` is true (default), pressing ESC restores original keystrokes.
/// When `enabled` is false, ESC key is passed through without restoration.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_esc_restore(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_esc_restore(enabled);
    }
}

/// Set whether to enable free tone placement (skip validation).
///
/// When `enabled` is true, allows placing diacritics anywhere without
/// spelling validation (e.g., "Zìa" is allowed).
/// When `enabled` is false (default), validates Vietnamese spelling rules.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_free_tone(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_free_tone(enabled);
    }
}

/// Set whether to use modern orthography for tone placement.
///
/// When `modern` is true: hoà, thuý (tone on second vowel - new style)
/// When `modern` is false (default): hòa, thúy (tone on first vowel - traditional)
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_modern(modern: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_modern_tone(modern);
    }
}

/// Enable/disable English auto-restore (experimental feature).
///
/// When `enabled` is true, automatically restores English words that were
/// accidentally transformed (e.g., "tẽt" → "text", "ễpct" → "expect").
/// When `enabled` is false (default), no auto-restore happens.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_english_auto_restore(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_english_auto_restore(enabled);
    }
}

/// Enable/disable auto-capitalize after sentence-ending punctuation.
///
/// When `enabled` is true, automatically capitalizes the first letter
/// after sentence-ending punctuation (. ! ? Enter).
/// When `enabled` is false (default), no auto-capitalize happens.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_auto_capitalize(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_auto_capitalize(enabled);
    }
}

/// Enable/disable foreign consonants (z, w, j, f) as valid initial consonants.
///
/// When `enabled` is true, allows z, w, j, f as valid Vietnamese consonants
/// for typing loanwords while still getting Vietnamese diacritics.
/// When `enabled` is false (default), these letters are treated as invalid initials.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_allow_foreign_consonants(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_allow_foreign_consonants(enabled);
    }
}

/// Enable/disable shortcut expansion.
///
/// When `enabled` is true (default), shortcuts are triggered as usual.
/// When `enabled` is false, shortcuts are not triggered.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_shortcuts_enabled(enabled: bool) {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.set_shortcuts_enabled(enabled);
    }
}

/// Clear the input buffer.
///
/// Call on word boundaries (space, punctuation).
/// Preserves word history for backspace-after-space feature.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_clear() {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.clear();
    }
}

/// Clear everything including word history.
///
/// Call when cursor position changes (mouse click, arrow keys, focus change).
/// This prevents accidental restore from stale history.
/// No-op if engine not initialized.
#[no_mangle]
pub extern "C" fn ime_clear_all() {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.clear_all();
    }
}
