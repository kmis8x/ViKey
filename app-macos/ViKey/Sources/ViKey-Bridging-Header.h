//
//  ViKey-Bridging-Header.h
//  ViKey macOS Input Method
//
//  C bridge for Rust FFI functions
//

#ifndef ViKey_Bridging_Header_h
#define ViKey_Bridging_Header_h

#include <stdint.h>
#include <stdbool.h>

// Result structure from Rust (must match core/src/lib.rs)
typedef struct {
    uint32_t chars[256];    // UTF-32 codepoints
    uint8_t action;         // 0=None, 1=Send, 2=Restore
    uint8_t backspace;      // Characters to delete
    uint8_t count;          // Valid chars count
    uint8_t flags;          // 0x01 = key consumed
} ImeResult;

// Core lifecycle
void ime_init(void);
void ime_free(ImeResult* result);

// Key processing
ImeResult* ime_key(uint16_t key, bool caps, bool ctrl);
ImeResult* ime_key_ext(uint16_t key, bool caps, bool ctrl, bool shift);

// State control
void ime_enabled(bool enabled);
void ime_method(uint8_t method);  // 0=Telex, 1=VNI
void ime_clear(void);
void ime_clear_all(void);

// Configuration
void ime_skip_w_shortcut(bool skip);
void ime_bracket_shortcut(bool enabled);
void ime_esc_restore(bool enabled);
void ime_free_tone(bool enabled);
void ime_modern(bool modern);
void ime_english_auto_restore(bool enabled);
void ime_auto_capitalize(bool enabled);
void ime_allow_foreign_consonants(bool enabled);
void ime_shortcuts_enabled(bool enabled);

// Shortcuts
void ime_add_shortcut(const char* trigger, const char* replacement);
void ime_remove_shortcut(const char* trigger);
void ime_clear_shortcuts(void);

#endif /* ViKey_Bridging_Header_h */
