/// Copy a UTF-8 Rust string into a fixed UTF-16 `TChar` destination buffer.
///
/// The destination is always null-terminated when non-empty.
pub fn copy_wstring(source: &str, destination: &mut [TChar]) {
    let mut written = 0usize;
    for (src, dst) in source.encode_utf16().zip(destination.iter_mut()) {
        *dst = src as TChar;
        written += 1;
    }

    if written < destination.len() {
        destination[written] = 0;
    } else if let Some(last) = destination.last_mut() {
        *last = 0;
    }
}

/// Compute the element length of a zero-terminated UTF-16 `TChar` string.
///
/// # Safety
///
/// `string` must point to a readable, zero-terminated `TChar` sequence.
pub unsafe fn tchar_len(string: *const TChar) -> usize {
    let mut len = 0;
    while unsafe { *string.add(len) } != 0 {
        len += 1;
    }
    len
}
