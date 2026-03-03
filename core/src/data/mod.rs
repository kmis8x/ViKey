//! Vietnamese Language Data Modules
//!
//! This module contains all linguistic data for Vietnamese input:
//! - `keys`: Virtual keycode definitions (platform-specific)
//! - `chars`: Unicode character conversion (includes tone/mark constants)
//! - `chars_parse`: Reverse parsing: Vietnamese Unicode → buffer components
//! - `vowel`: Vietnamese vowel phonology system (core types + HORN_PATTERNS)
//! - `vowel_tone_patterns`: Diphthong/triphthong tone placement tables
//! - `vowel_phonology`: Phonology struct with tone/horn placement logic
//! - `constants`: Vietnamese phonological constants (initials, finals, vowels)
//! - `constants_auto_restore`: Auto-restore specific detection patterns
//! - `telex_doubles`: English words with Telex double patterns for auto-restore

pub mod chars;
pub mod chars_parse;
pub mod constants;
pub mod constants_auto_restore;
pub mod english_dict;
pub mod keys;
pub mod telex_doubles;
pub mod vowel;
pub mod vowel_phonology;
pub mod vowel_tone_patterns;

pub use chars::{get_d, mark, to_char, tone};
pub use constants::*;
pub use keys::{is_break, is_letter, is_vowel};
pub use vowel::{Modifier, Phonology, Role, Vowel};
