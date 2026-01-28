// ViKey - Key Code Mapping Implementation
// keycodes.cpp

#include "keycodes.h"

// macOS keycodes (from core/src/data/keys.rs)
namespace MacKeyCodes {
    constexpr uint16_t MAC_A = 0;
    constexpr uint16_t MAC_S = 1;
    constexpr uint16_t MAC_D = 2;
    constexpr uint16_t MAC_F = 3;
    constexpr uint16_t MAC_H = 4;
    constexpr uint16_t MAC_G = 5;
    constexpr uint16_t MAC_Z = 6;
    constexpr uint16_t MAC_X = 7;
    constexpr uint16_t MAC_C = 8;
    constexpr uint16_t MAC_V = 9;
    constexpr uint16_t MAC_B = 11;
    constexpr uint16_t MAC_Q = 12;
    constexpr uint16_t MAC_W = 13;
    constexpr uint16_t MAC_E = 14;
    constexpr uint16_t MAC_R = 15;
    constexpr uint16_t MAC_Y = 16;
    constexpr uint16_t MAC_T = 17;
    constexpr uint16_t MAC_N1 = 18;
    constexpr uint16_t MAC_N2 = 19;
    constexpr uint16_t MAC_N3 = 20;
    constexpr uint16_t MAC_N4 = 21;
    constexpr uint16_t MAC_N6 = 22;
    constexpr uint16_t MAC_N5 = 23;
    constexpr uint16_t MAC_EQUAL = 24;
    constexpr uint16_t MAC_N9 = 25;
    constexpr uint16_t MAC_N7 = 26;
    constexpr uint16_t MAC_MINUS = 27;
    constexpr uint16_t MAC_N8 = 28;
    constexpr uint16_t MAC_N0 = 29;
    constexpr uint16_t MAC_RBRACKET = 30;
    constexpr uint16_t MAC_O = 31;
    constexpr uint16_t MAC_U = 32;
    constexpr uint16_t MAC_LBRACKET = 33;
    constexpr uint16_t MAC_I = 34;
    constexpr uint16_t MAC_P = 35;
    constexpr uint16_t MAC_RETURN = 36;
    constexpr uint16_t MAC_L = 37;
    constexpr uint16_t MAC_J = 38;
    constexpr uint16_t MAC_QUOTE = 39;
    constexpr uint16_t MAC_K = 40;
    constexpr uint16_t MAC_SEMICOLON = 41;
    constexpr uint16_t MAC_BACKSLASH = 42;
    constexpr uint16_t MAC_COMMA = 43;
    constexpr uint16_t MAC_SLASH = 44;
    constexpr uint16_t MAC_N = 45;
    constexpr uint16_t MAC_M = 46;
    constexpr uint16_t MAC_DOT = 47;
    constexpr uint16_t MAC_TAB = 48;
    constexpr uint16_t MAC_SPACE = 49;
    constexpr uint16_t MAC_BACKQUOTE = 50;
    constexpr uint16_t MAC_DELETE = 51;
    constexpr uint16_t MAC_ESC = 53;
    constexpr uint16_t MAC_ENTER = 76;
    constexpr uint16_t MAC_LEFT = 123;
    constexpr uint16_t MAC_RIGHT = 124;
    constexpr uint16_t MAC_DOWN = 125;
    constexpr uint16_t MAC_UP = 126;
    constexpr uint16_t INVALID = 0xFFFF;
}

namespace KeyCodes {

uint16_t ToMacKeycode(int vkCode) {
    using namespace MacKeyCodes;

    // Letters A-Z (VK 0x41-0x5A)
    if (vkCode >= VK_A_KEY && vkCode <= VK_Z_KEY) {
        switch (vkCode) {
            case 0x41: return MAC_A;
            case 0x42: return MAC_B;
            case 0x43: return MAC_C;
            case 0x44: return MAC_D;
            case 0x45: return MAC_E;
            case 0x46: return MAC_F;
            case 0x47: return MAC_G;
            case 0x48: return MAC_H;
            case 0x49: return MAC_I;
            case 0x4A: return MAC_J;
            case 0x4B: return MAC_K;
            case 0x4C: return MAC_L;
            case 0x4D: return MAC_M;
            case 0x4E: return MAC_N;
            case 0x4F: return MAC_O;
            case 0x50: return MAC_P;
            case 0x51: return MAC_Q;
            case 0x52: return MAC_R;
            case 0x53: return MAC_S;
            case 0x54: return MAC_T;
            case 0x55: return MAC_U;
            case 0x56: return MAC_V;
            case 0x57: return MAC_W;
            case 0x58: return MAC_X;
            case 0x59: return MAC_Y;
            case 0x5A: return MAC_Z;
            default: return INVALID;
        }
    }

    // Numbers 0-9 (VK 0x30-0x39)
    if (vkCode >= VK_0_KEY && vkCode <= VK_9_KEY) {
        switch (vkCode) {
            case 0x30: return MAC_N0;
            case 0x31: return MAC_N1;
            case 0x32: return MAC_N2;
            case 0x33: return MAC_N3;
            case 0x34: return MAC_N4;
            case 0x35: return MAC_N5;
            case 0x36: return MAC_N6;
            case 0x37: return MAC_N7;
            case 0x38: return MAC_N8;
            case 0x39: return MAC_N9;
            default: return INVALID;
        }
    }

    // Special keys
    switch (vkCode) {
        case VK_SPACE_KEY: return MAC_SPACE;
        case VK_BACK_KEY: return MAC_DELETE;
        case VK_TAB_KEY: return MAC_TAB;
        case VK_RETURN_KEY: return MAC_RETURN;
        case VK_ESCAPE_KEY: return MAC_ESC;
        case VK_LEFT_KEY: return MAC_LEFT;
        case VK_RIGHT_KEY: return MAC_RIGHT;
        case VK_UP_KEY: return MAC_UP;
        case VK_DOWN_KEY: return MAC_DOWN;
        case VK_OEM_1_KEY: return MAC_SEMICOLON;      // ;:
        case VK_OEM_PLUS_KEY: return MAC_EQUAL;       // =+
        case VK_OEM_COMMA_KEY: return MAC_COMMA;      // ,<
        case VK_OEM_MINUS_KEY: return MAC_MINUS;      // -_
        case VK_OEM_PERIOD_KEY: return MAC_DOT;       // .>
        case VK_OEM_2_KEY: return MAC_SLASH;          // /?
        case VK_OEM_3_KEY: return MAC_BACKQUOTE;      // `~
        case VK_OEM_4_KEY: return MAC_LBRACKET;       // [{
        case VK_OEM_5_KEY: return MAC_BACKSLASH;      // \|
        case VK_OEM_6_KEY: return MAC_RBRACKET;       // ]}
        case VK_OEM_7_KEY: return MAC_QUOTE;          // '"
        default: return INVALID;
    }
}

bool IsLetter(int vkCode) {
    return vkCode >= VK_A_KEY && vkCode <= VK_Z_KEY;
}

bool IsNumber(int vkCode) {
    return vkCode >= VK_0_KEY && vkCode <= VK_9_KEY;
}

bool IsRelevantKey(int vkCode) {
    return IsLetter(vkCode) ||
           IsNumber(vkCode) ||
           vkCode == VK_BACK_KEY ||
           vkCode == VK_ESCAPE_KEY ||
           vkCode == VK_SPACE_KEY ||
           vkCode == VK_TAB_KEY ||
           vkCode == VK_RETURN_KEY ||
           (vkCode >= VK_LEFT_KEY && vkCode <= VK_DOWN_KEY) ||
           (vkCode >= VK_OEM_1_KEY && vkCode <= VK_OEM_7_KEY) ||
           vkCode == VK_OEM_4_KEY || vkCode == VK_OEM_5_KEY || vkCode == VK_OEM_6_KEY;
}

bool IsBufferClearKey(int vkCode) {
    return vkCode == VK_SPACE_KEY ||
           vkCode == VK_TAB_KEY ||
           vkCode == VK_RETURN_KEY ||
           (vkCode >= VK_LEFT_KEY && vkCode <= VK_DOWN_KEY);
}

char ToChar(int vkCode, bool shift, bool capsLock) {
    bool upper = shift ^ capsLock;

    // Letters A-Z
    if (vkCode >= VK_A_KEY && vkCode <= VK_Z_KEY) {
        char c = 'a' + (char)(vkCode - VK_A_KEY);
        return upper ? (char)toupper(c) : c;
    }

    // Numbers 0-9
    if (vkCode >= VK_0_KEY && vkCode <= VK_9_KEY) {
        return '0' + (char)(vkCode - VK_0_KEY);
    }

    // Hyphen/minus key for shortcuts like --danger
    if (vkCode == VK_OEM_MINUS_KEY) {
        return shift ? '_' : '-';
    }

    return 0;
}

} // namespace KeyCodes
