//! Integration tests for FFI interface

use crate::*;
use crate::data::keys;
use serial_test::serial;
use std::ffi::CString;

#[test]
#[serial]
fn test_ffi_flow() {
    ime_init();
    ime_method(0); // Telex

    // Type 'a' + 's' -> á
    let r1 = ime_key(keys::A, false, false);
    assert!(!r1.is_null());
    unsafe { ime_free(r1) };

    let r2 = ime_key(keys::S, false, false);
    assert!(!r2.is_null());
    unsafe {
        assert_eq!((*r2).chars[0], 'á' as u32);
        ime_free(r2);
    }

    ime_clear();
}

#[test]
#[serial]
fn test_shortcut_ffi_add_and_clear() {
    ime_init();
    ime_clear_shortcuts(); // Clear any existing shortcuts
    ime_method(0); // Telex

    // Add a shortcut via FFI
    let trigger = CString::new("vn").unwrap();
    let replacement = CString::new("Việt Nam").unwrap();

    unsafe {
        ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
    }

    // Verify shortcut was added by checking engine state
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 1);
    }
    drop(guard);

    // Clear all shortcuts
    ime_clear_shortcuts();

    // Verify shortcuts cleared
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 0);
    }
    drop(guard);

    ime_clear();
}

#[test]
#[serial]
fn test_shortcut_ffi_remove() {
    ime_init();
    ime_clear_shortcuts(); // Clear any existing shortcuts
    ime_method(0); // Telex

    // Add two shortcuts
    let trigger1 = CString::new("hn").unwrap();
    let replacement1 = CString::new("Hà Nội").unwrap();
    let trigger2 = CString::new("hcm").unwrap();
    let replacement2 = CString::new("Hồ Chí Minh").unwrap();

    unsafe {
        ime_add_shortcut(trigger1.as_ptr(), replacement1.as_ptr());
        ime_add_shortcut(trigger2.as_ptr(), replacement2.as_ptr());
    }

    // Verify both added
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 2);
    }
    drop(guard);

    // Remove one shortcut
    unsafe {
        ime_remove_shortcut(trigger1.as_ptr());
    }

    // Verify only one remains
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 1);
    }
    drop(guard);

    // Clean up
    ime_clear_shortcuts();
    ime_clear();
}

#[test]
#[serial]
fn test_shortcut_ffi_null_safety() {
    ime_init();

    // Should not crash with null pointers
    unsafe {
        ime_add_shortcut(std::ptr::null(), std::ptr::null());
        ime_remove_shortcut(std::ptr::null());
    }

    // Engine should still work
    let r = ime_key(keys::A, false, false);
    assert!(!r.is_null());
    unsafe { ime_free(r) };

    ime_clear();
}

#[test]
#[serial]
fn test_shortcut_ffi_unicode() {
    ime_init();
    ime_clear_shortcuts(); // Clear any existing shortcuts
    ime_method(0);

    // Test with Unicode in both trigger and replacement
    let trigger = CString::new("tphcm").unwrap();
    let replacement = CString::new("Thành phố Hồ Chí Minh").unwrap();

    unsafe {
        ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
    }

    // Verify shortcut added with proper UTF-8 handling
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 1);
    }
    drop(guard);

    ime_clear_shortcuts();
    ime_clear();
}

#[test]
#[serial]
fn test_shortcut_ffi_symbol_trigger_immediate() {
    // Test that symbol-only triggers (like "->") are created as immediate shortcuts
    ime_init();
    ime_clear_shortcuts();
    ime_method(0); // Telex

    // Add arrow shortcut via FFI - should auto-detect as immediate
    let trigger = CString::new("->").unwrap();
    let replacement = CString::new("→").unwrap();

    unsafe {
        ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
    }

    // Verify shortcut was added with immediate trigger
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 1);
        let shortcut = e.shortcuts().lookup("->").unwrap().1;
        assert_eq!(
            shortcut.condition,
            engine::shortcut::TriggerCondition::Immediate,
            "Symbol-only trigger should be immediate"
        );
    }
    drop(guard);

    ime_clear_shortcuts();
    ime_clear();
}

#[test]
#[serial]
fn test_shortcut_ffi_letter_trigger_word_boundary() {
    // Test that letter triggers (like "vn") are created as word boundary shortcuts
    ime_init();
    ime_clear_shortcuts();
    ime_method(0); // Telex

    // Add abbreviation shortcut via FFI - should be word boundary
    let trigger = CString::new("vn").unwrap();
    let replacement = CString::new("Việt Nam").unwrap();

    unsafe {
        ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
    }

    // Verify shortcut was added with word boundary trigger
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 1);
        let shortcut = e.shortcuts().lookup("vn").unwrap().1;
        assert_eq!(
            shortcut.condition,
            engine::shortcut::TriggerCondition::OnWordBoundary,
            "Letter trigger should be word boundary"
        );
    }
    drop(guard);

    ime_clear_shortcuts();
    ime_clear();
}

/// Issue #161: Test that shortcuts containing numbers work correctly via FFI
#[test]
#[serial]
fn test_shortcut_ffi_with_numbers() {
    ime_init();
    ime_clear_shortcuts();
    ime_method(0); // Telex

    // Add shortcut with number via FFI
    let trigger = CString::new("f1").unwrap();
    let replacement = CString::new("formula one").unwrap();

    unsafe {
        ime_add_shortcut(trigger.as_ptr(), replacement.as_ptr());
    }

    // Verify shortcut was added
    let guard = lock_engine();
    if let Some(ref e) = *guard {
        assert_eq!(e.shortcuts().len(), 1);
        let shortcut = e.shortcuts().lookup("f1").unwrap().1;
        assert_eq!(
            shortcut.condition,
            engine::shortcut::TriggerCondition::OnWordBoundary,
            "Mixed letter+number trigger should be word boundary"
        );
    }
    drop(guard);

    // Type "f1" + space and verify shortcut triggers
    let _ = ime_key(keys::F, false, false);
    let _ = ime_key(keys::N1, false, false);
    let r = ime_key(keys::SPACE, false, false);

    assert!(!r.is_null());
    let result = unsafe { &*r };
    assert_eq!(
        result.action,
        engine::Action::Send as u8,
        "Shortcut should trigger"
    );
    assert_eq!(result.backspace, 2, "Should backspace 2 chars (f1)");

    // Verify output
    let output: String = (0..result.count as usize)
        .filter_map(|i| char::from_u32(result.chars[i]))
        .collect();
    assert_eq!(output, "formula one ", "Should output replacement + space");

    unsafe { ime_free(r) };
    ime_clear_shortcuts();
    ime_clear();
}

#[test]
#[serial]
fn test_restore_word_ffi() {
    ime_init();
    ime_method(0); // Telex

    // Restore a Vietnamese word
    let word = CString::new("việt").unwrap();
    unsafe {
        ime_restore_word(word.as_ptr());
    }

    // Type 's' to add sắc mark - should change ệ to ế
    // Engine returns replacement for changed portion
    let r = ime_key(keys::S, false, false);
    assert!(!r.is_null());
    unsafe {
        assert_eq!((*r).action, 1, "Should send replacement");
        // Engine outputs the modified result
        assert!((*r).count > 0, "Should have output chars");
        ime_free(r);
    }

    ime_clear();
}

#[test]
#[serial]
fn test_restore_word_ffi_null_safety() {
    ime_init();

    // Should not crash with null pointer
    unsafe {
        ime_restore_word(std::ptr::null());
    }

    // Engine should still work
    let r = ime_key(keys::A, false, false);
    assert!(!r.is_null());
    unsafe { ime_free(r) };

    ime_clear();
}
