use super::*;

// Helper: Create table with one word-boundary shortcut
fn table_with_shortcut(trigger: &str, replacement: &str) -> ShortcutTable {
    let mut table = ShortcutTable::new();
    table.add(Shortcut::new(trigger, replacement));
    table
}

// Helper: Create table with one immediate shortcut
fn table_with_immediate(trigger: &str, replacement: &str) -> ShortcutTable {
    let mut table = ShortcutTable::new();
    table.add(Shortcut::immediate(trigger, replacement));
    table
}

// Helper: Create table with Telex-specific shortcut
fn table_with_telex_shortcut(trigger: &str, replacement: &str) -> ShortcutTable {
    let mut table = ShortcutTable::new();
    table.add(Shortcut::telex(trigger, replacement));
    table
}

// Helper: Create table with VNI-specific shortcut
fn table_with_vni_shortcut(trigger: &str, replacement: &str) -> ShortcutTable {
    let mut table = ShortcutTable::new();
    table.add(Shortcut::vni(trigger, replacement));
    table
}

// Helper: Assert shortcut matches and check output/backspace
fn assert_shortcut_match(
    table: &ShortcutTable,
    buffer: &str,
    key_char: Option<char>,
    is_boundary: bool,
    expected_output: &str,
    expected_backspace: usize,
    method: InputMethod,
) {
    let result = table.try_match_for_method(buffer, key_char, is_boundary, method);
    assert!(
        result.is_some(),
        "Shortcut should match for buffer: {}",
        buffer
    );
    let m = result.unwrap();
    assert_eq!(m.output, expected_output);
    assert_eq!(m.backspace_count, expected_backspace);
}

// Helper: Assert no shortcut match
fn assert_no_match(
    table: &ShortcutTable,
    buffer: &str,
    key_char: Option<char>,
    is_boundary: bool,
    method: InputMethod,
) {
    let result = table.try_match_for_method(buffer, key_char, is_boundary, method);
    assert!(
        result.is_none(),
        "Shortcut should NOT match for buffer: {}",
        buffer
    );
}

#[test]
fn test_basic_shortcut() {
    let table = table_with_shortcut("vn", "Việt Nam");
    assert_shortcut_match(
        &table,
        "vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::All,
    );
}

#[test]
fn test_case_matching() {
    let table = table_with_shortcut("vn", "Việt Nam");

    // Lowercase "vn" → "Việt Nam" (as-is)
    assert_shortcut_match(
        &table,
        "vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::All,
    );

    // Issue #86: Uppercase "VN" → "VIỆT NAM" (smart case)
    assert_shortcut_match(
        &table,
        "VN",
        Some(' '),
        true,
        "VIỆT NAM ",
        2,
        InputMethod::All,
    );

    // Issue #86: Title case "Vn" → "Việt Nam" (smart case)
    assert_shortcut_match(
        &table,
        "Vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::All,
    );
}

#[test]
fn test_immediate_shortcut() {
    let table = table_with_immediate("w", "ư");

    // Immediate triggers without word boundary
    let result = table.try_match("w", None, false);
    assert!(result.is_some());
    let m = result.unwrap();
    assert_eq!(m.output, "ư");
    assert!(!m.include_trigger_key);
}

#[test]
fn test_word_boundary_required() {
    let table = table_with_shortcut("vn", "Việt Nam");

    // Without word boundary - should not match
    assert_no_match(&table, "vn", Some('a'), false, InputMethod::All);

    // With word boundary - should match
    assert_shortcut_match(
        &table,
        "vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::All,
    );
}

#[test]
fn test_longest_match() {
    let mut table = ShortcutTable::new();
    table.add(Shortcut::new("h", "họ"));
    table.add(Shortcut::new("hcm", "Hồ Chí Minh"));

    // "hcm" should match the longer shortcut
    let (trigger, _) = table.lookup("hcm").unwrap();
    assert_eq!(trigger, "hcm");
}

#[test]
fn test_disabled_shortcut() {
    let mut table = ShortcutTable::new();
    let mut shortcut = Shortcut::new("vn", "Việt Nam");
    shortcut.enabled = false;
    table.add(shortcut);

    let result = table.lookup("vn");
    assert!(result.is_none());
}

#[test]
fn test_telex_specific_shortcut() {
    let table = table_with_telex_shortcut("w", "ư");

    // Should match for Telex
    assert_shortcut_match(&table, "w", None, false, "ư", 1, InputMethod::Telex);

    // Should NOT match for VNI
    assert_no_match(&table, "w", None, false, InputMethod::Vni);

    // Should match for All (fallback)
    assert_shortcut_match(&table, "w", None, false, "ư", 1, InputMethod::All);
}

#[test]
fn test_vni_specific_shortcut() {
    let table = table_with_vni_shortcut("7", "ơ");

    // Should match for VNI
    assert_shortcut_match(&table, "7", None, false, "ơ", 1, InputMethod::Vni);

    // Should NOT match for Telex
    assert_no_match(&table, "7", None, false, InputMethod::Telex);
}

#[test]
fn test_all_input_method_shortcut() {
    let table = table_with_shortcut("vn", "Việt Nam");

    // Should match for Telex
    assert_shortcut_match(
        &table,
        "vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::Telex,
    );

    // Should match for VNI
    assert_shortcut_match(
        &table,
        "vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::Vni,
    );

    // Should match for All
    assert_shortcut_match(
        &table,
        "vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::All,
    );
}

#[test]
fn test_with_defaults_has_common_shortcuts() {
    let table = ShortcutTable::with_defaults();

    // Default shortcuts are empty — users add their own
    assert!(table.is_empty());

    // No preset shortcuts
    let result = table.lookup_for_method("vn", InputMethod::All);
    assert!(result.is_none());

    // "w" is NOT a shortcut (handled by engine)
    let result = table.lookup_for_method("w", InputMethod::Telex);
    assert!(result.is_none());
}

#[test]
fn test_shortcut_for_method_builder() {
    let shortcut = Shortcut::new("test", "Test").for_method(InputMethod::Telex);
    assert_eq!(shortcut.input_method, InputMethod::Telex);

    let shortcut = Shortcut::immediate("x", "y").for_method(InputMethod::Vni);
    assert_eq!(shortcut.input_method, InputMethod::Vni);
}

#[test]
fn test_applies_to() {
    let all_shortcut = Shortcut::new("vn", "Việt Nam");
    assert!(all_shortcut.applies_to(InputMethod::All));
    assert!(all_shortcut.applies_to(InputMethod::Telex));
    assert!(all_shortcut.applies_to(InputMethod::Vni));

    let telex_shortcut = Shortcut::telex("test", "Test");
    assert!(telex_shortcut.applies_to(InputMethod::All));
    assert!(telex_shortcut.applies_to(InputMethod::Telex));
    assert!(!telex_shortcut.applies_to(InputMethod::Vni));

    let vni_shortcut = Shortcut::vni("7", "ơ");
    assert!(vni_shortcut.applies_to(InputMethod::All));
    assert!(!vni_shortcut.applies_to(InputMethod::Telex));
    assert!(vni_shortcut.applies_to(InputMethod::Vni));
}

#[test]
fn test_replacement_validation_within_limit() {
    // Vietnamese text within limit (21 codepoints)
    let shortcut = Shortcut::new("tphcm", "Thành phố Hồ Chí Minh");
    assert_eq!(shortcut.replacement, "Thành phố Hồ Chí Minh");
    assert_eq!(shortcut.replacement.chars().count(), 21);
}

#[test]
fn test_replacement_validation_truncation() {
    // Create a very long replacement (>255 characters with Vietnamese)
    // MAX_REPLACEMENT_LEN is 255, so we need more than that
    let long_text = "Đây là một đoạn văn bản rất dài để kiểm tra việc cắt ngắn. Nó có nhiều ký tự tiếng Việt có dấu như ồ, ế, ẫ, ơ, ư. Tiếp tục thêm nhiều nội dung để vượt quá giới hạn 255 ký tự. Đây là một câu rất dài với nhiều từ tiếng Việt phức tạp để đảm bảo rằng chúng ta vượt quá giới hạn cho phép của hệ thống.";
    let char_count = long_text.chars().count();
    assert!(
        char_count > MAX_REPLACEMENT_LEN,
        "Test text should exceed limit (got {} chars, need > {})",
        char_count,
        MAX_REPLACEMENT_LEN
    );

    let shortcut = Shortcut::new("long", long_text);
    let result_count = shortcut.replacement.chars().count();
    assert_eq!(
        result_count, MAX_REPLACEMENT_LEN,
        "Should truncate to MAX_REPLACEMENT_LEN"
    );
}

#[test]
fn test_replacement_validation_vietnamese_diacritics() {
    // Each Vietnamese character with diacritic is 1 codepoint
    // "ồ" = 1 codepoint, "ế" = 1 codepoint, "ẫ" = 1 codepoint
    let vietnamese = "ồếẫơưáàảãạăắằẳẵặâấầẩẫậ"; // 22 Vietnamese chars
    let shortcut = Shortcut::new("viet", vietnamese);
    assert_eq!(shortcut.replacement.chars().count(), 22);
    assert_eq!(shortcut.replacement, vietnamese);
}

// =========================================================================
// Issue #178: Shortcuts > 63 chars should work
// Issue #178: Edge case handling
// =========================================================================

#[test]
fn issue178_long_shortcut_100_chars() {
    // Test that shortcuts with 100+ chars work (previously limited to 63)
    let long_replacement = "Đây là một đoạn văn bản dài hơn 63 ký tự để kiểm tra rằng gõ tắt giờ hỗ trợ nội dung dài hơn trước.";
    let char_count = long_replacement.chars().count();
    assert!(
        char_count > 63,
        "Test text should exceed old limit of 63 (got {})",
        char_count
    );

    let table = table_with_shortcut("long", long_replacement);
    let result = table.try_match("long", Some(' '), true);
    assert!(result.is_some(), "Shortcut should match");

    let m = result.unwrap();
    // Full replacement + trailing space
    assert_eq!(
        m.output.chars().count(),
        char_count + 1,
        "Output should contain full replacement + space"
    );
    assert!(
        m.output.starts_with(long_replacement),
        "Output should start with full replacement"
    );
}

#[test]
fn issue178_long_shortcut_200_chars() {
    // Test even longer shortcut (200 chars)
    let long_replacement = "Thành phố Hồ Chí Minh là thành phố lớn nhất Việt Nam về dân số và kinh tế. Đây là một trong những trung tâm kinh tế, chính trị, văn hóa và giáo dục quan trọng nhất của cả nước. Thành phố này còn được biết đến với nhiều tên gọi khác.";
    let char_count = long_replacement.chars().count();
    assert!(
        char_count > 100,
        "Test text should be > 100 chars (got {})",
        char_count
    );

    let shortcut = Shortcut::new("hcm2", long_replacement);
    assert_eq!(
        shortcut.replacement.chars().count(),
        char_count,
        "Replacement should not be truncated for {} chars",
        char_count
    );
    assert_eq!(shortcut.replacement, long_replacement);
}

// =========================================================================
// Issue #86: Smart Case-Aware Shortcuts
// Issue #86: Smart Case-Aware Shortcuts
// =========================================================================

#[test]
fn issue86_smart_case_lowercase() {
    let table = table_with_shortcut("ko", "không");
    assert_shortcut_match(&table, "ko", Some(' '), true, "không ", 2, InputMethod::All);
}

#[test]
fn issue86_smart_case_uppercase() {
    let table = table_with_shortcut("ko", "không");
    assert_shortcut_match(&table, "KO", Some(' '), true, "KHÔNG ", 2, InputMethod::All);
}

#[test]
fn issue86_smart_case_titlecase() {
    let table = table_with_shortcut("ko", "không");
    assert_shortcut_match(&table, "Ko", Some(' '), true, "Không ", 2, InputMethod::All);
}

#[test]
fn issue86_smart_case_vn_lowercase() {
    let table = table_with_shortcut("vn", "Việt Nam");
    assert_shortcut_match(
        &table,
        "vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::All,
    );
}

#[test]
fn issue86_smart_case_vn_uppercase() {
    let table = table_with_shortcut("vn", "Việt Nam");
    assert_shortcut_match(
        &table,
        "VN",
        Some(' '),
        true,
        "VIỆT NAM ",
        2,
        InputMethod::All,
    );
}

#[test]
fn issue86_smart_case_vn_titlecase() {
    let table = table_with_shortcut("vn", "Việt Nam");
    assert_shortcut_match(
        &table,
        "Vn",
        Some(' '),
        true,
        "Việt Nam ",
        2,
        InputMethod::All,
    );
}

#[test]
fn issue86_smart_case_immediate_shortcut() {
    let table = table_with_immediate("dc", "được");

    let result = table.try_match("dc", None, false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().output, "được");

    let result = table.try_match("DC", None, false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().output, "ĐƯỢC");

    let result = table.try_match("Dc", None, false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().output, "Được");
}

#[test]
fn issue86_smart_case_lookup_case_insensitive() {
    let table = table_with_shortcut("ko", "không");
    assert!(table.lookup("ko").is_some());
    assert!(table.lookup("Ko").is_some());
    assert!(table.lookup("KO").is_some());
    assert!(table.lookup("kO").is_some());
}

#[test]
fn issue86_smart_case_mixed_case_fallback() {
    let table = table_with_shortcut("ko", "không");
    assert_shortcut_match(&table, "kO", Some(' '), true, "không ", 2, InputMethod::All);
}

#[test]
fn issue86_smart_case_hcm() {
    let table = table_with_shortcut("hcm", "Hồ Chí Minh");
    assert_shortcut_match(
        &table,
        "hcm",
        Some(' '),
        true,
        "Hồ Chí Minh ",
        3,
        InputMethod::All,
    );
    assert_shortcut_match(
        &table,
        "HCM",
        Some(' '),
        true,
        "HỒ CHÍ MINH ",
        3,
        InputMethod::All,
    );
    assert_shortcut_match(
        &table,
        "Hcm",
        Some(' '),
        true,
        "Hồ Chí Minh ",
        3,
        InputMethod::All,
    );
}
