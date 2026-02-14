// ViKey - Dialog Procedures Implementation
// dialogs.cpp
// All dialog procs extracted from main.cpp for maintainability

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

// ============================================================
// Converter Dialog
// ============================================================

static INT_PTR CALLBACK ConverterDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);
    INT_PTR darkResult = HandleDarkModeColors(message, wParam);
    if (darkResult) return darkResult;

    switch (message) {
    case WM_INITDIALOG: {
        ScaleDialogForDpi(hDlg);
        SetWindowTextW(hDlg, L"Chuy\u1EC3n m\u00E3 ti\u1EBFng Vi\u1EC7t");
        SetDlgItemTextW(hDlg, IDC_BTN_CONVERT, L"Chuy\u1EC3n \u0111\u1ED5i");
        SetDlgItemTextW(hDlg, IDC_BTN_COPY, L"Sao ch\u00E9p");
        SetDlgItemTextW(hDlg, IDCANCEL, L"\u0110\u00F3ng");

        HWND hFrom = GetDlgItem(hDlg, IDC_COMBO_FROM);
        HWND hTo = GetDlgItem(hDlg, IDC_COMBO_TO);
        for (int i = 0; i <= 3; i++) {
            const wchar_t* name = EncodingConverter::GetEncodingName(static_cast<VietEncoding>(i));
            SendMessageW(hFrom, CB_ADDSTRING, 0, (LPARAM)name);
            SendMessageW(hTo, CB_ADDSTRING, 0, (LPARAM)name);
        }
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

void ShowConverterDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_CONVERTER), g_hWnd, ConverterDialogProc);
    isOpen = false;
}

// ============================================================
// Shortcuts Dialog
// ============================================================

static void InitShortcutsListView(HWND hList) {
    ListView_SetExtendedListViewStyle(hList, LVS_EX_FULLROWSELECT | LVS_EX_DOUBLEBUFFER);

    if (g_isDarkMode) {
        ListView_SetBkColor(hList, RGB(32, 32, 32));
        ListView_SetTextBkColor(hList, RGB(32, 32, 32));
        ListView_SetTextColor(hList, RGB(255, 255, 255));
    }

    RECT rcList;
    GetClientRect(hList, &rcList);
    int listWidth = rcList.right - rcList.left - GetSystemMetrics(SM_CXVSCROLL) - 4;

    LVCOLUMNW col = {};
    col.mask = LVCF_TEXT | LVCF_WIDTH;
    col.pszText = (LPWSTR)L"T\u1EAFt";
    col.cx = 60;
    ListView_InsertColumn(hList, 0, &col);
    col.pszText = (LPWSTR)L"Thay th\u1EBF";
    col.cx = listWidth - 60;
    ListView_InsertColumn(hList, 1, &col);

    for (size_t i = 0; i < Settings::Instance().shortcuts.size(); i++) {
        LVITEMW item = {};
        item.mask = LVIF_TEXT;
        item.iItem = static_cast<int>(i);
        item.pszText = (LPWSTR)Settings::Instance().shortcuts[i].key.c_str();
        ListView_InsertItem(hList, &item);
        ListView_SetItemText(hList, static_cast<int>(i), 1, (LPWSTR)Settings::Instance().shortcuts[i].value.c_str());
    }
}

static INT_PTR CALLBACK ShortcutsDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    UNREFERENCED_PARAMETER(lParam);
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
            wchar_t key[64] = {}, value[256] = {};
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
            if (sel >= 0) ListView_DeleteItem(hList, sel);
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
                wchar_t key[64] = {}, value[256] = {};
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
                wchar_t key[64] = {}, value[256] = {};
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
                    lplvcd->clrTextBk = (iRow % 2 == 0) ? RGB(45, 45, 45) : RGB(32, 32, 32);
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

void ShowShortcutsDialog() {
    static bool isOpen = false;
    if (isOpen) return;
    isOpen = true;
    DialogBoxW(g_hInstance, MAKEINTRESOURCEW(IDD_SHORTCUTS), g_hWnd, ShortcutsDialogProc);
    isOpen = false;
}

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
