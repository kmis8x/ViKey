// ViKey - Rust FFI Bridge Implementation
// rust_bridge.cpp
// Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey

#define _CRT_SECURE_NO_WARNINGS
#include "rust_bridge.h"
#include <codecvt>
#include <locale>
#include <string>

// ImeResult implementation
ImeResult::ImeResult(ImeAction a, uint8_t bs, uint8_t c, uint8_t f, const uint32_t* ch, size_t len)
    : action(a), backspace(bs), count(c), flags(f), m_charCount(0) {
    if (ch && len > 0) {
        m_charCount = len < 256 ? len : 256;
        memcpy(m_chars, ch, m_charCount * sizeof(uint32_t));
    }
}

std::wstring ImeResult::GetText() const {
    if (count == 0) return L"";

    std::wstring result;
    result.reserve(count);

    for (size_t i = 0; i < count && i < m_charCount; i++) {
        uint32_t cp = m_chars[i];
        if (cp == 0) continue;

        // Convert UTF-32 code point to UTF-16
        if (cp < 0x10000) {
            result += static_cast<wchar_t>(cp);
        } else {
            // Surrogate pair for characters outside BMP
            cp -= 0x10000;
            result += static_cast<wchar_t>(0xD800 + (cp >> 10));
            result += static_cast<wchar_t>(0xDC00 + (cp & 0x3FF));
        }
    }
    return result;
}

// RustBridge implementation
RustBridge& RustBridge::Instance() {
    static RustBridge instance;
    return instance;
}

RustBridge::RustBridge()
    : m_hModule(nullptr)
    , m_loaded(false)
    , m_ime_init(nullptr)
    , m_ime_clear(nullptr)
    , m_ime_clear_all(nullptr)
    , m_ime_free(nullptr)
    , m_ime_method(nullptr)
    , m_ime_enabled(nullptr)
    , m_ime_modern(nullptr)
    , m_ime_english_auto_restore(nullptr)
    , m_ime_auto_capitalize(nullptr)
    , m_ime_skip_w_shortcut(nullptr)
    , m_ime_bracket_shortcut(nullptr)
    , m_ime_esc_restore(nullptr)
    , m_ime_free_tone(nullptr)
    , m_ime_allow_foreign_consonants(nullptr)
    , m_ime_add_shortcut(nullptr)
    , m_ime_remove_shortcut(nullptr)
    , m_ime_clear_shortcuts(nullptr)
    , m_ime_key(nullptr)
    , m_ime_key_ext(nullptr) {
}

RustBridge::~RustBridge() {
    Shutdown();
}

bool RustBridge::Initialize() {
    if (m_loaded) return true;

    // Load the Rust core DLL
    m_hModule = LoadLibraryW(L"core.dll");
    if (!m_hModule) {
        // Try loading from same directory as exe
        wchar_t path[MAX_PATH];
        GetModuleFileNameW(nullptr, path, MAX_PATH);
        wchar_t* lastSlash = wcsrchr(path, L'\\');
        if (lastSlash) {
            wcscpy(lastSlash + 1, L"core.dll");
            m_hModule = LoadLibraryW(path);
        }
    }

    if (!m_hModule) {
        MessageBoxW(nullptr, L"Failed to load core.dll", L"IME Error", MB_ICONERROR);
        return false;
    }

    // Get function addresses
    m_ime_init = (FnInit)GetProcAddress(m_hModule, "ime_init");
    m_ime_clear = (FnClear)GetProcAddress(m_hModule, "ime_clear");
    m_ime_clear_all = (FnClearAll)GetProcAddress(m_hModule, "ime_clear_all");
    m_ime_free = (FnFree)GetProcAddress(m_hModule, "ime_free");
    m_ime_method = (FnMethod)GetProcAddress(m_hModule, "ime_method");
    m_ime_enabled = (FnEnabled)GetProcAddress(m_hModule, "ime_enabled");
    m_ime_modern = (FnModern)GetProcAddress(m_hModule, "ime_modern");
    m_ime_english_auto_restore = (FnEnglishAutoRestore)GetProcAddress(m_hModule, "ime_english_auto_restore");
    m_ime_auto_capitalize = (FnAutoCapitalize)GetProcAddress(m_hModule, "ime_auto_capitalize");
    m_ime_skip_w_shortcut = (FnSkipWShortcut)GetProcAddress(m_hModule, "ime_skip_w_shortcut");
    m_ime_bracket_shortcut = (FnBracketShortcut)GetProcAddress(m_hModule, "ime_bracket_shortcut");
    m_ime_esc_restore = (FnEscRestore)GetProcAddress(m_hModule, "ime_esc_restore");
    m_ime_free_tone = (FnFreeTone)GetProcAddress(m_hModule, "ime_free_tone");
    m_ime_allow_foreign_consonants = (FnAllowForeignConsonants)GetProcAddress(m_hModule, "ime_allow_foreign_consonants");
    m_ime_add_shortcut = (FnAddShortcut)GetProcAddress(m_hModule, "ime_add_shortcut");
    m_ime_remove_shortcut = (FnRemoveShortcut)GetProcAddress(m_hModule, "ime_remove_shortcut");
    m_ime_clear_shortcuts = (FnClearShortcuts)GetProcAddress(m_hModule, "ime_clear_shortcuts");
    m_ime_key = (FnKey)GetProcAddress(m_hModule, "ime_key");
    m_ime_key_ext = (FnKeyExt)GetProcAddress(m_hModule, "ime_key_ext");

    // Check required functions
    if (!m_ime_init || !m_ime_key || !m_ime_free) {
        MessageBoxW(nullptr, L"Failed to find required DLL functions", L"IME Error", MB_ICONERROR);
        FreeLibrary(m_hModule);
        m_hModule = nullptr;
        return false;
    }

    // Initialize the engine
    m_ime_init();
    m_loaded = true;
    return true;
}

void RustBridge::Shutdown() {
    if (m_hModule) {
        FreeLibrary(m_hModule);
        m_hModule = nullptr;
    }
    m_loaded = false;
}

void RustBridge::Clear() {
    if (m_ime_clear) m_ime_clear();
}

void RustBridge::ClearAll() {
    if (m_ime_clear_all) m_ime_clear_all();
}

void RustBridge::SetMethod(InputMethod method) {
    if (m_ime_method) m_ime_method(static_cast<uint8_t>(method));
}

void RustBridge::SetEnabled(bool enabled) {
    if (m_ime_enabled) m_ime_enabled(enabled);
}

void RustBridge::SetModernTone(bool modern) {
    if (m_ime_modern) m_ime_modern(modern);
}

void RustBridge::SetEnglishAutoRestore(bool enabled) {
    if (m_ime_english_auto_restore) m_ime_english_auto_restore(enabled);
}

void RustBridge::SetAutoCapitalize(bool enabled) {
    if (m_ime_auto_capitalize) m_ime_auto_capitalize(enabled);
}

void RustBridge::SetSkipWShortcut(bool skip) {
    if (m_ime_skip_w_shortcut) m_ime_skip_w_shortcut(skip);
}

void RustBridge::SetBracketShortcut(bool enabled) {
    if (m_ime_bracket_shortcut) m_ime_bracket_shortcut(enabled);
}

void RustBridge::SetEscRestore(bool enabled) {
    if (m_ime_esc_restore) m_ime_esc_restore(enabled);
}

void RustBridge::SetFreeTone(bool enabled) {
    if (m_ime_free_tone) m_ime_free_tone(enabled);
}

void RustBridge::SetAllowForeignConsonants(bool enabled) {
    if (m_ime_allow_foreign_consonants) m_ime_allow_foreign_consonants(enabled);
}

void RustBridge::AddShortcut(const wchar_t* trigger, const wchar_t* replacement) {
    if (!m_ime_add_shortcut || !trigger || !replacement) return;

    // Convert wide strings to UTF-8
    int triggerLen = WideCharToMultiByte(CP_UTF8, 0, trigger, -1, nullptr, 0, nullptr, nullptr);
    int replacementLen = WideCharToMultiByte(CP_UTF8, 0, replacement, -1, nullptr, 0, nullptr, nullptr);

    if (triggerLen <= 0 || replacementLen <= 0) return;

    std::string triggerUtf8(triggerLen, 0);
    std::string replacementUtf8(replacementLen, 0);

    WideCharToMultiByte(CP_UTF8, 0, trigger, -1, &triggerUtf8[0], triggerLen, nullptr, nullptr);
    WideCharToMultiByte(CP_UTF8, 0, replacement, -1, &replacementUtf8[0], replacementLen, nullptr, nullptr);

    m_ime_add_shortcut(triggerUtf8.c_str(), replacementUtf8.c_str());
}

void RustBridge::RemoveShortcut(const wchar_t* trigger) {
    if (!m_ime_remove_shortcut || !trigger) return;

    int triggerLen = WideCharToMultiByte(CP_UTF8, 0, trigger, -1, nullptr, 0, nullptr, nullptr);
    if (triggerLen <= 0) return;

    std::string triggerUtf8(triggerLen, 0);
    WideCharToMultiByte(CP_UTF8, 0, trigger, -1, &triggerUtf8[0], triggerLen, nullptr, nullptr);

    m_ime_remove_shortcut(triggerUtf8.c_str());
}

void RustBridge::ClearShortcuts() {
    if (m_ime_clear_shortcuts) m_ime_clear_shortcuts();
}

ImeResult RustBridge::ProcessKey(uint16_t keycode, bool caps, bool ctrl) {
    if (!m_ime_key) return ImeResult::Empty();

    NativeResult* ptr = m_ime_key(keycode, caps, ctrl);
    return ParseResult(ptr);
}

ImeResult RustBridge::ProcessKeyExt(uint16_t keycode, bool caps, bool ctrl, bool shift) {
    if (!m_ime_key_ext) return ProcessKey(keycode, caps, ctrl);

    NativeResult* ptr = m_ime_key_ext(keycode, caps, ctrl, shift);
    return ParseResult(ptr);
}

ImeResult RustBridge::ParseResult(NativeResult* ptr) {
    if (!ptr) return ImeResult::Empty();

    ImeResult result(
        static_cast<ImeAction>(ptr->action),
        ptr->backspace,
        ptr->count,
        ptr->flags,
        ptr->chars,
        ptr->count
    );

    if (m_ime_free) m_ime_free(ptr);

    return result;
}
