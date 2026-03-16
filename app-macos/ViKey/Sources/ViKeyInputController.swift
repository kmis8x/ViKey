//
//  ViKeyInputController.swift
//  ViKey macOS Input Method
//
//  Main IMKit input controller
//

import InputMethodKit
import AppKit

class ViKeyInputController: IMKInputController {

    // MARK: - Constants

    private let kSpaceKeyCode: UInt16 = 49

    // MARK: - Properties

    private let bridge = RustBridge.shared
    private var isEnabled = true

    // MARK: - IMKInputController Overrides

    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)

        // Load settings
        loadSettings()
    }

    /// Handle key events from the system
    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event else { return false }

        // Only handle key down events
        guard event.type == .keyDown else { return false }

        // Skip if disabled
        guard isEnabled else { return false }

        // Get client for text operations
        guard let client = sender as? IMKTextInput else { return false }

        // Check for toggle hotkey (Ctrl+Space by default)
        if event.modifierFlags.contains(.control) && event.keyCode == kSpaceKeyCode {
            toggleEnabled()
            return true
        }

        // Skip if any modifier (Cmd/Opt/Ctrl) is held — only Shift is allowed for typing
        let modifiers = event.modifierFlags.intersection([.command, .option, .control])
        if !modifiers.isEmpty {
            return false
        }

        // Process key through Rust engine
        let keyCode = event.keyCode
        let caps = calculateCaps(event: event)
        let ctrl = event.modifierFlags.contains(.control)
        let shift = event.modifierFlags.contains(.shift)

        if let result = bridge.processKey(keyCode: keyCode, caps: caps, ctrl: ctrl, shift: shift) {
            // Delete characters if needed
            if result.backspaceCount > 0 {
                deleteBackward(count: result.backspaceCount, client: client)
            }

            // Insert new text
            if !result.text.isEmpty {
                client.insertText(result.text, replacementRange: NSRange(location: NSNotFound, length: 0))
            }

            return result.keyConsumed
        }

        return false
    }

    /// Called when client changes (switching apps, text fields)
    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        bridge.clearAll()
    }

    /// Called when leaving this input method
    override func deactivateServer(_ sender: Any!) {
        super.deactivateServer(sender)
        bridge.clearAll()
    }

    // MARK: - Private Methods

    /// Calculate caps state from event (Shift XOR CapsLock)
    private func calculateCaps(event: NSEvent) -> Bool {
        let shift = event.modifierFlags.contains(.shift)
        let capsLock = event.modifierFlags.contains(.capsLock)
        return shift != capsLock // XOR
    }

    /// Delete characters by sending backspace events
    private func deleteBackward(count: Int, client: IMKTextInput) {
        for _ in 0..<count {
            // Use replacementRange to delete
            let range = NSRange(location: NSNotFound, length: 1)
            client.insertText("", replacementRange: range)
        }
    }

    /// Toggle IME enabled state
    private func toggleEnabled() {
        isEnabled.toggle()
        bridge.setEnabled(isEnabled)

        // Notify user (optional: show in status bar)
        NotificationCenter.default.post(
            name: NSNotification.Name("ViKeyEnabledChanged"),
            object: nil,
            userInfo: ["enabled": isEnabled]
        )
    }

    /// Load settings from UserDefaults
    private func loadSettings() {
        // Use Settings singleton to apply all settings
        Settings.shared.applyAll()
    }
}
