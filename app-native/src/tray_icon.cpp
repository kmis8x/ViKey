// ViKey - System Tray Icon Implementation
// tray_icon.cpp

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

    // GDI+ already initialized in main.cpp - no duplicate init needed

    // Load icons from embedded resources
    m_iconVN = (HICON)LoadImageW(hInstance, MAKEINTRESOURCEW(IDI_ICON_VN), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);
    m_iconEN = (HICON)LoadImageW(hInstance, MAKEINTRESOURCEW(IDI_ICON_EN), IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR);

    // Fallback to generated icons if resources not found
    if (!m_iconVN) m_iconVN = CreateLetterIcon(true);
    if (!m_iconEN) m_iconEN = CreateLetterIcon(false);

    // Load menu from resources
    InitializeMenu(hInstance);

    // Setup notification icon data
    m_nid.cbSize = sizeof(NOTIFYICONDATAW);
    m_nid.hWnd = hWnd;
    m_nid.uID = 1;
    m_nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    m_nid.uCallbackMessage = WM_TRAYICON_MSG;
    m_nid.hIcon = m_iconVN;
    wcscpy_s(m_nid.szTip, L"Ti\u1EBFng Vi\u1EC7t [VN]\nCtrl+Space \u0111\u1EC3 chuy\u1EC3n");

    // Add to system tray
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

    // Update enabled state
    CheckMenuItem(hPopup, IDM_TOGGLE_ENABLED, vietnamese ? MF_CHECKED : MF_UNCHECKED);

    // Update menu text
    ModifyMenuW(hPopup, IDM_TOGGLE_ENABLED, MF_BYCOMMAND | MF_STRING,
                IDM_TOGGLE_ENABLED, vietnamese ? L"Ti\u1EBFng Vi\u1EC7t (VN)" : L"Ti\u1EBFng Anh (EN)");

    // Update method selection
    CheckMenuItem(hPopup, IDM_METHOD_TELEX, method == InputMethod::Telex ? MF_CHECKED : MF_UNCHECKED);
    CheckMenuItem(hPopup, IDM_METHOD_VNI, method == InputMethod::VNI ? MF_CHECKED : MF_UNCHECKED);
}

void TrayIcon::ShowContextMenu(HWND hWnd) {
    if (!m_hMenu) return;

    POINT pt;
    GetCursorPos(&pt);

    // Required to make menu close when clicking outside
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

HICON TrayIcon::LoadIconWithBorder(const wchar_t* iconPath) {
    // Load icon from file
    HICON hOriginal = (HICON)LoadImageW(nullptr, iconPath, IMAGE_ICON, 16, 16, LR_LOADFROMFILE);
    if (!hOriginal) return nullptr;

    // Create a new icon with white border
    HDC hdcScreen = GetDC(nullptr);
    HDC hdcMem = CreateCompatibleDC(hdcScreen);

    BITMAPINFO bmi = {};
    bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bmi.bmiHeader.biWidth = 16;
    bmi.bmiHeader.biHeight = -16;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    void* bits = nullptr;
    HBITMAP hBitmap = CreateDIBSection(hdcMem, &bmi, DIB_RGB_COLORS, &bits, nullptr, 0);
    HBITMAP hOldBitmap = (HBITMAP)SelectObject(hdcMem, hBitmap);

    Gdiplus::Graphics graphics(hdcMem);
    graphics.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);

    // Clear with transparent
    graphics.Clear(Gdiplus::Color(0, 0, 0, 0));

    // Draw white circular border first
    Gdiplus::Pen borderPen(Gdiplus::Color(255, 255, 255, 255), 1.5f);
    graphics.DrawEllipse(&borderPen, 0.5f, 0.5f, 14.0f, 14.0f);

    // Draw the original icon on top
    DrawIconEx(hdcMem, 0, 0, hOriginal, 16, 16, 0, nullptr, DI_NORMAL);

    SelectObject(hdcMem, hOldBitmap);

    // Create mask
    HBITMAP hMask = CreateBitmap(16, 16, 1, 1, nullptr);

    ICONINFO iconInfo = {};
    iconInfo.fIcon = TRUE;
    iconInfo.hbmMask = hMask;
    iconInfo.hbmColor = hBitmap;

    HICON hNewIcon = CreateIconIndirect(&iconInfo);

    // Cleanup
    DeleteObject(hMask);
    DeleteObject(hBitmap);
    DeleteDC(hdcMem);
    ReleaseDC(nullptr, hdcScreen);
    DestroyIcon(hOriginal);

    return hNewIcon;
}

HICON TrayIcon::CreateLetterIcon(bool vietnamese) {
    // Create a 16x16 icon with a letter
    HDC hdcScreen = GetDC(nullptr);
    HDC hdcMem = CreateCompatibleDC(hdcScreen);

    BITMAPINFO bmi = {};
    bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bmi.bmiHeader.biWidth = 16;
    bmi.bmiHeader.biHeight = -16; // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    void* bits = nullptr;
    HBITMAP hBitmap = CreateDIBSection(hdcMem, &bmi, DIB_RGB_COLORS, &bits, nullptr, 0);
    HBITMAP hOldBitmap = (HBITMAP)SelectObject(hdcMem, hBitmap);

    // Initialize GDI+ graphics
    Gdiplus::Graphics graphics(hdcMem);
    graphics.SetSmoothingMode(Gdiplus::SmoothingModeAntiAlias);
    graphics.SetTextRenderingHint(Gdiplus::TextRenderingHintAntiAlias);

    // Clear background (transparent)
    graphics.Clear(Gdiplus::Color(0, 0, 0, 0));

    // Choose background color: Blue for VN, Gray for EN
    Gdiplus::Color bgColor = vietnamese ?
        Gdiplus::Color(255, 0, 120, 212) :   // Windows Blue
        Gdiplus::Color(255, 100, 100, 100);  // Dark Gray

    // Draw circular background for visibility on dark/light taskbars
    Gdiplus::SolidBrush bgBrush(bgColor);
    graphics.FillEllipse(&bgBrush, 0, 0, 15, 15);

    // White border for contrast
    Gdiplus::Pen borderPen(Gdiplus::Color(255, 255, 255, 255), 1.0f);
    graphics.DrawEllipse(&borderPen, 0, 0, 15, 15);

    // White text on colored background
    Gdiplus::SolidBrush textBrush(Gdiplus::Color(255, 255, 255, 255));

    // Draw letter
    Gdiplus::FontFamily fontFamily(L"Arial");
    Gdiplus::Font font(&fontFamily, 8, Gdiplus::FontStyleBold, Gdiplus::UnitPixel);

    const wchar_t* letter = vietnamese ? L"V" : L"E";

    // Center the text
    Gdiplus::RectF layoutRect(0, 0, 16, 16);
    Gdiplus::StringFormat format;
    format.SetAlignment(Gdiplus::StringAlignmentCenter);
    format.SetLineAlignment(Gdiplus::StringAlignmentCenter);
    graphics.DrawString(letter, -1, &font, layoutRect, &format, &textBrush);

    SelectObject(hdcMem, hOldBitmap);

    // Create mask bitmap
    HBITMAP hMask = CreateBitmap(16, 16, 1, 1, nullptr);

    // Create icon
    ICONINFO iconInfo = {};
    iconInfo.fIcon = TRUE;
    iconInfo.hbmMask = hMask;
    iconInfo.hbmColor = hBitmap;

    HICON hIcon = CreateIconIndirect(&iconInfo);

    // Cleanup
    DeleteObject(hMask);
    DeleteObject(hBitmap);
    DeleteDC(hdcMem);
    ReleaseDC(nullptr, hdcScreen);

    return hIcon;
}

void TrayIcon::InitializeMenu(HINSTANCE hInstance) {
    m_hMenu = ::LoadMenuW(hInstance, MAKEINTRESOURCEW(IDM_TRAY_MENU));

    // Fix Vietnamese text (resource file encoding issue)
    HMENU hPopup = GetSubMenu(m_hMenu, 0);
    if (hPopup) {
        ModifyMenuW(hPopup, IDM_TOGGLE_ENABLED, MF_BYCOMMAND | MF_STRING, IDM_TOGGLE_ENABLED, L"Ti\u1EBFng Vi\u1EC7t (VN)");
        ModifyMenuW(hPopup, IDM_SETTINGS, MF_BYCOMMAND | MF_STRING, IDM_SETTINGS, L"C\u00E0i \u0111\u1EB7t...");
        ModifyMenuW(hPopup, IDM_EXCLUDE_APPS, MF_BYCOMMAND | MF_STRING, IDM_EXCLUDE_APPS, L"Lo\u1EA1i tr\u1EEB \u1EE9ng d\u1EE5ng...");
        ModifyMenuW(hPopup, IDM_CONVERTER, MF_BYCOMMAND | MF_STRING, IDM_CONVERTER, L"Chuy\u1EC3n m\u00E3...");
        ModifyMenuW(hPopup, IDM_EXPORT_SETTINGS, MF_BYCOMMAND | MF_STRING, IDM_EXPORT_SETTINGS, L"Xu\u1EA5t c\u00E0i \u0111\u1EB7t...");

        // Fix encoding submenu text
        HMENU hEncMenu = GetSubMenu(hPopup, 4);  // Position of encoding submenu
        if (hEncMenu) {
            ModifyMenuW(hPopup, 4, MF_BYPOSITION | MF_POPUP | MF_STRING, (UINT_PTR)hEncMenu, L"M\u00E3 xu\u1EA5t");
        }
        ModifyMenuW(hPopup, IDM_IMPORT_SETTINGS, MF_BYCOMMAND | MF_STRING, IDM_IMPORT_SETTINGS, L"Nh\u1EADp c\u00E0i \u0111\u1EB7t...");
        ModifyMenuW(hPopup, IDM_CHECK_UPDATE, MF_BYCOMMAND | MF_STRING, IDM_CHECK_UPDATE, L"Ki\u1EC3m tra c\u1EADp nh\u1EADt");
        ModifyMenuW(hPopup, IDM_ABOUT, MF_BYCOMMAND | MF_STRING, IDM_ABOUT, L"Gi\u1EDBi thi\u1EC7u");
        ModifyMenuW(hPopup, IDM_EXIT, MF_BYCOMMAND | MF_STRING, IDM_EXIT, L"Tho\u00E1t");
    }
}
