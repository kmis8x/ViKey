// ViKey - Settings Manager Implementation
// settings.cpp
// Registry helpers, Load, Save, AutoStart, Shortcuts/ExcludedApps

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
    , shortcutsEnabled(true)
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
