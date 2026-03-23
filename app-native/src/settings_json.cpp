// ViKey - Settings JSON Helpers
// settings_json.cpp
// JSON escape/parse functions and Settings::ImportFromJson, ExportToJson, ExportShortcutsToJson, ImportShortcutsFromJson

#include "settings.h"
#include <shlwapi.h>
#include <sstream>
#include <vector>

#pragma comment(lib, "shlwapi.lib")

// JSON helpers
static std::wstring EscapeJsonString(const std::wstring& s) {
    std::wstring result;
    result.reserve(s.size());
    for (wchar_t c : s) {
        if (c == L'\\') result += L"\\\\";
        else if (c == L'"') result += L"\\\"";
        else result += c;
    }
    return result;
}

static void WriteShortcutsJsonArray(std::wstringstream& ss, const std::vector<TextShortcut>& shortcuts) {
    for (size_t i = 0; i < shortcuts.size(); i++) {
        ss << L"    {\"key\": \"" << EscapeJsonString(shortcuts[i].key)
           << L"\", \"value\": \"" << EscapeJsonString(shortcuts[i].value) << L"\"}";
        if (i < shortcuts.size() - 1) ss << L",";
        ss << L"\n";
    }
}

// Forward declaration
static std::wstring ExtractJsonString(const std::wstring& json, const std::wstring& key);

static std::vector<TextShortcut> ParseShortcutsJsonArray(const std::wstring& json, const std::wstring& key) {
    std::vector<TextShortcut> result;
    size_t shortcutsPos = json.find(L"\"" + key + L"\":");
    if (shortcutsPos == std::wstring::npos) return result;
    size_t arrStart = json.find(L"[", shortcutsPos);
    size_t arrEnd = json.find(L"]", arrStart);
    if (arrStart == std::wstring::npos || arrEnd == std::wstring::npos) return result;
    std::wstring arrSection = json.substr(arrStart, arrEnd - arrStart + 1);
    size_t pos = 0;
    while ((pos = arrSection.find(L"{", pos)) != std::wstring::npos) {
        size_t objEnd = arrSection.find(L"}", pos);
        if (objEnd == std::wstring::npos) break;
        std::wstring obj = arrSection.substr(pos, objEnd - pos + 1);
        std::wstring k = ExtractJsonString(obj, L"key");
        std::wstring v = ExtractJsonString(obj, L"value");
        if (!k.empty() && !v.empty()) {
            result.push_back({k, v});
        }
        pos = objEnd + 1;
    }
    return result;
}

static std::wstring ExtractJsonString(const std::wstring& json, const std::wstring& key) {
    std::wstring searchKey = L"\"" + key + L"\":";
    size_t pos = json.find(searchKey);
    if (pos == std::wstring::npos) return L"";
    pos += searchKey.length();
    while (pos < json.length() && (json[pos] == L' ' || json[pos] == L'\t')) pos++;
    if (pos >= json.length()) return L"";
    if (json[pos] == L'"') {
        pos++;
        std::wstring result;
        while (pos < json.length() && json[pos] != L'"') {
            if (json[pos] == L'\\' && pos + 1 < json.length()) {
                pos++;
                if (json[pos] == L'n') result += L'\n';
                else if (json[pos] == L't') result += L'\t';
                else result += json[pos];
            } else {
                result += json[pos];
            }
            pos++;
        }
        return result;
    }
    return L"";
}

static bool ExtractJsonBool(const std::wstring& json, const std::wstring& key, bool defaultVal) {
    std::wstring searchKey = L"\"" + key + L"\":";
    size_t pos = json.find(searchKey);
    if (pos == std::wstring::npos) return defaultVal;
    pos += searchKey.length();
    while (pos < json.length() && (json[pos] == L' ' || json[pos] == L'\t')) pos++;
    if (pos + 4 <= json.length() && json.substr(pos, 4) == L"true") return true;
    if (pos + 5 <= json.length() && json.substr(pos, 5) == L"false") return false;
    return defaultVal;
}

static int ExtractJsonInt(const std::wstring& json, const std::wstring& key, int defaultVal) {
    std::wstring searchKey = L"\"" + key + L"\":";
    size_t pos = json.find(searchKey);
    if (pos == std::wstring::npos) return defaultVal;
    pos += searchKey.length();
    while (pos < json.length() && (json[pos] == L' ' || json[pos] == L'\t')) pos++;
    std::wstring numStr;
    while (pos < json.length() && (json[pos] >= L'0' && json[pos] <= L'9')) {
        numStr += json[pos++];
    }
    if (numStr.empty()) return defaultVal;
    return std::stoi(numStr);
}

// JSON Export/Import
std::wstring Settings::ExportToJson() const {
    std::wstringstream ss;
    ss << L"{\n";
    ss << L"  \"version\": 1,\n";
    ss << L"  \"settings\": {\n";
    ss << L"    \"enabled\": " << (enabled ? L"true" : L"false") << L",\n";
    ss << L"    \"method\": " << static_cast<int>(method) << L",\n";
    ss << L"    \"modernTone\": " << (modernTone ? L"true" : L"false") << L",\n";
    ss << L"    \"englishAutoRestore\": " << (englishAutoRestore ? L"true" : L"false") << L",\n";
    ss << L"    \"autoCapitalize\": " << (autoCapitalize ? L"true" : L"false") << L",\n";
    ss << L"    \"escRestore\": " << (escRestore ? L"true" : L"false") << L",\n";
    ss << L"    \"freeTone\": " << (freeTone ? L"true" : L"false") << L",\n";
    ss << L"    \"allowForeignConsonants\": " << (allowForeignConsonants ? L"true" : L"false") << L",\n";
    ss << L"    \"skipWShortcut\": " << (skipWShortcut ? L"true" : L"false") << L",\n";
    ss << L"    \"bracketShortcut\": " << (bracketShortcut ? L"true" : L"false") << L",\n";
    ss << L"    \"slowMode\": " << (slowMode ? L"true" : L"false") << L",\n";
    ss << L"    \"clipboardMode\": " << (clipboardMode ? L"true" : L"false") << L",\n";
    ss << L"    \"smartSwitch\": " << (smartSwitch ? L"true" : L"false") << L",\n";
    ss << L"    \"autoStart\": " << (autoStart ? L"true" : L"false") << L",\n";
    ss << L"    \"silentStartup\": " << (silentStartup ? L"true" : L"false") << L"\n";
    ss << L"  },\n";
    ss << L"  \"hotkey\": {\n";
    ss << L"    \"ctrl\": " << (toggleHotkey.ctrl ? L"true" : L"false") << L",\n";
    ss << L"    \"shift\": " << (toggleHotkey.shift ? L"true" : L"false") << L",\n";
    ss << L"    \"alt\": " << (toggleHotkey.alt ? L"true" : L"false") << L",\n";
    ss << L"    \"win\": " << (toggleHotkey.win ? L"true" : L"false") << L",\n";
    ss << L"    \"key\": " << toggleHotkey.vkCode << L"\n";
    ss << L"  },\n";
    ss << L"  \"excludedApps\": [\n";
    for (size_t i = 0; i < excludedApps.size(); i++) {
        ss << L"    \"" << excludedApps[i] << L"\"";
        if (i < excludedApps.size() - 1) ss << L",";
        ss << L"\n";
    }
    ss << L"  ],\n";
    ss << L"  \"shortcuts\": [\n";
    WriteShortcutsJsonArray(ss, shortcuts);
    ss << L"  ]\n";
    ss << L"}\n";
    return ss.str();
}

bool Settings::ImportFromJson(const std::wstring& json) {
    int version = ExtractJsonInt(json, L"version", 0);
    if (version != 1) return false;

    size_t settingsPos = json.find(L"\"settings\":");
    if (settingsPos == std::wstring::npos) return false;
    size_t settingsEnd = json.find(L"}", settingsPos);
    if (settingsEnd == std::wstring::npos) return false;
    std::wstring settingsSection = json.substr(settingsPos, settingsEnd - settingsPos + 1);

    enabled = ExtractJsonBool(settingsSection, L"enabled", true);
    int methodInt = ExtractJsonInt(settingsSection, L"method", 0);
    method = (methodInt >= 0 && methodInt <= 1) ? static_cast<InputMethod>(methodInt) : InputMethod::Telex;
    modernTone = ExtractJsonBool(settingsSection, L"modernTone", true);
    englishAutoRestore = ExtractJsonBool(settingsSection, L"englishAutoRestore", true);
    autoCapitalize = ExtractJsonBool(settingsSection, L"autoCapitalize", false);
    escRestore = ExtractJsonBool(settingsSection, L"escRestore", true);
    freeTone = ExtractJsonBool(settingsSection, L"freeTone", false);
    allowForeignConsonants = ExtractJsonBool(settingsSection, L"allowForeignConsonants", false);
    skipWShortcut = ExtractJsonBool(settingsSection, L"skipWShortcut", false);
    bracketShortcut = ExtractJsonBool(settingsSection, L"bracketShortcut", false);
    slowMode = ExtractJsonBool(settingsSection, L"slowMode", false);
    clipboardMode = ExtractJsonBool(settingsSection, L"clipboardMode", false);
    smartSwitch = ExtractJsonBool(settingsSection, L"smartSwitch", false);
    autoStart = ExtractJsonBool(settingsSection, L"autoStart", false);
    silentStartup = ExtractJsonBool(settingsSection, L"silentStartup", false);

    size_t hotkeyPos = json.find(L"\"hotkey\":");
    if (hotkeyPos != std::wstring::npos) {
        size_t hotkeyEnd = json.find(L"}", hotkeyPos);
        if (hotkeyEnd == std::wstring::npos) return false;
        std::wstring hotkeySection = json.substr(hotkeyPos, hotkeyEnd - hotkeyPos + 1);
        toggleHotkey.ctrl = ExtractJsonBool(hotkeySection, L"ctrl", true);
        toggleHotkey.shift = ExtractJsonBool(hotkeySection, L"shift", false);
        toggleHotkey.alt = ExtractJsonBool(hotkeySection, L"alt", false);
        toggleHotkey.win = ExtractJsonBool(hotkeySection, L"win", false);
        toggleHotkey.vkCode = static_cast<UINT>(ExtractJsonInt(hotkeySection, L"key", VK_SPACE));
    }

    excludedApps.clear();
    size_t excludedPos = json.find(L"\"excludedApps\":");
    if (excludedPos != std::wstring::npos) {
        size_t arrStart = json.find(L"[", excludedPos);
        size_t arrEnd = json.find(L"]", arrStart);
        if (arrStart != std::wstring::npos && arrEnd != std::wstring::npos) {
            std::wstring arrSection = json.substr(arrStart + 1, arrEnd - arrStart - 1);
            size_t pos = 0;
            while ((pos = arrSection.find(L"\"", pos)) != std::wstring::npos) {
                size_t endQuote = arrSection.find(L"\"", pos + 1);
                if (endQuote == std::wstring::npos) break;
                std::wstring app = arrSection.substr(pos + 1, endQuote - pos - 1);
                if (!app.empty()) {
                    excludedApps.push_back(app);
                }
                pos = endQuote + 1;
            }
        }
    }

    shortcuts = ParseShortcutsJsonArray(json, L"shortcuts");

    return true;
}

std::wstring Settings::ExportShortcutsToJson() const {
    std::wstringstream ss;
    ss << L"{\n";
    ss << L"  \"version\": 1,\n";
    ss << L"  \"shortcuts\": [\n";
    WriteShortcutsJsonArray(ss, shortcuts);
    ss << L"  ]\n";
    ss << L"}\n";
    return ss.str();
}

bool Settings::ImportShortcutsFromJson(const std::wstring& json) {
    int version = ExtractJsonInt(json, L"version", 0);
    if (version != 1) return false;

    auto newShortcuts = ParseShortcutsJsonArray(json, L"shortcuts");
    if (newShortcuts.empty()) return false;
    shortcuts = newShortcuts;
    return true;
}
