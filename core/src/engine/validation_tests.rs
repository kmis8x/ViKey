use super::*;
use crate::data::chars::tone;
use crate::utils::keys_from_str;

/// Valid Vietnamese syllables
const VALID: &[&str] = &[
    "ba", "ca", "an", "em", "gi", "gia", "giau", "ke", "ki", "ky", "nghe", "nghi", "nghieng",
    "truong", "nguoi", "duoc",
];

/// Invalid: no vowel
const INVALID_NO_VOWEL: &[&str] = &["bcd", "bcdfgh"];

/// Invalid: bad initial
const INVALID_INITIAL: &[&str] = &["clau", "john", "bla", "string", "chrome"];

/// Invalid: spelling violations
const INVALID_SPELLING: &[&str] = &["ci", "ce", "cy", "ka", "ko", "ku", "ngi", "nge", "ge"];

/// Invalid: foreign words
const INVALID_FOREIGN: &[&str] = &["exp", "expect", "test", "claudeco", "claus", "gues"];

fn assert_all_valid(words: &[&str]) {
    for w in words {
        assert!(is_valid(&keys_from_str(w)), "'{}' should be valid", w);
    }
}

fn assert_all_invalid(words: &[&str]) {
    for w in words {
        assert!(!is_valid(&keys_from_str(w)), "'{}' should be invalid", w);
    }
}

#[test]
fn test_valid() {
    assert_all_valid(VALID);
}

#[test]
fn test_invalid_no_vowel() {
    assert_all_invalid(INVALID_NO_VOWEL);
}

#[test]
fn test_invalid_initial() {
    assert_all_invalid(INVALID_INITIAL);
}

#[test]
fn test_invalid_spelling() {
    assert_all_invalid(INVALID_SPELLING);
}

#[test]
fn test_invalid_foreign() {
    assert_all_invalid(INVALID_FOREIGN);
}

// New tests for whitelist validation
#[test]
fn test_eu_invalid_without_circumflex() {
    // "eu" without circumflex should be invalid
    let keys = keys_from_str("neu");
    let tones = vec![0, 0, 0]; // no modifiers
    assert!(
        !is_valid_with_tones(&keys, &tones),
        "'neu' without circumflex should be invalid"
    );
}

#[test]
fn test_eu_valid_with_circumflex() {
    // "êu" with circumflex should be valid
    let keys = keys_from_str("neu");
    let tones = vec![0, tone::CIRCUMFLEX, 0]; // circumflex on E
    assert!(
        is_valid_with_tones(&keys, &tones),
        "'nêu' with circumflex should be valid"
    );
}

#[test]
fn test_valid_diphthongs() {
    // Test some valid diphthong patterns
    let valid_patterns = [
        "ai", "ao", "au", "eo", "ia", "iu", "oa", "oe", "oi", "ui", "uy",
    ];
    for pattern in valid_patterns {
        let keys = keys_from_str(pattern);
        assert!(is_valid(&keys), "'{}' should be valid diphthong", pattern);
    }
}

#[test]
fn test_invalid_diphthongs() {
    // Test some invalid diphthong patterns (not in whitelist)
    let invalid_patterns = ["ou", "yo", "ae", "yi"];
    for pattern in invalid_patterns {
        let keys = keys_from_str(pattern);
        assert!(
            !is_valid(&keys),
            "'{}' should be invalid diphthong",
            pattern
        );
    }
}

#[test]
fn test_gues_invalid_final() {
    // "gues" has invalid final 's' - should fail validation
    let keys = keys_from_str("gues");
    let tones = vec![0; 4]; // no tones
    assert!(
        !is_valid_with_tones(&keys, &tones),
        "'gues' should be invalid (S is not valid Vietnamese final)"
    );
}

#[test]
fn test_breve_followed_by_vowel_invalid() {
    // Issue #44: "taiw" → "tăi" should be invalid
    // Breve (ă) cannot be followed by another vowel in Vietnamese
    // Valid: ăm, ăn, ăng (consonant endings), oă (xoăn)
    // Invalid: ăi, ăo, ău, ăy
    let keys = keys_from_str("tai");
    let tones = vec![0, tone::HORN, 0]; // breve on 'a'
    assert!(
        !is_valid_with_tones(&keys, &tones),
        "'tăi' (breve + vowel) should be invalid"
    );

    // Also test standalone ăi
    let keys = keys_from_str("ai");
    let tones = vec![tone::HORN, 0]; // breve on 'a'
    assert!(
        !is_valid_with_tones(&keys, &tones),
        "'ăi' should be invalid"
    );
}
