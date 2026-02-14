// ViKey - Settings Manager Implementation
// settings.cpp
// Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey

#include "settings.h"
#include <shlwapi.h>
#include <sstream>
#include <vector>

#pragma comment(lib, "shlwapi.lib")

Settings& Settings::Instance() {
    static Settings instance;
    return instance;
}

Settings::Settings()
    : enabled(true)
    , method(InputMethod::Telex)
    , modernTone(true)
    , englishAutoRestore(true)
    , autoCapitalize(false)
    , escRestore(true)
    , freeTone(false)
    , allowForeignConsonants(false)
    , skipWShortcut(false)
    , bracketShortcut(false)
    , slowMode(false)
    , clipboardMode(false)
    , smartSwitch(false)
    , autoStart(false)
    , silentStartup(false)
    , checkForUpdates(true) {
}

// Batch registry helpers (single key open for all reads/writes)
static bool ReadBool(HKEY hKey, const wchar_t* name, bool defaultValue) {
    DWORD value = 0, size = sizeof(value), type = REG_DWORD;
    if (RegQueryValueExW(hKey, name, nullptr, &type, (LPBYTE)&value, &size) == ERROR_SUCCESS)
        return value != 0;
    return defaultValue;
}

static int ReadInt(HKEY hKey, const wchar_t* name, int defaultValue) {
    DWORD value = 0, size = sizeof(value), type = REG_DWORD;
    if (RegQueryValueExW(hKey, name, nullptr, &type, (LPBYTE)&value, &size) == ERROR_SUCCESS)
        return static_cast<int>(value);
    return defaultValue;
}

static void WriteBool(HKEY hKey, const wchar_t* name, bool value) {
    DWORD dw = value ? 1 : 0;
    RegSetValueExW(hKey, name, 0, REG_DWORD, (LPBYTE)&dw, sizeof(dw));
}

static void WriteInt(HKEY hKey, const wchar_t* name, int value) {
    DWORD dw = static_cast<DWORD>(value);
    RegSetValueExW(hKey, name, 0, REG_DWORD, (LPBYTE)&dw, sizeof(dw));
}

void Settings::Load() {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REGISTRY_PATH, 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        enabled = ReadBool(hKey, L"Enabled", true);
        int methodInt = ReadInt(hKey, L"Method", 0);
        method = (methodInt >= 0 && methodInt <= 1) ? static_cast<InputMethod>(methodInt) : InputMethod::Telex;
        modernTone = ReadBool(hKey, L"ModernTone", true);
        englishAutoRestore = ReadBool(hKey, L"EnglishAutoRestore", true);
        autoCapitalize = ReadBool(hKey, L"AutoCapitalize", false);
        escRestore = ReadBool(hKey, L"EscRestore", true);
        freeTone = ReadBool(hKey, L"FreeTone", false);
        allowForeignConsonants = ReadBool(hKey, L"AllowForeignConsonants", false);
        skipWShortcut = ReadBool(hKey, L"SkipWTextShortcut", false);
        bracketShortcut = ReadBool(hKey, L"BracketTextShortcut", false);
        slowMode = ReadBool(hKey, L"SlowMode", false);
        clipboardMode = ReadBool(hKey, L"ClipboardMode", false);
        smartSwitch = ReadBool(hKey, L"SmartSwitch", false);
        silentStartup = ReadBool(hKey, L"SilentStartup", false);
        shortcutsEnabled = ReadBool(hKey, L"ShortcutsEnabled", true);
        checkForUpdates = ReadBool(hKey, L"CheckForUpdates", true);
        toggleHotkey.ctrl = ReadBool(hKey, L"HotkeyCtrl", true);
        toggleHotkey.shift = ReadBool(hKey, L"HotkeyShift", false);
        toggleHotkey.alt = ReadBool(hKey, L"HotkeyAlt", false);
        toggleHotkey.win = ReadBool(hKey, L"HotkeyWin", false);
        toggleHotkey.vkCode = static_cast<UINT>(ReadInt(hKey, L"HotkeyKey", VK_SPACE));
        RegCloseKey(hKey);
    }
    autoStart = GetAutoStart();
    LoadShortcuts();
    LoadExcludedApps();
}

void Settings::Save() {
    HKEY hKey;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REGISTRY_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) == ERROR_SUCCESS) {
        WriteBool(hKey, L"Enabled", enabled);
        WriteInt(hKey, L"Method", static_cast<int>(method));
        WriteBool(hKey, L"ModernTone", modernTone);
        WriteBool(hKey, L"EnglishAutoRestore", englishAutoRestore);
        WriteBool(hKey, L"AutoCapitalize", autoCapitalize);
        WriteBool(hKey, L"EscRestore", escRestore);
        WriteBool(hKey, L"FreeTone", freeTone);
        WriteBool(hKey, L"AllowForeignConsonants", allowForeignConsonants);
        WriteBool(hKey, L"SkipWTextShortcut", skipWShortcut);
        WriteBool(hKey, L"BracketTextShortcut", bracketShortcut);
        WriteBool(hKey, L"SlowMode", slowMode);
        WriteBool(hKey, L"ClipboardMode", clipboardMode);
        WriteBool(hKey, L"SmartSwitch", smartSwitch);
        WriteBool(hKey, L"SilentStartup", silentStartup);
        WriteBool(hKey, L"ShortcutsEnabled", shortcutsEnabled);
        WriteBool(hKey, L"CheckForUpdates", checkForUpdates);
        WriteBool(hKey, L"HotkeyCtrl", toggleHotkey.ctrl);
        WriteBool(hKey, L"HotkeyShift", toggleHotkey.shift);
        WriteBool(hKey, L"HotkeyAlt", toggleHotkey.alt);
        WriteBool(hKey, L"HotkeyWin", toggleHotkey.win);
        WriteInt(hKey, L"HotkeyKey", static_cast<int>(toggleHotkey.vkCode));
        RegCloseKey(hKey);
    }
    SetAutoStart(autoStart);
    SaveShortcuts();
    SaveExcludedApps();
}

std::vector<TextShortcut> Settings::DefaultShortcuts() {
    return {
        {L"vn", L"Vi\u1EC7t Nam"},
        {L"hn", L"H\u00E0 N\u1ED9i"},
        {L"tphcm", L"Th\u00E0nh ph\u1ED1 H\u1ED3 Ch\u00ED Minh"},
        {L"sg", L"S\u00E0i G\u00F2n"},
        {L"ko", L"kh\u00F4ng"},
        {L"dc", L"\u0111\u01B0\u1EE3c"},
        {L"nc", L"n\u01B0\u1EDBc"},
        {L"bn", L"b\u1EA1n"},
        {L"mk", L"m\u00ECnh"},
        {L"ns", L"n\u00F3i"},
        {L"vs", L"v\u1EDBi"},
        {L"ntn", L"nh\u01B0 th\u1EBF n\u00E0o"},
        {L"j", L"g\u00EC"},
        {L"cx", L"c\u0169ng"},
        {L"ng", L"ng\u01B0\u1EDDi"},
        {L"ck", L"ch\u1ED3ng"},
        {L"vk", L"v\u1EE3"},
        {L"bt", L"b\u00ECnh th\u01B0\u1EDDng"},
        {L"nt", L"nh\u1EAFn tin"},
        {L"ctv", L"c\u1ED9ng t\u00E1c vi\u00EAn"},
    };
}

std::wstring Settings::GetString(const wchar_t* name, const wchar_t* defaultValue) {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REGISTRY_PATH, 0, KEY_READ, &hKey) != ERROR_SUCCESS) {
        return defaultValue;
    }

    wchar_t buffer[4096] = {0};
    DWORD size = sizeof(buffer);
    DWORD type = REG_SZ;
    std::wstring result = defaultValue;

    if (RegQueryValueExW(hKey, name, nullptr, &type, (LPBYTE)buffer, &size) == ERROR_SUCCESS) {
        result = buffer;
    }

    RegCloseKey(hKey);
    return result;
}

void Settings::SetString(const wchar_t* name, const std::wstring& value) {
    HKEY hKey;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REGISTRY_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) != ERROR_SUCCESS) {
        return;
    }

    RegSetValueExW(hKey, name, 0, REG_SZ,
                   (LPBYTE)value.c_str(), static_cast<DWORD>((value.length() + 1) * sizeof(wchar_t)));
    RegCloseKey(hKey);
}

bool Settings::GetAutoStart() const {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, STARTUP_PATH, 0, KEY_READ, &hKey) != ERROR_SUCCESS) {
        return false;
    }

    wchar_t buffer[MAX_PATH] = {0};
    DWORD size = sizeof(buffer);
    bool exists = RegQueryValueExW(hKey, APP_NAME, nullptr, nullptr, (LPBYTE)buffer, &size) == ERROR_SUCCESS;

    RegCloseKey(hKey);
    return exists;
}

void Settings::SetAutoStart(bool enabled) {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, STARTUP_PATH, 0, KEY_WRITE, &hKey) != ERROR_SUCCESS) {
        return;
    }

    if (enabled) {
        wchar_t exePath[MAX_PATH];
        GetModuleFileNameW(nullptr, exePath, MAX_PATH);
        std::wstring value = L"\"" + std::wstring(exePath) + L"\"";
        RegSetValueExW(hKey, APP_NAME, 0, REG_SZ,
                       (LPBYTE)value.c_str(), static_cast<DWORD>((value.length() + 1) * sizeof(wchar_t)));
    } else {
        RegDeleteValueW(hKey, APP_NAME);
    }

    RegCloseKey(hKey);
}

void Settings::LoadShortcuts() {
    std::wstring json = GetString(L"TextShortcuts", L"");
    if (json.empty()) {
        shortcuts = DefaultShortcuts();
        return;
    }

    // Simple JSON-like parsing: key1|value1;key2|value2;...
    shortcuts.clear();
    std::wstring::size_type pos = 0;
    while (pos < json.length()) {
        auto semicolon = json.find(L';', pos);
        if (semicolon == std::wstring::npos) semicolon = json.length();

        std::wstring pair = json.substr(pos, semicolon - pos);
        auto pipe = pair.find(L'|');
        if (pipe != std::wstring::npos) {
            TextShortcut s;
            s.key = pair.substr(0, pipe);
            s.value = pair.substr(pipe + 1);
            if (!s.key.empty() && !s.value.empty()) {
                shortcuts.push_back(s);
            }
        }

        pos = semicolon + 1;
    }

    if (shortcuts.empty()) {
        shortcuts = DefaultShortcuts();
    }
}

void Settings::SaveShortcuts() {
    // Simple serialization: key1|value1;key2|value2;...
    std::wstring json;
    for (const auto& s : shortcuts) {
        if (!json.empty()) json += L';';
        json += s.key + L'|' + s.value;
    }
    SetString(L"TextShortcuts", json);
}

void Settings::LoadExcludedApps() {
    std::wstring list = GetString(L"ExcludedApps", L"");
    excludedApps.clear();
    if (list.empty()) return;

    // Parse pipe-delimited list
    std::wstring::size_type pos = 0;
    while (pos < list.length()) {
        auto pipe = list.find(L'|', pos);
        if (pipe == std::wstring::npos) pipe = list.length();
        std::wstring app = list.substr(pos, pipe - pos);
        if (!app.empty()) {
            excludedApps.push_back(app);
        }
        pos = pipe + 1;
    }
}

void Settings::SaveExcludedApps() {
    std::wstring list;
    for (size_t i = 0; i < excludedApps.size(); i++) {
        if (i > 0) list += L'|';
        list += excludedApps[i];
    }
    SetString(L"ExcludedApps", list);
}

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

// Forward declaration (defined after ExportShortcutsToJson)
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

// JSON Export/Import (Feature 5)
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

// Simple JSON parser helpers
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

bool Settings::ImportFromJson(const std::wstring& json) {
    // Check version
    int version = ExtractJsonInt(json, L"version", 0);
    if (version != 1) return false;

    // Find settings section
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

    // Find hotkey section
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

    // Find excludedApps array
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

    // Find shortcuts array
    shortcuts = ParseShortcutsJsonArray(json, L"shortcuts");
    if (shortcuts.empty()) {
        shortcuts = DefaultShortcuts();
    }

    return true;
}

// Common file I/O helpers (DRY: shared by settings + shortcuts export/import)
static bool WriteWideStringToFile(const wchar_t* path, const std::wstring& content) {
    HANDLE hFile = CreateFileW(path, GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (hFile == INVALID_HANDLE_VALUE) return false;
    BYTE bom[2] = {0xFF, 0xFE};
    DWORD written;
    WriteFile(hFile, bom, 2, &written, nullptr);
    WriteFile(hFile, content.c_str(), static_cast<DWORD>(content.length() * sizeof(wchar_t)), &written, nullptr);
    CloseHandle(hFile);
    return true;
}

static std::wstring ReadWideStringFromFile(const wchar_t* path) {
    HANDLE hFile = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (hFile == INVALID_HANDLE_VALUE) return L"";
    DWORD fileSize = GetFileSize(hFile, nullptr);
    if (fileSize == INVALID_FILE_SIZE || fileSize < 4) {
        CloseHandle(hFile);
        return L"";
    }
    std::vector<BYTE> buffer(fileSize + 2);
    DWORD bytesRead;
    if (!ReadFile(hFile, buffer.data(), fileSize, &bytesRead, nullptr)) {
        CloseHandle(hFile);
        return L"";
    }
    CloseHandle(hFile);
    buffer[bytesRead] = 0;
    buffer[bytesRead + 1] = 0;
    if (buffer[0] == 0xFF && buffer[1] == 0xFE) {
        return reinterpret_cast<const wchar_t*>(buffer.data() + 2);
    }
    int wideLen = MultiByteToWideChar(CP_UTF8, 0, reinterpret_cast<const char*>(buffer.data()), bytesRead, nullptr, 0);
    if (wideLen > 0) {
        std::wstring result(wideLen, 0);
        MultiByteToWideChar(CP_UTF8, 0, reinterpret_cast<const char*>(buffer.data()), bytesRead, &result[0], wideLen);
        return result;
    }
    return L"";
}

bool Settings::ExportToFile(const wchar_t* path) {
    return WriteWideStringToFile(path, Instance().ExportToJson());
}

bool Settings::ImportFromFile(const wchar_t* path) {
    std::wstring json = ReadWideStringFromFile(path);
    if (json.empty()) return false;
    if (!Instance().ImportFromJson(json)) return false;
    Instance().Save();
    return true;
}

// Export shortcuts only to JSON
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

// Import shortcuts only from JSON
bool Settings::ImportShortcutsFromJson(const std::wstring& json) {
    int version = ExtractJsonInt(json, L"version", 0);
    if (version != 1) return false;

    auto newShortcuts = ParseShortcutsJsonArray(json, L"shortcuts");
    if (newShortcuts.empty()) return false;
    shortcuts = newShortcuts;
    return true;
}

bool Settings::ExportShortcutsToFile(const wchar_t* path) {
    return WriteWideStringToFile(path, Instance().ExportShortcutsToJson());
}

bool Settings::ImportShortcutsFromFile(const wchar_t* path) {
    std::wstring json = ReadWideStringFromFile(path);
    if (json.empty()) return false;
    if (!Instance().ImportShortcutsFromJson(json)) return false;
    Instance().Save();
    return true;
}
