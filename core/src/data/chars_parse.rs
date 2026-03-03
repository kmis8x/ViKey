//! Vietnamese character reverse parsing: Unicode char → buffer components
//!
//! O(1) via compiler-optimized match statement.
//! Used for buffer restoration from Vietnamese text.

use super::keys;
use super::chars::{mark, tone, ParsedChar};

/// Parse Vietnamese character back to buffer components
///
/// Returns None for unknown characters (symbols, numbers handled separately).
pub fn parse_char(c: char) -> Option<ParsedChar> {
    use keys::*;
    use mark::{HOI, HUYEN, NANG, NGA, SAC};
    use tone::{CIRCUMFLEX, HORN};
    const T0: u8 = 0;
    const M0: u8 = 0;

    macro_rules! vowel {
        ($key:expr, $caps:expr, $tone:expr, $mark:expr) => {
            Some(ParsedChar::new($key, $caps, $tone, $mark))
        };
    }

    match c {
        // ===== A variants =====
        'a' => vowel!(A, false, T0, M0),
        'A' => vowel!(A, true, T0, M0),
        'á' => vowel!(A, false, T0, SAC),
        'Á' => vowel!(A, true, T0, SAC),
        'à' => vowel!(A, false, T0, HUYEN),
        'À' => vowel!(A, true, T0, HUYEN),
        'ả' => vowel!(A, false, T0, HOI),
        'Ả' => vowel!(A, true, T0, HOI),
        'ã' => vowel!(A, false, T0, NGA),
        'Ã' => vowel!(A, true, T0, NGA),
        'ạ' => vowel!(A, false, T0, NANG),
        'Ạ' => vowel!(A, true, T0, NANG),
        // ă (breve)
        'ă' => vowel!(A, false, HORN, M0),
        'Ă' => vowel!(A, true, HORN, M0),
        'ắ' => vowel!(A, false, HORN, SAC),
        'Ắ' => vowel!(A, true, HORN, SAC),
        'ằ' => vowel!(A, false, HORN, HUYEN),
        'Ằ' => vowel!(A, true, HORN, HUYEN),
        'ẳ' => vowel!(A, false, HORN, HOI),
        'Ẳ' => vowel!(A, true, HORN, HOI),
        'ẵ' => vowel!(A, false, HORN, NGA),
        'Ẵ' => vowel!(A, true, HORN, NGA),
        'ặ' => vowel!(A, false, HORN, NANG),
        'Ặ' => vowel!(A, true, HORN, NANG),
        // â (circumflex)
        'â' => vowel!(A, false, CIRCUMFLEX, M0),
        'Â' => vowel!(A, true, CIRCUMFLEX, M0),
        'ấ' => vowel!(A, false, CIRCUMFLEX, SAC),
        'Ấ' => vowel!(A, true, CIRCUMFLEX, SAC),
        'ầ' => vowel!(A, false, CIRCUMFLEX, HUYEN),
        'Ầ' => vowel!(A, true, CIRCUMFLEX, HUYEN),
        'ẩ' => vowel!(A, false, CIRCUMFLEX, HOI),
        'Ẩ' => vowel!(A, true, CIRCUMFLEX, HOI),
        'ẫ' => vowel!(A, false, CIRCUMFLEX, NGA),
        'Ẫ' => vowel!(A, true, CIRCUMFLEX, NGA),
        'ậ' => vowel!(A, false, CIRCUMFLEX, NANG),
        'Ậ' => vowel!(A, true, CIRCUMFLEX, NANG),

        // ===== E variants =====
        'e' => vowel!(E, false, T0, M0),
        'E' => vowel!(E, true, T0, M0),
        'é' => vowel!(E, false, T0, SAC),
        'É' => vowel!(E, true, T0, SAC),
        'è' => vowel!(E, false, T0, HUYEN),
        'È' => vowel!(E, true, T0, HUYEN),
        'ẻ' => vowel!(E, false, T0, HOI),
        'Ẻ' => vowel!(E, true, T0, HOI),
        'ẽ' => vowel!(E, false, T0, NGA),
        'Ẽ' => vowel!(E, true, T0, NGA),
        'ẹ' => vowel!(E, false, T0, NANG),
        'Ẹ' => vowel!(E, true, T0, NANG),
        // ê (circumflex)
        'ê' => vowel!(E, false, CIRCUMFLEX, M0),
        'Ê' => vowel!(E, true, CIRCUMFLEX, M0),
        'ế' => vowel!(E, false, CIRCUMFLEX, SAC),
        'Ế' => vowel!(E, true, CIRCUMFLEX, SAC),
        'ề' => vowel!(E, false, CIRCUMFLEX, HUYEN),
        'Ề' => vowel!(E, true, CIRCUMFLEX, HUYEN),
        'ể' => vowel!(E, false, CIRCUMFLEX, HOI),
        'Ể' => vowel!(E, true, CIRCUMFLEX, HOI),
        'ễ' => vowel!(E, false, CIRCUMFLEX, NGA),
        'Ễ' => vowel!(E, true, CIRCUMFLEX, NGA),
        'ệ' => vowel!(E, false, CIRCUMFLEX, NANG),
        'Ệ' => vowel!(E, true, CIRCUMFLEX, NANG),

        // ===== I variants =====
        'i' => vowel!(I, false, T0, M0),
        'I' => vowel!(I, true, T0, M0),
        'í' => vowel!(I, false, T0, SAC),
        'Í' => vowel!(I, true, T0, SAC),
        'ì' => vowel!(I, false, T0, HUYEN),
        'Ì' => vowel!(I, true, T0, HUYEN),
        'ỉ' => vowel!(I, false, T0, HOI),
        'Ỉ' => vowel!(I, true, T0, HOI),
        'ĩ' => vowel!(I, false, T0, NGA),
        'Ĩ' => vowel!(I, true, T0, NGA),
        'ị' => vowel!(I, false, T0, NANG),
        'Ị' => vowel!(I, true, T0, NANG),

        // ===== O variants =====
        'o' => vowel!(O, false, T0, M0),
        'O' => vowel!(O, true, T0, M0),
        'ó' => vowel!(O, false, T0, SAC),
        'Ó' => vowel!(O, true, T0, SAC),
        'ò' => vowel!(O, false, T0, HUYEN),
        'Ò' => vowel!(O, true, T0, HUYEN),
        'ỏ' => vowel!(O, false, T0, HOI),
        'Ỏ' => vowel!(O, true, T0, HOI),
        'õ' => vowel!(O, false, T0, NGA),
        'Õ' => vowel!(O, true, T0, NGA),
        'ọ' => vowel!(O, false, T0, NANG),
        'Ọ' => vowel!(O, true, T0, NANG),
        // ô (circumflex)
        'ô' => vowel!(O, false, CIRCUMFLEX, M0),
        'Ô' => vowel!(O, true, CIRCUMFLEX, M0),
        'ố' => vowel!(O, false, CIRCUMFLEX, SAC),
        'Ố' => vowel!(O, true, CIRCUMFLEX, SAC),
        'ồ' => vowel!(O, false, CIRCUMFLEX, HUYEN),
        'Ồ' => vowel!(O, true, CIRCUMFLEX, HUYEN),
        'ổ' => vowel!(O, false, CIRCUMFLEX, HOI),
        'Ổ' => vowel!(O, true, CIRCUMFLEX, HOI),
        'ỗ' => vowel!(O, false, CIRCUMFLEX, NGA),
        'Ỗ' => vowel!(O, true, CIRCUMFLEX, NGA),
        'ộ' => vowel!(O, false, CIRCUMFLEX, NANG),
        'Ộ' => vowel!(O, true, CIRCUMFLEX, NANG),
        // ơ (horn)
        'ơ' => vowel!(O, false, HORN, M0),
        'Ơ' => vowel!(O, true, HORN, M0),
        'ớ' => vowel!(O, false, HORN, SAC),
        'Ớ' => vowel!(O, true, HORN, SAC),
        'ờ' => vowel!(O, false, HORN, HUYEN),
        'Ờ' => vowel!(O, true, HORN, HUYEN),
        'ở' => vowel!(O, false, HORN, HOI),
        'Ở' => vowel!(O, true, HORN, HOI),
        'ỡ' => vowel!(O, false, HORN, NGA),
        'Ỡ' => vowel!(O, true, HORN, NGA),
        'ợ' => vowel!(O, false, HORN, NANG),
        'Ợ' => vowel!(O, true, HORN, NANG),

        // ===== U variants =====
        'u' => vowel!(U, false, T0, M0),
        'U' => vowel!(U, true, T0, M0),
        'ú' => vowel!(U, false, T0, SAC),
        'Ú' => vowel!(U, true, T0, SAC),
        'ù' => vowel!(U, false, T0, HUYEN),
        'Ù' => vowel!(U, true, T0, HUYEN),
        'ủ' => vowel!(U, false, T0, HOI),
        'Ủ' => vowel!(U, true, T0, HOI),
        'ũ' => vowel!(U, false, T0, NGA),
        'Ũ' => vowel!(U, true, T0, NGA),
        'ụ' => vowel!(U, false, T0, NANG),
        'Ụ' => vowel!(U, true, T0, NANG),
        // ư (horn)
        'ư' => vowel!(U, false, HORN, M0),
        'Ư' => vowel!(U, true, HORN, M0),
        'ứ' => vowel!(U, false, HORN, SAC),
        'Ứ' => vowel!(U, true, HORN, SAC),
        'ừ' => vowel!(U, false, HORN, HUYEN),
        'Ừ' => vowel!(U, true, HORN, HUYEN),
        'ử' => vowel!(U, false, HORN, HOI),
        'Ử' => vowel!(U, true, HORN, HOI),
        'ữ' => vowel!(U, false, HORN, NGA),
        'Ữ' => vowel!(U, true, HORN, NGA),
        'ự' => vowel!(U, false, HORN, NANG),
        'Ự' => vowel!(U, true, HORN, NANG),

        // ===== Y variants =====
        'y' => vowel!(Y, false, T0, M0),
        'Y' => vowel!(Y, true, T0, M0),
        'ý' => vowel!(Y, false, T0, SAC),
        'Ý' => vowel!(Y, true, T0, SAC),
        'ỳ' => vowel!(Y, false, T0, HUYEN),
        'Ỳ' => vowel!(Y, true, T0, HUYEN),
        'ỷ' => vowel!(Y, false, T0, HOI),
        'Ỷ' => vowel!(Y, true, T0, HOI),
        'ỹ' => vowel!(Y, false, T0, NGA),
        'Ỹ' => vowel!(Y, true, T0, NGA),
        'ỵ' => vowel!(Y, false, T0, NANG),
        'Ỵ' => vowel!(Y, true, T0, NANG),

        // ===== Consonants =====
        'đ' => Some(ParsedChar::stroke(D, false)),
        'Đ' => Some(ParsedChar::stroke(D, true)),
        'd' => Some(ParsedChar::new(D, false, 0, 0)),
        'D' => Some(ParsedChar::new(D, true, 0, 0)),
        'b' => Some(ParsedChar::new(B, false, 0, 0)),
        'B' => Some(ParsedChar::new(B, true, 0, 0)),
        'c' => Some(ParsedChar::new(C, false, 0, 0)),
        'C' => Some(ParsedChar::new(C, true, 0, 0)),
        'f' => Some(ParsedChar::new(F, false, 0, 0)),
        'F' => Some(ParsedChar::new(F, true, 0, 0)),
        'g' => Some(ParsedChar::new(G, false, 0, 0)),
        'G' => Some(ParsedChar::new(G, true, 0, 0)),
        'h' => Some(ParsedChar::new(H, false, 0, 0)),
        'H' => Some(ParsedChar::new(H, true, 0, 0)),
        'j' => Some(ParsedChar::new(J, false, 0, 0)),
        'J' => Some(ParsedChar::new(J, true, 0, 0)),
        'k' => Some(ParsedChar::new(K, false, 0, 0)),
        'K' => Some(ParsedChar::new(K, true, 0, 0)),
        'l' => Some(ParsedChar::new(L, false, 0, 0)),
        'L' => Some(ParsedChar::new(L, true, 0, 0)),
        'm' => Some(ParsedChar::new(M, false, 0, 0)),
        'M' => Some(ParsedChar::new(M, true, 0, 0)),
        'n' => Some(ParsedChar::new(N, false, 0, 0)),
        'N' => Some(ParsedChar::new(N, true, 0, 0)),
        'p' => Some(ParsedChar::new(P, false, 0, 0)),
        'P' => Some(ParsedChar::new(P, true, 0, 0)),
        'q' => Some(ParsedChar::new(Q, false, 0, 0)),
        'Q' => Some(ParsedChar::new(Q, true, 0, 0)),
        'r' => Some(ParsedChar::new(R, false, 0, 0)),
        'R' => Some(ParsedChar::new(R, true, 0, 0)),
        's' => Some(ParsedChar::new(S, false, 0, 0)),
        'S' => Some(ParsedChar::new(S, true, 0, 0)),
        't' => Some(ParsedChar::new(T, false, 0, 0)),
        'T' => Some(ParsedChar::new(T, true, 0, 0)),
        'v' => Some(ParsedChar::new(V, false, 0, 0)),
        'V' => Some(ParsedChar::new(V, true, 0, 0)),
        'w' => Some(ParsedChar::new(W, false, 0, 0)),
        'W' => Some(ParsedChar::new(W, true, 0, 0)),
        'x' => Some(ParsedChar::new(X, false, 0, 0)),
        'X' => Some(ParsedChar::new(X, true, 0, 0)),
        'z' => Some(ParsedChar::new(Z, false, 0, 0)),
        'Z' => Some(ParsedChar::new(Z, true, 0, 0)),

        _ => None,
    }
}
