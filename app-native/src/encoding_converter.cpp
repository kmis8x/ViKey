// ViKey - Encoding Converter Implementation
// encoding_converter.cpp
// Project: ViKey | Author: Tran Cong Sinh | https://github.com/kmis8x/ViKey

#include "encoding_converter.h"
#include <unordered_map>

// VNI Windows mapping (VNI char -> Unicode char)
static const std::unordered_map<wchar_t, wchar_t> g_vniToUnicode = {
    // Lowercase vowels with diacritics
    {0x00E1, 0x00E1}, // a acute - same
    {0x00E0, 0x00E0}, // a grave - same
    {0x1EA3, 0x1EA3}, // a hook - same
    {0x00E3, 0x00E3}, // a tilde - same
    {0x1EA1, 0x1EA1}, // a dot - same
    // VNI specific characters
    {0x00E2, 0x00E2}, // a circumflex - same
    {0x0103, 0x0103}, // a breve - same
    // ... more mappings
};

// Unicode characters for Vietnamese
static const wchar_t UNICODE_VIET[] = L"aàảãáạăằẳẵắặâầẩẫấậeèẻẽéẹêềểễếệiìỉĩíịoòỏõóọôồổỗốộơờởỡớợuùủũúụưừửữứựyỳỷỹýỵđAÀẢÃÁẠĂẰẲẴẮẶÂẦẨẪẤẬEÈẺẼÉẸÊỀỂỄẾỆIÌỈĨÍỊOÒỎÕÓỌÔỒỔỖỐỘƠỜỞỠỚỢUÙỦŨÚỤƯỪỬỮỨỰYỲỶỸÝỴĐ";

// VNI Windows character codes (corresponding to UNICODE_VIET)
static const wchar_t VNI_VIET[] = L"aµ¶·¸¹¨»¼½¾¿©ÇÈÉÊËeÌÍÎÏÐª«Ñ®ÒÓiÔÕÖ×Øo¹º»¼½¤åæçèé¥êëìíîuïðñòó¦ôõö÷øyùúûüýđAÙÚÛÜÝ¡ßàáâã¢äåæçèEéêëìí£ïðñòóIôõö÷øO¿ÀÁÂÃ¬ÄÅÆÇÈÊËÌÍÎUÏÐÑÒÓYôõö÷øD";

// TCVN3 character codes
static const unsigned char TCVN3_VIET[] = {
    0x61, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xA8, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
    0xA9, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0x65, 0xCC, 0xCD, 0xCE, 0xCF, 0xD0,
    0xAA, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0x69, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
    0x6F, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF, 0xA4, 0xE0, 0xE1, 0xE2, 0xE3, 0xE4,
    0xA5, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0x75, 0xEA, 0xEB, 0xEC, 0xED, 0xEE,
    0xA6, 0xEF, 0xF0, 0xF1, 0xF2, 0xF3, 0x79, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xAE, // d
    // Uppercase
    0x41, 0x80, 0x81, 0x82, 0x83, 0x84, 0xA1, 0x85, 0x86, 0x87, 0x88, 0x89,
    0xA2, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x45, 0x8F, 0x90, 0x91, 0x92, 0x93,
    0xA3, 0x94, 0x95, 0x96, 0x97, 0x98, 0x49, 0x99, 0x9A, 0x9B, 0x9C, 0x9D,
    0x4F, 0x9E, 0x9F, 0xA0, 0xAF, 0xB0, 0xAC, 0xB1, 0xB2, 0xB3, 0xB4, 0xBA,
    0xAD, 0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0x55, 0xC5, 0xC6, 0xFA, 0xFB, 0xFC,
    0xFD, 0xFE, 0x59, 0xFF,
    0xD0 // D
};

EncodingConverter& EncodingConverter::Instance() {
    static EncodingConverter instance;
    return instance;
}

const wchar_t* EncodingConverter::GetEncodingName(VietEncoding enc) {
    switch (enc) {
        case VietEncoding::Unicode: return L"Unicode";
        case VietEncoding::VNI_Windows: return L"VNI Windows";
        case VietEncoding::TCVN3: return L"TCVN3 (ABC)";
        case VietEncoding::Unicode_Comp: return L"Unicode Composite";
        default: return L"Unknown";
    }
}

std::wstring EncodingConverter::Convert(const std::wstring& text, VietEncoding from, VietEncoding to) {
    if (from == to) return text;

    // First convert to Unicode (intermediate format)
    std::wstring unicode;
    switch (from) {
        case VietEncoding::Unicode:
            unicode = text;
            break;
        case VietEncoding::VNI_Windows:
            unicode = VNIToUnicode(text);
            break;
        case VietEncoding::TCVN3:
            unicode = TCVN3ToUnicode(text);
            break;
        case VietEncoding::Unicode_Comp:
            unicode = CompositeToUnicode(text);
            break;
    }

    // Then convert from Unicode to target encoding
    switch (to) {
        case VietEncoding::Unicode:
            return unicode;
        case VietEncoding::VNI_Windows:
            return UnicodeToVNI(unicode);
        case VietEncoding::TCVN3:
            return UnicodeToTCVN3(unicode);
        case VietEncoding::Unicode_Comp:
            return UnicodeToComposite(unicode);
    }

    return text;
}

std::wstring EncodingConverter::UnicodeToVNI(const std::wstring& text) {
    std::wstring result;
    result.reserve(text.size());

    for (wchar_t c : text) {
        bool found = false;
        size_t len = wcslen(UNICODE_VIET);
        for (size_t i = 0; i < len && i < wcslen(VNI_VIET); i++) {
            if (c == UNICODE_VIET[i]) {
                result += VNI_VIET[i];
                found = true;
                break;
            }
        }
        if (!found) {
            result += c;
        }
    }

    return result;
}

std::wstring EncodingConverter::VNIToUnicode(const std::wstring& text) {
    std::wstring result;
    result.reserve(text.size());

    for (wchar_t c : text) {
        bool found = false;
        size_t len = wcslen(VNI_VIET);
        for (size_t i = 0; i < len && i < wcslen(UNICODE_VIET); i++) {
            if (c == VNI_VIET[i]) {
                result += UNICODE_VIET[i];
                found = true;
                break;
            }
        }
        if (!found) {
            result += c;
        }
    }

    return result;
}

std::wstring EncodingConverter::UnicodeToTCVN3(const std::wstring& text) {
    std::wstring result;
    result.reserve(text.size());

    size_t unicodeLen = wcslen(UNICODE_VIET);
    size_t tcvnLen = sizeof(TCVN3_VIET);

    for (wchar_t c : text) {
        bool found = false;
        for (size_t i = 0; i < unicodeLen && i < tcvnLen; i++) {
            if (c == UNICODE_VIET[i]) {
                result += static_cast<wchar_t>(TCVN3_VIET[i]);
                found = true;
                break;
            }
        }
        if (!found) {
            result += c;
        }
    }

    return result;
}

std::wstring EncodingConverter::TCVN3ToUnicode(const std::wstring& text) {
    std::wstring result;
    result.reserve(text.size());

    size_t unicodeLen = wcslen(UNICODE_VIET);
    size_t tcvnLen = sizeof(TCVN3_VIET);

    for (wchar_t c : text) {
        bool found = false;
        unsigned char byte = static_cast<unsigned char>(c);
        for (size_t i = 0; i < tcvnLen && i < unicodeLen; i++) {
            if (byte == TCVN3_VIET[i]) {
                result += UNICODE_VIET[i];
                found = true;
                break;
            }
        }
        if (!found) {
            result += c;
        }
    }

    return result;
}

std::wstring EncodingConverter::UnicodeToComposite(const std::wstring& text) {
    // Unicode NFC -> NFD conversion (precomposed -> decomposed)
    // Using Windows API
    int len = NormalizeString(NormalizationD, text.c_str(), -1, nullptr, 0);
    if (len <= 0) return text;

    std::wstring result(len, L'\0');
    NormalizeString(NormalizationD, text.c_str(), -1, &result[0], len);

    // Remove trailing null
    while (!result.empty() && result.back() == L'\0') {
        result.pop_back();
    }

    return result;
}

std::wstring EncodingConverter::CompositeToUnicode(const std::wstring& text) {
    // Unicode NFD -> NFC conversion (decomposed -> precomposed)
    int len = NormalizeString(NormalizationC, text.c_str(), -1, nullptr, 0);
    if (len <= 0) return text;

    std::wstring result(len, L'\0');
    NormalizeString(NormalizationC, text.c_str(), -1, &result[0], len);

    // Remove trailing null
    while (!result.empty() && result.back() == L'\0') {
        result.pop_back();
    }

    return result;
}
