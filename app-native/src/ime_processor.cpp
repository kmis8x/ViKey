// ViKey - Main Processor Implementation
// ime_processor.cpp
// Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey

#define _CRT_SECURE_NO_WARNINGS
#include "ime_processor.h"
#include "keycodes.h"
#include <cstdio>

ImeProcessor& ImeProcessor::Instance() {
    static ImeProcessor instance;
    return instance;
}

ImeProcessor::ImeProcessor()
    : m_enabled(true)
    , m_method(InputMethod::Telex)
    , m_initialized(false)
    , m_lastAppName(L"") {
}

bool ImeProcessor::Initialize() {
    if (m_initialized) return true;

    // Initialize Rust bridge
    if (!RustBridge::Instance().Initialize()) {
        return false;
    }

    // Set up keyboard hook callback
    KeyboardHook::Instance().SetCallback([this](KeyEventData& event) {
        OnKeyPressed(event);
    });

    m_initialized = true;
    return true;
}

void ImeProcessor::Start() {
    KeyboardHook::Instance().Start();
}

void ImeProcessor::Stop() {
    KeyboardHook::Instance().Stop();
}

void ImeProcessor::SetEnabled(bool enabled) {
    m_enabled = enabled;
    RustBridge::Instance().SetEnabled(enabled);
}

void ImeProcessor::ToggleEnabled() {
    SetEnabled(!m_enabled);

    // Save state for current app if smart switch is enabled
    Settings& settings = Settings::Instance();
    if (settings.smartSwitch) {
        std::wstring currentApp = AppDetector::Instance().GetForegroundAppName();
        if (!currentApp.empty()) {
            AppDetector::Instance().SaveAppState(currentApp, m_enabled);
        }
    }
}

void ImeProcessor::SetMethod(InputMethod method) {
    m_method = method;
    RustBridge::Instance().SetMethod(method);
}

void ImeProcessor::ApplySettings() {
    Settings& settings = Settings::Instance();

    SetEnabled(settings.enabled);
    SetMethod(settings.method);

    RustBridge& bridge = RustBridge::Instance();
    bridge.SetModernTone(settings.modernTone);
    bridge.SetEnglishAutoRestore(settings.englishAutoRestore);
    bridge.SetAutoCapitalize(settings.autoCapitalize);
    bridge.SetEscRestore(settings.escRestore);
    bridge.SetFreeTone(settings.freeTone);
    bridge.SetSkipWShortcut(settings.skipWShortcut);
    bridge.SetBracketShortcut(settings.bracketShortcut);

    TextSender::Instance().SetSlowMode(settings.slowMode);
    TextSender::Instance().SetClipboardMode(settings.clipboardMode);

    // Sync excluded apps to AppDetector
    AppDetector::Instance().SetExcludedApps(settings.excludedApps);

    UpdateShortcuts();
}

void ImeProcessor::UpdateShortcuts() {
    const auto& shortcuts = Settings::Instance().shortcuts;

    // Update native shortcut manager (for SPACE expansion)
    ShortcutManager::Instance().SetShortcuts(shortcuts);

    // Sync shortcuts to Rust engine (for punctuation expansion)
    RustBridge& bridge = RustBridge::Instance();
    bridge.ClearShortcuts();
    for (const auto& s : shortcuts) {
        if (!s.key.empty() && !s.value.empty()) {
            bridge.AddShortcut(s.key.c_str(), s.value.c_str());
        }
    }
}

void ImeProcessor::CheckAppChange() {
    Settings& settings = Settings::Instance();

    AppDetector& detector = AppDetector::Instance();
    std::wstring currentApp = detector.GetForegroundAppName();

    if (currentApp.empty()) return;

    if (currentApp != m_lastAppName) {
        // App changed - save state for old app, restore state for new app
        if (!m_lastAppName.empty() && settings.smartSwitch) {
            detector.SaveAppState(m_lastAppName, m_enabled);
        }

        m_lastAppName = currentApp;

        // Check if new app is in exclusion list (Feature 3)
        if (detector.IsCurrentAppExcluded()) {
            if (m_enabled) {
                m_enabled = false;
                RustBridge::Instance().SetEnabled(false);
            }
        } else if (settings.smartSwitch) {
            // Restore state for new app (Feature 2)
            bool newState = detector.GetAppState(currentApp, settings.enabled);
            if (newState != m_enabled) {
                m_enabled = newState;
                RustBridge::Instance().SetEnabled(newState);
            }
        }

        // Apply per-app encoding (Feature 8)
        int encoding = detector.GetAppEncoding(currentApp, 0);
        TextSender::Instance().SetOutputEncoding(static_cast<OutputEncoding>(encoding));
    }
}

void ImeProcessor::OnKeyPressed(KeyEventData& event) {
    // Check for app changes (smart switch)
    CheckAppChange();

    if (!m_enabled) {
        event.handled = false;
        return;
    }

    int vk = event.vkCode;

    // Debug: log key presses to file
    static FILE* debugFile = nullptr;
    if (!debugFile) {
        debugFile = fopen("D:\\CK\\ViKey\\app-native\\debug.log", "w");
    }
    if (debugFile) {
        fprintf(debugFile, "Key: 0x%02X, shift=%d, caps=%d\n", vk, event.shift, event.capsLock);
        fflush(debugFile);
    }

    // Handle backspace for shortcut buffer
    if (vk == VK_BACK_KEY) {
        ShortcutManager::Instance().OnBackspace();
    }

    // Check shortcut expansion on Space
    if (vk == VK_SPACE_KEY) {
        auto [expansion, len] = ShortcutManager::Instance().CheckExpansion();
        if (!expansion.empty() && len > 0) {
            // Replace shortcut with expansion (delete shortcut + send expansion + space)
            TextSender::Instance().SendText(expansion + L" ", static_cast<int>(len));
            event.handled = true;
            return;
        }
    }

    // Convert VK code to macOS-style keycode
    uint16_t macKeycode = KeyCodes::ToMacKeycode(vk);
    if (macKeycode == 0xFFFF) {
        event.handled = false;
        return;
    }

    // Track typed character for shortcut matching
    char typedChar = KeyCodes::ToChar(vk, event.shift, event.capsLock);
    if (typedChar != 0) {
        ShortcutManager::Instance().OnChar(typedChar);
    }

    // Determine caps state (XOR of Shift and CapsLock)
    bool caps = event.shift ^ event.capsLock;

    // Process through Rust engine
    ImeResult result = RustBridge::Instance().ProcessKeyExt(macKeycode, caps, false, event.shift);

    // Debug: log Rust result
    if (debugFile) {
        fprintf(debugFile, "  -> action=%d, count=%d, bs=%d, consumed=%d\n",
                (int)result.action, result.count, result.backspace, result.IsKeyConsumed() ? 1 : 0);
        fflush(debugFile);
    }

    if (result.action == ImeAction::Send && result.count > 0) {
        // Send replacement text
        std::wstring text = result.GetText();
        if (debugFile) {
            fprintf(debugFile, "  -> Sending text (len=%zu), backspaces=%d\n",
                    text.length(), result.backspace);
            fflush(debugFile);
        }
        TextSender::Instance().SendText(text, result.backspace);
        event.handled = true;
    } else if (result.IsKeyConsumed()) {
        // Key consumed but no text to send
        event.handled = true;
    } else {
        event.handled = false;
    }
}
