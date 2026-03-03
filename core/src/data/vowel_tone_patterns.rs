//! Vietnamese tone position patterns for vowel pairs and triphthongs
//!
//! Data tables used by Phonology to determine where tone marks are placed.
//! Based on docs/vietnamese-language-system.md section 7.3

use super::keys;
use super::vowel::TonePosition;

/// Triphthong pattern for tone placement
pub struct TriphthongTonePattern {
    pub v1: u16,
    pub v2: u16,
    pub v3: u16,
    pub position: TonePosition,
}

/// Diphthongs with tone on FIRST vowel (âm chính + glide)
///
/// Section 7.3.1: ai, ao, au, ay, âu, ây, eo, êu, ia, iu, oi, ôi, ơi, ui, ưi, ưu, ua*, ưa
/// *ua is First only when NOT preceded by 'q'
pub const TONE_FIRST_PATTERNS: &[[u16; 2]] = &[
    [keys::A, keys::I], // ai: mái, hài
    [keys::A, keys::O], // ao: cáo, sào
    [keys::A, keys::U], // au: sáu, màu
    [keys::A, keys::Y], // ay: máy, tày
    [keys::E, keys::O], // eo: kéo, trèo
    [keys::E, keys::U], // êu: nếu, kêu (circumflex on e, tone on e)
    [keys::I, keys::A], // ia: kìa, mía (not after gi)
    [keys::I, keys::U], // iu: dịu, kíu
    [keys::O, keys::I], // oi: đói, còi
    [keys::U, keys::I], // ui: túi, mùi
    // Note: "ua" removed - context-dependent, handled specially in find_diphthong_position()
    // Open syllable: tone on u (mùa), Closed syllable: tone on a (chuẩn)
    [keys::U, keys::U], // ưu: lưu, hưu (when first has horn)
];

/// Diphthongs with tone on SECOND vowel (âm đệm + chính, compound)
///
/// Section 7.3.1: oa, oă, oe, uê, uy, ua (after q), iê, uô, ươ
pub const TONE_SECOND_PATTERNS: &[[u16; 2]] = &[
    [keys::O, keys::A], // oa: hoà, toá
    [keys::O, keys::E], // oe: khoẻ, xoè
    [keys::U, keys::E], // uê: huế, tuệ
    [keys::U, keys::Y], // uy: quý, thuỳ
    [keys::I, keys::E], // iê: tiên (compound)
    [keys::U, keys::O], // uô/ươ: (compound - when both have horn)
];

/// Triphthongs - all use middle (position 2) except uyê
///
/// Section 7.3.3
pub const TRIPHTHONG_PATTERNS: &[TriphthongTonePattern] = &[
    TriphthongTonePattern {
        v1: keys::I,
        v2: keys::E,
        v3: keys::U,
        position: TonePosition::Second,
    }, // iêu: tiếu
    TriphthongTonePattern {
        v1: keys::Y,
        v2: keys::E,
        v3: keys::U,
        position: TonePosition::Second,
    }, // yêu: yếu
    TriphthongTonePattern {
        v1: keys::O,
        v2: keys::A,
        v3: keys::I,
        position: TonePosition::Second,
    }, // oai: ngoài
    TriphthongTonePattern {
        v1: keys::O,
        v2: keys::A,
        v3: keys::Y,
        position: TonePosition::Second,
    }, // oay: xoáy
    TriphthongTonePattern {
        v1: keys::O,
        v2: keys::E,
        v3: keys::O,
        position: TonePosition::Second,
    }, // oeo: khoèo
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::A,
        v3: keys::Y,
        position: TonePosition::Second,
    }, // uây: khuấy (â in middle)
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::O,
        v3: keys::I,
        position: TonePosition::Second,
    }, // uôi: cuối
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::O,
        v3: keys::I,
        position: TonePosition::Second,
    }, // ươi: mười (ư+ơ+i, both have horn)
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::O,
        v3: keys::U,
        position: TonePosition::Second,
    }, // ươu: rượu
    TriphthongTonePattern {
        v1: keys::I,
        v2: keys::U,
        v3: keys::O,
        position: TonePosition::Last,
    }, // iươ: giường (gi + ươ, tone on ơ)
    // Special: uyê uses Last position
    TriphthongTonePattern {
        v1: keys::U,
        v2: keys::Y,
        v3: keys::E,
        position: TonePosition::Last,
    }, // uyê: khuyến, quyền
];
