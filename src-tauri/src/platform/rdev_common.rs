//! Common rdev key mapping utilities shared between Linux and macOS.
//!
//! Both platforms use the `rdev` crate for keyboard hooking, and share
//! identical key mapping logic.

#![cfg(any(target_os = "linux", target_os = "macos"))]

use crate::platform::keyboard_hook::Key;

/// Converts an rdev key to our internal Key representation.
pub(crate) fn rdev_key_to_key(rdev_key: &rdev::Key) -> Key {
    use rdev::Key as RK;
    match rdev_key {
        RK::BackSpace => Key::Backspace,
        RK::Return => Key::Enter,
        RK::Tab => Key::Tab,
        RK::Escape => Key::Escape,
        RK::Space => Key::Space,
        RK::Delete => Key::Delete,
        RK::LeftArrow => Key::Left,
        RK::RightArrow => Key::Right,
        RK::UpArrow => Key::Up,
        RK::DownArrow => Key::Down,
        RK::Home => Key::Home,
        RK::End => Key::End,
        RK::PageUp => Key::PageUp,
        RK::PageDown => Key::PageDown,
        RK::F1 => Key::F(1),
        RK::F2 => Key::F(2),
        RK::F3 => Key::F(3),
        RK::F4 => Key::F(4),
        RK::F5 => Key::F(5),
        RK::F6 => Key::F(6),
        RK::F7 => Key::F(7),
        RK::F8 => Key::F(8),
        RK::F9 => Key::F(9),
        RK::F10 => Key::F(10),
        RK::F11 => Key::F(11),
        RK::F12 => Key::F(12),
        RK::KeyA => Key::Char('a'),
        RK::KeyB => Key::Char('b'),
        RK::KeyC => Key::Char('c'),
        RK::KeyD => Key::Char('d'),
        RK::KeyE => Key::Char('e'),
        RK::KeyF => Key::Char('f'),
        RK::KeyG => Key::Char('g'),
        RK::KeyH => Key::Char('h'),
        RK::KeyI => Key::Char('i'),
        RK::KeyJ => Key::Char('j'),
        RK::KeyK => Key::Char('k'),
        RK::KeyL => Key::Char('l'),
        RK::KeyM => Key::Char('m'),
        RK::KeyN => Key::Char('n'),
        RK::KeyO => Key::Char('o'),
        RK::KeyP => Key::Char('p'),
        RK::KeyQ => Key::Char('q'),
        RK::KeyR => Key::Char('r'),
        RK::KeyS => Key::Char('s'),
        RK::KeyT => Key::Char('t'),
        RK::KeyU => Key::Char('u'),
        RK::KeyV => Key::Char('v'),
        RK::KeyW => Key::Char('w'),
        RK::KeyX => Key::Char('x'),
        RK::KeyY => Key::Char('y'),
        RK::KeyZ => Key::Char('z'),
        RK::Num0 => Key::Char('0'),
        RK::Num1 => Key::Char('1'),
        RK::Num2 => Key::Char('2'),
        RK::Num3 => Key::Char('3'),
        RK::Num4 => Key::Char('4'),
        RK::Num5 => Key::Char('5'),
        RK::Num6 => Key::Char('6'),
        RK::Num7 => Key::Char('7'),
        RK::Num8 => Key::Char('8'),
        RK::Num9 => Key::Char('9'),
        other => Key::Other(format!("{:?}", other)),
    }
}

/// Track modifier state from rdev events.
pub(crate) fn is_modifier(key: &rdev::Key) -> bool {
    matches!(
        key,
        rdev::Key::ShiftLeft
            | rdev::Key::ShiftRight
            | rdev::Key::ControlLeft
            | rdev::Key::ControlRight
            | rdev::Key::Alt
            | rdev::Key::AltGr
            | rdev::Key::MetaLeft
            | rdev::Key::MetaRight
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdev_key_mapping_chars() {
        assert_eq!(rdev_key_to_key(&rdev::Key::KeyA), Key::Char('a'));
        assert_eq!(rdev_key_to_key(&rdev::Key::Num5), Key::Char('5'));
    }

    #[test]
    fn test_rdev_key_mapping_special() {
        assert_eq!(rdev_key_to_key(&rdev::Key::BackSpace), Key::Backspace);
        assert_eq!(rdev_key_to_key(&rdev::Key::Return), Key::Enter);
        assert_eq!(rdev_key_to_key(&rdev::Key::F1), Key::F(1));
    }

    #[test]
    fn test_is_modifier() {
        assert!(is_modifier(&rdev::Key::ShiftLeft));
        assert!(is_modifier(&rdev::Key::ControlRight));
        assert!(!is_modifier(&rdev::Key::KeyA));
    }
}
