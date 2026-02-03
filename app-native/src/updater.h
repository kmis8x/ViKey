// ViKey - Auto Update Manager
// updater.h
// Checks for updates from GitHub releases

#pragma once

#include <windows.h>
#include <string>
#include <functional>

// Current version - update this when releasing new versions
constexpr const wchar_t* VIKEY_VERSION = L"1.3.4";
constexpr const char* VIKEY_VERSION_A = "1.3.4";

// GitHub API endpoint for releases
constexpr const wchar_t* GITHUB_API_HOST = L"api.github.com";
constexpr const wchar_t* GITHUB_API_PATH = L"/repos/kmis8x/ViKey/releases/latest";
constexpr const wchar_t* GITHUB_RELEASES_URL = L"https://github.com/kmis8x/ViKey/releases/latest";

// Update check result
struct UpdateInfo {
    bool available;           // True if update is available
    std::wstring latestVersion;  // Latest version string
    std::wstring downloadUrl;    // Download URL for the release
    std::wstring releaseNotes;   // Release notes (brief)
    std::wstring error;          // Error message if check failed
};

class Updater {
public:
    static Updater& Instance();

    // Check for updates (blocking call)
    // Returns UpdateInfo with results
    UpdateInfo CheckForUpdates();

    // Check for updates asynchronously
    // Callback will be called on completion (on main thread via PostMessage)
    void CheckForUpdatesAsync(HWND hWnd, std::function<void(const UpdateInfo&)> callback);

    // Open download page in browser
    static void OpenDownloadPage();

    // Download and install update automatically
    // Returns true if download started successfully
    bool DownloadAndInstall(const std::wstring& version, HWND hWnd);

    // Get current version
    static const wchar_t* GetCurrentVersion() { return VIKEY_VERSION; }

private:
    Updater() = default;
    ~Updater() = default;
    Updater(const Updater&) = delete;
    Updater& operator=(const Updater&) = delete;

    // HTTP request helper using WinHTTP
    std::string HttpGet(const wchar_t* host, const wchar_t* path);

    // Parse GitHub release JSON
    UpdateInfo ParseReleaseJson(const std::string& json);

    // Compare version strings (using Rust FFI)
    bool IsNewerVersion(const char* current, const char* latest);

    // Async callback storage
    HWND m_callbackWnd = nullptr;
    std::function<void(const UpdateInfo&)> m_callback;
};

// Custom message for async update check completion
#define WM_UPDATE_CHECK_COMPLETE (WM_USER + 100)
