// ViKey - Shortcuts Dialog
// dialogs_shortcuts.cpp
// InitShortcutsListView, ShortcutsDialogProc, ShowShortcutsDialog

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
