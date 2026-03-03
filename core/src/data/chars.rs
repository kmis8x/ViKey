//! Vietnamese Unicode Character System
//!
//! Forward: base vowels + modifiers + marks → Vietnamese Unicode
//! Reverse: delegates to `chars_parse` module
//!
//! ## Design Principles
//! - Single lookup table for all vowel combinations (12 bases × 6 marks = 72)
//! - O(1) reverse lookup via match (compiler-optimized jump table)
//! - Uses Rust's built-in `to_uppercase()` for case conversion

use super::keys;

/// Tone modifiers (dấu phụ) - changes base vowel form
pub mod tone {
    pub const NONE: u8 = 0;
    pub const CIRCUMFLEX: u8 = 1; // ^ (mũ): a→â, e→ê, o→ô
    pub const HORN: u8 = 2;       // ơ, ư or breve ă
}

/// Marks (dấu thanh) - Vietnamese tone marks
pub mod mark {
    pub const NONE: u8 = 0;
    pub const SAC: u8 = 1;   // sắc (á)
    pub const HUYEN: u8 = 2; // huyền (à)
    pub const HOI: u8 = 3;   // hỏi (ả)
    pub const NGA: u8 = 4;   // ngã (ã)
    pub const NANG: u8 = 5;  // nặng (ạ)
}

/// Vietnamese vowel lookup table
/// Each entry: (base_char, [sắc, huyền, hỏi, ngã, nặng])
const VOWEL_TABLE: [(char, [char; 5]); 12] = [
    ('a', ['á', 'à', 'ả', 'ã', 'ạ']),
    ('ă', ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ']),
    ('â', ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ']),
    ('e', ['é', 'è', 'ẻ', 'ẽ', 'ẹ']),
    ('ê', ['ế', 'ề', 'ể', 'ễ', 'ệ']),
    ('i', ['í', 'ì', 'ỉ', 'ĩ', 'ị']),
    ('o', ['ó', 'ò', 'ỏ', 'õ', 'ọ']),
    ('ô', ['ố', 'ồ', 'ổ', 'ỗ', 'ộ']),
    ('ơ', ['ớ', 'ờ', 'ở', 'ỡ', 'ợ']),
    ('u', ['ú', 'ù', 'ủ', 'ũ', 'ụ']),
    ('ư', ['ứ', 'ừ', 'ử', 'ữ', 'ự']),
    ('y', ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ']),
];

/// Get base character from key + tone modifier
fn get_base_char(key: u16, t: u8) -> Option<char> {
    match key {
        keys::A => Some(match t {
            tone::CIRCUMFLEX => 'â',
            tone::HORN => 'ă',
            _ => 'a',
        }),
        keys::E => Some(match t {
            tone::CIRCUMFLEX => 'ê',
            _ => 'e',
        }),
        keys::I => Some('i'),
        keys::O => Some(match t {
            tone::CIRCUMFLEX => 'ô',
            tone::HORN => 'ơ',
            _ => 'o',
        }),
        keys::U => Some(match t {
            tone::HORN => 'ư',
            _ => 'u',
        }),
        keys::Y => Some('y'),
        _ => None,
    }
}

/// Apply mark to base vowel character
fn apply_mark(base: char, m: u8) -> char {
    if m == mark::NONE || m > mark::NANG {
        return base;
    }
    VOWEL_TABLE
        .iter()
        .find(|(b, _)| *b == base)
        .map(|(_, marks)| marks[(m - 1) as usize])
        .unwrap_or(base)
}

/// Convert to uppercase using Rust's Unicode-aware method
fn to_upper(ch: char) -> char {
    ch.to_uppercase().next().unwrap_or(ch)
}

/// Convert key + modifiers to Vietnamese character
pub fn to_char(key: u16, caps: bool, tone: u8, mark: u8) -> Option<char> {
    if key == keys::D {
        return Some(if caps { 'D' } else { 'd' });
    }
    let base = get_base_char(key, tone)?;
    let marked = apply_mark(base, mark);
    Some(if caps { to_upper(marked) } else { marked })
}

/// Get đ/Đ character
pub fn get_d(caps: bool) -> char {
    if caps { 'Đ' } else { 'đ' }
}

// ============================================================
// REVERSE PARSING: Vietnamese char → buffer components
// ============================================================

/// Parsed character components for buffer restoration
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParsedChar {
    pub key: u16,
    pub caps: bool,
    pub tone: u8,
    pub mark: u8,
    pub stroke: bool,
}

impl ParsedChar {
    pub const fn new(key: u16, caps: bool, t: u8, m: u8) -> Self {
        Self { key, caps, tone: t, mark: m, stroke: false }
    }

    pub const fn stroke(key: u16, caps: bool) -> Self {
        Self { key, caps, tone: 0, mark: 0, stroke: true }
    }
}

/// Parse Vietnamese character back to buffer components
///
/// Delegates to chars_parse for the full match table.
pub use super::chars_parse::parse_char;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vowels() {
        assert_eq!(to_char(keys::A, false, 0, 0), Some('a'));
        assert_eq!(to_char(keys::E, false, 0, 0), Some('e'));
        assert_eq!(to_char(keys::I, false, 0, 0), Some('i'));
        assert_eq!(to_char(keys::O, false, 0, 0), Some('o'));
        assert_eq!(to_char(keys::U, false, 0, 0), Some('u'));
        assert_eq!(to_char(keys::Y, false, 0, 0), Some('y'));
    }

    #[test]
    fn test_tone_modifiers() {
        assert_eq!(to_char(keys::A, false, 1, 0), Some('â'));
        assert_eq!(to_char(keys::E, false, 1, 0), Some('ê'));
        assert_eq!(to_char(keys::O, false, 1, 0), Some('ô'));
        assert_eq!(to_char(keys::A, false, 2, 0), Some('ă'));
        assert_eq!(to_char(keys::O, false, 2, 0), Some('ơ'));
        assert_eq!(to_char(keys::U, false, 2, 0), Some('ư'));
    }

    #[test]
    fn test_marks() {
        assert_eq!(to_char(keys::A, false, 0, 1), Some('á'));
        assert_eq!(to_char(keys::A, false, 0, 2), Some('à'));
        assert_eq!(to_char(keys::A, false, 0, 3), Some('ả'));
        assert_eq!(to_char(keys::A, false, 0, 4), Some('ã'));
        assert_eq!(to_char(keys::A, false, 0, 5), Some('ạ'));
    }

    #[test]
    fn test_combined_tone_and_mark() {
        assert_eq!(to_char(keys::A, false, 1, 1), Some('ấ'));
        assert_eq!(to_char(keys::O, false, 2, 2), Some('ờ'));
        assert_eq!(to_char(keys::U, false, 2, 5), Some('ự'));
    }

    #[test]
    fn test_uppercase() {
        assert_eq!(to_char(keys::A, true, 0, 0), Some('A'));
        assert_eq!(to_char(keys::A, true, 0, 1), Some('Á'));
        assert_eq!(to_char(keys::A, true, 1, 1), Some('Ấ'));
        assert_eq!(to_char(keys::O, true, 2, 2), Some('Ờ'));
        assert_eq!(to_char(keys::U, true, 2, 5), Some('Ự'));
    }

    #[test]
    fn test_d() {
        assert_eq!(get_d(false), 'đ');
        assert_eq!(get_d(true), 'Đ');
    }

    #[test]
    fn test_parse_basic_vowels() {
        let p = parse_char('a').unwrap();
        assert_eq!((p.key, p.caps, p.tone, p.mark), (keys::A, false, 0, 0));
        let p = parse_char('E').unwrap();
        assert_eq!((p.key, p.caps, p.tone, p.mark), (keys::E, true, 0, 0));
    }

    #[test]
    fn test_parse_vowels_with_marks() {
        let p = parse_char('á').unwrap();
        assert_eq!((p.key, p.tone, p.mark), (keys::A, 0, mark::SAC));
        let p = parse_char('ụ').unwrap();
        assert_eq!((p.key, p.tone, p.mark), (keys::U, 0, mark::NANG));
        let p = parse_char('Ề').unwrap();
        assert_eq!(
            (p.key, p.caps, p.tone, p.mark),
            (keys::E, true, tone::CIRCUMFLEX, mark::HUYEN)
        );
    }

    #[test]
    fn test_parse_complex_vowels() {
        let p = parse_char('ự').unwrap();
        assert_eq!((p.key, p.tone, p.mark), (keys::U, tone::HORN, mark::NANG));
        let p = parse_char('ấ').unwrap();
        assert_eq!(
            (p.key, p.tone, p.mark),
            (keys::A, tone::CIRCUMFLEX, mark::SAC)
        );
    }

    #[test]
    fn test_parse_d_stroke() {
        let p = parse_char('đ').unwrap();
        assert!(p.stroke);
        assert_eq!(p.key, keys::D);
        assert!(!p.caps);
        let p = parse_char('Đ').unwrap();
        assert!(p.stroke);
        assert!(p.caps);
    }

    #[test]
    fn test_parse_consonants() {
        let p = parse_char('n').unwrap();
        assert_eq!(p.key, keys::N);
        assert!(!p.stroke);
        let p = parse_char('T').unwrap();
        assert_eq!(p.key, keys::T);
        assert!(p.caps);
    }

    #[test]
    fn test_parse_roundtrip() {
        let test_cases = [
            ('a', keys::A, 0, 0),
            ('á', keys::A, 0, 1),
            ('ả', keys::A, 0, 3),
            ('â', keys::A, 1, 0),
            ('ấ', keys::A, 1, 1),
            ('ậ', keys::A, 1, 5),
            ('ă', keys::A, 2, 0),
            ('ắ', keys::A, 2, 1),
            ('ư', keys::U, 2, 0),
            ('ự', keys::U, 2, 5),
            ('ơ', keys::O, 2, 0),
            ('ợ', keys::O, 2, 5),
        ];
        for (ch, key, t, m) in test_cases {
            let p = parse_char(ch).unwrap();
            assert_eq!((p.key, p.tone, p.mark), (key, t, m), "Failed for '{}'", ch);
        }
    }
}
