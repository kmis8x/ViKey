// ViKey - App Detector Implementation
// app_detector.cpp
// Project: ViKey | Author: Tran Cong Sinh | https://github.com/kmis8x/ViKey

#include "app_detector.h"
#include <psapi.h>
#include <algorithm>

#pragma comment(lib, "psapi.lib")

AppDetector& AppDetector::Instance() {
    static AppDetector instance;
    return instance;
}

AppDetector::AppDetector()
    : m_lastHwnd(nullptr) {
}

std::wstring AppDetector::GetForegroundAppName() {
    HWND hwnd = GetForegroundWindow();
    if (!hwnd) return L"";

    DWORD processId = 0;
    GetWindowThreadProcessId(hwnd, &processId);
    if (processId == 0) return L"";

    HANDLE hProcess = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, processId);
    if (!hProcess) return L"";

    wchar_t exePath[MAX_PATH] = {};
    DWORD size = MAX_PATH;
    std::wstring appName;

    if (QueryFullProcessImageNameW(hProcess, 0, exePath, &size)) {
        // Extract filename from full path
        std::wstring fullPath(exePath);
        size_t lastSlash = fullPath.find_last_of(L"\\");
        if (lastSlash != std::wstring::npos) {
            appName = fullPath.substr(lastSlash + 1);
        } else {
            appName = fullPath;
        }
        // Convert to lowercase for consistency
        std::transform(appName.begin(), appName.end(), appName.begin(), ::towlower);
    }

    CloseHandle(hProcess);
    return appName;
}

bool AppDetector::HasAppChanged() {
    HWND hwnd = GetForegroundWindow();
    if (hwnd != m_lastHwnd) {
        m_lastHwnd = hwnd;
        std::wstring currentApp = GetForegroundAppName();
        if (currentApp != m_lastAppName) {
            m_lastAppName = currentApp;
            return true;
        }
    }
    return false;
}

void AppDetector::SaveAppState(const std::wstring& app, bool enabled) {
    if (app.empty()) return;
    m_appStates[app].enabled = enabled;

    // Save to registry
    HKEY hKey;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, APP_STATES_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) == ERROR_SUCCESS) {
        DWORD value = enabled ? 1 : 0;
        RegSetValueExW(hKey, app.c_str(), 0, REG_DWORD, (LPBYTE)&value, sizeof(value));
        RegCloseKey(hKey);
    }
}

bool AppDetector::GetAppState(const std::wstring& app, bool defaultEnabled) {
    if (app.empty()) return defaultEnabled;

    auto it = m_appStates.find(app);
    if (it != m_appStates.end()) {
        return it->second.enabled;
    }
    return defaultEnabled;
}

void AppDetector::ClearAppState(const std::wstring& app) {
    if (app.empty()) return;
    m_appStates.erase(app);

    // Remove from registry
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, APP_STATES_PATH, 0, KEY_WRITE, &hKey) == ERROR_SUCCESS) {
        RegDeleteValueW(hKey, app.c_str());
        RegCloseKey(hKey);
    }
}

void AppDetector::SetExcludedApps(const std::vector<std::wstring>& apps) {
    m_excludedApps = apps;
}

bool AppDetector::IsCurrentAppExcluded() {
    std::wstring currentApp = GetForegroundAppName();
    if (currentApp.empty()) return false;

    for (const auto& excluded : m_excludedApps) {
        std::wstring lowerExcluded = excluded;
        std::transform(lowerExcluded.begin(), lowerExcluded.end(), lowerExcluded.begin(), ::towlower);
        if (currentApp == lowerExcluded) {
            return true;
        }
    }
    return false;
}

void AppDetector::SetAppEncoding(const std::wstring& app, int encoding) {
    if (app.empty()) return;
    m_appStates[app].encoding = encoding;

    // Save to registry
    HKEY hKey;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, APP_ENCODINGS_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) == ERROR_SUCCESS) {
        DWORD value = static_cast<DWORD>(encoding);
        RegSetValueExW(hKey, app.c_str(), 0, REG_DWORD, (LPBYTE)&value, sizeof(value));
        RegCloseKey(hKey);
    }
}

int AppDetector::GetAppEncoding(const std::wstring& app, int defaultEncoding) {
    if (app.empty()) return defaultEncoding;

    auto it = m_appStates.find(app);
    if (it != m_appStates.end() && it->second.encoding != 0) {
        return it->second.encoding;
    }
    return defaultEncoding;
}

void AppDetector::Load() {
    // Load app states from registry
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, APP_STATES_PATH, 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        wchar_t valueName[256];
        DWORD valueNameSize;
        DWORD valueData;
        DWORD valueDataSize;
        DWORD type;
        DWORD index = 0;

        while (true) {
            valueNameSize = 256;
            valueDataSize = sizeof(valueData);
            LONG result = RegEnumValueW(hKey, index++, valueName, &valueNameSize,
                                         nullptr, &type, (LPBYTE)&valueData, &valueDataSize);
            if (result != ERROR_SUCCESS) break;
            if (type == REG_DWORD) {
                m_appStates[valueName].enabled = (valueData != 0);
            }
        }
        RegCloseKey(hKey);
    }

    // Load app encodings from registry
    if (RegOpenKeyExW(HKEY_CURRENT_USER, APP_ENCODINGS_PATH, 0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        wchar_t valueName[256];
        DWORD valueNameSize;
        DWORD valueData;
        DWORD valueDataSize;
        DWORD type;
        DWORD index = 0;

        while (true) {
            valueNameSize = 256;
            valueDataSize = sizeof(valueData);
            LONG result = RegEnumValueW(hKey, index++, valueName, &valueNameSize,
                                         nullptr, &type, (LPBYTE)&valueData, &valueDataSize);
            if (result != ERROR_SUCCESS) break;
            if (type == REG_DWORD) {
                m_appStates[valueName].encoding = static_cast<int>(valueData);
            }
        }
        RegCloseKey(hKey);
    }

    // Load excluded apps
    HKEY hMainKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, REGISTRY_PATH, 0, KEY_READ, &hMainKey) == ERROR_SUCCESS) {
        wchar_t buffer[4096] = {};
        DWORD size = sizeof(buffer);
        DWORD type = REG_SZ;
        if (RegQueryValueExW(hMainKey, L"ExcludedApps", nullptr, &type, (LPBYTE)buffer, &size) == ERROR_SUCCESS) {
            // Parse pipe-delimited list
            std::wstring apps(buffer);
            m_excludedApps.clear();
            size_t pos = 0;
            while (pos < apps.length()) {
                size_t pipe = apps.find(L'|', pos);
                if (pipe == std::wstring::npos) pipe = apps.length();
                std::wstring app = apps.substr(pos, pipe - pos);
                if (!app.empty()) {
                    m_excludedApps.push_back(app);
                }
                pos = pipe + 1;
            }
        }
        RegCloseKey(hMainKey);
    }
}

void AppDetector::Save() {
    // Save excluded apps to registry
    HKEY hKey;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, REGISTRY_PATH, 0, nullptr,
                        REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr) == ERROR_SUCCESS) {
        std::wstring apps;
        for (size_t i = 0; i < m_excludedApps.size(); i++) {
            if (i > 0) apps += L'|';
            apps += m_excludedApps[i];
        }
        RegSetValueExW(hKey, L"ExcludedApps", 0, REG_SZ,
                       (LPBYTE)apps.c_str(), static_cast<DWORD>((apps.length() + 1) * sizeof(wchar_t)));
        RegCloseKey(hKey);
    }
}
