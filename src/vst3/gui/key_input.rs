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

    // Virtual key constants differ between generated targets (`u32` vs `i32`).
    // Compare through a neutral integer type so this code stays portable.
    let key_code = i64::from(key_code);
    if key_code == KEY_BACK as i64 {
        return Some('\u{8}');
    }
    if key_code == KEY_DELETE as i64 {
        return Some('\u{7f}');
    }
    if key_code == KEY_ENTER as i64 || key_code == KEY_RETURN as i64 {
        return Some('\r');
    }
    if key_code == KEY_ESCAPE as i64 {
        return Some('\u{1b}');
    }
    if key_code == KEY_TAB as i64 {
        return Some('\t');
    }
    if key_code == KEY_SPACE as i64 {
        return Some(' ');
    }
    if key_code == KEY_LEFT as i64 {
        return Some('\u{1c}');
    }
    if key_code == KEY_RIGHT as i64 {
        return Some('\u{1d}');
    }
    if key_code == KEY_HOME as i64 {
        return Some('\u{1e}');
    }
    if key_code == KEY_END as i64 {
        return Some('\u{1f}');
    }

    let code = key as u32;
    if code != 0 {
        return char::from_u32(code);
    }

    match key_code {
        value @ 0x21..=0x7e => char::from_u32(value as u32),
        _ => None,
    }
}
