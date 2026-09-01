//! External compile coverage for the shared Radiant GUI adoption surface.

#![cfg(all(
    feature = "radiant-gui",
    any(target_os = "macos", target_os = "windows")
))]

#[test]
fn bundled_text_options_exports_stable_face_order() {
    let options = toybox::radiant_gui::bundled_text_options();

    assert_eq!(options.font_paths, Vec::<std::path::PathBuf>::new());
    assert_eq!(options.embedded_fonts.len(), 2);
    assert_eq!(
        options.embedded_fonts[0].bytes(),
        include_bytes!("../assets/IoskeleyMono/IoskeleyMono-Regular.ttf")
    );
    assert_eq!(
        options.embedded_fonts[1].bytes(),
        include_bytes!("../assets/Sometype_Mono/static/SometypeMono-Regular.ttf")
    );
}

#[test]
fn bundled_offscreen_capture_exports_constructible_public_signature() {
    let _: fn(
        radiant::gui::types::Vector2,
        radiant::theme::DpiScale,
    ) -> Result<
        radiant::gui_runtime::OffscreenVelloCapture,
        radiant::gui_runtime::EmbeddedVelloError,
    > = toybox::radiant_gui::bundled_offscreen_capture;
}

#[test]
fn bundled_faces_are_complementary_for_documented_private_use_scalar() {
    // U+E0FF is deliberately covered by Sometype Mono, but not Ioskeley Mono.
    // Keep this assertion against the actual embedded TTF charmaps so a future
    // asset replacement cannot silently remove the fallback coverage.
    const COMPLEMENTARY_SCALAR: u32 = 0xE0FF;
    let primary = include_bytes!("../assets/IoskeleyMono/IoskeleyMono-Regular.ttf");
    let fallback = include_bytes!("../assets/Sometype_Mono/static/SometypeMono-Regular.ttf");

    assert!(!ttf_cmap_contains(primary, COMPLEMENTARY_SCALAR));
    assert!(ttf_cmap_contains(fallback, COMPLEMENTARY_SCALAR));
}

#[test]
fn adopted_radiant_surface_remains_publicly_constructible() {
    let _knob = radiant::application::knob(0.5)
        .automation_active(true)
        .message(|_: radiant::widgets::KnobMessage| ());
    let _icon = radiant::gui::svg::IconName::ChevronDown;
}

fn ttf_cmap_contains(font: &[u8], scalar: u32) -> bool {
    let Some(num_tables) = be_u16(font, 4) else {
        return false;
    };
    let mut cmap_offset = None;
    for table in 0..usize::from(num_tables) {
        let record = 12 + table * 16;
        if font.get(record..record + 4) == Some(b"cmap") {
            cmap_offset = be_u32(font, record + 8).map(|offset| offset as usize);
            break;
        }
    }
    let Some(cmap_offset) = cmap_offset else {
        return false;
    };
    let Some(num_subtables) = be_u16(font, cmap_offset + 2) else {
        return false;
    };

    for subtable in 0..usize::from(num_subtables) {
        let record = cmap_offset + 4 + subtable * 8;
        let Some(offset) = be_u32(font, record + 4) else {
            continue;
        };
        let offset = cmap_offset + offset as usize;
        match be_u16(font, offset) {
            Some(4) if scalar <= u32::from(u16::MAX) => {
                let Some(seg_count_x2) = be_u16(font, offset + 6) else {
                    continue;
                };
                let seg_count = usize::from(seg_count_x2 / 2);
                let start_codes = offset + 14 + (seg_count * 2) + 2;
                let end_codes = offset + 14;
                for segment in 0..seg_count {
                    let Some(start) = be_u16(font, start_codes + segment * 2) else {
                        continue;
                    };
                    let Some(end) = be_u16(font, end_codes + segment * 2) else {
                        continue;
                    };
                    if end != u16::MAX && u32::from(start) <= scalar && scalar <= u32::from(end) {
                        return true;
                    }
                }
            }
            Some(12) => {
                let Some(group_count) = be_u32(font, offset + 12) else {
                    continue;
                };
                for group in 0..group_count as usize {
                    let group_offset = offset + 16 + group * 12;
                    let Some(start) = be_u32(font, group_offset) else {
                        continue;
                    };
                    let Some(end) = be_u32(font, group_offset + 4) else {
                        continue;
                    };
                    if start <= scalar && scalar <= end {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
}

fn be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ]))
}
