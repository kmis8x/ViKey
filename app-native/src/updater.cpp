// ViKey - Auto Update Manager
// updater.cpp
// CheckForUpdates, HTTP request, JSON parse, version comparison

#include "updater.h"
#include "rust_bridge.h"
#include <winhttp.h>
#include <shellapi.h>
#include <thread>
#include <sstream>
#pragma comment(lib, "winhttp.lib")

// Function pointer for version comparison from core.dll
using FnVersionHasUpdate = int(*)(const char*, const char*);
static FnVersionHasUpdate g_versionHasUpdate = nullptr;
static HMODULE g_hCoreDll = nullptr;

// Load version comparison function from core.dll
static bool LoadVersionFunction() {
    if (g_versionHasUpdate) return true;

    // Use GetModuleHandleW since core.dll is already loaded by RustBridge
    g_hCoreDll = GetModuleHandleW(L"core.dll");
    if (!g_hCoreDll) return false;

    g_versionHasUpdate = (FnVersionHasUpdate)GetProcAddress(g_hCoreDll, "version_has_update");
    return g_versionHasUpdate != nullptr;
}

Updater& Updater::Instance() {
    static Updater instance;
    return instance;
}

UpdateInfo Updater::CheckForUpdates() {
    UpdateInfo info = {};
    info.available = false;

    std::string response = HttpGet(GITHUB_API_HOST, GITHUB_API_PATH);

    if (response.empty()) {
        info.error = L"Không thể kết nối đến máy chủ";
        return info;
    }

    info = ParseReleaseJson(response);

    return info;
}

void Updater::CheckForUpdatesAsync(HWND hWnd, std::function<void(const UpdateInfo&)> callback) {
    m_callbackWnd = hWnd;

    HWND capturedWnd = hWnd;
    std::thread([this, capturedWnd]() {
        UpdateInfo info = CheckForUpdates();

        if (capturedWnd && IsWindow(capturedWnd)) {
            UpdateInfo* pInfo = new UpdateInfo(info);
            if (!PostMessage(capturedWnd, WM_UPDATE_CHECK_COMPLETE, 0, (LPARAM)pInfo)) {
                delete pInfo;
            }
        }
    }).detach();
}

void Updater::OpenDownloadPage() {
    ShellExecuteW(nullptr, L"open", GITHUB_RELEASES_URL, nullptr, nullptr, SW_SHOWNORMAL);
}

std::string Updater::HttpGet(const wchar_t* host, const wchar_t* path) {
    std::string result;

    HINTERNET hSession = WinHttpOpen(
        L"ViKey-Updater/1.0",
        WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
        WINHTTP_NO_PROXY_NAME,
        WINHTTP_NO_PROXY_BYPASS,
        0);

    if (!hSession) {
        return result;
    }

    HINTERNET hConnect = WinHttpConnect(hSession, host, INTERNET_DEFAULT_HTTPS_PORT, 0);
    if (!hConnect) {
        WinHttpCloseHandle(hSession);
        return result;
    }

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

    WinHttpAddRequestHeaders(hRequest,
        L"User-Agent: ViKey-Updater/1.0\r\n",
        (DWORD)-1, WINHTTP_ADDREQ_FLAG_ADD);

    BOOL bResult = WinHttpSendRequest(
        hRequest,
        WINHTTP_NO_ADDITIONAL_HEADERS, 0,
        WINHTTP_NO_REQUEST_DATA, 0,
        0, 0);

    if (bResult) {
        bResult = WinHttpReceiveResponse(hRequest, nullptr);
    }

    if (bResult) {
        DWORD statusCode = 0;
        DWORD statusSize = sizeof(statusCode);
        WinHttpQueryHeaders(hRequest, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                            WINHTTP_HEADER_NAME_BY_INDEX, &statusCode, &statusSize, WINHTTP_NO_HEADER_INDEX);
        if (statusCode != 200) {
            WinHttpCloseHandle(hRequest);
            WinHttpCloseHandle(hConnect);
            WinHttpCloseHandle(hSession);
            return result;
        }
    }

    if (bResult) {
        constexpr size_t MAX_RESPONSE_SIZE = 1048576; // 1MB cap
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

                if (result.size() > MAX_RESPONSE_SIZE) break;
            }
        } while (dwSize > 0);
    }

    WinHttpCloseHandle(hRequest);
    WinHttpCloseHandle(hConnect);
    WinHttpCloseHandle(hSession);

    return result;
}

UpdateInfo Updater::ParseReleaseJson(const std::string& json) {
    UpdateInfo info = {};
    info.available = false;

    size_t tagPos = json.find("\"tag_name\"");
    if (tagPos == std::string::npos) {
        info.error = L"Không thể đọc thông tin phiên bản";
        return info;
    }

    size_t colonPos = json.find(':', tagPos);
    size_t quoteStart = json.find('"', colonPos);
    size_t quoteEnd = json.find('"', quoteStart + 1);

    if (quoteStart == std::string::npos || quoteEnd == std::string::npos) {
        info.error = L"Lỗi phân tích dữ liệu";
        return info;
    }

    std::string tagName = json.substr(quoteStart + 1, quoteEnd - quoteStart - 1);

    if (!tagName.empty() && tagName[0] == 'v') {
        tagName = tagName.substr(1);
    }

    info.latestVersion = std::wstring(tagName.begin(), tagName.end());

    if (LoadVersionFunction() && g_versionHasUpdate) {
        int hasUpdate = g_versionHasUpdate(VIKEY_VERSION_A, tagName.c_str());
        info.available = (hasUpdate == 1);
    } else {
        info.available = (tagName > std::string(VIKEY_VERSION_A));
    }

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

    size_t bodyPos = json.find("\"body\"");
    if (bodyPos != std::string::npos) {
        size_t bodyColonPos = json.find(':', bodyPos);
        size_t bodyQuoteStart = json.find('"', bodyColonPos);
        size_t bodyQuoteEnd = json.find('"', bodyQuoteStart + 1);

        if (bodyQuoteStart != std::string::npos && bodyQuoteEnd != std::string::npos) {
            size_t maxLen = 200;
            size_t bodyLen = bodyQuoteEnd - bodyQuoteStart - 1;
            std::string body = json.substr(bodyQuoteStart + 1, bodyLen < maxLen ? bodyLen : maxLen);

            size_t pos = 0;
            while ((pos = body.find("\\n", pos)) != std::string::npos) {
                body.replace(pos, 2, "\n");
                pos += 1;
            }

            info.releaseNotes = std::wstring(body.begin(), body.end());
        }
    }

    return info;
}

bool Updater::IsNewerVersion(const char* current, const char* latest) {
    if (LoadVersionFunction() && g_versionHasUpdate) {
        return g_versionHasUpdate(current, latest) == 1;
    }
    return std::string(latest) > std::string(current);
}
