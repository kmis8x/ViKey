// ViKey - Settings Manager
// settings.h
// Persists user settings to Windows Registry

#pragma once

#include <windows.h>
#include <string>
#include <vector>
#include "rust_bridge.h"
#include "shortcut_manager.h"

// Hotkey configuration for language toggle
struct HotkeyConfig {
    bool ctrl = true;
    bool shift = false;
    bool alt = false;
    bool win = false;
    UINT vkCode = VK_SPACE;  // Virtual key code (default: Space)

    // Get combined modifiers for RegisterHotKey
    UINT GetModifiers() const {
        UINT mods = 0x4000;  // MOD_NOREPEAT
        if (ctrl)  mods |= 0x0002;  // MOD_CONTROL
        if (shift) mods |= 0x0004;  // MOD_SHIFT
        if (alt)   mods |= 0x0001;  // MOD_ALT
        if (win)   mods |= 0x0008;  // MOD_WIN
        return mods;
    }
};

class Settings {
public:
    static Settings& Instance();

    // Load all settings from registry
    void Load();

    // Save all settings to registry
    void Save();

    // Settings properties
    bool enabled;
    InputMethod method;
    bool modernTone;
    bool englishAutoRestore;
    bool autoCapitalize;
    bool escRestore;
    bool freeTone;
    bool allowForeignConsonants;  // Allow f, j, w, z as valid consonants
    bool skipWShortcut;
    bool bracketShortcut;
    bool slowMode;
    bool clipboardMode;  // Use clipboard for text injection (for stubborn apps)
    bool smartSwitch;    // Remember IME state per app (Feature 2)
    bool autoStart;
    bool silentStartup;  // Hide Settings on startup, show Toast notification instead
    bool shortcutsEnabled;  // Enable/disable text shortcuts
    bool checkForUpdates;   // Check for updates on startup
    std::vector<TextShortcut> shortcuts;
    std::vector<std::wstring> excludedApps;  // Apps to auto-disable (Feature 3)
    HotkeyConfig toggleHotkey;  // Configurable toggle hotkey

    // Get default shortcuts
    static std::vector<TextShortcut> DefaultShortcuts();

    // Auto-start management
    void SetAutoStart(bool enabled);
    bool GetAutoStart() const;

    // Import/Export settings (Feature 5)
    std::wstring ExportToJson() const;
    bool ImportFromJson(const std::wstring& json);
    static bool ExportToFile(const wchar_t* path);
    static bool ImportFromFile(const wchar_t* path);

    // Import/Export shortcuts only
    std::wstring ExportShortcutsToJson() const;
    bool ImportShortcutsFromJson(const std::wstring& json);
    static bool ExportShortcutsToFile(const wchar_t* path);
    static bool ImportShortcutsFromFile(const wchar_t* path);

private:
    Settings();
    ~Settings() = default;
    Settings(const Settings&) = delete;
    Settings& operator=(const Settings&) = delete;

    // Registry helpers
    std::wstring GetString(const wchar_t* name, const wchar_t* defaultValue);
    void SetString(const wchar_t* name, const std::wstring& value);

    // Shortcut serialization
    void LoadShortcuts();
    void SaveShortcuts();

    // Excluded apps serialization (Feature 3)
    void LoadExcludedApps();
    void SaveExcludedApps();

    static constexpr const wchar_t* REGISTRY_PATH = L"SOFTWARE\\ViKey";
    static constexpr const wchar_t* STARTUP_PATH = L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
    static constexpr const wchar_t* APP_NAME = L"ViKey";
};
