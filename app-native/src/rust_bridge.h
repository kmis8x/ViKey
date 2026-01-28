// ViKey - Rust FFI Bridge
// rust_bridge.h
// Interface to core.dll

#pragma once

#include <windows.h>
#include <cstdint>
#include <string>

// Input method type
enum class InputMethod : uint8_t {
    Telex = 0,
    VNI = 1
};

// IME action type
enum class ImeAction : uint8_t {
    None = 0,    // No action needed, pass key through
    Send = 1,    // Send text replacement
    Restore = 2  // Restore original text (unused)
};

// Native result structure from Rust (must match core/src/engine/mod.rs)
// chars[256] (1024 bytes) + action (1) + backspace (1) + count (1) + flags (1) = 1028 bytes
// Note: No packing needed - Rust's #[repr(C)] uses natural alignment
struct NativeResult {
    uint32_t chars[256];
    uint8_t action;
    uint8_t backspace;
    uint8_t count;
    uint8_t flags;
};

// Managed IME result
class ImeResult {
public:
    static constexpr uint8_t FLAG_KEY_CONSUMED = 0x01;

    ImeAction action;
    uint8_t backspace;
    uint8_t count;
    uint8_t flags;

    ImeResult() : action(ImeAction::None), backspace(0), count(0), flags(0) {}
    ImeResult(ImeAction a, uint8_t bs, uint8_t c, uint8_t f, const uint32_t* ch, size_t len);

    // Check if key should be consumed (not passed through)
    bool IsKeyConsumed() const { return (flags & FLAG_KEY_CONSUMED) != 0; }

    // Get the result text as a wstring
    std::wstring GetText() const;

    static ImeResult Empty() { return ImeResult(); }

private:
    uint32_t m_chars[256];
    size_t m_charCount;
};

// Rust bridge singleton
class RustBridge {
public:
    static RustBridge& Instance();

    // Initialize the IME engine. Call once at startup.
    bool Initialize();

    // Cleanup
    void Shutdown();

    // Check if loaded
    bool IsLoaded() const { return m_loaded; }

    // Clear the typing buffer (on word boundary)
    void Clear();

    // Clear everything including word history (on cursor change)
    void ClearAll();

    // Set input method (Telex=0, VNI=1)
    void SetMethod(InputMethod method);

    // Enable or disable IME processing
    void SetEnabled(bool enabled);

    // Set tone style (modern=true: hoa`, old=false: ho`a)
    void SetModernTone(bool modern);

    // Enable/disable English auto-restore
    void SetEnglishAutoRestore(bool enabled);

    // Enable/disable auto-capitalize after sentence end
    void SetAutoCapitalize(bool enabled);

    // Skip w shortcut (w stays as w at word start)
    void SetSkipWShortcut(bool skip);

    // Enable bracket shortcuts: ] -> u+, [ -> o+
    void SetBracketShortcut(bool enabled);

    // Enable ESC key restore (restore raw ASCII)
    void SetEscRestore(bool enabled);

    // Enable free tone placement (skip validation)
    void SetFreeTone(bool enabled);

    // Allow foreign consonants (f, j, w, z) as valid initials
    void SetAllowForeignConsonants(bool enabled);

    // Shortcut management
    void AddShortcut(const wchar_t* trigger, const wchar_t* replacement);
    void RemoveShortcut(const wchar_t* trigger);
    void ClearShortcuts();

    // Process a keystroke and get the result
    ImeResult ProcessKey(uint16_t keycode, bool caps, bool ctrl);

    // Process a keystroke with shift parameter (for VNI symbols)
    ImeResult ProcessKeyExt(uint16_t keycode, bool caps, bool ctrl, bool shift);

private:
    RustBridge();
    ~RustBridge();
    RustBridge(const RustBridge&) = delete;
    RustBridge& operator=(const RustBridge&) = delete;

    // Function pointer types
    using FnInit = void(*)();
    using FnClear = void(*)();
    using FnClearAll = void(*)();
    using FnFree = void(*)(void*);
    using FnMethod = void(*)(uint8_t);
    using FnEnabled = void(*)(bool);
    using FnModern = void(*)(bool);
    using FnEnglishAutoRestore = void(*)(bool);
    using FnAutoCapitalize = void(*)(bool);
    using FnSkipWShortcut = void(*)(bool);
    using FnBracketShortcut = void(*)(bool);
    using FnEscRestore = void(*)(bool);
    using FnFreeTone = void(*)(bool);
    using FnAllowForeignConsonants = void(*)(bool);
    using FnAddShortcut = void(*)(const char*, const char*);
    using FnRemoveShortcut = void(*)(const char*);
    using FnClearShortcuts = void(*)();
    using FnKey = NativeResult*(*)(uint16_t, bool, bool);
    using FnKeyExt = NativeResult*(*)(uint16_t, bool, bool, bool);

    HMODULE m_hModule;
    bool m_loaded;

    // Function pointers
    FnInit m_ime_init;
    FnClear m_ime_clear;
    FnClearAll m_ime_clear_all;
    FnFree m_ime_free;
    FnMethod m_ime_method;
    FnEnabled m_ime_enabled;
    FnModern m_ime_modern;
    FnEnglishAutoRestore m_ime_english_auto_restore;
    FnAutoCapitalize m_ime_auto_capitalize;
    FnSkipWShortcut m_ime_skip_w_shortcut;
    FnBracketShortcut m_ime_bracket_shortcut;
    FnEscRestore m_ime_esc_restore;
    FnFreeTone m_ime_free_tone;
    FnAllowForeignConsonants m_ime_allow_foreign_consonants;
    FnAddShortcut m_ime_add_shortcut;
    FnRemoveShortcut m_ime_remove_shortcut;
    FnClearShortcuts m_ime_clear_shortcuts;
    FnKey m_ime_key;
    FnKeyExt m_ime_key_ext;

    ImeResult ParseResult(NativeResult* ptr);
};
