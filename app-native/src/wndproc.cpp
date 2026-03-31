// ViKey - Window Procedure
// wndproc.cpp
// WndProc message handler callback

#include <windows.h>
#include <objidl.h>
#include <gdiplus.h>
#include <commctrl.h>
#include <shellscalingapi.h>

#include "resource.h"
#include "dark_mode.h"
#include "dialogs.h"
#include "ime_processor.h"
#include "text_sender.h"
#include "tray_icon.h"
#include "hotkey.h"
#include "settings.h"
#include "keyboard_hook.h"
#include "app_detector.h"
#include "updater.h"

// Tray icon message ID (must match main.cpp and tray_icon.cpp)
constexpr UINT WM_TRAYICON_MSG = WM_USER + 1;

// Globals defined in main.cpp
extern HINSTANCE g_hInstance;
extern HWND g_hWnd;

// Forward declaration for UpdateUI defined in main.cpp
void UpdateUI();

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

    case WM_DEFERRED_CLIPBOARD: {
        auto* data = reinterpret_cast<DeferredClipboardData*>(lParam);
        TextSender::ExecuteDeferredClipboard(data);
        return 0;
    }

    case WM_SETTINGCHANGE: {
        if (lParam && wcscmp(reinterpret_cast<LPCWSTR>(lParam), L"ImmersiveColorSet") == 0) {
            RefreshDarkMode();
        }
        return 0;
    }

    case WM_HOTKEY:
        HotkeyManager::Instance().ProcessHotkey(wParam);
        return 0;

    case WM_TIMER:
        if (wParam == TIMER_HOOK_CHECK) {
            KeyboardHook::Instance().EnsureInstalled();
        }
        return 0;

    case WM_DESTROY:
        PostQuitMessage(0);
        return 0;

    default:
        return DefWindowProcW(hWnd, message, wParam, lParam);
    }
}
