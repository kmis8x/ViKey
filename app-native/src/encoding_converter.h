// ViKey - Encoding Converter (Feature 6: Code Conversion Tool)
// encoding_converter.h
// Converts Vietnamese text between different encodings

#pragma once

#include <windows.h>
#include <string>

// Supported Vietnamese encodings
enum class VietEncoding {
    Unicode = 0,      // UTF-16/UTF-8 (default)
    VNI_Windows = 1,  // VNI Windows
    TCVN3 = 2,        // TCVN3 (ABC)
    Unicode_Comp = 3  // Unicode Composite (NFD)
};

class EncodingConverter {
public:
    static EncodingConverter& Instance();

    // Convert text between encodings
    std::wstring Convert(const std::wstring& text, VietEncoding from, VietEncoding to);

    // Get encoding name for display
    static const wchar_t* GetEncodingName(VietEncoding enc);

private:
    EncodingConverter() = default;
    ~EncodingConverter() = default;
    EncodingConverter(const EncodingConverter&) = delete;
    EncodingConverter& operator=(const EncodingConverter&) = delete;

    // Conversion helpers
    std::wstring UnicodeToVNI(const std::wstring& text);
    std::wstring VNIToUnicode(const std::wstring& text);
    std::wstring UnicodeToTCVN3(const std::wstring& text);
    std::wstring TCVN3ToUnicode(const std::wstring& text);
    std::wstring UnicodeToComposite(const std::wstring& text);
    std::wstring CompositeToUnicode(const std::wstring& text);
};
