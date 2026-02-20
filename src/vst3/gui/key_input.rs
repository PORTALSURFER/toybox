/// Translate one VST3 key-down payload into a patchbay input character.
///
/// VST3 delivers both a Unicode `key` and an optional virtual `key_code`.
/// This helper prioritizes control/navigation virtual keys so text editing can
/// handle arrows/home/end/delete consistently across hosts.
pub fn vst3_key_down_to_input_char(key: u16, key_code: i16) -> Option<char> {
    use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::{
        KEY_BACK, KEY_DELETE, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT, KEY_RETURN,
        KEY_RIGHT, KEY_SPACE, KEY_TAB,
    };

    let key_code = key_code as u32;
    match key_code {
        KEY_BACK => return Some('\u{8}'),
        KEY_DELETE => return Some('\u{7f}'),
        KEY_ENTER | KEY_RETURN => return Some('\r'),
        KEY_ESCAPE => return Some('\u{1b}'),
        KEY_TAB => return Some('\t'),
        KEY_SPACE => return Some(' '),
        KEY_LEFT => return Some('\u{1c}'),
        KEY_RIGHT => return Some('\u{1d}'),
        KEY_HOME => return Some('\u{1e}'),
        KEY_END => return Some('\u{1f}'),
        _ => {}
    }

    let code = key as u32;
    if code != 0 {
        return char::from_u32(code);
    }

    match key_code {
        value @ 0x21..=0x7e => char::from_u32(value),
        _ => None,
    }
}
