// ViKey - Updater Download and Install
// updater_download.cpp
// Updater::DownloadAndInstall implementation

#include "updater.h"
#include <winhttp.h>
#include <shellapi.h>
#pragma comment(lib, "winhttp.lib")

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

    // Escape single quotes for PowerShell string embedding
    auto escapePS = [](const std::wstring& s) -> std::wstring {
        std::wstring result = s;
        size_t pos = 0;
        while ((pos = result.find(L'\'', pos)) != std::wstring::npos) {
            result.replace(pos, 1, L"''");
            pos += 2;
        }
        return result;
    };

    // Create PowerShell update script with error handling
    std::wstring scriptPath = std::wstring(tempPath) + L"vikey-update.ps1";
    std::wstring script =
        L"# ViKey Auto-Update Script\n"
        L"$ErrorActionPreference = 'Stop'\n"
        L"$zipUrl = '" + escapePS(downloadUrl) + L"'\n"
        L"$zipPath = '" + escapePS(zipPath) + L"'\n"
        L"$extractPath = '" + escapePS(extractPath) + L"'\n"
        L"$installPath = '" + escapePS(exeDir) + L"'\n"
        L"$exePath = '" + escapePS(std::wstring(exePath)) + L"'\n"
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
