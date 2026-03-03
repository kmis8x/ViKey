// ViKey - Tray Icon Drawing
// tray_icon_drawing.cpp
// CreateLetterIcon and LoadIconWithBorder GDI+ icon creation

#include "tray_icon.h"
#include "resource.h"
#include <objidl.h>
#include <gdiplus.h>

#pragma comment(lib, "gdiplus.lib")

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
