// ViKey - Win32 Native Application
// main.cpp
// Entry point, message loop, window management
//
// Project: ViKey | Author: Trần Công Sinh | https://github.com/kmis8x/ViKey

#define _CRT_SECURE_NO_WARNINGS

#include <windows.h>
#include <objidl.h>
#include <gdiplus.h>
#include <commctrl.h>
#include <commdlg.h>
#include <shobjidl.h>
#include <shellscalingapi.h>
#include <dwmapi.h>
#include <uxtheme.h>

#pragma comment(lib, "shcore.lib")
#pragma comment(lib, "dwmapi.lib")
#pragma comment(lib, "uxtheme.lib")

#include "resource.h"
#include "ime_processor.h"
#include "tray_icon.h"
#include "hotkey.h"
#include "settings.h"
#include "keyboard_hook.h"
#include "app_detector.h"
#include "encoding_converter.h"
#include "updater.h"
#include <cstdio>
#include <ctime>

#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "gdiplus.lib")
// Manifest is now embedded via resource.rc (includes Common Controls + DPI awareness)

// Application name and class
constexpr const wchar_t* APP_NAME = L"ViKey";
constexpr const wchar_t* WINDOW_CLASS = L"ViKey_Hidden";
constexpr const wchar_t* MUTEX_NAME = L"Global\\ViKey_SingleInstance";

// Custom messages
constexpr UINT WM_TRAYICON_MSG = WM_USER + 1;

// Global variables
HINSTANCE g_hInstance = nullptr;
HWND g_hWnd = nullptr;
ULONG_PTR g_gdiplusToken = 0;

// Dark mode support
bool g_isDarkMode = false;
HBRUSH g_hDarkBrush = nullptr;
HBRUSH g_hDarkEditBrush = nullptr;
constexpr COLORREF DARK_BG_COLOR = RGB(32, 32, 32);
constexpr COLORREF DARK_EDIT_BG = RGB(45, 45, 45);
constexpr COLORREF DARK_TEXT_COLOR = RGB(255, 255, 255);

// Undocumented dark mode APIs from uxtheme.dll
enum PreferredAppMode { Default, AllowDark, ForceDark, ForceLight, Max };
typedef PreferredAppMode (WINAPI *fnSetPreferredAppMode)(PreferredAppMode appMode);
typedef BOOL (WINAPI *fnAllowDarkModeForWindow)(HWND hWnd, BOOL allow);
typedef void (WINAPI *fnRefreshImmersiveColorPolicyState)();
typedef void (WINAPI *fnFlushMenuThemes)();

fnSetPreferredAppMode SetPreferredAppMode = nullptr;
fnAllowDarkModeForWindow AllowDarkModeForWindow = nullptr;
fnRefreshImmersiveColorPolicyState RefreshImmersiveColorPolicyState = nullptr;
fnFlushMenuThemes FlushMenuThemes = nullptr;

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

// Forward declarations
LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
bool InitInstance(HINSTANCE hInstance);
void CleanupInstance();
INT_PTR CALLBACK SettingsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
INT_PTR CALLBACK AboutDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
void UpdateUI();
void ShowSettingsDialog();
void ShowAboutDialog();
void ExportSettings();
void ImportSettings();
void ShowExcludeAppsDialog();
INT_PTR CALLBACK ExcludeAppsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
void ShowConverterDialog();
INT_PTR CALLBACK ConverterDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
void ShowShortcutsDialog();
INT_PTR CALLBACK ShortcutsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
void CheckForUpdatesOnStartup();
void ShowUpdateDialog(const UpdateInfo& info);
INT_PTR CALLBACK UpdateDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
void CheckForUpdatesManual();

// DPI helper - get scaling factor for current monitor
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

// Entry point
int APIENTRY wWinMain(_In_ HINSTANCE hInstance,
                      _In_opt_ HINSTANCE hPrevInstance,
                      _In_ LPWSTR lpCmdLine,
                      _In_ int nCmdShow) {
    UNREFERENCED_PARAMETER(hPrevInstance);
    UNREFERENCED_PARAMETER(lpCmdLine);
    UNREFERENCED_PARAMETER(nCmdShow);

    // Enable Per-Monitor DPI awareness (Windows 10 1703+)
    HMODULE hUser32 = GetModuleHandleW(L"user32.dll");
    if (hUser32) {
        typedef BOOL (WINAPI *SetProcessDpiAwarenessContextFunc)(DPI_AWARENESS_CONTEXT);
        auto pSetDpiContext = (SetProcessDpiAwarenessContextFunc)GetProcAddress(hUser32, "SetProcessDpiAwarenessContext");
        if (pSetDpiContext) {
            pSetDpiContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        } else {
            // Fallback for Windows 8.1
            HMODULE hShcore = LoadLibraryW(L"shcore.dll");
            if (hShcore) {
                typedef HRESULT (WINAPI *SetProcessDpiAwarenessFunc)(PROCESS_DPI_AWARENESS);
                auto pSetDpi = (SetProcessDpiAwarenessFunc)GetProcAddress(hShcore, "SetProcessDpiAwareness");
                if (pSetDpi) {
                    pSetDpi(PROCESS_PER_MONITOR_DPI_AWARE);
                }
                FreeLibrary(hShcore);
            }
        }
    }

    // Initialize dark mode support (Windows 10 1809+)
    InitDarkModeAPIs();

    // Check for single instance
    HANDLE hMutex = CreateMutexW(nullptr, TRUE, MUTEX_NAME);
    if (GetLastError() == ERROR_ALREADY_EXISTS) {
        // Another instance is already running
        if (hMutex) CloseHandle(hMutex);
        return 0;
    }

    // Initialize GDI+
    Gdiplus::GdiplusStartupInput gdiplusStartupInput;
    Gdiplus::GdiplusStartup(&g_gdiplusToken, &gdiplusStartupInput, nullptr);

    // Initialize common controls
    INITCOMMONCONTROLSEX icc = {};
    icc.dwSize = sizeof(icc);
    icc.dwICC = ICC_WIN95_CLASSES | ICC_LISTVIEW_CLASSES;
    InitCommonControlsEx(&icc);

    // Initialize application
    if (!InitInstance(hInstance)) {
        CleanupInstance();
        if (hMutex) CloseHandle(hMutex);
        return 1;
    }

    // Debug: log that we're entering message loop
    {
        FILE* f = fopen("D:\\CK\\ViKey\\app-native\\msgloop_start.log", "w");
        if (f) {
            fprintf(f, "Entering message loop, g_hWnd=%p, thread=%lu\n",
                    (void*)g_hWnd, GetCurrentThreadId());
            fclose(f);
        }
    }

    // Set up a timer to periodically verify hook status
    SetTimer(g_hWnd, 999, 3000, nullptr);  // Every 3 seconds

    // Message loop using GetMessage (standard approach for LL hooks)
    MSG msg;
    static int checkCount = 0;
    while (GetMessage(&msg, nullptr, 0, 0)) {
        // Log timer ticks to verify message loop is running
        if (msg.message == WM_TIMER && msg.wParam == 999) {
            checkCount++;
            FILE* f = fopen("D:\\CK\\ViKey\\app-native\\hookcheck.log", "a");
            if (f) {
                fprintf(f, "Check #%d: hook=%d\n", checkCount,
                        KeyboardHook::Instance().IsActive() ? 1 : 0);
                fclose(f);
            }
        }
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    // Cleanup
    CleanupInstance();
    if (hMutex) CloseHandle(hMutex);

    return static_cast<int>(msg.wParam);
}

bool InitInstance(HINSTANCE hInstance) {
    g_hInstance = hInstance;

    // Register window class
    WNDCLASSEXW wcex = {};
    wcex.cbSize = sizeof(WNDCLASSEXW);
    wcex.lpfnWndProc = WndProc;
    wcex.hInstance = hInstance;
    wcex.lpszClassName = WINDOW_CLASS;

    if (!RegisterClassExW(&wcex)) {
        return false;
    }

    // Create hidden window (for message handling)
    // Using a regular window (not HWND_MESSAGE) for better compatibility with low-level hooks
    g_hWnd = CreateWindowExW(
        WS_EX_TOOLWINDOW,  // Don't show in taskbar
        WINDOW_CLASS,
        APP_NAME,
        WS_POPUP,          // No border, no title bar
        -32000, -32000,    // Off-screen position
        0, 0,              // Zero size
        nullptr,           // No parent (not HWND_MESSAGE)
        nullptr,
        hInstance,
        nullptr);
    if (!g_hWnd) {
        return false;
    }

    // Load settings
    Settings::Instance().Load();

    // Load app detector data (for smart switch)
    AppDetector::Instance().Load();

    // Initialize IME processor
    if (!ImeProcessor::Instance().Initialize()) {
        MessageBoxW(nullptr, L"Failed to load core.dll", APP_NAME, MB_ICONERROR);
        return false;
    }

    // Apply settings to processor
    ImeProcessor::Instance().ApplySettings();

    // Initialize tray icon
    TrayIcon& tray = TrayIcon::Instance();
    tray.Initialize(g_hWnd, hInstance);

    // Set up tray callbacks
    tray.onToggleEnabled = []() {
        ImeProcessor::Instance().ToggleEnabled();
        Settings::Instance().enabled = ImeProcessor::Instance().IsEnabled();
        Settings::Instance().Save();
        UpdateUI();
    };

    tray.onSetMethod = [](InputMethod method) {
        ImeProcessor::Instance().SetMethod(method);
        Settings::Instance().method = method;
        Settings::Instance().Save();
        UpdateUI();
    };

    tray.onSettings = []() {
        ShowSettingsDialog();
    };

    tray.onAbout = []() {
        ShowAboutDialog();
    };

    tray.onExit = []() {
        PostMessage(g_hWnd, WM_CLOSE, 0, 0);
    };

    // Register global hotkey (Ctrl+Space)
    HotkeyManager& hotkey = HotkeyManager::Instance();
    hotkey.Register(g_hWnd);
    hotkey.SetCallback([]() {
        ImeProcessor::Instance().ToggleEnabled();
        Settings::Instance().enabled = ImeProcessor::Instance().IsEnabled();
        Settings::Instance().Save();
        UpdateUI();
    });

    // Start IME processor
    ImeProcessor::Instance().Start();

    // Update UI state
    UpdateUI();

    // Show Settings dialog or Toast on startup based on silentStartup setting
    if (Settings::Instance().silentStartup) {
        // Silent mode: show Toast notification only
        const wchar_t* lang = Settings::Instance().enabled ? L"Ti\u1EBFng Vi\u1EC7t" : L"Ti\u1EBFng Anh";
        wchar_t msg[128];
        swprintf_s(msg, L"\u0110ang ch\u1EA1y \u1EDF ch\u1EBF \u0111\u1ED9 %s\nCtrl+Space \u0111\u1EC3 chuy\u1EC3n", lang);
        tray.ShowBalloon(L"B\u1ED9 g\u00F5 ti\u1EBFng Vi\u1EC7t", msg);
    } else {
        // Normal mode: show Settings dialog
        PostMessage(g_hWnd, WM_COMMAND, IDM_SETTINGS, 0);
    }

    // Check for updates on startup (async)
    if (Settings::Instance().checkForUpdates) {
        CheckForUpdatesOnStartup();
    }

    return true;
}

void CleanupInstance() {
    // Stop processor
    ImeProcessor::Instance().Stop();

    // Unregister hotkey
    if (g_hWnd) {
        HotkeyManager::Instance().Unregister(g_hWnd);
    }

    // Remove tray icon
    TrayIcon::Instance().Shutdown();

    // Shutdown GDI+
    if (g_gdiplusToken) {
        Gdiplus::GdiplusShutdown(g_gdiplusToken);
    }
}

void UpdateUI() {
    bool enabled = ImeProcessor::Instance().IsEnabled();
    InputMethod method = ImeProcessor::Instance().GetMethod();

    TrayIcon& tray = TrayIcon::Instance();
    tray.UpdateIcon(enabled);
    tray.UpdateTooltip(enabled, method);
    tray.UpdateMenu(enabled, method);
}

LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    switch (message) {
    case WM_TRAYICON_MSG:
        TrayIcon::Instance().ProcessTrayMessage(hWnd, wParam, lParam);
        return 0;

    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDM_TOGGLE_ENABLED:
            if (TrayIcon::Instance().onToggleEnabled)
                TrayIcon::Instance().onToggleEnabled();
            break;

        case IDM_METHOD_TELEX:
            if (TrayIcon::Instance().onSetMethod)
                TrayIcon::Instance().onSetMethod(InputMethod::Telex);
            break;

        case IDM_METHOD_VNI:
            if (TrayIcon::Instance().onSetMethod)
                TrayIcon::Instance().onSetMethod(InputMethod::VNI);
            break;

        case IDM_ENC_UNICODE:
        case IDM_ENC_VNI:
        case IDM_ENC_TCVN3: {
            OutputEncoding enc = OutputEncoding::Unicode;
            if (LOWORD(wParam) == IDM_ENC_VNI) enc = OutputEncoding::VNI;
            else if (LOWORD(wParam) == IDM_ENC_TCVN3) enc = OutputEncoding::TCVN3;

            // Set encoding for current app
            TextSender::Instance().SetOutputEncoding(enc);

            // Save encoding for current app
            std::wstring currentApp = AppDetector::Instance().GetForegroundAppName();
            if (!currentApp.empty()) {
                AppDetector::Instance().SetAppEncoding(currentApp, static_cast<int>(enc));
            }

            UpdateUI();
            break;
        }

        case IDM_SETTINGS:
            if (TrayIcon::Instance().onSettings)
                TrayIcon::Instance().onSettings();
            break;

        case IDM_ABOUT:
            if (TrayIcon::Instance().onAbout)
                TrayIcon::Instance().onAbout();
            break;

        case IDM_EXPORT_SETTINGS:
            ExportSettings();
            break;

        case IDM_IMPORT_SETTINGS:
            ImportSettings();
            break;

        case IDM_EXCLUDE_APPS:
            ShowExcludeAppsDialog();
            break;

        case IDM_SHORTCUTS:
            ShowShortcutsDialog();
            break;

        case IDM_CONVERTER:
            ShowConverterDialog();
            break;

        case IDM_CHECK_UPDATE:
            CheckForUpdatesManual();
            break;

        case IDM_EXIT:
            if (TrayIcon::Instance().onExit)
                TrayIcon::Instance().onExit();
            break;
        }
        return 0;

    case WM_UPDATE_CHECK_COMPLETE: {
        UpdateInfo* pInfo = reinterpret_cast<UpdateInfo*>(lParam);
        if (pInfo) {
            if (pInfo->available) {
                ShowUpdateDialog(*pInfo);
            }
            delete pInfo;
        }
        return 0;
    }

    case WM_HOTKEY:
        HotkeyManager::Instance().ProcessHotkey(wParam);
        return 0;

    case WM_DESTROY:
        PostQuitMessage(0);
        return 0;

    default:
        return DefWindowProcW(hWnd, message, wParam, lParam);
    }
}

void ShowSettingsDialog() {
    static bool isOpen = false;
    if (isOpen) return;  // Prevent multiple dialogs
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_SETTINGS), g_hWnd, SettingsDialogProc);
    isOpen = false;
}

void ShowAboutDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_ABOUT), g_hWnd, AboutDialogProc);
    isOpen = false;
}

// Check if system dark mode is enabled
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
    return value == 0;  // 0 = dark mode, 1 = light mode
}

// Handle dark mode color messages - returns TRUE if handled
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

// Enable dark mode for dialog (Windows 10 1809+)
void EnableDarkModeForDialog(HWND hDlg) {
    g_isDarkMode = IsSystemDarkMode();

    if (g_isDarkMode) {
        // Allow dark mode for this window
        if (AllowDarkModeForWindow) {
            AllowDarkModeForWindow(hDlg, TRUE);
        }

        // DWMWA_USE_IMMERSIVE_DARK_MODE = 20
        BOOL darkMode = TRUE;
        DwmSetWindowAttribute(hDlg, 20, &darkMode, sizeof(darkMode));

        // Create dark mode brushes if not already created
        if (!g_hDarkBrush) {
            g_hDarkBrush = CreateSolidBrush(DARK_BG_COLOR);
        }
        if (!g_hDarkEditBrush) {
            g_hDarkEditBrush = CreateSolidBrush(DARK_EDIT_BG);
        }

        // Apply dark theme to all child controls
        EnumChildWindows(hDlg, [](HWND hChild, LPARAM) -> BOOL {
            if (AllowDarkModeForWindow) {
                AllowDarkModeForWindow(hChild, TRUE);
            }
            wchar_t className[64] = {};
            GetClassNameW(hChild, className, 64);
            if (_wcsicmp(className, L"Button") == 0) {
                // Use DarkMode_Explorer for buttons and checkboxes
                SetWindowTheme(hChild, L"Explorer", nullptr);
            } else if (_wcsicmp(className, L"ComboBox") == 0) {
                SetWindowTheme(hChild, L"DarkMode_CFD", nullptr);
            } else if (_wcsicmp(className, L"Edit") == 0) {
                SetWindowTheme(hChild, L"DarkMode_CFD", nullptr);
            } else if (_wcsicmp(className, L"SysListView32") == 0) {
                SetWindowTheme(hChild, L"DarkMode_Explorer", nullptr);
            } else if (_wcsicmp(className, L"ListBox") == 0) {
                SetWindowTheme(hChild, L"DarkMode_Explorer", nullptr);
            } else {
                SetWindowTheme(hChild, L"DarkMode_Explorer", nullptr);
            }
            return TRUE;
        }, 0);

        // Send theme change notification
        SendMessageW(hDlg, WM_THEMECHANGED, 0, 0);
    }
}

// Center dialog on screen
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

// Scale dialog for DPI and enable dark mode
void ScaleDialogForDpi(HWND hDlg) {
    EnableDarkModeForDialog(hDlg);
    CenterDialog(hDlg);
}

// Settings dialog helper functions
void InitSettingsDialog(HWND hDlg) {
    Settings& settings = Settings::Instance();

    // Set title like EVKey style with icon
    SetWindowTextW(hDlg, L"ViKey x64 - 1.3.3 - Nh\u1EB9, nhanh, chu\u1EA9n Vi\u1EC7t");
    // Set dialog icon (shows in title bar)
    HICON hIcon = LoadIconW(g_hInstance, MAKEINTRESOURCEW(IDI_LOGO));
    SendMessageW(hDlg, WM_SETICON, ICON_SMALL, (LPARAM)hIcon);
    SendMessageW(hDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);

    // Fix Vietnamese text (resource file encoding workaround)
    SetDlgItemTextW(hDlg, IDC_CHECK_MODERN, L"B\u1ECF d\u1EA5u ki\u1EC3u m\u1EDBi");  // Bỏ dấu kiểu mới
    SetDlgItemTextW(hDlg, IDC_CHECK_AUTORESTORE, L"T\u1EF1 \u0111\u1ED9ng kh\u00F4i ph\u1EE5c ti\u1EBFng Anh");
    SetDlgItemTextW(hDlg, IDC_CHECK_AUTOCAP, L"T\u1EF1 \u0111\u1ED9ng vi\u1EBFt hoa");
    SetDlgItemTextW(hDlg, IDC_CHECK_ESCRESTORE, L"ESC kh\u00F4i ph\u1EE5c ASCII");
    SetDlgItemTextW(hDlg, IDC_CHECK_FREETONE, L"B\u1ECF d\u1EA5u t\u1EF1 do");
    SetDlgItemTextW(hDlg, IDC_CHECK_SKIPW, L"B\u1ECF qua ph\u00EDm t\u1EAFt w");
    SetDlgItemTextW(hDlg, IDC_CHECK_BRACKET, L"D\u1EA5u ngo\u1EB7c l\u00E0m ph\u00EDm t\u1EAFt");
    SetDlgItemTextW(hDlg, IDC_CHECK_FOREIGN, L"f,j,w,z ph\u1EE5 \u00E2m");
    SetDlgItemTextW(hDlg, IDC_CHECK_SLOWMODE, L"Ch\u1EBF \u0111\u1ED9 ch\u1EADm (terminal)");
    SetDlgItemTextW(hDlg, IDC_CHECK_CLIPBOARD, L"Ch\u1EBF \u0111\u1ED9 clipboard");
    SetDlgItemTextW(hDlg, IDC_CHECK_SMARTSWITCH, L"Nh\u1EDB theo \u1EE9ng d\u1EE5ng");
    SetDlgItemTextW(hDlg, IDC_CHECK_AUTOSTART, L"Kh\u1EDFi \u0111\u1ED9ng c\u00F9ng Windows");
    SetDlgItemTextW(hDlg, IDC_CHECK_SILENT, L"\u1EA8n khi kh\u1EDFi \u0111\u1ED9ng");
    SetDlgItemTextW(hDlg, IDC_CHECK_SHORTCUT_ENABLED, L"Cho ph\u00E9p g\u00F5 t\u1EAFt");
    SetDlgItemTextW(hDlg, IDC_CHECK_AUTO_UPDATE, L"Ki\u1EC3m tra c\u1EADp nh\u1EADt");
    SetDlgItemTextW(hDlg, IDC_BTN_OK, L"L\u01B0u");
    SetDlgItemTextW(hDlg, IDC_BTN_CANCEL, L"Hu\u1EF7");

    // Input method combobox
    HWND hComboMethod = GetDlgItem(hDlg, IDC_COMBO_METHOD);
    SendMessageW(hComboMethod, CB_ADDSTRING, 0, (LPARAM)L"Telex");
    SendMessageW(hComboMethod, CB_ADDSTRING, 0, (LPARAM)L"VNI");
    SendMessageW(hComboMethod, CB_SETCURSEL, settings.method == InputMethod::Telex ? 0 : 1, 0);

    // Shortcuts enabled
    CheckDlgButton(hDlg, IDC_CHECK_SHORTCUT_ENABLED, settings.shortcutsEnabled ? BST_CHECKED : BST_UNCHECKED);

    // Checkboxes
    CheckDlgButton(hDlg, IDC_CHECK_MODERN, settings.modernTone ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_AUTORESTORE, settings.englishAutoRestore ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_AUTOCAP, settings.autoCapitalize ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_ESCRESTORE, settings.escRestore ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_FREETONE, settings.freeTone ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_SKIPW, settings.skipWShortcut ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_BRACKET, settings.bracketShortcut ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_FOREIGN, settings.allowForeignConsonants ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_SLOWMODE, settings.slowMode ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_CLIPBOARD, settings.clipboardMode ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_SMARTSWITCH, settings.smartSwitch ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_AUTOSTART, settings.autoStart ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_SILENT, settings.silentStartup ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_AUTO_UPDATE, settings.checkForUpdates ? BST_CHECKED : BST_UNCHECKED);

    // Hotkey configuration
    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_CTRL, settings.toggleHotkey.ctrl ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_SHIFT, settings.toggleHotkey.shift ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_ALT, settings.toggleHotkey.alt ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_WIN, settings.toggleHotkey.win ? BST_CHECKED : BST_UNCHECKED);

    // Set hotkey character (convert VK code to displayable text)
    wchar_t hotkeyChar[16] = {0};
    if (settings.toggleHotkey.vkCode == VK_SPACE) {
        wcscpy_s(hotkeyChar, L"Space");
    } else if (settings.toggleHotkey.vkCode >= 0x41 && settings.toggleHotkey.vkCode <= 0x5A) {
        hotkeyChar[0] = static_cast<wchar_t>(settings.toggleHotkey.vkCode);
    } else if (settings.toggleHotkey.vkCode >= 0x30 && settings.toggleHotkey.vkCode <= 0x39) {
        hotkeyChar[0] = static_cast<wchar_t>(settings.toggleHotkey.vkCode);
    }
    SetDlgItemTextW(hDlg, IDC_EDIT_HOTKEY, hotkeyChar);
}

void SaveSettingsFromDialog(HWND hDlg) {
    Settings& settings = Settings::Instance();

    // Input method from combobox
    HWND hComboMethod = GetDlgItem(hDlg, IDC_COMBO_METHOD);
    int methodIdx = (int)SendMessageW(hComboMethod, CB_GETCURSEL, 0, 0);
    settings.method = (methodIdx == 0) ? InputMethod::Telex : InputMethod::VNI;

    // Shortcuts enabled
    settings.shortcutsEnabled = IsDlgButtonChecked(hDlg, IDC_CHECK_SHORTCUT_ENABLED) == BST_CHECKED;

    // Checkboxes
    settings.modernTone = IsDlgButtonChecked(hDlg, IDC_CHECK_MODERN) == BST_CHECKED;
    settings.englishAutoRestore = IsDlgButtonChecked(hDlg, IDC_CHECK_AUTORESTORE) == BST_CHECKED;
    settings.autoCapitalize = IsDlgButtonChecked(hDlg, IDC_CHECK_AUTOCAP) == BST_CHECKED;
    settings.escRestore = IsDlgButtonChecked(hDlg, IDC_CHECK_ESCRESTORE) == BST_CHECKED;
    settings.freeTone = IsDlgButtonChecked(hDlg, IDC_CHECK_FREETONE) == BST_CHECKED;
    settings.skipWShortcut = IsDlgButtonChecked(hDlg, IDC_CHECK_SKIPW) == BST_CHECKED;
    settings.bracketShortcut = IsDlgButtonChecked(hDlg, IDC_CHECK_BRACKET) == BST_CHECKED;
    settings.allowForeignConsonants = IsDlgButtonChecked(hDlg, IDC_CHECK_FOREIGN) == BST_CHECKED;
    settings.slowMode = IsDlgButtonChecked(hDlg, IDC_CHECK_SLOWMODE) == BST_CHECKED;
    settings.clipboardMode = IsDlgButtonChecked(hDlg, IDC_CHECK_CLIPBOARD) == BST_CHECKED;
    settings.smartSwitch = IsDlgButtonChecked(hDlg, IDC_CHECK_SMARTSWITCH) == BST_CHECKED;
    settings.autoStart = IsDlgButtonChecked(hDlg, IDC_CHECK_AUTOSTART) == BST_CHECKED;
    settings.silentStartup = IsDlgButtonChecked(hDlg, IDC_CHECK_SILENT) == BST_CHECKED;
    settings.checkForUpdates = IsDlgButtonChecked(hDlg, IDC_CHECK_AUTO_UPDATE) == BST_CHECKED;

    // Hotkey configuration
    settings.toggleHotkey.ctrl = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_CTRL) == BST_CHECKED;
    settings.toggleHotkey.shift = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_SHIFT) == BST_CHECKED;
    settings.toggleHotkey.alt = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_ALT) == BST_CHECKED;
    settings.toggleHotkey.win = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_WIN) == BST_CHECKED;

    // Get hotkey character
    wchar_t hotkeyChar[16] = {0};
    GetDlgItemTextW(hDlg, IDC_EDIT_HOTKEY, hotkeyChar, 16);
    if (_wcsicmp(hotkeyChar, L"Space") == 0 || hotkeyChar[0] == L' ' || hotkeyChar[0] == 0) {
        settings.toggleHotkey.vkCode = VK_SPACE;
    } else if (hotkeyChar[0] >= L'A' && hotkeyChar[0] <= L'Z') {
        settings.toggleHotkey.vkCode = static_cast<UINT>(hotkeyChar[0]);
    } else if (hotkeyChar[0] >= L'a' && hotkeyChar[0] <= L'z') {
        settings.toggleHotkey.vkCode = static_cast<UINT>(hotkeyChar[0] - L'a' + L'A');
    } else if (hotkeyChar[0] >= L'0' && hotkeyChar[0] <= L'9') {
        settings.toggleHotkey.vkCode = static_cast<UINT>(hotkeyChar[0]);
    }

    // Save to registry
    settings.Save();

    // Apply to processor
    ImeProcessor::Instance().ApplySettings();

    // Update hotkey registration
    HotkeyManager::Instance().UpdateHotkey(g_hWnd);

    UpdateUI();
}

INT_PTR CALLBACK SettingsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    // Handle dark mode colors
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG:
        InitSettingsDialog(hDlg);
        ScaleDialogForDpi(hDlg);
        return TRUE;

    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDC_BTN_SHORTCUTS:
            ShowShortcutsDialog();
            return TRUE;

        case IDC_BTN_EXCLUDE:
            ShowExcludeAppsDialog();
            return TRUE;

        case IDC_BTN_CONVERTER:
            ShowConverterDialog();
            return TRUE;

        case IDOK:
            SaveSettingsFromDialog(hDlg);
            EndDialog(hDlg, IDOK);
            return TRUE;

        case IDCANCEL:
            EndDialog(hDlg, IDCANCEL);
            return TRUE;
        }
        break;
    }
    return FALSE;
}

INT_PTR CALLBACK AboutDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);

    // Handle dark mode colors
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG:
        ScaleDialogForDpi(hDlg);
        return TRUE;

    case WM_COMMAND:
        if (LOWORD(wParam) == IDOK || LOWORD(wParam) == IDCANCEL) {
            EndDialog(hDlg, LOWORD(wParam));
            return TRUE;
        }
        break;
    }
    return FALSE;
}

// Export/Import Settings (Feature 5)
void ExportSettings() {
    wchar_t filePath[MAX_PATH] = L"vikey-settings.json";

    OPENFILENAMEW ofn = {};
    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = g_hWnd;
    ofn.lpstrFilter = L"JSON Files (*.json)\0*.json\0All Files (*.*)\0*.*\0";
    ofn.lpstrFile = filePath;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrTitle = L"Xu\u1EA5t c\u00E0i \u0111\u1EB7t ViKey";
    ofn.Flags = OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST;
    ofn.lpstrDefExt = L"json";

    if (GetSaveFileNameW(&ofn)) {
        if (Settings::ExportToFile(filePath)) {
            TrayIcon::Instance().ShowBalloon(L"Xu\u1EA5t th\u00E0nh c\u00F4ng",
                L"\u0110\u00E3 l\u01B0u c\u00E0i \u0111\u1EB7t v\u00E0o file");
        } else {
            MessageBoxW(g_hWnd, L"Kh\u00F4ng th\u1EC3 l\u01B0u file", L"L\u1ED7i", MB_ICONERROR);
        }
    }
}

void ImportSettings() {
    wchar_t filePath[MAX_PATH] = L"";

    OPENFILENAMEW ofn = {};
    ofn.lStructSize = sizeof(ofn);
    ofn.hwndOwner = g_hWnd;
    ofn.lpstrFilter = L"JSON Files (*.json)\0*.json\0All Files (*.*)\0*.*\0";
    ofn.lpstrFile = filePath;
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrTitle = L"Nh\u1EADp c\u00E0i \u0111\u1EB7t ViKey";
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

    if (GetOpenFileNameW(&ofn)) {
        if (Settings::ImportFromFile(filePath)) {
            // Apply imported settings
            ImeProcessor::Instance().ApplySettings();
            HotkeyManager::Instance().UpdateHotkey(g_hWnd);
            UpdateUI();
            TrayIcon::Instance().ShowBalloon(L"Nh\u1EADp th\u00E0nh c\u00F4ng",
                L"\u0110\u00E3 kh\u00F4i ph\u1EE5c c\u00E0i \u0111\u1EB7t t\u1EEB file");
        } else {
            MessageBoxW(g_hWnd, L"File kh\u00F4ng h\u1EE3p l\u1EC7 ho\u1EB7c kh\u00F4ng \u0111\u1ECDc \u0111\u01B0\u1EE3c",
                L"L\u1ED7i", MB_ICONERROR);
        }
    }
}

// Exclude Apps Dialog (Feature 3)
void ShowExcludeAppsDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_EXCLUDE_APPS), g_hWnd, ExcludeAppsDialogProc);
    isOpen = false;
}

INT_PTR CALLBACK ExcludeAppsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);

    // Handle dark mode colors
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG: {
        ScaleDialogForDpi(hDlg);
        // Set Vietnamese text
        SetWindowTextW(hDlg, L"Lo\u1EA1i tr\u1EEB \u1EE9ng d\u1EE5ng");
        SetDlgItemTextW(hDlg, IDC_BTN_GET_CURRENT, L"L\u1EA5y app");
        SetDlgItemTextW(hDlg, IDOK, L"L\u01B0u");
        SetDlgItemTextW(hDlg, IDCANCEL, L"Hu\u1EF7");

        // Populate list with current excluded apps
        HWND hList = GetDlgItem(hDlg, IDC_LIST_EXCLUDED);
        for (const auto& app : Settings::Instance().excludedApps) {
            SendMessageW(hList, LB_ADDSTRING, 0, (LPARAM)app.c_str());
        }
        return TRUE;
    }

    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDC_BTN_ADD_EXCLUDED: {
            wchar_t appName[256] = {};
            GetDlgItemTextW(hDlg, IDC_EDIT_APP_NAME, appName, 256);
            if (wcslen(appName) > 0) {
                HWND hList = GetDlgItem(hDlg, IDC_LIST_EXCLUDED);
                SendMessageW(hList, LB_ADDSTRING, 0, (LPARAM)appName);
                SetDlgItemTextW(hDlg, IDC_EDIT_APP_NAME, L"");
            }
            return TRUE;
        }

        case IDC_BTN_REMOVE_EXCLUDED: {
            HWND hList = GetDlgItem(hDlg, IDC_LIST_EXCLUDED);
            int sel = (int)SendMessageW(hList, LB_GETCURSEL, 0, 0);
            if (sel != LB_ERR) {
                SendMessageW(hList, LB_DELETESTRING, sel, 0);
            }
            return TRUE;
        }

        case IDC_BTN_GET_CURRENT: {
            std::wstring currentApp = AppDetector::Instance().GetForegroundAppName();
            if (!currentApp.empty()) {
                SetDlgItemTextW(hDlg, IDC_EDIT_APP_NAME, currentApp.c_str());
            }
            return TRUE;
        }

        case IDOK: {
            // Save list to settings
            Settings& settings = Settings::Instance();
            settings.excludedApps.clear();

            HWND hList = GetDlgItem(hDlg, IDC_LIST_EXCLUDED);
            int count = (int)SendMessageW(hList, LB_GETCOUNT, 0, 0);
            for (int i = 0; i < count; i++) {
                wchar_t app[256] = {};
                SendMessageW(hList, LB_GETTEXT, i, (LPARAM)app);
                if (wcslen(app) > 0) {
                    settings.excludedApps.push_back(app);
                }
            }

            settings.Save();
            ImeProcessor::Instance().ApplySettings();

            EndDialog(hDlg, IDOK);
            return TRUE;
        }

        case IDCANCEL:
            EndDialog(hDlg, IDCANCEL);
            return TRUE;
        }
        break;
    }
    return FALSE;
}

// Converter Dialog (Feature 6)
void ShowConverterDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_CONVERTER), g_hWnd, ConverterDialogProc);
    isOpen = false;
}

INT_PTR CALLBACK ConverterDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);

    // Handle dark mode colors
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG: {
        ScaleDialogForDpi(hDlg);
        // Set Vietnamese text
        SetWindowTextW(hDlg, L"Chuy\u1EC3n m\u00E3 ti\u1EBFng Vi\u1EC7t");
        SetDlgItemTextW(hDlg, IDC_BTN_CONVERT, L"Chuy\u1EC3n \u0111\u1ED5i");
        SetDlgItemTextW(hDlg, IDC_BTN_COPY, L"Sao ch\u00E9p");
        SetDlgItemTextW(hDlg, IDCANCEL, L"\u0110\u00F3ng");

        // Populate encoding combos
        HWND hFrom = GetDlgItem(hDlg, IDC_COMBO_FROM);
        HWND hTo = GetDlgItem(hDlg, IDC_COMBO_TO);

        for (int i = 0; i <= 3; i++) {
            const wchar_t* name = EncodingConverter::GetEncodingName(static_cast<VietEncoding>(i));
            SendMessageW(hFrom, CB_ADDSTRING, 0, (LPARAM)name);
            SendMessageW(hTo, CB_ADDSTRING, 0, (LPARAM)name);
        }

        // Default: VNI -> Unicode
        SendMessageW(hFrom, CB_SETCURSEL, 1, 0);
        SendMessageW(hTo, CB_SETCURSEL, 0, 0);

        return TRUE;
    }

    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDC_BTN_CONVERT: {
            int len = GetWindowTextLengthW(GetDlgItem(hDlg, IDC_EDIT_SOURCE));
            if (len > 0) {
                std::wstring source(len + 1, L'\0');
                GetDlgItemTextW(hDlg, IDC_EDIT_SOURCE, &source[0], len + 1);
                source.resize(len);

                int fromIdx = (int)SendMessageW(GetDlgItem(hDlg, IDC_COMBO_FROM), CB_GETCURSEL, 0, 0);
                int toIdx = (int)SendMessageW(GetDlgItem(hDlg, IDC_COMBO_TO), CB_GETCURSEL, 0, 0);

                VietEncoding from = static_cast<VietEncoding>(fromIdx);
                VietEncoding to = static_cast<VietEncoding>(toIdx);

                std::wstring result = EncodingConverter::Instance().Convert(source, from, to);
                SetDlgItemTextW(hDlg, IDC_EDIT_TARGET, result.c_str());
            }
            return TRUE;
        }

        case IDC_BTN_SWAP: {
            HWND hFrom = GetDlgItem(hDlg, IDC_COMBO_FROM);
            HWND hTo = GetDlgItem(hDlg, IDC_COMBO_TO);
            int fromIdx = (int)SendMessageW(hFrom, CB_GETCURSEL, 0, 0);
            int toIdx = (int)SendMessageW(hTo, CB_GETCURSEL, 0, 0);
            SendMessageW(hFrom, CB_SETCURSEL, toIdx, 0);
            SendMessageW(hTo, CB_SETCURSEL, fromIdx, 0);
            return TRUE;
        }

        case IDC_BTN_COPY: {
            int len = GetWindowTextLengthW(GetDlgItem(hDlg, IDC_EDIT_TARGET));
            if (len > 0) {
                std::wstring text(len + 1, L'\0');
                GetDlgItemTextW(hDlg, IDC_EDIT_TARGET, &text[0], len + 1);

                if (OpenClipboard(hDlg)) {
                    EmptyClipboard();
                    size_t size = (len + 1) * sizeof(wchar_t);
                    HGLOBAL hGlobal = GlobalAlloc(GMEM_MOVEABLE, size);
                    if (hGlobal) {
                        wchar_t* pGlobal = static_cast<wchar_t*>(GlobalLock(hGlobal));
                        if (pGlobal) {
                            wcscpy_s(pGlobal, len + 1, text.c_str());
                            GlobalUnlock(hGlobal);
                            SetClipboardData(CF_UNICODETEXT, hGlobal);
                        }
                    }
                    CloseClipboard();
                }
            }
            return TRUE;
        }

        case IDCANCEL:
            EndDialog(hDlg, IDCANCEL);
            return TRUE;
        }
        break;
    }
    return FALSE;
}

// Shortcuts Dialog (Gõ tắt)
void ShowShortcutsDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_SHORTCUTS), g_hWnd, ShortcutsDialogProc);
    isOpen = false;
}

void InitShortcutsListView(HWND hList) {
    ListView_SetExtendedListViewStyle(hList, LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER);

    // Set dark mode colors for ListView
    if (g_isDarkMode) {
        ListView_SetBkColor(hList, RGB(32, 32, 32));
        ListView_SetTextBkColor(hList, RGB(32, 32, 32));
        ListView_SetTextColor(hList, RGB(255, 255, 255));
    }

    // Get ListView width to calculate column sizes
    RECT rcList;
    GetClientRect(hList, &rcList);
    int listWidth = rcList.right - rcList.left - GetSystemMetrics(SM_CXVSCROLL) - 4;

    LVCOLUMNW col = {};
    col.mask = LVCF_TEXT | LVCF_WIDTH;

    col.pszText = (LPWSTR)L"T\u1EAFt";
    col.cx = 60;
    ListView_InsertColumn(hList, 0, &col);

    col.pszText = (LPWSTR)L"Thay th\u1EBF";
    col.cx = listWidth - 60;  // Fill remaining space
    ListView_InsertColumn(hList, 1, &col);

    // Load shortcuts from settings
    for (size_t i = 0; i < Settings::Instance().shortcuts.size(); i++) {
        LVITEMW item = {};
        item.mask = LVIF_TEXT;
        item.iItem = static_cast<int>(i);
        item.pszText = (LPWSTR)Settings::Instance().shortcuts[i].key.c_str();
        ListView_InsertItem(hList, &item);
        ListView_SetItemText(hList, static_cast<int>(i), 1, (LPWSTR)Settings::Instance().shortcuts[i].value.c_str());
    }
}

INT_PTR CALLBACK ShortcutsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);

    // Handle dark mode colors
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG: {
        ScaleDialogForDpi(hDlg);
        SetWindowTextW(hDlg, L"G\u00F5 t\u1EAFt - ViKey");
        SetDlgItemTextW(hDlg, IDC_BTN_ADD, L"+");
        SetDlgItemTextW(hDlg, IDC_BTN_REMOVE, L"-");
        SetDlgItemTextW(hDlg, IDC_BTN_DEFAULT, L"M\u1EB7c \u0111\u1ECBnh");
        SetDlgItemTextW(hDlg, IDC_BTN_EXPORT_SC, L"Xu\u1EA5t...");
        SetDlgItemTextW(hDlg, IDC_BTN_IMPORT_SC, L"Nh\u1EADp...");
        SetDlgItemTextW(hDlg, IDC_BTN_OK, L"L\u01B0u");
        SetDlgItemTextW(hDlg, IDC_BTN_CANCEL, L"Hu\u1EF7");

        HWND hList = GetDlgItem(hDlg, IDC_LIST_SHORTCUTS);
        InitShortcutsListView(hList);
        return TRUE;
    }

    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDC_BTN_ADD: {
            wchar_t key[64] = {};
            wchar_t value[256] = {};
            GetDlgItemTextW(hDlg, IDC_EDIT_KEY, key, 64);
            GetDlgItemTextW(hDlg, IDC_EDIT_VALUE, value, 256);

            if (wcslen(key) > 0 && wcslen(value) > 0) {
                HWND hList = GetDlgItem(hDlg, IDC_LIST_SHORTCUTS);
                int idx = ListView_GetItemCount(hList);

                LVITEMW item = {};
                item.mask = LVIF_TEXT;
                item.iItem = idx;
                item.pszText = key;
                ListView_InsertItem(hList, &item);
                ListView_SetItemText(hList, idx, 1, value);

                SetDlgItemTextW(hDlg, IDC_EDIT_KEY, L"");
                SetDlgItemTextW(hDlg, IDC_EDIT_VALUE, L"");
            }
            return TRUE;
        }

        case IDC_BTN_REMOVE: {
            HWND hList = GetDlgItem(hDlg, IDC_LIST_SHORTCUTS);
            int sel = ListView_GetNextItem(hList, -1, LVNI_SELECTED);
            if (sel >= 0) {
                ListView_DeleteItem(hList, sel);
            }
            return TRUE;
        }

        case IDC_BTN_DEFAULT: {
            if (MessageBoxW(hDlg, L"Kh\u00F4i ph\u1EE5c danh s\u00E1ch g\u00F5 t\u1EAFt m\u1EB7c \u0111\u1ECBnh?",
                           L"X\u00E1c nh\u1EADn", MB_YESNO | MB_ICONQUESTION) == IDYES) {
                HWND hList = GetDlgItem(hDlg, IDC_LIST_SHORTCUTS);
                ListView_DeleteAllItems(hList);
                auto defaults = Settings::DefaultShortcuts();
                for (size_t i = 0; i < defaults.size(); i++) {
                    LVITEMW item = {};
                    item.mask = LVIF_TEXT;
                    item.iItem = static_cast<int>(i);
                    item.pszText = (LPWSTR)defaults[i].key.c_str();
                    ListView_InsertItem(hList, &item);
                    ListView_SetItemText(hList, static_cast<int>(i), 1, (LPWSTR)defaults[i].value.c_str());
                }
            }
            return TRUE;
        }

        case IDC_BTN_EXPORT_SC: {
            HWND hList = GetDlgItem(hDlg, IDC_LIST_SHORTCUTS);
            int count = ListView_GetItemCount(hList);
            Settings::Instance().shortcuts.clear();
            for (int i = 0; i < count; i++) {
                wchar_t key[64] = {};
                wchar_t value[256] = {};
                ListView_GetItemText(hList, i, 0, key, 64);
                ListView_GetItemText(hList, i, 1, value, 256);
                if (wcslen(key) > 0 && wcslen(value) > 0) {
                    Settings::Instance().shortcuts.push_back({key, value});
                }
            }

            wchar_t filePath[MAX_PATH] = L"vikey-shortcuts.json";
            OPENFILENAMEW ofn = {};
            ofn.lStructSize = sizeof(ofn);
            ofn.hwndOwner = hDlg;
            ofn.lpstrFilter = L"JSON Files (*.json)\0*.json\0All Files (*.*)\0*.*\0";
            ofn.lpstrFile = filePath;
            ofn.nMaxFile = MAX_PATH;
            ofn.lpstrTitle = L"Xu\u1EA5t g\u00F5 t\u1EAFt";
            ofn.Flags = OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST;
            ofn.lpstrDefExt = L"json";

            if (GetSaveFileNameW(&ofn)) {
                if (Settings::ExportShortcutsToFile(filePath)) {
                    MessageBoxW(hDlg, L"\u0110\u00E3 xu\u1EA5t th\u00E0nh c\u00F4ng!", L"Th\u00E0nh c\u00F4ng", MB_ICONINFORMATION);
                } else {
                    MessageBoxW(hDlg, L"Kh\u00F4ng th\u1EC3 l\u01B0u file", L"L\u1ED7i", MB_ICONERROR);
                }
            }
            return TRUE;
        }

        case IDC_BTN_IMPORT_SC: {
            wchar_t filePath[MAX_PATH] = L"";
            OPENFILENAMEW ofn = {};
            ofn.lStructSize = sizeof(ofn);
            ofn.hwndOwner = hDlg;
            ofn.lpstrFilter = L"JSON Files (*.json)\0*.json\0All Files (*.*)\0*.*\0";
            ofn.lpstrFile = filePath;
            ofn.nMaxFile = MAX_PATH;
            ofn.lpstrTitle = L"Nh\u1EADp g\u00F5 t\u1EAFt";
            ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST;

            if (GetOpenFileNameW(&ofn)) {
                if (Settings::ImportShortcutsFromFile(filePath)) {
                    HWND hList = GetDlgItem(hDlg, IDC_LIST_SHORTCUTS);
                    ListView_DeleteAllItems(hList);
                    for (size_t i = 0; i < Settings::Instance().shortcuts.size(); i++) {
                        LVITEMW item = {};
                        item.mask = LVIF_TEXT;
                        item.iItem = static_cast<int>(i);
                        item.pszText = (LPWSTR)Settings::Instance().shortcuts[i].key.c_str();
                        ListView_InsertItem(hList, &item);
                        ListView_SetItemText(hList, static_cast<int>(i), 1, (LPWSTR)Settings::Instance().shortcuts[i].value.c_str());
                    }
                    MessageBoxW(hDlg, L"\u0110\u00E3 nh\u1EADp th\u00E0nh c\u00F4ng!", L"Th\u00E0nh c\u00F4ng", MB_ICONINFORMATION);
                } else {
                    MessageBoxW(hDlg, L"File kh\u00F4ng h\u1EE3p l\u1EC7", L"L\u1ED7i", MB_ICONERROR);
                }
            }
            return TRUE;
        }

        case IDOK: {
            HWND hList = GetDlgItem(hDlg, IDC_LIST_SHORTCUTS);
            int count = ListView_GetItemCount(hList);

            Settings::Instance().shortcuts.clear();
            for (int i = 0; i < count; i++) {
                wchar_t key[64] = {};
                wchar_t value[256] = {};
                ListView_GetItemText(hList, i, 0, key, 64);
                ListView_GetItemText(hList, i, 1, value, 256);
                if (wcslen(key) > 0 && wcslen(value) > 0) {
                    Settings::Instance().shortcuts.push_back({key, value});
                }
            }

            Settings::Instance().Save();
            ImeProcessor::Instance().ApplySettings();
            EndDialog(hDlg, IDOK);
            return TRUE;
        }

        case IDCANCEL:
            EndDialog(hDlg, IDCANCEL);
            return TRUE;
        }
        break;

    case WM_NOTIFY: {
        LPNMHDR pnmh = (LPNMHDR)lParam;
        if (pnmh->idFrom == IDC_LIST_SHORTCUTS && pnmh->code == NM_CUSTOMDRAW) {
            LPNMLVCUSTOMDRAW lplvcd = (LPNMLVCUSTOMDRAW)lParam;
            switch (lplvcd->nmcd.dwDrawStage) {
            case CDDS_PREPAINT:
                return CDRF_NOTIFYITEMDRAW;

            case CDDS_ITEMPREPAINT: {
                int iRow = (int)lplvcd->nmcd.dwItemSpec;
                if (g_isDarkMode) {
                    // Alternating row colors for dark mode
                    if (iRow % 2 == 0) {
                        lplvcd->clrTextBk = RGB(45, 45, 45);
                    } else {
                        lplvcd->clrTextBk = RGB(32, 32, 32);
                    }
                    lplvcd->clrText = RGB(255, 255, 255);
                }
                return CDRF_NEWFONT;
            }
            }
        }
        break;
    }
    }
    return FALSE;
}

// ============================================================
// Update Check Functions
// ============================================================

// Global to store update info for dialog
static UpdateInfo g_updateInfo;

void CheckForUpdatesOnStartup() {
    // Check async to not block startup
    Updater::Instance().CheckForUpdatesAsync(g_hWnd, [](const UpdateInfo& info) {
        // Callback is handled via WM_UPDATE_CHECK_COMPLETE message
    });
}

void CheckForUpdatesManual() {
    // Show checking message
    TrayIcon::Instance().ShowBalloon(L"Ki\u1EC3m tra c\u1EADp nh\u1EADt",
        L"\u0110ang ki\u1EC3m tra phi\u00EAn b\u1EA3n m\u1EDBi...");

    // Check sync for manual request
    UpdateInfo info = Updater::Instance().CheckForUpdates();

    if (!info.error.empty()) {
        MessageBoxW(g_hWnd, info.error.c_str(), L"L\u1ED7i ki\u1EC3m tra c\u1EADp nh\u1EADt", MB_ICONERROR);
    } else if (info.available) {
        ShowUpdateDialog(info);
    } else {
        wchar_t msg[256];
        swprintf_s(msg, L"B\u1EA1n \u0111ang s\u1EED d\u1EE5ng phi\u00EAn b\u1EA3n m\u1EDBi nh\u1EA5t (%s)", Updater::GetCurrentVersion());
        MessageBoxW(g_hWnd, msg, L"Ki\u1EC3m tra c\u1EADp nh\u1EADt", MB_ICONINFORMATION);
    }
}

void ShowUpdateDialog(const UpdateInfo& info) {
    g_updateInfo = info;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_UPDATE), g_hWnd, UpdateDialogProc);
}

INT_PTR CALLBACK UpdateDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);

    // Handle dark mode colors
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG: {
        ScaleDialogForDpi(hDlg);

        // Set dialog title and icon
        SetWindowTextW(hDlg, L"C\u00F3 phi\u00EAn b\u1EA3n m\u1EDBi!");
        HICON hIcon = LoadIconW(g_hInstance, MAKEINTRESOURCEW(IDI_LOGO));
        SendMessageW(hDlg, WM_SETICON, ICON_SMALL, (LPARAM)hIcon);
        SendMessageW(hDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);

        // Set version info
        wchar_t versionMsg[256];
        swprintf_s(versionMsg, L"Phi\u00EAn b\u1EA3n hi\u1EC7n t\u1EA1i: %s\nPhi\u00EAn b\u1EA3n m\u1EDBi: %s",
            Updater::GetCurrentVersion(), g_updateInfo.latestVersion.c_str());
        SetDlgItemTextW(hDlg, IDC_STATIC_VERSION, versionMsg);

        // Set release notes if available
        if (!g_updateInfo.releaseNotes.empty()) {
            SetDlgItemTextW(hDlg, IDC_STATIC_NOTES, g_updateInfo.releaseNotes.c_str());
        }

        // Set button text
        SetDlgItemTextW(hDlg, IDC_BTN_DOWNLOAD, L"C\u1EADp nh\u1EADt ngay");
        SetDlgItemTextW(hDlg, IDC_BTN_SKIP, L"\u0110\u1EC3 sau");
        SetDlgItemTextW(hDlg, IDC_CHECK_DISABLE_UPDATE, L"Kh\u00F4ng ki\u1EC3m tra t\u1EF1 \u0111\u1ED9ng");

        return TRUE;
    }

    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDC_BTN_DOWNLOAD: {
            // Check if user wants to disable auto-check
            if (IsDlgButtonChecked(hDlg, IDC_CHECK_DISABLE_UPDATE) == BST_CHECKED) {
                Settings::Instance().checkForUpdates = false;
                Settings::Instance().Save();
            }

            // Auto download and install update
            EndDialog(hDlg, IDOK);
            if (!Updater::Instance().DownloadAndInstall(g_updateInfo.latestVersion, g_hWnd)) {
                // Fallback to opening download page if auto-update fails
                MessageBoxW(g_hWnd, L"Kh\u00F4ng th\u1EC3 t\u1EF1 \u0111\u1ED9ng c\u1EADp nh\u1EADt.\nM\u1EDF trang t\u1EA3i v\u1EC1...",
                    L"L\u1ED7i c\u1EADp nh\u1EADt", MB_ICONWARNING);
                Updater::OpenDownloadPage();
            }
            return TRUE;
        }

        case IDC_BTN_SKIP:
        case IDCANCEL:
            // Check if user wants to disable auto-check
            if (IsDlgButtonChecked(hDlg, IDC_CHECK_DISABLE_UPDATE) == BST_CHECKED) {
                Settings::Instance().checkForUpdates = false;
                Settings::Instance().Save();
            }

            EndDialog(hDlg, IDCANCEL);
            return TRUE;
        }
        break;
    }
    return FALSE;
}
