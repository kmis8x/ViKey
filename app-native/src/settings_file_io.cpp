// ViKey - Settings File I/O
// settings_file_io.cpp
// File read/write helpers and Export/Import file functions

#include "settings.h"
#include <shlwapi.h>
#include <vector>

#pragma comment(lib, "shlwapi.lib")

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
