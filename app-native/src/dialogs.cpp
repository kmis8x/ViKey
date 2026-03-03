// ViKey - Dialog Procedures Implementation
// dialogs.cpp
// Settings dialog, About dialog, Import/Export settings

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
// Settings Dialog
// ============================================================

static void InitSettingsDialog(HWND hDlg) {
    Settings& settings = Settings::Instance();

    wchar_t titleBuf[128];
    swprintf_s(titleBuf, L"ViKey x64 - %s - Nh\u1EB9, nhanh, chu\u1EA9n Vi\u1EC7t", VIKEY_VERSION);
    SetWindowTextW(hDlg, titleBuf);
    HICON hIcon = LoadIconW(g_hInstance, MAKEINTRESOURCEW(IDI_LOGO));
    SendMessageW(hDlg, WM_SETICON, ICON_SMALL, (LPARAM)hIcon);
    SendMessageW(hDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);

    SetDlgItemTextW(hDlg, IDC_CHECK_MODERN, L"B\u1ECF d\u1EA5u ki\u1EC3u m\u1EDBi");
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

    HWND hComboMethod = GetDlgItem(hDlg, IDC_COMBO_METHOD);
    SendMessageW(hComboMethod, CB_ADDSTRING, 0, (LPARAM)L"Telex");
    SendMessageW(hComboMethod, CB_ADDSTRING, 0, (LPARAM)L"VNI");
    SendMessageW(hComboMethod, CB_SETCURSEL, settings.method == InputMethod::Telex ? 0 : 1, 0);

    CheckDlgButton(hDlg, IDC_CHECK_SHORTCUT_ENABLED, settings.shortcutsEnabled ? BST_CHECKED : BST_UNCHECKED);
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

    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_CTRL, settings.toggleHotkey.ctrl ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_SHIFT, settings.toggleHotkey.shift ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_ALT, settings.toggleHotkey.alt ? BST_CHECKED : BST_UNCHECKED);
    CheckDlgButton(hDlg, IDC_CHECK_HOTKEY_WIN, settings.toggleHotkey.win ? BST_CHECKED : BST_UNCHECKED);

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

static void SaveSettingsFromDialog(HWND hDlg) {
    Settings& settings = Settings::Instance();

    HWND hComboMethod = GetDlgItem(hDlg, IDC_COMBO_METHOD);
    int methodIdx = (int)SendMessageW(hComboMethod, CB_GETCURSEL, 0, 0);
    settings.method = (methodIdx == 0) ? InputMethod::Telex : InputMethod::VNI;

    settings.shortcutsEnabled = IsDlgButtonChecked(hDlg, IDC_CHECK_SHORTCUT_ENABLED) == BST_CHECKED;
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

    settings.toggleHotkey.ctrl = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_CTRL) == BST_CHECKED;
    settings.toggleHotkey.shift = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_SHIFT) == BST_CHECKED;
    settings.toggleHotkey.alt = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_ALT) == BST_CHECKED;
    settings.toggleHotkey.win = IsDlgButtonChecked(hDlg, IDC_CHECK_HOTKEY_WIN) == BST_CHECKED;

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

    settings.Save();
    ImeProcessor::Instance().ApplySettings();
    HotkeyManager::Instance().UpdateHotkey(g_hWnd);
    UpdateUI();
}

static INT_PTR CALLBACK SettingsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
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

void ShowSettingsDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_SETTINGS), g_hWnd, SettingsDialogProc);
    isOpen = false;
}

// ============================================================
// About Dialog
// ============================================================

static INT_PTR CALLBACK AboutDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);
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

void ShowAboutDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_ABOUT), g_hWnd, AboutDialogProc);
    isOpen = false;
}

// ============================================================
// Import/Export Settings
// ============================================================

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
