// ViKey - Text Sender
// text_sender.h
// Sends text using Win32 SendInput API with KEYEVENTF_UNICODE

#pragma once

#include <windows.h>
#include <string>

// Output encoding for per-app encoding (Feature 8)
enum class OutputEncoding {
    Unicode = 0,
    VNI = 1,
    TCVN3 = 2
};

class TextSender {
public:
    static TextSender& Instance();

    // Slow mode: send events individually with small delays (for terminals)
    void SetSlowMode(bool slow) { m_slowMode = slow; }
    bool IsSlowMode() const { return m_slowMode; }

    // Clipboard mode: use clipboard + Ctrl+V for stubborn apps (Feature 4)
    void SetClipboardMode(bool clipboard) { m_clipboardMode = clipboard; }
    bool IsClipboardMode() const { return m_clipboardMode; }

    // Output encoding for per-app encoding (Feature 8)
    void SetOutputEncoding(OutputEncoding enc) { m_outputEncoding = enc; }
    OutputEncoding GetOutputEncoding() const { return m_outputEncoding; }

    // Send text replacement: delete characters then insert new text
    void SendText(const std::wstring& text, int backspaces);

    // Clipboard mode: use clipboard + Ctrl+V (for stubborn apps)
    void SendTextClipboard(const std::wstring& text, int backspaces);

private:
    TextSender();
    ~TextSender() = default;
    TextSender(const TextSender&) = delete;
    TextSender& operator=(const TextSender&) = delete;

    // Fast mode: batch all events (default)
    void SendTextFast(const std::wstring& text, int backspaces);

    // Slow mode: send events one by one with delays (for problematic apps)
    void SendTextSlow(const std::wstring& text, int backspaces);

    bool m_slowMode;
    bool m_clipboardMode;
    OutputEncoding m_outputEncoding;
};
