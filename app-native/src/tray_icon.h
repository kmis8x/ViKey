// ViKey - System Tray Icon
// tray_icon.h
// System tray icon and context menu

#pragma once

#include <windows.h>
#include <shellapi.h>
#include <functional>
#include "rust_bridge.h"

class TrayIcon {
public:
    static TrayIcon& Instance();

    // Initialize the tray icon
    bool Initialize(HWND hWnd, HINSTANCE hInstance);

    // Cleanup
    void Shutdown();

    // Update icon based on enabled state
    void UpdateIcon(bool vietnamese);

    // Update tooltip text
    void UpdateTooltip(bool vietnamese, InputMethod method);

    // Update menu checkmarks
    void UpdateMenu(bool vietnamese, InputMethod method);

    // Show context menu at cursor position
    void ShowContextMenu(HWND hWnd);

    // Show balloon notification
    void ShowBalloon(const wchar_t* title, const wchar_t* text);

    // Process tray icon messages
    bool ProcessTrayMessage(HWND hWnd, WPARAM wParam, LPARAM lParam);

    // Callbacks
    std::function<void()> onToggleEnabled;
    std::function<void(InputMethod)> onSetMethod;
    std::function<void()> onSettings;
    std::function<void()> onAbout;
    std::function<void()> onExit;

private:
    TrayIcon();
    ~TrayIcon();
    TrayIcon(const TrayIcon&) = delete;
    TrayIcon& operator=(const TrayIcon&) = delete;

    // Create V or E icon programmatically
    HICON CreateLetterIcon(bool vietnamese);

    // Load icon from file and add white border
    HICON LoadIconWithBorder(const wchar_t* iconPath);

    // Load menu from resources
    void InitializeMenu(HINSTANCE hInstance);

    NOTIFYICONDATAW m_nid;
    HMENU m_hMenu;
    HICON m_iconVN;
    HICON m_iconEN;
    bool m_initialized;
};
