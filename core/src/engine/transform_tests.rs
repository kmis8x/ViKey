use super::super::buffer::Char;
use super::*;
use crate::data::chars::{mark, tone};
use crate::data::keys;

fn setup_buffer(s: &str) -> Buffer {
    let mut buf = Buffer::new();
    for ch in s.chars() {
        let key = match ch.to_ascii_lowercase() {
            'a' => keys::A,
            'b' => keys::B,
            'c' => keys::C,
            'd' => keys::D,
            'e' => keys::E,
            'g' => keys::G,
            'h' => keys::H,
            'i' => keys::I,
            'n' => keys::N,
            'o' => keys::O,
            'u' => keys::U,
            _ => continue,
        };
        buf.push(Char::new(key, ch.is_uppercase()));
    }
    buf
}

#[test]
fn test_apply_stroke() {
    let mut buf = setup_buffer("do");
    let result = apply_stroke(&mut buf);
    assert!(result.applied);
    assert!(buf.get(0).unwrap().stroke);
}

#[test]
fn test_apply_stroke_anywhere() {
    // "dod" should stroke the first 'd'
    let mut buf = setup_buffer("dod");
    let result = apply_stroke(&mut buf);
    assert!(result.applied);
    assert!(buf.get(0).unwrap().stroke); // First d is stroked
}

#[test]
fn test_apply_mark() {
    let mut buf = setup_buffer("an");
    let result = apply_mark(&mut buf, mark::SAC, true);
    assert!(result.applied);
    assert_eq!(buf.get(0).unwrap().mark, mark::SAC);
}

#[test]
fn test_uo_compound() {
    let mut buf = setup_buffer("duoc");
    let result = apply_tone(&mut buf, keys::W, tone::HORN, 0);
    assert!(result.applied);
    // Both u and o should have horn
    assert_eq!(buf.get(1).unwrap().tone, tone::HORN); // u
    assert_eq!(buf.get(2).unwrap().tone, tone::HORN); // o
}
