// ViKey - Main Processor
// ime_processor.h
// Connects keyboard hook to Rust engine

#pragma once

#include <windows.h>
#include "rust_bridge.h"
#include "keyboard_hook.h"
#include "text_sender.h"
#include "shortcut_manager.h"
#include "settings.h"
#include "app_detector.h"

class ImeProcessor {
public:
    static ImeProcessor& Instance();

    // Initialize the processor
    bool Initialize();

    // Start/stop processing
    void Start();
    void Stop();

    // Enable/disable IME
    void SetEnabled(bool enabled);
    bool IsEnabled() const { return m_enabled; }

    // Toggle enabled state
    void ToggleEnabled();

    // Set input method
    void SetMethod(InputMethod method);
    InputMethod GetMethod() const { return m_method; }

    // Apply settings from Settings class
    void ApplySettings();

    // Update shortcuts from Settings
    void UpdateShortcuts();

private:
    ImeProcessor();
    ~ImeProcessor() = default;
    ImeProcessor(const ImeProcessor&) = delete;
    ImeProcessor& operator=(const ImeProcessor&) = delete;

    // Key press handler
    void OnKeyPressed(KeyEventData& event);

    // Check and handle app changes (for smart switch)
    void CheckAppChange();

    bool m_enabled;
    std::wstring m_lastAppName;  // Track last app for smart switch
    InputMethod m_method;
    bool m_initialized;
};
