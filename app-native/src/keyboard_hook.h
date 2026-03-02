// ViKey - Low-level Keyboard Hook
// keyboard_hook.h
// Uses SetWindowsHookEx with WH_KEYBOARD_LL

#pragma once

#include <windows.h>
#include <functional>
#include <cstdint>

// Unique marker for injected keys (prevents recursion) - "VNIM" in hex
constexpr ULONG_PTR INJECTED_KEY_MARKER = 0x564E494D;

// Key event data
struct KeyEventData {
    int vkCode;
    bool shift;
    bool capsLock;
    bool handled;

    KeyEventData(int vk, bool s, bool caps)
        : vkCode(vk), shift(s), capsLock(caps), handled(false) {}
};

// Callback function type for key events
using KeyPressedCallback = std::function<void(KeyEventData&)>;

// Forward declaration
static LRESULT CALLBACK GlobalLowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam);

class KeyboardHook {
    friend LRESULT CALLBACK GlobalLowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam);
public:
    static KeyboardHook& Instance();

    // Start the keyboard hook
    bool Start();

    // Stop the keyboard hook
    void Stop();

    // Check if hook is active
    bool IsActive() const { return m_hookId != nullptr; }

    // Set callback for key events
    void SetCallback(KeyPressedCallback callback) { m_callback = callback; }

private:
    KeyboardHook();
    ~KeyboardHook();
    KeyboardHook(const KeyboardHook&) = delete;
    KeyboardHook& operator=(const KeyboardHook&) = delete;

    // Process key event
    LRESULT ProcessKey(int nCode, WPARAM wParam, LPARAM lParam);

    // Check key states
    static bool IsKeyDown(int vKey);
    static bool IsCapsLockOn();

    HHOOK m_hookId;
    bool m_isProcessing;
    KeyPressedCallback m_callback;
};
