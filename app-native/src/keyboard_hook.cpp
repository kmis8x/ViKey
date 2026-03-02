// ViKey - Low-level Keyboard Hook Implementation
// keyboard_hook.cpp
// Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey

#include "keyboard_hook.h"
#include "keycodes.h"
#include "rust_bridge.h"

// Win32 Constants
// WH_KEYBOARD_LL is defined in Windows.h as 13
#ifndef WH_KEYBOARD_LL
#define WH_KEYBOARD_LL 13
#endif
constexpr int WM_KEYDOWN_MSG = 0x0100;
constexpr int WM_SYSKEYDOWN_MSG = 0x0104;
constexpr DWORD LLKHF_INJECTED_FLAG = 0x10;

// KBDLLHOOKSTRUCT structure
struct KBDLLHOOKSTRUCT_DATA {
    DWORD vkCode;
    DWORD scanCode;
    DWORD flags;
    DWORD time;
    ULONG_PTR dwExtraInfo;
};

// Static instance pointer for callback
static KeyboardHook* g_instance = nullptr;

// Global keyboard hook procedure (not a class member)
// Use __declspec(noinline) to prevent optimization that might affect the callback
__declspec(noinline) static LRESULT CALLBACK GlobalLowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    if (g_instance) {
        return g_instance->ProcessKey(nCode, wParam, lParam);
    }
    return CallNextHookEx(nullptr, nCode, wParam, lParam);
}

KeyboardHook& KeyboardHook::Instance() {
    static KeyboardHook instance;
    return instance;
}

KeyboardHook::KeyboardHook()
    : m_hookId(nullptr)
    , m_isProcessing(false)
    , m_callback(nullptr) {
    g_instance = this;
}

KeyboardHook::~KeyboardHook() {
    Stop();
    g_instance = nullptr;
}

bool KeyboardHook::Start() {
    if (m_hookId != nullptr) return true;

    SetLastError(0);

    // For WH_KEYBOARD_LL, use user32.dll module handle
    HMODULE hMod = LoadLibraryW(L"user32.dll");

    m_hookId = SetWindowsHookExW(
        WH_KEYBOARD_LL,
        GlobalLowLevelKeyboardProc,
        hMod,
        0
    );

    if (m_hookId == nullptr) {
        MessageBoxW(nullptr, L"Failed to install keyboard hook!", L"Hook Error", MB_ICONERROR);
    }

    return m_hookId != nullptr;
}

void KeyboardHook::Stop() {
    if (m_hookId != nullptr) {
        UnhookWindowsHookEx(m_hookId);
        m_hookId = nullptr;
    }
}

LRESULT KeyboardHook::ProcessKey(int nCode, WPARAM wParam, LPARAM lParam) {
    // Prevent recursion
    if (m_isProcessing) {
        return CallNextHookEx(m_hookId, nCode, wParam, lParam);
    }

    // Only process key down events
    if (nCode >= 0 && (wParam == WM_KEYDOWN_MSG || wParam == WM_SYSKEYDOWN_MSG)) {
        auto* hookStruct = reinterpret_cast<KBDLLHOOKSTRUCT_DATA*>(lParam);

        // Skip ONLY our own injected keys (identified by our marker)
        // Don't skip other injected keys - they may come from remote desktop/AnyDesk
        if (hookStruct->dwExtraInfo == INJECTED_KEY_MARKER) {
            return CallNextHookEx(m_hookId, nCode, wParam, lParam);
        }

        int vkCode = static_cast<int>(hookStruct->vkCode);

        // Clear buffer on Ctrl key press
        if (vkCode == VK_CONTROL_KEY) {
            RustBridge::Instance().Clear();
            return CallNextHookEx(m_hookId, nCode, wParam, lParam);
        }

        // Only process relevant keys
        if (KeyCodes::IsRelevantKey(vkCode)) {
            bool shift = IsKeyDown(VK_SHIFT_KEY);
            bool capsLock = IsCapsLockOn();
            bool ctrl = IsKeyDown(VK_CONTROL_KEY);
            bool alt = IsKeyDown(VK_MENU_KEY);

            // Skip Ctrl/Alt combinations (shortcuts)
            if (ctrl || alt) {
                if (ctrl) RustBridge::Instance().Clear();
                return CallNextHookEx(m_hookId, nCode, wParam, lParam);
            }

            // Clear buffer on word boundary keys (except Space which needs shortcut check)
            bool isBufferClearKey = KeyCodes::IsBufferClearKey(vkCode);
            if (isBufferClearKey && vkCode != VK_SPACE_KEY) {
                RustBridge::Instance().Clear();
                return CallNextHookEx(m_hookId, nCode, wParam, lParam);
            }

            // Process through callback if set
            if (m_callback) {
                KeyEventData event(vkCode, shift, capsLock);

                m_isProcessing = true;
                m_callback(event);
                m_isProcessing = false;

                // Clear buffer after processing word boundary keys
                if (isBufferClearKey) {
                    RustBridge::Instance().Clear();
                }

                // Block original key if handled
                if (event.handled) {
                    return 1;
                }
            }
        }
    }

    return CallNextHookEx(m_hookId, nCode, wParam, lParam);
}

bool KeyboardHook::IsKeyDown(int vKey) {
    return (GetAsyncKeyState(vKey) & 0x8000) != 0;
}

bool KeyboardHook::IsCapsLockOn() {
    return (GetKeyState(VK_CAPITAL_KEY) & 0x0001) != 0;
}
