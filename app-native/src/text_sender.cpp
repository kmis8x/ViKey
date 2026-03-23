// ViKey - Text Sender Implementation
// text_sender.cpp
// Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey

#include "text_sender.h"
#include "keyboard_hook.h"
#include "keycodes.h"
#include "encoding_converter.h"
#include <vector>

// Win32 Constants
constexpr DWORD INPUT_KEYBOARD_TYPE = 1;
constexpr DWORD KEYEVENTF_KEYUP_FLAG = 0x0002;
constexpr DWORD KEYEVENTF_UNICODE_FLAG = 0x0004;

// INPUT struct for SendInput (explicit layout for 64-bit compatibility)
#pragma pack(push, 8)
struct INPUT_DATA {
    DWORD type;
    union {
        struct {
            WORD wVk;
            WORD wScan;
            DWORD dwFlags;
            DWORD time;
            ULONG_PTR dwExtraInfo;
            DWORD padding[2]; // Padding for 64-bit alignment
        } ki;
    };
};
#pragma pack(pop)

TextSender& TextSender::Instance() {
    static TextSender instance;
    return instance;
}

TextSender::TextSender() : m_slowMode(false), m_clipboardMode(false), m_outputEncoding(OutputEncoding::Unicode) {}

void TextSender::SendText(const std::wstring& text, int backspaces) {
    if (text.empty() && backspaces == 0) return;

    // Convert text if needed (Feature 8: App Encoding Memory)
    std::wstring outputText = text;
    if (m_outputEncoding != OutputEncoding::Unicode && !text.empty()) {
        VietEncoding targetEnc = (m_outputEncoding == OutputEncoding::VNI) ?
            VietEncoding::VNI_Windows : VietEncoding::TCVN3;
        outputText = EncodingConverter::Instance().Convert(text, VietEncoding::Unicode, targetEnc);
    }

    if (m_clipboardMode) {
        SendTextClipboard(outputText, backspaces);
    } else if (m_slowMode) {
        SendTextSlow(outputText, backspaces);
    } else {
        SendTextFast(outputText, backspaces);
    }
}

void TextSender::SendTextFast(const std::wstring& text, int backspaces) {
    // Combine backspaces + Unicode chars into a SINGLE SendInput call.
    // This is atomic: no gaps between events, preventing race conditions
    // where the hook callback re-enters during Sleep() pauses.
    std::vector<INPUT> inputs;
    inputs.reserve(backspaces * 2 + text.length() * 2);

    // Backspace down/up pairs
    for (int i = 0; i < backspaces; i++) {
        INPUT down = {};
        down.type = INPUT_KEYBOARD;
        down.ki.wVk = VK_BACK;
        down.ki.wScan = 0x0E;
        down.ki.dwFlags = 0;
        down.ki.dwExtraInfo = INJECTED_KEY_MARKER;
        inputs.push_back(down);

        INPUT up = {};
        up.type = INPUT_KEYBOARD;
        up.ki.wVk = VK_BACK;
        up.ki.wScan = 0x0E;
        up.ki.dwFlags = KEYEVENTF_KEYUP_FLAG;
        up.ki.dwExtraInfo = INJECTED_KEY_MARKER;
        inputs.push_back(up);
    }

    // Unicode char down/up pairs
    for (wchar_t c : text) {
        if (c >= 0xD800 && c <= 0xDBFF) continue;

        INPUT down = {};
        down.type = INPUT_KEYBOARD;
        down.ki.wVk = 0;
        down.ki.wScan = c;
        down.ki.dwFlags = KEYEVENTF_UNICODE_FLAG;
        down.ki.dwExtraInfo = INJECTED_KEY_MARKER;
        inputs.push_back(down);

        INPUT up = {};
        up.type = INPUT_KEYBOARD;
        up.ki.wVk = 0;
        up.ki.wScan = c;
        up.ki.dwFlags = KEYEVENTF_UNICODE_FLAG | KEYEVENTF_KEYUP_FLAG;
        up.ki.dwExtraInfo = INJECTED_KEY_MARKER;
        inputs.push_back(up);
    }

    if (!inputs.empty()) {
        SendInput(static_cast<UINT>(inputs.size()), inputs.data(), sizeof(INPUT));
    }
}

void TextSender::SendTextSlow(const std::wstring& text, int backspaces) {
    // Use keybd_event for backspaces with longer delays
    for (int i = 0; i < backspaces; i++) {
        keybd_event(VK_BACK, 0x0E, 0, INJECTED_KEY_MARKER);
        Sleep(15);
        keybd_event(VK_BACK, 0x0E, KEYEVENTF_KEYUP_FLAG, INJECTED_KEY_MARKER);
        Sleep(15);
    }

    // Longer delay between backspaces and text
    if (backspaces > 0) {
        Sleep(30);
    }

    // Send Unicode characters using SendInput
    INPUT input[2] = {};
    input[0].type = INPUT_KEYBOARD;
    input[1].type = INPUT_KEYBOARD;

    for (wchar_t c : text) {
        if (c >= 0xD800 && c <= 0xDBFF) continue;

        input[0].ki.wVk = 0;
        input[0].ki.wScan = c;
        input[0].ki.dwFlags = KEYEVENTF_UNICODE_FLAG;
        input[0].ki.dwExtraInfo = INJECTED_KEY_MARKER;

        input[1].ki.wVk = 0;
        input[1].ki.wScan = c;
        input[1].ki.dwFlags = KEYEVENTF_UNICODE_FLAG | KEYEVENTF_KEYUP_FLAG;
        input[1].ki.dwExtraInfo = INJECTED_KEY_MARKER;

        SendInput(2, input, sizeof(INPUT));
        Sleep(15);
    }
}

// Clipboard mode: use clipboard + Ctrl+V for stubborn apps (Feature 4)
void TextSender::SendTextClipboard(const std::wstring& text, int backspaces) {
    // Step 1: Send backspaces as a single batch SendInput
    if (backspaces > 0) {
        std::vector<INPUT> bsInputs;
        bsInputs.reserve(backspaces * 2);
        for (int i = 0; i < backspaces; i++) {
            INPUT down = {};
            down.type = INPUT_KEYBOARD;
            down.ki.wVk = VK_BACK;
            down.ki.wScan = 0x0E;
            down.ki.dwFlags = 0;
            down.ki.dwExtraInfo = INJECTED_KEY_MARKER;
            bsInputs.push_back(down);

            INPUT up = {};
            up.type = INPUT_KEYBOARD;
            up.ki.wVk = VK_BACK;
            up.ki.wScan = 0x0E;
            up.ki.dwFlags = KEYEVENTF_KEYUP_FLAG;
            up.ki.dwExtraInfo = INJECTED_KEY_MARKER;
            bsInputs.push_back(up);
        }
        SendInput(static_cast<UINT>(bsInputs.size()), bsInputs.data(), sizeof(INPUT));
    }

    if (text.empty()) return;

    // Delay between backspaces and clipboard paste
    if (backspaces > 0) {
        Sleep(20);
    }

    // Step 2: Save previous clipboard content, set our text
    if (!OpenClipboard(nullptr)) return;

    // Save previous clipboard content
    HGLOBAL hSaved = nullptr;
    if (IsClipboardFormatAvailable(CF_UNICODETEXT)) {
        HGLOBAL hClip = GetClipboardData(CF_UNICODETEXT);
        if (hClip) {
            SIZE_T clipSize = GlobalSize(hClip);
            hSaved = GlobalAlloc(GMEM_MOVEABLE, clipSize);
            if (hSaved) {
                void* pSrc = GlobalLock(hClip);
                void* pDst = GlobalLock(hSaved);
                if (pSrc && pDst) {
                    memcpy(pDst, pSrc, clipSize);
                }
                GlobalUnlock(hClip);
                GlobalUnlock(hSaved);
            }
        }
    }

    // Set our text to clipboard
    EmptyClipboard();
    size_t size = (text.length() + 1) * sizeof(wchar_t);
    HGLOBAL hGlobal = GlobalAlloc(GMEM_MOVEABLE, size);
    if (hGlobal) {
        wchar_t* pGlobal = static_cast<wchar_t*>(GlobalLock(hGlobal));
        if (pGlobal) {
            wcscpy_s(pGlobal, text.length() + 1, text.c_str());
            GlobalUnlock(hGlobal);
            SetClipboardData(CF_UNICODETEXT, hGlobal);
        } else {
            GlobalFree(hGlobal);
        }
    }
    CloseClipboard();

    // Step 3: Send Ctrl+V
    Sleep(10);
    keybd_event(VK_CONTROL, 0x1D, 0, INJECTED_KEY_MARKER);
    Sleep(5);
    keybd_event('V', 0x2F, 0, INJECTED_KEY_MARKER);
    Sleep(10);
    keybd_event('V', 0x2F, KEYEVENTF_KEYUP_FLAG, INJECTED_KEY_MARKER);
    Sleep(5);
    keybd_event(VK_CONTROL, 0x1D, KEYEVENTF_KEYUP_FLAG, INJECTED_KEY_MARKER);

    // Step 4: Restore previous clipboard content (or clear ViKey's text)
    // Wait for Ctrl+V to be fully processed by the target application
    Sleep(150);

    // Retry OpenClipboard up to 3 times (another app may hold the clipboard briefly)
    bool clipboardOpened = false;
    for (int attempt = 0; attempt < 3; attempt++) {
        if (OpenClipboard(nullptr)) {
            clipboardOpened = true;
            break;
        }
        Sleep(50);
    }

    if (clipboardOpened) {
        EmptyClipboard();
        if (hSaved) {
            // Restore the user's previous clipboard content
            if (!SetClipboardData(CF_UNICODETEXT, hSaved)) {
                GlobalFree(hSaved);
            }
            hSaved = nullptr; // Ownership transferred to clipboard or freed
        }
        // If hSaved was null (clipboard was empty), EmptyClipboard already
        // cleared ViKey's text — user won't accidentally paste it later
        CloseClipboard();
    } else if (hSaved) {
        GlobalFree(hSaved);
    }
}
