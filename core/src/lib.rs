//! ViKey Vietnamese Keyboard Core
//!
//! Simple Vietnamese input method engine supporting Telex and VNI.
//!
//! Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey
//!
//! # FFI Usage
//!
//! ```c
//! // Initialize once at app start
//! ime_init();
//! ime_method(0);  // 0=Telex, 1=VNI
//!
//! // Process each keystroke
//! ImeResult* r = ime_key(keycode, is_shift, is_ctrl);
//! if (r && r->action == 1) {
//!     // Send r->backspace deletes, then r->chars
//! }
//! ime_free(r);
//!
//! // Clean up on word boundary
//! ime_clear();
//! ```

pub mod data;
pub mod engine;
pub mod input;
pub mod updater;
pub mod utils;

mod ffi_restore;
mod ffi_settings;
mod ffi_shortcuts;

pub use ffi_restore::*;
pub use ffi_settings::*;
pub use ffi_shortcuts::*;

use engine::{Engine, Result};
use std::sync::Mutex;

// Global engine instance (thread-safe via Mutex)
static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

/// Lock the engine mutex, recovering from poisoned state if needed (for tests)
pub(crate) fn lock_engine() -> std::sync::MutexGuard<'static, Option<Engine>> {
    ENGINE.lock().unwrap_or_else(|e| e.into_inner())
}

// ============================================================
// FFI Interface
// ============================================================

/// Initialize the IME engine.
///
/// Must be called exactly once before any other `ime_*` functions.
/// Thread-safe: uses internal mutex.
#[no_mangle]
pub extern "C" fn ime_init() {
    let mut guard = lock_engine();
    *guard = Some(Engine::new());
}

/// Process a key event and return the result.
///
/// # Arguments
/// * `key` - macOS virtual keycode (0-127 for standard keys)
/// * `caps` - true if CapsLock is pressed (for uppercase letters)
/// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
///
/// # Returns
/// * Pointer to `Result` struct (caller must free with `ime_free`)
/// * `null` if engine not initialized
#[no_mangle]
pub extern "C" fn ime_key(key: u16, caps: bool, ctrl: bool) -> *mut Result {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        let r = e.on_key(key, caps, ctrl);
        Box::into_raw(Box::new(r))
    } else {
        std::ptr::null_mut()
    }
}

/// Process a key event with extended parameters.
///
/// # Arguments
/// * `key` - macOS virtual keycode (0-127 for standard keys)
/// * `caps` - true if CapsLock is pressed (for uppercase letters)
/// * `ctrl` - true if Cmd/Ctrl/Alt is pressed (bypasses IME)
/// * `shift` - true if Shift key is pressed (for symbols like @, #, $)
///
/// # Returns
/// * Pointer to `Result` struct (caller must free with `ime_free`)
/// * `null` if engine not initialized
#[no_mangle]
pub extern "C" fn ime_key_ext(key: u16, caps: bool, ctrl: bool, shift: bool) -> *mut Result {
    let mut guard = lock_engine();
    if let Some(ref mut e) = *guard {
        let r = e.on_key_ext(key, caps, ctrl, shift);
        Box::into_raw(Box::new(r))
    } else {
        std::ptr::null_mut()
    }
}

/// Get the full composed buffer as UTF-32 codepoints.
///
/// # Arguments
/// * `out` - Pointer to output buffer for UTF-32 codepoints
/// * `max_len` - Maximum number of codepoints to write
///
/// # Returns
/// Number of codepoints written to `out`.
///
/// # Safety
/// `out` must point to valid memory of at least `max_len * sizeof(u32)` bytes.
#[no_mangle]
pub unsafe extern "C" fn ime_get_buffer(out: *mut u32, max_len: i64) -> i64 {
    if out.is_null() || max_len <= 0 {
        return 0;
    }

    let guard = lock_engine();
    if let Some(ref e) = *guard {
        let full = e.get_buffer_string();
        let utf32: Vec<u32> = full.chars().map(|c| c as u32).collect();
        let len = utf32.len().min(max_len as usize);
        std::ptr::copy_nonoverlapping(utf32.as_ptr(), out, len);
        len as i64
    } else {
        0
    }
}

/// Free a result pointer returned by `ime_key`.
///
/// # Safety
/// * `r` must be a pointer returned by `ime_key`, or null
/// * Must be called exactly once per non-null `ime_key` return
#[no_mangle]
pub unsafe extern "C" fn ime_free(r: *mut Result) {
    if !r.is_null() {
        drop(Box::from_raw(r));
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
#[path = "ffi_tests.rs"]
mod tests;
