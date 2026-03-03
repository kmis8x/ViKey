//! FFI word restore function for Vietnamese IME

use crate::lock_engine;

/// Restore buffer from a Vietnamese word string.
///
/// Used when native app detects cursor at word boundary and user
/// wants to continue editing (e.g., backspace into previous word).
/// Parses Vietnamese characters back to buffer components.
///
/// # Arguments
/// * `word` - C string containing the Vietnamese word to restore
///
/// # Safety
/// Pointer must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ime_restore_word(word: *const std::os::raw::c_char) {
    if word.is_null() {
        return;
    }
    let word_str = match std::ffi::CStr::from_ptr(word).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.restore_word(word_str);
    }
}
