//! Helpers for VST3 factory and component metadata.

use std::ffi::{CString, c_char};

use crate::vst3::gui::copy_wstring;
use toybox_vst3_ffi::Steinberg::Vst::TChar;
use toybox_vst3_ffi::Steinberg::{PClassInfo, PClassInfo_::ClassCardinality_, TUID, int32};

/// VST3 category label used for audio processing components.
pub const CATEGORY_AUDIO_MODULE_CLASS: &str = "Audio Module Class";
/// VST3 category label used for edit controller components.
pub const CATEGORY_COMPONENT_CONTROLLER_CLASS: &str = "Component Controller Class";

/// Copy a UTF-8 Rust string into a fixed C string buffer.
///
/// The destination is always null terminated when non-empty.
pub fn copy_cstring(source: &str, destination: &mut [c_char]) {
    let c_string = CString::new(source).unwrap_or_default();
    let bytes = c_string.as_bytes_with_nul();

    for (src, dst) in bytes.iter().zip(destination.iter_mut()) {
        *dst = *src as c_char;
    }

    if bytes.len() > destination.len() && let Some(last) = destination.last_mut() {
        *last = 0;
    }
}

/// Fill a `PClassInfo` entry with common class metadata.
pub fn write_class_info(
    info: &mut PClassInfo,
    class_id: TUID,
    category: &str,
    class_name: &str,
    cardinality: int32,
) {
    info.cid = class_id;
    info.cardinality = cardinality;
    copy_cstring(category, &mut info.category);
    copy_cstring(class_name, &mut info.name);
}

/// Fill a `PClassInfo` entry for classes that allow many instances.
pub fn write_class_info_many(
    info: &mut PClassInfo,
    class_id: TUID,
    category: &str,
    class_name: &str,
) {
    write_class_info(
        info,
        class_id,
        category,
        class_name,
        ClassCardinality_::kManyInstances as int32,
    );
}

/// Fill a UTF-16 `TChar` name field used by bus and parameter info objects.
pub fn write_wide_name(name: &str, destination: &mut [TChar]) {
    copy_wstring(name, destination);
}
