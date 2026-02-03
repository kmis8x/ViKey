// ViKey - Auto Update Manager
// updater.cpp
// Checks for updates from GitHub releases using WinHTTP

#include "updater.h"
#include "rust_bridge.h"
#include <winhttp.h>
#include <shellapi.h>
#include <thread>
#include <sstream>

#pragma comment(lib, "winhttp.lib")

// External Rust FFI function for version comparison
extern "C" int version_has_update(const char* current, const char* latest);

Updater& Updater::Instance() {
    static Updater instance;
    return instance;
}

UpdateInfo Updater::CheckForUpdates() {
    UpdateInfo info = {};
    info.available = false;

    // Make HTTP request to GitHub API
    std::string response = HttpGet(GITHUB_API_HOST, GITHUB_API_PATH);

    if (response.empty()) {
        info.error = L"Không thể kết nối đến máy chủ";
        return info;
    }

    // Parse the JSON response
    info = ParseReleaseJson(response);

    return info;
}

void Updater::CheckForUpdatesAsync(HWND hWnd, std::function<void(const UpdateInfo&)> callback) {
    m_callbackWnd = hWnd;
    m_callback = callback;

    // Run check in background thread
    std::thread([this]() {
        UpdateInfo info = CheckForUpdates();

        // Post message to main thread
        if (m_callbackWnd && m_callback) {
            // Allocate info on heap to pass to main thread
            UpdateInfo* pInfo = new UpdateInfo(info);
            PostMessage(m_callbackWnd, WM_UPDATE_CHECK_COMPLETE, 0, (LPARAM)pInfo);
        }
    }).detach();
}

void Updater::OpenDownloadPage() {
    ShellExecuteW(nullptr, L"open", GITHUB_RELEASES_URL, nullptr, nullptr, SW_SHOWNORMAL);
}

std::string Updater::HttpGet(const wchar_t* host, const wchar_t* path) {
    std::string result;

    // Initialize WinHTTP
    HINTERNET hSession = WinHttpOpen(
        L"ViKey-Updater/1.0",
        WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
        WINHTTP_NO_PROXY_NAME,
        WINHTTP_NO_PROXY_BYPASS,
        0);

    if (!hSession) {
        return result;
    }

    // Connect to server
    HINTERNET hConnect = WinHttpConnect(hSession, host, INTERNET_DEFAULT_HTTPS_PORT, 0);
    if (!hConnect) {
        WinHttpCloseHandle(hSession);
        return result;
    }

    // Create request
    HINTERNET hRequest = WinHttpOpenRequest(
        hConnect,
        L"GET",
        path,
        nullptr,
        WINHTTP_NO_REFERER,
        WINHTTP_DEFAULT_ACCEPT_TYPES,
        WINHTTP_FLAG_SECURE);

    if (!hRequest) {
        WinHttpCloseHandle(hConnect);
        WinHttpCloseHandle(hSession);
        return result;
    }

    // Add User-Agent header (required by GitHub API)
    WinHttpAddRequestHeaders(hRequest,
        L"User-Agent: ViKey-Updater/1.0\r\n",
        (DWORD)-1, WINHTTP_ADDREQ_FLAG_ADD);

    // Send request
    BOOL bResult = WinHttpSendRequest(
        hRequest,
        WINHTTP_NO_ADDITIONAL_HEADERS, 0,
        WINHTTP_NO_REQUEST_DATA, 0,
        0, 0);

    if (bResult) {
        bResult = WinHttpReceiveResponse(hRequest, nullptr);
    }

    // Read response
    if (bResult) {
        DWORD dwSize = 0;
        DWORD dwDownloaded = 0;

        do {
            dwSize = 0;
            WinHttpQueryDataAvailable(hRequest, &dwSize);

            if (dwSize > 0) {
                char* buffer = new char[dwSize + 1];
                ZeroMemory(buffer, dwSize + 1);

                if (WinHttpReadData(hRequest, buffer, dwSize, &dwDownloaded)) {
                    result.append(buffer, dwDownloaded);
                }

                delete[] buffer;
            }
        } while (dwSize > 0);
    }

    // Cleanup
    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);

    return result;
}

UpdateInfo Updater::ParseReleaseJson(const std::string& json) {
    UpdateInfo info = {};
    info.available = false;

    // Simple JSON parsing (no external library needed)
    // Looking for: "tag_name": "v1.2.3"

    // Find tag_name
    size_t tagPos = json.find("\"tag_name\"");
    if (tagPos == std::string::npos) {
        info.error = L"Không thể đọc thông tin phiên bản";
        return info;
    }

    // Find the version string after tag_name
    size_t colonPos = json.find(':', tagPos);
    size_t quoteStart = json.find('"', colonPos);
    size_t quoteEnd = json.find('"', quoteStart + 1);

    if (quoteStart == std::string::npos || quoteEnd == std::string::npos) {
        info.error = L"Lỗi phân tích dữ liệu";
        return info;
    }

    std::string tagName = json.substr(quoteStart + 1, quoteEnd - quoteStart - 1);

    // Remove 'v' prefix if present
    if (!tagName.empty() && tagName[0] == 'v') {
        tagName = tagName.substr(1);
    }

    // Convert to wide string
    info.latestVersion = std::wstring(tagName.begin(), tagName.end());

    // Check if update is available using Rust FFI
    int hasUpdate = version_has_update(VIKEY_VERSION_A, tagName.c_str());
    info.available = (hasUpdate == 1);

    // Find html_url for download
    size_t htmlUrlPos = json.find("\"html_url\"");
    if (htmlUrlPos != std::string::npos) {
        size_t urlColonPos = json.find(':', htmlUrlPos);
        size_t urlQuoteStart = json.find('"', urlColonPos);
        size_t urlQuoteEnd = json.find('"', urlQuoteStart + 1);

        if (urlQuoteStart != std::string::npos && urlQuoteEnd != std::string::npos) {
            std::string url = json.substr(urlQuoteStart + 1, urlQuoteEnd - urlQuoteStart - 1);
            info.downloadUrl = std::wstring(url.begin(), url.end());
        }
    }

    // Find body (release notes) - just get first 200 chars
    size_t bodyPos = json.find("\"body\"");
    if (bodyPos != std::string::npos) {
        size_t bodyColonPos = json.find(':', bodyPos);
        size_t bodyQuoteStart = json.find('"', bodyColonPos);
        size_t bodyQuoteEnd = json.find('"', bodyQuoteStart + 1);

        if (bodyQuoteStart != std::string::npos && bodyQuoteEnd != std::string::npos) {
            std::string body = json.substr(bodyQuoteStart + 1,
                std::min(bodyQuoteEnd - bodyQuoteStart - 1, (size_t)200));

            // Convert escaped newlines
            size_t pos = 0;
            while ((pos = body.find("\\n", pos)) != std::string::npos) {
                body.replace(pos, 2, "\n");
                pos += 1;
            }

            // Convert to wide string (simple ASCII conversion)
            info.releaseNotes = std::wstring(body.begin(), body.end());
        }
    }

    return info;
}

bool Updater::IsNewerVersion(const char* current, const char* latest) {
    int result = version_has_update(current, latest);
    return result == 1;
}
