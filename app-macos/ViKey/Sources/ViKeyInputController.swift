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

    /// Enabled state uses Settings as single source of truth.
    /// IMKit creates separate controller instances per client app,
    /// so a per-instance variable would desync across apps.
    private var isEnabled: Bool {
        get { Settings.shared.enabled }
        set { Settings.shared.enabled = newValue }
    }

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

        // Check for toggle hotkey (Ctrl+Space) before the enabled guard,
        // so users can re-enable ViKey after disabling it.
        if event.modifierFlags.contains(.control) && event.keyCode == kSpaceKeyCode {
            toggleEnabled()
            return true
        }

        // Skip if disabled
        guard isEnabled else { return false }

        // Get client for text operations
        guard let client = sender as? IMKTextInput else { return false }

        // Skip if any modifier (Cmd/Opt/Ctrl) is held — only Shift is allowed for typing
        let modifiers = event.modifierFlags.intersection([.command, .option, .control])
        if !modifiers.isEmpty {
            return false
        }

        // Process key through Rust engine
        let keyCode = event.keyCode
        let caps = calculateCaps(event: event)
        let shift = event.modifierFlags.contains(.shift)
        // ctrl is always false here: the modifier filter above already
        // returns false when Control is held, so it can never be true
        // at this point. Pass false directly to avoid dead-code confusion.

        if let result = bridge.processKey(keyCode: keyCode, caps: caps, ctrl: false, shift: shift) {
            // Combine delete + insert into a single IPC call when possible.
            // This reduces N+1 IPC round-trips to 1 for Vietnamese transforms.
            if result.backspaceCount > 0 && !result.text.isEmpty {
                let currentRange = client.selectedRange()
                if currentRange.location != NSNotFound
                    && currentRange.location >= result.backspaceCount {
                    let replaceRange = NSRange(
                        location: currentRange.location - result.backspaceCount,
                        length: result.backspaceCount
                    )
                    client.insertText(result.text, replacementRange: replaceRange)
                    return result.keyConsumed
                }
            }

            // Fallback: separate delete then insert
            if result.backspaceCount > 0 {
                deleteBackward(count: result.backspaceCount, client: client)
            }
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

    /// Delete characters using a single replacement range when possible.
    /// Falls back to per-character deletion for apps where selectedRange
    /// returns NSNotFound (e.g. some Electron apps, Terminal).
    private func deleteBackward(count: Int, client: IMKTextInput) {
        let currentRange = client.selectedRange()
        if currentRange.location != NSNotFound && currentRange.location >= count {
            let replaceRange = NSRange(location: currentRange.location - count, length: count)
            client.insertText("", replacementRange: replaceRange)
        } else {
            // Fallback: loop per character for clients that don't report selectedRange
            for _ in 0..<count {
                client.insertText("", replacementRange: NSRange(location: NSNotFound, length: 1))
            }
        }
    }

    private static let enabledChangedNotification = NSNotification.Name("ViKeyEnabledChanged")

    /// Toggle IME enabled state and persist to UserDefaults.
    /// The computed `isEnabled` property writes through to Settings.shared.enabled,
    /// which also calls RustBridge.shared.setEnabled(), so no separate bridge call needed.
    private func toggleEnabled() {
        isEnabled.toggle()

        NotificationCenter.default.post(
            name: Self.enabledChangedNotification,
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
