// ViKey - App Detector (Feature 2: Smart Switch per App)
// app_detector.h
// Detects foreground app changes and manages per-app IME states

#pragma once

#include <windows.h>
#include <string>
#include <unordered_map>

// Per-app state storage
struct AppState {
    bool enabled;
    int encoding;  // For Feature 8: App Encoding Memory
};

class AppDetector {
public:
    static AppDetector& Instance();

    // Get current foreground app name (e.g., "notepad.exe")
    std::wstring GetForegroundAppName();

    // Check if foreground app has changed since last call
    bool HasAppChanged();

    // Smart Switch per App (Feature 2)
    void SaveAppState(const std::wstring& app, bool enabled);
    bool GetAppState(const std::wstring& app, bool defaultEnabled);
    void ClearAppState(const std::wstring& app);

    // Exclusion list (Feature 3)
    void SetExcludedApps(const std::vector<std::wstring>& apps);
    bool IsCurrentAppExcluded();
    const std::vector<std::wstring>& GetExcludedApps() const { return m_excludedApps; }

    // App Encoding Memory (Feature 8)
    void SetAppEncoding(const std::wstring& app, int encoding);
    int GetAppEncoding(const std::wstring& app, int defaultEncoding);

    // Load/Save to registry
    void Load();
    void Save();

private:
    AppDetector();
    ~AppDetector() = default;
    AppDetector(const AppDetector&) = delete;
    AppDetector& operator=(const AppDetector&) = delete;

    HWND m_lastHwnd;
    std::wstring m_lastAppName;
    std::unordered_map<std::wstring, AppState> m_appStates;
    std::vector<std::wstring> m_excludedApps;

    static constexpr const wchar_t* REGISTRY_PATH = L"SOFTWARE\\ViKey";
    static constexpr const wchar_t* APP_STATES_PATH = L"SOFTWARE\\ViKey\\AppStates";
    static constexpr const wchar_t* APP_ENCODINGS_PATH = L"SOFTWARE\\ViKey\\AppEncodings";
};
