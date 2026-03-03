// ViKey - Exclude Apps Dialog
// dialogs_exclude_apps.cpp
// ExcludeAppsDialogProc and ShowExcludeAppsDialog

#include "dialogs.h"
#include "dark_mode.h"
#include "resource.h"
#include "settings.h"
#include "ime_processor.h"
#include "tray_icon.h"
#include "hotkey.h"
#include "app_detector.h"
#include "encoding_converter.h"
#include <commctrl.h>
#include <commdlg.h>

// ============================================================
// Exclude Apps Dialog
// ============================================================

static INT_PTR CALLBACK ExcludeAppsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG: {
        ScaleDialogForDpi(hDlg);
        SetWindowTextW(hDlg, L"Lo\u1EA1i tr\u1EEB \u1EE9ng d\u1EE5ng");
        SetDlgItemTextW(hDlg, IDC_BTN_GET_CURRENT, L"L\u1EA5y app");
        SetDlgItemTextW(hDlg, IDOK, L"L\u01B0u");
        SetDlgItemTextW(hDlg, IDCANCEL, L"Hu\u1EF7");

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

void ShowExcludeAppsDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_EXCLUDE_APPS), g_hWnd, ExcludeAppsDialogProc);
    isOpen = false;
}
