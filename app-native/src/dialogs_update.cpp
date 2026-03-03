// ViKey - Update Dialog
// dialogs_update.cpp
// CheckForUpdates*, UpdateDialogProc, ShowUpdateDialog

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
// Update Dialog
// ============================================================

static UpdateInfo g_updateInfo;

void CheckForUpdatesOnStartup() {
    Updater::Instance().CheckForUpdatesAsync(g_hWnd, [](const UpdateInfo& info) {
        // Handled via WM_UPDATE_CHECK_COMPLETE message
    });
}

void CheckForUpdatesManual() {
    TrayIcon::Instance().ShowBalloon(L"Ki\u1EC3m tra c\u1EADp nh\u1EADt",
        L"\u0110ang ki\u1EC3m tra phi\u00EAn b\u1EA3n m\u1EDBi...");

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

static INT_PTR CALLBACK UpdateDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG: {
        ScaleDialogForDpi(hDlg);
        SetWindowTextW(hDlg, L"C\u00F3 phi\u00EAn b\u1EA3n m\u1EDBi!");
        HICON hIcon = LoadIconW(g_hInstance, MAKEINTRESOURCEW(IDI_LOGO));
        SendMessageW(hDlg, WM_SETICON, ICON_SMALL, (LPARAM)hIcon);
        SendMessageW(hDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);

        wchar_t versionMsg[256];
        swprintf_s(versionMsg, L"Phi\u00EAn b\u1EA3n hi\u1EC7n t\u1EA1i: %s\nPhi\u00EAn b\u1EA3n m\u1EDBi: %s",
            Updater::GetCurrentVersion(), g_updateInfo.latestVersion.c_str());
        SetDlgItemTextW(hDlg, IDC_STATIC_VERSION, versionMsg);

        if (!g_updateInfo.releaseNotes.empty()) {
            SetDlgItemTextW(hDlg, IDC_STATIC_NOTES, g_updateInfo.releaseNotes.c_str());
        }

        SetDlgItemTextW(hDlg, IDC_BTN_DOWNLOAD, L"C\u1EADp nh\u1EADt ngay");
        SetDlgItemTextW(hDlg, IDC_BTN_SKIP, L"\u0110\u1EC3 sau");
        SetDlgItemTextW(hDlg, IDC_CHECK_DISABLE_UPDATE, L"Kh\u00F4ng ki\u1EC3m tra t\u1EF1 \u0111\u1ED9ng");
        return TRUE;
    }

    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDC_BTN_DOWNLOAD: {
            if (IsDlgButtonChecked(hDlg, IDC_CHECK_DISABLE_UPDATE) == BST_CHECKED) {
                Settings::Instance().checkForUpdates = false;
                Settings::Instance().Save();
            }
            EndDialog(hDlg, IDOK);
            if (!Updater::Instance().DownloadAndInstall(g_updateInfo.latestVersion, g_hWnd)) {
                MessageBoxW(g_hWnd, L"Kh\u00F4ng th\u1EC3 t\u1EF1 \u0111\u1ED9ng c\u1EADp nh\u1EADt.\nM\u1EDF trang t\u1EA3i v\u1EC1...",
                    L"L\u1ED7i c\u1EADp nh\u1EADt", MB_ICONWARNING);
                Updater::OpenDownloadPage();
            }
            return TRUE;
        }
        case IDC_BTN_SKIP:
        case IDCANCEL:
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

void ShowUpdateDialog(const UpdateInfo& info) {
    g_updateInfo = info;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_UPDATE), g_hWnd, UpdateDialogProc);
}
