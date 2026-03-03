//! FFI shortcut management functions for Vietnamese IME

use crate::lock_engine;

/// Add a shortcut to the engine.
///
/// # Arguments
/// * `trigger` - C string for trigger (e.g., "vn")
/// * `replacement` - C string for replacement (e.g., "Việt Nam")
///
/// # Safety
/// Both pointers must be valid null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn ime_add_shortcut(
    trigger: *const std::os::raw::c_char,
    replacement: *const std::os::raw::c_char,
) {
    if trigger.is_null() || replacement.is_null() {
        return;
    }

    let trigger_str = match std::ffi::CStr::from_ptr(trigger).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let replacement_str = match std::ffi::CStr::from_ptr(replacement).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        // Auto-detect shortcut type:
        // - If trigger contains only non-letter chars (like "->", "=>"), use immediate trigger
        // - Otherwise use word boundary trigger (traditional abbreviations like "vn" → "Việt Nam")
        let is_symbol_trigger = trigger_str.chars().all(|c| !c.is_alphabetic());
        let shortcut = if is_symbol_trigger {
            crate::engine::shortcut::Shortcut::immediate(trigger_str, replacement_str)
        } else {
            crate::engine::shortcut::Shortcut::new(trigger_str, replacement_str)
        };
        e.shortcuts_mut().add(shortcut);
    }
}

/// Remove a shortcut from the engine.
///
/// # Arguments
/// * `trigger` - C string for trigger to remove
///
/// # Safety
/// Pointer must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ime_remove_shortcut(trigger: *const std::os::raw::c_char) {
    if trigger.is_null() {
        return;
    }

    let trigger_str = match std::ffi::CStr::from_ptr(trigger).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.shortcuts_mut().remove(trigger_str);
    }
}

/// Clear all shortcuts from the engine.
#[no_mangle]
pub extern "C" fn ime_clear_shortcuts() {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        e.shortcuts_mut().clear();
    }
}
