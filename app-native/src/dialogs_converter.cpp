// ViKey - Converter Dialog
// dialogs_converter.cpp
// ConverterDialogProc and ShowConverterDialog

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
