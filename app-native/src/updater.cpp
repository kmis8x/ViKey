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

// Function pointer for version comparison from core.dll
using FnVersionHasUpdate = int(*)(const char*, const char*);
static FnVersionHasUpdate g_versionHasUpdate = nullptr;
static HMODULE g_hCoreDll = nullptr;

// Load version comparison function from core.dll
static bool LoadVersionFunction() {
    if (g_versionHasUpdate) return true;

    // Get exe directory
    wchar_t exePath[MAX_PATH];
    GetModuleFileNameW(nullptr, exePath, MAX_PATH);
    std::wstring exeDir(exePath);
    size_t lastSlash = exeDir.find_last_of(L"\\/");
    if (lastSlash != std::wstring::npos) {
        exeDir = exeDir.substr(0, lastSlash);
    }

    std::wstring dllPath = exeDir + L"\\core.dll";
    g_hCoreDll = LoadLibraryW(dllPath.c_str());
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
    // Capture hWnd by value to avoid accessing member after destruction
    HWND capturedWnd = hWnd;
    std::thread([this, capturedWnd]() {
        UpdateInfo info = CheckForUpdates();

        // Verify window still exists before posting message
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

    // Validate HTTP status code
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
    if (LoadVersionFunction() && g_versionHasUpdate) {
        int hasUpdate = g_versionHasUpdate(VIKEY_VERSION_A, tagName.c_str());
        info.available = (hasUpdate == 1);
    } else {
        // Fallback: simple string comparison
        info.available = (tagName > std::string(VIKEY_VERSION_A));
    }

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
            size_t maxLen = 200;
            size_t bodyLen = bodyQuoteEnd - bodyQuoteStart - 1;
            std::string body = json.substr(bodyQuoteStart + 1, bodyLen < maxLen ? bodyLen : maxLen);

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
    if (LoadVersionFunction() && g_versionHasUpdate) {
        return g_versionHasUpdate(current, latest) == 1;
    }
    // Fallback: simple string comparison
    return std::string(latest) > std::string(current);
}

// Validate version string format (only digits and dots, e.g. "1.3.4")
static bool IsValidVersionString(const std::wstring& v) {
    if (v.empty() || v.length() > 20) return false;
    for (wchar_t c : v) {
        if (c != L'.' && (c < L'0' || c > L'9')) return false;
    }
    return true;
}

bool Updater::DownloadAndInstall(const std::wstring& version, HWND hWnd) {
    // Validate version to prevent script injection
    if (!IsValidVersionString(version)) {
        MessageBoxW(hWnd, L"Phi\u00EAn b\u1EA3n kh\u00F4ng h\u1EE3p l\u1EC7", L"L\u1ED7i", MB_ICONERROR);
        return false;
    }

    // Build download URL
    std::wstring downloadUrl = L"https://github.com/kmis8x/ViKey/releases/download/v" + version +
                               L"/ViKey-v" + version + L"-win64.zip";

    // Get temp folder
    wchar_t tempPath[MAX_PATH];
    GetTempPathW(MAX_PATH, tempPath);

    std::wstring zipPath = std::wstring(tempPath) + L"ViKey-update.zip";
    std::wstring extractPath = std::wstring(tempPath) + L"ViKey-update\\";

    // Get exe directory
    wchar_t exePath[MAX_PATH];
    GetModuleFileNameW(nullptr, exePath, MAX_PATH);
    std::wstring exeDir(exePath);
    size_t lastSlash = exeDir.find_last_of(L"\\/");
    if (lastSlash != std::wstring::npos) {
        exeDir = exeDir.substr(0, lastSlash);
    }

    // Create PowerShell update script with error handling
    std::wstring scriptPath = std::wstring(tempPath) + L"vikey-update.ps1";
    std::wstring script =
        L"# ViKey Auto-Update Script\n"
        L"$ErrorActionPreference = 'Stop'\n"
        L"$zipUrl = '" + downloadUrl + L"'\n"
        L"$zipPath = '" + zipPath + L"'\n"
        L"$extractPath = '" + extractPath + L"'\n"
        L"$installPath = '" + exeDir + L"'\n"
        L"$exePath = '" + std::wstring(exePath) + L"'\n"
        L"\n"
        L"try {\n"
        L"    # Download update\n"
        L"    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12\n"
        L"    Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing\n"
        L"\n"
        L"    # Kill ViKey process\n"
        L"    $proc = Get-Process -Name 'ViKey' -ErrorAction SilentlyContinue\n"
        L"    if ($proc) { $proc | Stop-Process -Force; Start-Sleep -Seconds 1 }\n"
        L"\n"
        L"    # Extract update\n"
        L"    if (Test-Path $extractPath) { Remove-Item $extractPath -Recurse -Force }\n"
        L"    Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force\n"
        L"\n"
        L"    # Install update\n"
        L"    $updateDir = Get-ChildItem -Path $extractPath -Directory | Select-Object -First 1\n"
        L"    if ($updateDir) {\n"
        L"        Copy-Item -Path (Join-Path $updateDir.FullName '*') -Destination $installPath -Recurse -Force\n"
        L"    } else {\n"
        L"        Copy-Item -Path (Join-Path $extractPath '*') -Destination $installPath -Recurse -Force\n"
        L"    }\n"
        L"\n"
        L"    # Cleanup\n"
        L"    Remove-Item $zipPath -Force -ErrorAction SilentlyContinue\n"
        L"    Remove-Item $extractPath -Recurse -Force -ErrorAction SilentlyContinue\n"
        L"\n"
        L"    # Restart ViKey\n"
        L"    Start-Process -FilePath $exePath\n"
        L"} catch {\n"
        L"    # Restart ViKey on error\n"
        L"    $proc = Get-Process -Name 'ViKey' -ErrorAction SilentlyContinue\n"
        L"    if (-not $proc) { Start-Process -FilePath $exePath }\n"
        L"}\n";

    // Write script to file
    HANDLE hFile = CreateFileW(scriptPath.c_str(), GENERIC_WRITE, 0, nullptr,
                               CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (hFile == INVALID_HANDLE_VALUE) {
        return false;
    }

    // Convert to UTF-8 for PowerShell
    int utf8Len = WideCharToMultiByte(CP_UTF8, 0, script.c_str(), -1, nullptr, 0, nullptr, nullptr);
    std::string utf8Script(utf8Len, 0);
    WideCharToMultiByte(CP_UTF8, 0, script.c_str(), -1, &utf8Script[0], utf8Len, nullptr, nullptr);

    DWORD written;
    WriteFile(hFile, utf8Script.c_str(), (DWORD)utf8Script.length(), &written, nullptr);
    CloseHandle(hFile);

    // Show updating message
    MessageBoxW(hWnd, L"\u0110ang t\u1EA3i b\u1EA3n c\u1EADp nh\u1EADt...\nViKey s\u1EBD t\u1EF1 \u0111\u1ED9ng kh\u1EDFi \u0111\u1ED9ng l\u1EA1i.",
        L"C\u1EADp nh\u1EADt ViKey", MB_ICONINFORMATION);

    // Launch PowerShell script (it will kill this process after download)
    std::wstring cmdLine = L"powershell.exe -ExecutionPolicy Bypass -WindowStyle Hidden -File \"" + scriptPath + L"\"";

    STARTUPINFOW si = { sizeof(si) };
    PROCESS_INFORMATION pi = {};

    if (!CreateProcessW(nullptr, &cmdLine[0], nullptr, nullptr, FALSE,
                        CREATE_NO_WINDOW, nullptr, nullptr, &si, &pi)) {
        return false;
    }

    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);

    return true;
}
