// ViKey - Dialog Procedures
// dialogs.h
// Forward declarations for all dialog procedures and show functions

#pragma once

#include <windows.h>
#include "updater.h"

// Application globals (defined in main.cpp)
extern HINSTANCE g_hInstance;
extern HWND g_hWnd;

// Show dialog functions
void ShowSettingsDialog();
void ShowAboutDialog();
void ShowExcludeAppsDialog();
void ShowConverterDialog();
void ShowShortcutsDialog();
void ShowUpdateDialog(const UpdateInfo& info);

// Import/Export settings
void ExportSettings();
void ImportSettings();

// Update check functions
void CheckForUpdatesOnStartup();
void CheckForUpdatesManual();

// UI state update (defined in main.cpp)
void UpdateUI();
