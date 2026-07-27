//! External compile coverage for the shared Radiant GUI adoption surface.

#![cfg(all(feature = "radiant-gui", target_os = "macos"))]

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
fn adopted_radiant_surface_remains_publicly_constructible() {
    let _knob = radiant::application::knob(0.5)
        .automation_active(true)
        .message(|_: radiant::widgets::KnobMessage| ());
    let _icon = radiant::gui::svg::IconName::ChevronDown;
}
