// ViKey - Shortcut Manager Implementation
// shortcut_manager.cpp

#include "shortcut_manager.h"
#include <algorithm>
#include <cctype>

ShortcutManager& ShortcutManager::Instance() {
    static ShortcutManager instance;
    return instance;
}

ShortcutManager::ShortcutManager() {}

void ShortcutManager::SetShortcuts(const std::vector<TextShortcut>& shortcuts) {
    m_shortcuts.clear();
    for (const auto& s : shortcuts) {
        if (!s.key.empty() && !s.value.empty()) {
            m_shortcuts.push_back(s);
        }
    }
}

void ShortcutManager::OnChar(char c) {
    // Allow letters, digits, and hyphens for shortcuts like --danger
    if (std::isalnum(static_cast<unsigned char>(c)) || c == '-') {
        if (m_buffer.length() < MAX_BUFFER_LENGTH) {
            m_buffer += static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
        }
    } else {
        // Non-alphanumeric clears buffer (except space which triggers expansion)
        m_buffer.clear();
    }
}

void ShortcutManager::OnBackspace() {
    if (!m_buffer.empty()) {
        m_buffer.pop_back();
    }
}

void ShortcutManager::Clear() {
    m_buffer.clear();
}

std::pair<std::wstring, size_t> ShortcutManager::CheckExpansion() {
    if (m_buffer.empty() || m_shortcuts.empty()) {
        return {L"", 0};
    }

    // Convert buffer to wide string for comparison
    std::wstring wbuffer(m_buffer.begin(), m_buffer.end());

    // Find matching shortcut (case-insensitive)
    for (const auto& shortcut : m_shortcuts) {
        // Convert shortcut key to lowercase for comparison
        std::wstring lowerKey = shortcut.key;
        std::transform(lowerKey.begin(), lowerKey.end(), lowerKey.begin(),
            [](wchar_t c) { return static_cast<wchar_t>(std::tolower(c)); });

        if (lowerKey == wbuffer) {
            size_t len = m_buffer.length();
            m_buffer.clear();
            return {shortcut.value, len};
        }
    }

    m_buffer.clear();
    return {L"", 0};
}
