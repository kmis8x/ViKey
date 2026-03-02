// ViKey - Dark Mode Support
// dark_mode.h
// Windows 10 1809+ dark mode APIs and dialog helpers

#pragma once

#include <windows.h>

// Dark mode state and colors
extern bool g_isDarkMode;
extern HBRUSH g_hDarkBrush;
extern HBRUSH g_hDarkEditBrush;
constexpr COLORREF DARK_BG_COLOR = RGB(32, 32, 32);
constexpr COLORREF DARK_EDIT_BG = RGB(45, 45, 45);
constexpr COLORREF DARK_TEXT_COLOR = RGB(255, 255, 255);

// Initialize undocumented dark mode APIs from uxtheme.dll
void InitDarkModeAPIs();

// Check if system dark mode is enabled
bool IsSystemDarkMode();

// Handle dark mode color messages for dialog controls
INT_PTR HandleDarkModeColors(UINT message, WPARAM wParam);

// Enable dark mode for a dialog window
void EnableDarkModeForDialog(HWND hDlg);

// Center dialog on screen
void CenterDialog(HWND hDlg);

// Scale dialog for DPI and enable dark mode
void ScaleDialogForDpi(HWND hDlg);

// DPI helper - get scaling factor for current monitor
float GetDpiScale(HWND hWnd);
