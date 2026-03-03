use super::*;
use crate::utils::keys_from_str;

#[test]
fn parse_simple_syllable() {
    let s = parse(&keys_from_str("ba"));
    assert_eq!(s.initial.len(), 1);
    assert_eq!(s.vowel.len(), 1);
    assert!(s.final_c.is_empty());
}

#[test]
fn parse_ngh_initial() {
    let s = parse(&keys_from_str("nghieng"));
    assert_eq!(s.initial.len(), 3);
    assert_eq!(s.vowel.len(), 2);
    assert_eq!(s.final_c.len(), 2);
}

#[test]
fn parse_qu_initial() {
    let s = parse(&keys_from_str("qua"));
    assert_eq!(s.initial.len(), 2);
    assert_eq!(s.vowel.len(), 1);
    assert!(s.glide.is_none());
}

#[test]
fn parse_hoa_with_glide() {
    let s = parse(&keys_from_str("hoa"));
    assert_eq!(s.initial.len(), 1);
    assert!(s.glide.is_some());
    assert_eq!(s.vowel.len(), 1);
}

#[test]
fn parse_gi_initial() {
    let s = parse(&keys_from_str("giau"));
    assert_eq!(s.initial.len(), 2);
    assert_eq!(s.vowel.len(), 2);
}

#[test]
fn parse_duoc() {
    let s = parse(&keys_from_str("duoc"));
    assert_eq!(s.initial.len(), 1);
    assert_eq!(s.vowel.len(), 2);
    assert_eq!(s.final_c.len(), 1);
}

#[test]
fn parse_vowel_only() {
    let s = parse(&keys_from_str("a"));
    assert!(s.initial.is_empty());
    assert_eq!(s.vowel.len(), 1);
}

#[test]
fn invalid_no_vowel() {
    let s = parse(&keys_from_str("bcd"));
    assert!(s.is_empty());
}

#[test]
fn test_is_valid_structure() {
    assert!(is_valid_structure(&keys_from_str("ba")));
    assert!(is_valid_structure(&keys_from_str("nghieng")));
    assert!(is_valid_structure(&keys_from_str("a")));
    assert!(!is_valid_structure(&keys_from_str("bcd")));
    assert!(!is_valid_structure(&keys_from_str("")));
}
