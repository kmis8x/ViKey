//! ViKey IBus Engine for Linux
//!
//! Vietnamese Input Method using IBus framework.
//! This is a simple standalone engine that uses vikey_core FFI.
//!
//! Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey
//!
//! Note: IBus integration is done via C FFI. This binary is called by IBus daemon.

use std::ffi::CString;
use std::os::raw::c_char;

mod keymap;

// Re-export vikey_core FFI functions
pub use vikey_core::{
    ime_init, ime_key_ext, ime_free, ime_method, ime_enabled,
    ime_modern, ime_esc_restore, ime_english_auto_restore,
    ime_clear, ime_clear_all,
};

// IBus C FFI bindings (minimal)
#[link(name = "ibus-1.0")]
extern "C" {
    fn ibus_init();
    fn ibus_bus_new() -> *mut std::ffi::c_void;
    fn ibus_bus_is_connected(bus: *mut std::ffi::c_void) -> i32;
    fn ibus_bus_request_name(bus: *mut std::ffi::c_void, name: *const c_char, flags: u32) -> u32;
    fn ibus_main();
    fn ibus_quit();
}

/// Main entry point for IBus engine
fn main() {
    println!("ViKey IBus Engine starting...");

    // Initialize vikey_core
    unsafe {
        ime_init();
        ime_method(0); // Telex default
        ime_enabled(true);
        ime_modern(true);
        ime_esc_restore(true);
        ime_english_auto_restore(true);
    }

    // Initialize IBus
    unsafe {
        ibus_init();

        let bus = ibus_bus_new();
        if bus.is_null() {
            eprintln!("Error: Cannot create IBus bus");
            std::process::exit(1);
        }

        if ibus_bus_is_connected(bus) == 0 {
            eprintln!("Error: Cannot connect to IBus daemon");
            std::process::exit(1);
        }

        let name = CString::new("vn.vikey.ibus.ViKey").unwrap();
        let result = ibus_bus_request_name(bus, name.as_ptr(), 0x4); // IBUS_BUS_NAME_FLAG_REPLACE_EXISTING

        if result == 0 {
            eprintln!("Error: Cannot request name on IBus bus");
            std::process::exit(1);
        }

        println!("ViKey IBus engine started successfully!");
        println!("Engine name: vikey");
        println!("Language: Vietnamese (vi)");
        println!("Methods: Telex, VNI");

        // Run IBus main loop
        ibus_main();
    }
}

/// Process a key event from IBus
///
/// # Arguments
/// * `keycode` - Linux X11 keycode
/// * `modifiers` - IBus modifier flags
///
/// # Returns
/// * `(handled, text, backspace)` - Whether key was handled, text to commit, backspace count
#[no_mangle]
pub extern "C" fn vikey_process_key(keycode: u32, keyval: u32, modifiers: u32) -> *mut ViKeyResult {
    // Convert Linux keycode to macOS keycode
    let mac_key = keymap::linux_to_mac(keycode);
    if mac_key == 0xFF {
        return std::ptr::null_mut();
    }

    // Check modifiers
    let caps = (modifiers & 0x2) != 0;     // IBUS_LOCK_MASK
    let shift = (modifiers & 0x1) != 0;    // IBUS_SHIFT_MASK
    let ctrl = (modifiers & 0x4) != 0;     // IBUS_CONTROL_MASK

    // Skip if Ctrl/Alt is pressed
    if ctrl || (modifiers & 0x8) != 0 {    // IBUS_MOD1_MASK (Alt)
        return std::ptr::null_mut();
    }

    // Call vikey_core
    let result = unsafe { ime_key_ext(mac_key, caps != shift, ctrl, shift) };

    if result.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let r = &*result;

        if r.action == 1 {
            // Send action - convert chars to string
            let text: String = (0..r.count as usize)
                .filter_map(|i| char::from_u32(r.chars[i]))
                .collect();

            let c_text = CString::new(text.clone()).unwrap_or_default();

            let out = Box::new(ViKeyResult {
                handled: 1,
                text: c_text.into_raw(),
                backspace: r.backspace,
            });

            ime_free(result);
            return Box::into_raw(out);
        }

        ime_free(result);
    }

    std::ptr::null_mut()
}

/// Result structure for FFI
#[repr(C)]
pub struct ViKeyResult {
    pub handled: i32,
    pub text: *mut c_char,
    pub backspace: u8,
}

/// Free a ViKeyResult
#[no_mangle]
pub unsafe extern "C" fn vikey_free_result(r: *mut ViKeyResult) {
    if !r.is_null() {
        let boxed = Box::from_raw(r);
        if !boxed.text.is_null() {
            drop(CString::from_raw(boxed.text));
        }
    }
}

/// Clear buffer on word boundary
#[no_mangle]
pub extern "C" fn vikey_clear() {
    unsafe { ime_clear(); }
}

/// Clear all including history
#[no_mangle]
pub extern "C" fn vikey_clear_all() {
    unsafe { ime_clear_all(); }
}

/// Set input method (0=Telex, 1=VNI)
#[no_mangle]
pub extern "C" fn vikey_set_method(method: u8) {
    unsafe { ime_method(method); }
}

/// Enable/disable engine
#[no_mangle]
pub extern "C" fn vikey_set_enabled(enabled: bool) {
    unsafe { ime_enabled(enabled); }
}
