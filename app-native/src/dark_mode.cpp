// ViKey - Dark Mode Support Implementation
// dark_mode.cpp
// Windows 10 1809+ dark mode using undocumented uxtheme APIs

#include "dark_mode.h"
#include <dwmapi.h>
#include <uxtheme.h>

#pragma comment(lib, "dwmapi.lib")
#pragma comment(lib, "uxtheme.lib")

// Dark mode state
bool g_isDarkMode = false;
HBRUSH g_hDarkBrush = nullptr;
HBRUSH g_hDarkEditBrush = nullptr;

// Undocumented dark mode APIs from uxtheme.dll
enum PreferredAppMode { Default, AllowDark, ForceDark, ForceLight, Max };
typedef PreferredAppMode (WINAPI *fnSetPreferredAppMode)(PreferredAppMode appMode);
typedef BOOL (WINAPI *fnAllowDarkModeForWindow)(HWND hWnd, BOOL allow);
typedef void (WINAPI *fnRefreshImmersiveColorPolicyState)();
typedef void (WINAPI *fnFlushMenuThemes)();

static fnSetPreferredAppMode SetPreferredAppMode = nullptr;
fnAllowDarkModeForWindow AllowDarkModeForWindow = nullptr;
static fnRefreshImmersiveColorPolicyState RefreshImmersiveColorPolicyState = nullptr;
static fnFlushMenuThemes FlushMenuThemes = nullptr;

void InitDarkModeAPIs() {
    HMODULE hUxTheme = LoadLibraryW(L"uxtheme.dll");
    if (hUxTheme) {
        SetPreferredAppMode = (fnSetPreferredAppMode)GetProcAddress(hUxTheme, MAKEINTRESOURCEA(135));
        AllowDarkModeForWindow = (fnAllowDarkModeForWindow)GetProcAddress(hUxTheme, MAKEINTRESOURCEA(133));
        RefreshImmersiveColorPolicyState = (fnRefreshImmersiveColorPolicyState)GetProcAddress(hUxTheme, MAKEINTRESOURCEA(104));
        FlushMenuThemes = (fnFlushMenuThemes)GetProcAddress(hUxTheme, MAKEINTRESOURCEA(136));

        if (SetPreferredAppMode) {
            SetPreferredAppMode(AllowDark);
        }
        if (RefreshImmersiveColorPolicyState) {
            RefreshImmersiveColorPolicyState();
        }
        if (FlushMenuThemes) {
            FlushMenuThemes();
        }
    }
}

bool IsSystemDarkMode() {
    HKEY hKey;
    DWORD value = 0;
    DWORD size = sizeof(value);
    if (RegOpenKeyExW(HKEY_CURRENT_USER,
        L"Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
        0, KEY_READ, &hKey) == ERROR_SUCCESS) {
        RegQueryValueExW(hKey, L"AppsUseLightTheme", nullptr, nullptr, (LPBYTE)&value, &size);
        RegCloseKey(hKey);
    }
    return value == 0;
}

INT_PTR HandleDarkModeColors(UINT message, WPARAM wParam) {
    if (!g_isDarkMode) return FALSE;

    switch (message) {
    case WM_CTLCOLORDLG:
        return (INT_PTR)g_hDarkBrush;

    case WM_CTLCOLORSTATIC: {
        HDC hdc = (HDC)wParam;
        SetTextColor(hdc, DARK_TEXT_COLOR);
        SetBkColor(hdc, DARK_BG_COLOR);
        return (INT_PTR)g_hDarkBrush;
    }

    case WM_CTLCOLOREDIT:
    case WM_CTLCOLORLISTBOX: {
        HDC hdc = (HDC)wParam;
        SetTextColor(hdc, DARK_TEXT_COLOR);
        SetBkColor(hdc, DARK_EDIT_BG);
        return (INT_PTR)g_hDarkEditBrush;
    }

    case WM_CTLCOLORBTN: {
        HDC hdc = (HDC)wParam;
        SetTextColor(hdc, DARK_TEXT_COLOR);
        SetBkColor(hdc, DARK_BG_COLOR);
        return (INT_PTR)g_hDarkBrush;
    }
    }
    return FALSE;
}

void EnableDarkModeForDialog(HWND hDlg) {
    g_isDarkMode = IsSystemDarkMode();

    if (g_isDarkMode) {
        if (AllowDarkModeForWindow) {
            AllowDarkModeForWindow(hDlg, TRUE);
        }

        BOOL darkMode = TRUE;
        DwmSetWindowAttribute(hDlg, 20, &darkMode, sizeof(darkMode));

        if (!g_hDarkBrush) {
            g_hDarkBrush = CreateSolidBrush(DARK_BG_COLOR);
        }
        if (!g_hDarkEditBrush) {
            g_hDarkEditBrush = CreateSolidBrush(DARK_EDIT_BG);
        }

        EnumChildWindows(hDlg, [](HWND hChild, LPARAM) -> BOOL {
            if (AllowDarkModeForWindow) {
                AllowDarkModeForWindow(hChild, TRUE);
            }
            wchar_t className[64] = {};
            GetClassNameW(hChild, className, 64);
            if (_wcsicmp(className, L"Button") == 0) {
                SetWindowTheme(hChild, L"Explorer", nullptr);
            } else if (_wcsicmp(className, L"ComboBox") == 0) {
                SetWindowTheme(hChild, L"DarkMode_CFD", nullptr);
            } else if (_wcsicmp(className, L"Edit") == 0) {
                SetWindowTheme(hChild, L"DarkMode_CFD", nullptr);
            } else if (_wcsicmp(className, L"SysListView32") == 0) {
                SetWindowTheme(hChild, L"DarkMode_Explorer", nullptr);
            } else {
                SetWindowTheme(hChild, L"DarkMode_Explorer", nullptr);
            }
            return TRUE;
        }, 0);

        SendMessageW(hDlg, WM_THEMECHANGED, 0, 0);
    }
}

void CenterDialog(HWND hDlg) {
    RECT dlgRect;
    GetWindowRect(hDlg, &dlgRect);
    int dlgWidth = dlgRect.right - dlgRect.left;
    int dlgHeight = dlgRect.bottom - dlgRect.top;
    int screenWidth = GetSystemMetrics(SM_CXSCREEN);
    int screenHeight = GetSystemMetrics(SM_CYSCREEN);
    int x = (screenWidth - dlgWidth) / 2;
    int y = (screenHeight - dlgHeight) / 2;
    SetWindowPos(hDlg, nullptr, x, y, 0, 0, SWP_NOZORDER | SWP_NOSIZE);
}

void ScaleDialogForDpi(HWND hDlg) {
    EnableDarkModeForDialog(hDlg);
    CenterDialog(hDlg);
}

float GetDpiScale(HWND hWnd) {
    UINT dpi = 96;
    HMODULE hUser32 = GetModuleHandleW(L"user32.dll");
    if (hUser32) {
        typedef UINT (WINAPI *GetDpiForWindowFunc)(HWND);
        auto pGetDpiForWindow = (GetDpiForWindowFunc)GetProcAddress(hUser32, "GetDpiForWindow");
        if (pGetDpiForWindow && hWnd) {
            dpi = pGetDpiForWindow(hWnd);
        } else {
            HDC hdc = GetDC(nullptr);
            dpi = GetDeviceCaps(hdc, LOGPIXELSX);
            ReleaseDC(nullptr, hdc);
        }
    }
    return dpi / 96.0f;
}
