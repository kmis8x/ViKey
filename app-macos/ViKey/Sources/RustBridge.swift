//
//  RustBridge.swift
//  ViKey macOS Input Method
//
//  Swift wrapper for Rust FFI
//

import Foundation

/// Result from IME processing
struct ImeProcessResult {
    let text: String
    let backspaceCount: Int
    let keyConsumed: Bool
}

/// Action types from Rust engine
enum ImeAction: UInt8 {
    case none = 0
    case send = 1
    case restore = 2
}

/// Input method types
enum InputMethod: UInt8 {
    case telex = 0
    case vni = 1
}

/// Bridge to Rust core engine
class RustBridge {
    static let shared = RustBridge()

    private init() {
        ime_init()
    }

    /// Process a key event
    /// - Parameters:
    ///   - keyCode: macOS virtual keycode (0-127)
    ///   - caps: Whether output should be uppercase
    ///   - ctrl: Control key pressed
    ///   - shift: Shift key pressed
    /// - Returns: Result with text to insert and backspace count, or nil if no action
    func processKey(keyCode: UInt16, caps: Bool, ctrl: Bool, shift: Bool) -> ImeProcessResult? {
        guard let resultPtr = ime_key_ext(keyCode, caps, ctrl, shift) else {
            return nil
        }
        defer { ime_free(resultPtr) }

        let result = resultPtr.pointee

        // Check if key was consumed (for shortcuts like -> or :))
        let keyConsumed = (result.flags & 0x01) != 0

        // Only return if action is Send
        guard result.action == ImeAction.send.rawValue else {
            if keyConsumed {
                return ImeProcessResult(text: "", backspaceCount: 0, keyConsumed: true)
            }
            return nil
        }

        // Convert UTF-32 chars to Swift String
        let text = convertToString(chars: result.chars, count: Int(result.count))

        return ImeProcessResult(
            text: text,
            backspaceCount: Int(result.backspace),
            keyConsumed: keyConsumed || result.count > 0
        )
    }

    /// Convert UTF-32 codepoints array to String
    private func convertToString(chars: (UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32, UInt32), count: Int) -> String {
        // Use withUnsafePointer to access tuple as array
        return withUnsafePointer(to: chars) { ptr in
            ptr.withMemoryRebound(to: UInt32.self, capacity: 256) { buffer in
                var scalars = [Unicode.Scalar]()
                scalars.reserveCapacity(count)

                for i in 0..<count {
                    if let scalar = Unicode.Scalar(buffer[i]) {
                        scalars.append(scalar)
                    }
                }

                return String(scalars.map { Character($0) })
            }
        }
    }

    // MARK: - State Control

    func setEnabled(_ enabled: Bool) {
        ime_enabled(enabled)
    }

    func setMethod(_ method: InputMethod) {
        ime_method(method.rawValue)
    }

    func clear() {
        ime_clear()
    }

    func clearAll() {
        ime_clear_all()
    }

    // MARK: - Configuration

    func setSkipWShortcut(_ skip: Bool) {
        ime_skip_w_shortcut(skip)
    }

    func setBracketShortcut(_ enabled: Bool) {
        ime_bracket_shortcut(enabled)
    }

    func setEscRestore(_ enabled: Bool) {
        ime_esc_restore(enabled)
    }

    func setFreeTone(_ enabled: Bool) {
        ime_free_tone(enabled)
    }

    func setModernTone(_ modern: Bool) {
        ime_modern(modern)
    }

    func setEnglishAutoRestore(_ enabled: Bool) {
        ime_english_auto_restore(enabled)
    }

    func setAutoCapitalize(_ enabled: Bool) {
        ime_auto_capitalize(enabled)
    }

    func setAllowForeignConsonants(_ enabled: Bool) {
        ime_allow_foreign_consonants(enabled)
    }

    func setShortcutsEnabled(_ enabled: Bool) {
        ime_shortcuts_enabled(enabled)
    }

    // MARK: - Shortcuts

    func addShortcut(trigger: String, replacement: String) {
        trigger.withCString { triggerPtr in
            replacement.withCString { replacementPtr in
                ime_add_shortcut(triggerPtr, replacementPtr)
            }
        }
    }

    func removeShortcut(trigger: String) {
        trigger.withCString { triggerPtr in
            ime_remove_shortcut(triggerPtr)
        }
    }

    func clearShortcuts() {
        ime_clear_shortcuts()
    }
}
