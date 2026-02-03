//
//  Settings.swift
//  ViKey macOS Input Method
//
//  Settings management via UserDefaults
//

import Foundation

/// ViKey settings manager
class Settings {
    static let shared = Settings()

    private let defaults = UserDefaults.standard

    // MARK: - Keys

    private enum Keys {
        static let inputMethod = "InputMethod"
        static let modernTone = "ModernTone"
        static let escRestore = "EscRestore"
        static let skipWShortcut = "SkipWShortcut"
        static let bracketShortcut = "BracketShortcut"
        static let freeTone = "FreeTone"
        static let englishAutoRestore = "EnglishAutoRestore"
        static let autoCapitalize = "AutoCapitalize"
        static let enabled = "Enabled"
        static let allowForeignConsonants = "AllowForeignConsonants"  // f,j,w,z as consonants
        static let shortcutsEnabled = "ShortcutsEnabled"  // Enable/disable shortcuts
        static let shortcuts = "Shortcuts"  // Text shortcuts array
    }

    // MARK: - Properties

    var inputMethod: InputMethod {
        get {
            InputMethod(rawValue: UInt8(defaults.integer(forKey: Keys.inputMethod))) ?? .telex
        }
        set {
            defaults.set(Int(newValue.rawValue), forKey: Keys.inputMethod)
            RustBridge.shared.setMethod(newValue)
        }
    }

    var modernTone: Bool {
        get { defaults.bool(forKey: Keys.modernTone) }
        set {
            defaults.set(newValue, forKey: Keys.modernTone)
            RustBridge.shared.setModernTone(newValue)
        }
    }

    var escRestore: Bool {
        get { defaults.bool(forKey: Keys.escRestore) }
        set {
            defaults.set(newValue, forKey: Keys.escRestore)
            RustBridge.shared.setEscRestore(newValue)
        }
    }

    var skipWShortcut: Bool {
        get { defaults.bool(forKey: Keys.skipWShortcut) }
        set {
            defaults.set(newValue, forKey: Keys.skipWShortcut)
            RustBridge.shared.setSkipWShortcut(newValue)
        }
    }

    var bracketShortcut: Bool {
        get { defaults.bool(forKey: Keys.bracketShortcut) }
        set {
            defaults.set(newValue, forKey: Keys.bracketShortcut)
            RustBridge.shared.setBracketShortcut(newValue)
        }
    }

    var freeTone: Bool {
        get { defaults.bool(forKey: Keys.freeTone) }
        set {
            defaults.set(newValue, forKey: Keys.freeTone)
            RustBridge.shared.setFreeTone(newValue)
        }
    }

    var englishAutoRestore: Bool {
        get { defaults.bool(forKey: Keys.englishAutoRestore) }
        set {
            defaults.set(newValue, forKey: Keys.englishAutoRestore)
            RustBridge.shared.setEnglishAutoRestore(newValue)
        }
    }

    var autoCapitalize: Bool {
        get { defaults.bool(forKey: Keys.autoCapitalize) }
        set {
            defaults.set(newValue, forKey: Keys.autoCapitalize)
            RustBridge.shared.setAutoCapitalize(newValue)
        }
    }

    var enabled: Bool {
        get { defaults.object(forKey: Keys.enabled) as? Bool ?? true }
        set {
            defaults.set(newValue, forKey: Keys.enabled)
            RustBridge.shared.setEnabled(newValue)
        }
    }

    var allowForeignConsonants: Bool {
        get { defaults.bool(forKey: Keys.allowForeignConsonants) }
        set {
            defaults.set(newValue, forKey: Keys.allowForeignConsonants)
            RustBridge.shared.setAllowForeignConsonants(newValue)
        }
    }

    var shortcutsEnabled: Bool {
        get { defaults.object(forKey: Keys.shortcutsEnabled) as? Bool ?? true }
        set {
            defaults.set(newValue, forKey: Keys.shortcutsEnabled)
            RustBridge.shared.setShortcutsEnabled(newValue)
        }
    }

    // MARK: - Shortcuts

    /// Get all text shortcuts as array of (trigger, replacement) tuples
    var shortcuts: [(String, String)] {
        get {
            guard let array = defaults.array(forKey: Keys.shortcuts) as? [[String]] else {
                return []
            }
            return array.compactMap { pair in
                guard pair.count == 2 else { return nil }
                return (pair[0], pair[1])
            }
        }
        set {
            let array = newValue.map { [$0.0, $0.1] }
            defaults.set(array, forKey: Keys.shortcuts)
            // Sync to Rust engine
            RustBridge.shared.clearShortcuts()
            for (trigger, replacement) in newValue {
                RustBridge.shared.addShortcut(trigger: trigger, replacement: replacement)
            }
        }
    }

    func addShortcut(trigger: String, replacement: String) {
        var current = shortcuts
        // Remove existing if any
        current.removeAll { $0.0 == trigger }
        current.append((trigger, replacement))
        shortcuts = current
    }

    func removeShortcut(trigger: String) {
        var current = shortcuts
        current.removeAll { $0.0 == trigger }
        shortcuts = current
    }

    // MARK: - Initialization

    private init() {
        registerDefaults()
    }

    private func registerDefaults() {
        defaults.register(defaults: [
            Keys.inputMethod: 0, // Telex
            Keys.modernTone: true,
            Keys.escRestore: true,
            Keys.skipWShortcut: false,
            Keys.bracketShortcut: false,
            Keys.freeTone: false,
            Keys.englishAutoRestore: true,
            Keys.autoCapitalize: false,
            Keys.enabled: true,
            Keys.allowForeignConsonants: false,
            Keys.shortcutsEnabled: true,
            Keys.shortcuts: [] as [[String]]
        ])
    }

    /// Apply all settings to Rust bridge
    func applyAll() {
        RustBridge.shared.setMethod(inputMethod)
        RustBridge.shared.setModernTone(modernTone)
        RustBridge.shared.setEscRestore(escRestore)
        RustBridge.shared.setSkipWShortcut(skipWShortcut)
        RustBridge.shared.setBracketShortcut(bracketShortcut)
        RustBridge.shared.setFreeTone(freeTone)
        RustBridge.shared.setEnglishAutoRestore(englishAutoRestore)
        RustBridge.shared.setAutoCapitalize(autoCapitalize)
        RustBridge.shared.setAllowForeignConsonants(allowForeignConsonants)
        RustBridge.shared.setShortcutsEnabled(shortcutsEnabled)
        RustBridge.shared.setEnabled(enabled)

        // Load shortcuts to Rust engine
        RustBridge.shared.clearShortcuts()
        for (trigger, replacement) in shortcuts {
            RustBridge.shared.addShortcut(trigger: trigger, replacement: replacement)
        }
    }
}
