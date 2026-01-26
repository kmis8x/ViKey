//! Linux to macOS keycode mapping
//!
//! Maps Linux X11/evdev keycodes to macOS virtual keycodes used by vikey_core.

/// Convert Linux evdev keycode to macOS virtual keycode
/// Returns 0xFF if key is not mapped
pub fn linux_to_mac(keycode: u32) -> u16 {
    // Linux evdev keycodes (from /usr/include/linux/input-event-codes.h)
    // macOS virtual keycodes (from Carbon HIToolbox/Events.h)

    match keycode {
        // Letters (Linux evdev + 8 offset from X11)
        38 => 0,   // A -> kVK_ANSI_A (0)
        56 => 11,  // B -> kVK_ANSI_B (11)
        54 => 8,   // C -> kVK_ANSI_C (8)
        40 => 2,   // D -> kVK_ANSI_D (2)
        26 => 14,  // E -> kVK_ANSI_E (14)
        41 => 3,   // F -> kVK_ANSI_F (3)
        42 => 5,   // G -> kVK_ANSI_G (5)
        43 => 4,   // H -> kVK_ANSI_H (4)
        31 => 34,  // I -> kVK_ANSI_I (34)
        44 => 38,  // J -> kVK_ANSI_J (38)
        45 => 40,  // K -> kVK_ANSI_K (40)
        46 => 37,  // L -> kVK_ANSI_L (37)
        58 => 46,  // M -> kVK_ANSI_M (46)
        57 => 45,  // N -> kVK_ANSI_N (45)
        32 => 31,  // O -> kVK_ANSI_O (31)
        33 => 35,  // P -> kVK_ANSI_P (35)
        24 => 12,  // Q -> kVK_ANSI_Q (12)
        27 => 15,  // R -> kVK_ANSI_R (15)
        39 => 1,   // S -> kVK_ANSI_S (1)
        28 => 17,  // T -> kVK_ANSI_T (17)
        30 => 32,  // U -> kVK_ANSI_U (32)
        55 => 9,   // V -> kVK_ANSI_V (9)
        25 => 13,  // W -> kVK_ANSI_W (13)
        53 => 7,   // X -> kVK_ANSI_X (7)
        29 => 16,  // Y -> kVK_ANSI_Y (16)
        52 => 6,   // Z -> kVK_ANSI_Z (6)

        // Numbers
        10 => 18,  // 1 -> kVK_ANSI_1 (18)
        11 => 19,  // 2 -> kVK_ANSI_2 (19)
        12 => 20,  // 3 -> kVK_ANSI_3 (20)
        13 => 21,  // 4 -> kVK_ANSI_4 (21)
        14 => 23,  // 5 -> kVK_ANSI_5 (23)
        15 => 22,  // 6 -> kVK_ANSI_6 (22)
        16 => 26,  // 7 -> kVK_ANSI_7 (26)
        17 => 28,  // 8 -> kVK_ANSI_8 (28)
        18 => 25,  // 9 -> kVK_ANSI_9 (25)
        19 => 29,  // 0 -> kVK_ANSI_0 (29)

        // Special keys
        65 => 49,  // Space -> kVK_Space (49)
        22 => 51,  // Backspace -> kVK_Delete (51)
        23 => 48,  // Tab -> kVK_Tab (48)
        36 => 36,  // Return -> kVK_Return (36)
        9 => 53,   // Escape -> kVK_Escape (53)

        // Arrow keys
        113 => 123, // Left -> kVK_LeftArrow (123)
        114 => 124, // Right -> kVK_RightArrow (124)
        116 => 125, // Down -> kVK_DownArrow (125)
        111 => 126, // Up -> kVK_UpArrow (126)

        // Punctuation
        60 => 47,  // . -> kVK_ANSI_Period (47)
        59 => 43,  // , -> kVK_ANSI_Comma (43)
        61 => 44,  // / -> kVK_ANSI_Slash (44)
        47 => 41,  // ; -> kVK_ANSI_Semicolon (41)
        48 => 39,  // ' -> kVK_ANSI_Quote (39)
        34 => 33,  // [ -> kVK_ANSI_LeftBracket (33)
        35 => 30,  // ] -> kVK_ANSI_RightBracket (30)
        51 => 42,  // \ -> kVK_ANSI_Backslash (42)
        20 => 27,  // - -> kVK_ANSI_Minus (27)
        21 => 24,  // = -> kVK_ANSI_Equal (24)
        49 => 50,  // ` -> kVK_ANSI_Grave (50)

        _ => 0xFF, // Unknown key
    }
}

/// Convert X11 keysym to macOS virtual keycode (alternative method)
/// Used when keycode is not available
pub fn keysym_to_mac(keysym: u32) -> u16 {
    match keysym {
        // Lowercase letters
        0x0061 => 0,   // a
        0x0062 => 11,  // b
        0x0063 => 8,   // c
        0x0064 => 2,   // d
        0x0065 => 14,  // e
        0x0066 => 3,   // f
        0x0067 => 5,   // g
        0x0068 => 4,   // h
        0x0069 => 34,  // i
        0x006a => 38,  // j
        0x006b => 40,  // k
        0x006c => 37,  // l
        0x006d => 46,  // m
        0x006e => 45,  // n
        0x006f => 31,  // o
        0x0070 => 35,  // p
        0x0071 => 12,  // q
        0x0072 => 15,  // r
        0x0073 => 1,   // s
        0x0074 => 17,  // t
        0x0075 => 32,  // u
        0x0076 => 9,   // v
        0x0077 => 13,  // w
        0x0078 => 7,   // x
        0x0079 => 16,  // y
        0x007a => 6,   // z

        // Uppercase letters (same keycodes)
        0x0041 => 0,   // A
        0x0042 => 11,  // B
        0x0043 => 8,   // C
        0x0044 => 2,   // D
        0x0045 => 14,  // E
        0x0046 => 3,   // F
        0x0047 => 5,   // G
        0x0048 => 4,   // H
        0x0049 => 34,  // I
        0x004a => 38,  // J
        0x004b => 40,  // K
        0x004c => 37,  // L
        0x004d => 46,  // M
        0x004e => 45,  // N
        0x004f => 31,  // O
        0x0050 => 35,  // P
        0x0051 => 12,  // Q
        0x0052 => 15,  // R
        0x0053 => 1,   // S
        0x0054 => 17,  // T
        0x0055 => 32,  // U
        0x0056 => 9,   // V
        0x0057 => 13,  // W
        0x0058 => 7,   // X
        0x0059 => 16,  // Y
        0x005a => 6,   // Z

        // Numbers
        0x0030 => 29,  // 0
        0x0031 => 18,  // 1
        0x0032 => 19,  // 2
        0x0033 => 20,  // 3
        0x0034 => 21,  // 4
        0x0035 => 23,  // 5
        0x0036 => 22,  // 6
        0x0037 => 26,  // 7
        0x0038 => 28,  // 8
        0x0039 => 25,  // 9

        // Special
        0x0020 => 49,  // space
        0xff08 => 51,  // BackSpace
        0xff09 => 48,  // Tab
        0xff0d => 36,  // Return
        0xff1b => 53,  // Escape

        // Punctuation
        0x002e => 47,  // .
        0x002c => 43,  // ,
        0x002f => 44,  // /
        0x003b => 41,  // ;
        0x0027 => 39,  // '
        0x005b => 33,  // [
        0x005d => 30,  // ]
        0x005c => 42,  // \
        0x002d => 27,  // -
        0x003d => 24,  // =
        0x0060 => 50,  // `

        _ => 0xFF,
    }
}
