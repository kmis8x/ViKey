// ViKey - System Tray Icon Implementation
// tray_icon.cpp
// Initialize, Shutdown, UpdateIcon, UpdateTooltip, UpdateMenu, ShowContextMenu, ShowBalloon

#include "tray_icon.h"
#include "resource.h"
#include <objidl.h>
#include <gdiplus.h>
#include <string>

#pragma comment(lib, "gdiplus.lib")

// Custom WM_TRAYICON message
constexpr UINT WM_TRAYICON_MSG = WM_USER + 1;

TrayIcon& TrayIcon::Instance() {
    static TrayIcon instance;
    return instance;
}

TrayIcon::TrayIcon()
    : m_hMenu(nullptr)
    , m_iconVN(nullptr)
    , m_iconEN(nullptr)
    , m_initialized(false) {
    ZeroMemory(&m_nid, sizeof(m_nid));
}

TrayIcon::~TrayIcon() {
    Shutdown();
}

bool TrayIcon::Initialize(HWND hWnd, HINSTANCE hInstance) {
    if (m_initialized) return true;

    // Load icons from embedded resources
    m_iconVN = (HICON)LoadImageW(hInstance, MAKEINTRESOURCEW(IDI_ICON_VN), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);
    m_iconEN = (HICON)LoadImageW(hInstance, MAKEINTRESOURCEW(IDI_ICON_EN), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);

    // Fallback to generated icons if resources not found
    if (!m_iconVN) m_iconVN = CreateLetterIcon(true);
    if (!m_iconEN) m_iconEN = CreateLetterIcon(false);

    InitializeMenu(hInstance);

    m_nid.cbSize = sizeof(NOTIFYICONDATAW);
    m_nid.hWnd = hWnd;
    m_nid.uID = 1;
    m_nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    m_nid.uCallbackMessage = WM_TRAYICON_MSG;
    m_nid.hIcon = m_iconVN;
    wcscpy_s(m_nid.szTip, L"Ti\u1EBFng Vi\u1EC7t [VN]\nCtrl+Space \u0111\u1EC3 chuy\u1EC3n");

    Shell_NotifyIconW(NIM_ADD, &m_nid);

    m_initialized = true;
    return true;
}

void TrayIcon::Shutdown() {
    if (!m_initialized) return;

    Shell_NotifyIconW(NIM_DELETE, &m_nid);

    if (m_hMenu) {
        DestroyMenu(m_hMenu);
        m_hMenu = nullptr;
    }

    if (m_iconVN) {
        DestroyIcon(m_iconVN);
        m_iconVN = nullptr;
    }

    if (m_iconEN) {
        DestroyIcon(m_iconEN);
        m_iconEN = nullptr;
    }

    m_initialized = false;
}

void TrayIcon::UpdateIcon(bool vietnamese) {
    if (!m_initialized) return;

    m_nid.hIcon = vietnamese ? m_iconVN : m_iconEN;
    Shell_NotifyIconW(NIM_MODIFY, &m_nid);
}

void TrayIcon::UpdateTooltip(bool vietnamese, InputMethod method) {
    if (!m_initialized) return;

    const wchar_t* lang = vietnamese ? L"Ti\u1EBFng Vi\u1EC7t" : L"Ti\u1EBFng Anh";
    const wchar_t* methodStr = method == InputMethod::Telex ? L"Telex" : L"VNI";

    swprintf_s(m_nid.szTip, L"%s (%s)\nCtrl+Space \u0111\u1EC3 chuy\u1EC3n", lang, methodStr);
    Shell_NotifyIconW(NIM_MODIFY, &m_nid);
}

void TrayIcon::UpdateMenu(bool vietnamese, InputMethod method) {
    if (!m_hMenu) return;

    HMENU hPopup = GetSubMenu(m_hMenu, 0);
    if (!hPopup) return;

    CheckMenuItem(hPopup, IDM_TOGGLE_ENABLED, vietnamese ? MF_CHECKED : MF_UNCHECKED);

    ModifyMenuW(hPopup, IDM_TOGGLE_ENABLED, MF_BYCOMMAND | MF_STRING,
                IDM_TOGGLE_ENABLED, vietnamese ? L"Ti\u1EBFng Vi\u1EC7t (VN)" : L"Ti\u1EBFng Anh (EN)");

    CheckMenuItem(hPopup, IDM_METHOD_TELEX, method == InputMethod::Telex ? MF_CHECKED : MF_UNCHECKED);
    CheckMenuItem(hPopup, IDM_METHOD_VNI, method == InputMethod::VNI ? MF_CHECKED : MF_UNCHECKED);
}

void TrayIcon::ShowContextMenu(HWND hWnd) {
    if (!m_hMenu) return;

    POINT pt;
    GetCursorPos(&pt);

    SetForegroundWindow(hWnd);

    HMENU hPopup = GetSubMenu(m_hMenu, 0);
    TrackPopupMenu(hPopup, TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
                   pt.x, pt.y, 0, hWnd, nullptr);

    PostMessage(hWnd, WM_NULL, 0, 0);
}

void TrayIcon::ShowBalloon(const wchar_t* title, const wchar_t* text) {
    if (!m_initialized) return;

    m_nid.uFlags |= NIF_INFO;
    wcscpy_s(m_nid.szInfoTitle, title);
    wcscpy_s(m_nid.szInfo, text);
    m_nid.dwInfoFlags = NIIF_INFO;

    Shell_NotifyIconW(NIM_MODIFY, &m_nid);

    m_nid.uFlags &= ~NIF_INFO;
}

bool TrayIcon::ProcessTrayMessage(HWND hWnd, WPARAM wParam, LPARAM lParam) {
    if (LOWORD(lParam) == WM_RBUTTONUP || LOWORD(lParam) == WM_CONTEXTMENU) {
        ShowContextMenu(hWnd);
        return true;
    }

    if (LOWORD(lParam) == WM_LBUTTONDBLCLK) {
        if (onSettings) onSettings();
        return true;
    }

    return false;
}

void TrayIcon::InitializeMenu(HINSTANCE hInstance) {
    m_hMenu = ::LoadMenuW(hInstance, MAKEINTRESOURCEW(IDM_TRAY_MENU));

    HMENU hPopup = GetSubMenu(m_hMenu, 0);
    if (hPopup) {
        ModifyMenuW(hPopup, IDM_TOGGLE_ENABLED, MF_BYCOMMAND | MF_STRING, IDM_TOGGLE_ENABLED, L"Ti\u1EBFng Vi\u1EC7t (VN)");
        ModifyMenuW(hPopup, IDM_SETTINGS, MF_BYCOMMAND | MF_STRING, IDM_SETTINGS, L"C\u00E0i \u0111\u1EB7t...");
        ModifyMenuW(hPopup, IDM_EXCLUDE_APPS, MF_BYCOMMAND | MF_STRING, IDM_EXCLUDE_APPS, L"Lo\u1EA1i tr\u1EEB \u1EE9ng d\u1EE5ng...");
        ModifyMenuW(hPopup, IDM_CONVERTER, MF_BYCOMMAND | MF_STRING, IDM_CONVERTER, L"Chuy\u1EC3n m\u00E3...");
        ModifyMenuW(hPopup, IDM_EXPORT_SETTINGS, MF_BYCOMMAND | MF_STRING, IDM_EXPORT_SETTINGS, L"Xu\u1EA5t c\u00E0i \u0111\u1EB7t...");

        HMENU hEncMenu = GetSubMenu(hPopup, 4);
        if (hEncMenu) {
            ModifyMenuW(hPopup, 4, MF_BYPOSITION | MF_POPUP | MF_STRING, (UINT_PTR)hEncMenu, L"M\u00E3 xu\u1EA5t");
        }
        ModifyMenuW(hPopup, IDM_IMPORT_SETTINGS, MF_BYCOMMAND | MF_STRING, IDM_IMPORT_SETTINGS, L"Nh\u1EADp c\u00E0i \u0111\u1EB7t...");
        ModifyMenuW(hPopup, IDM_CHECK_UPDATE, MF_BYCOMMAND | MF_STRING, IDM_CHECK_UPDATE, L"Ki\u1EC3m tra c\u1EADp nh\u1EADt");
        ModifyMenuW(hPopup, IDM_ABOUT, MF_BYCOMMAND | MF_STRING, IDM_ABOUT, L"Gi\u1EDBi thi\u1EC7u");
        ModifyMenuW(hPopup, IDM_EXIT, MF_BYCOMMAND | MF_STRING, IDM_EXIT, L"Tho\u00E1t");
    }
}
