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
#include <shellscalingapi.h>

#pragma comment(lib, "shcore.lib")
#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "gdiplus.lib")

#include "resource.h"
#include "dark_mode.h"
#include "dialogs.h"
#include "ime_processor.h"
#include "tray_icon.h"
#include "hotkey.h"
#include "settings.h"
#include "keyboard_hook.h"
#include "app_detector.h"
#include "text_sender.h"
#include "updater.h"

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

// Forward declarations
LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
bool InitInstance(HINSTANCE hInstance);
void CleanupInstance();

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

    // Message loop
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

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

    // Create hidden window for message handling
    g_hWnd = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        WINDOW_CLASS,
        APP_NAME,
        WS_POPUP,
        -32000, -32000,
        0, 0,
        nullptr,
        nullptr,
        hInstance,
        nullptr);
    if (!g_hWnd) {
        return false;
    }

    // Load settings
    Settings::Instance().Load();
    AppDetector::Instance().Load();

    // Initialize IME processor
    if (!ImeProcessor::Instance().Initialize()) {
        MessageBoxW(nullptr, L"Failed to load core.dll", APP_NAME, MB_ICONERROR);
        return false;
    }
    ImeProcessor::Instance().ApplySettings();

    // Initialize tray icon
    TrayIcon& tray = TrayIcon::Instance();
    tray.Initialize(g_hWnd, hInstance);

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

    tray.onSettings = []() { ShowSettingsDialog(); };
    tray.onAbout = []() { ShowAboutDialog(); };
    tray.onExit = []() { PostMessage(g_hWnd, WM_CLOSE, 0, 0); };

    // Register global hotkey
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
    UpdateUI();

    // Show Settings dialog or Toast on startup
    if (Settings::Instance().silentStartup) {
        const wchar_t* lang = Settings::Instance().enabled ? L"Ti\u1EBFng Vi\u1EC7t" : L"Ti\u1EBFng Anh";
        wchar_t msg[128];
        swprintf_s(msg, L"\u0110ang ch\u1EA1y \u1EDF ch\u1EBF \u0111\u1ED9 %s\nCtrl+Space \u0111\u1EC3 chuy\u1EC3n", lang);
        tray.ShowBalloon(L"B\u1ED9 g\u00F5 ti\u1EBFng Vi\u1EC7t", msg);
    } else {
        PostMessage(g_hWnd, WM_COMMAND, IDM_SETTINGS, 0);
    }

    // Check for updates on startup (async)
    if (Settings::Instance().checkForUpdates) {
        CheckForUpdatesOnStartup();
    }

    return true;
}

void CleanupInstance() {
    ImeProcessor::Instance().Stop();
    if (g_hWnd) {
        HotkeyManager::Instance().Unregister(g_hWnd);
    }
    TrayIcon::Instance().Shutdown();
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

            TextSender::Instance().SetOutputEncoding(enc);

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
