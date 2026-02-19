use super::super::*;

#[test]
fn fixed_root_layout_expands_to_intrinsic_content() {
    let spec = UiSpec::new(
        RootFrameSpec::new("root", panel("panel", textbox("VeryWideLabel")).pad_all(0))
            .padding(0)
            .layout(LayoutBox::fixed(1, 1)),
    );
    let measured = measure_checked(&spec).expect("measurement should succeed");
    let intrinsic = text_size(
        "VeryWideLabel",
        ThemeTokens::default().typography.text_scale,
    );
    assert_eq!(measured, intrinsic);
}

#[test]
fn auto_panel_layout_expands_to_intrinsic_content() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            panel("panel", textbox("WidePanelText")).pad_all(0),
        )
        .padding(0),
    );
    let measured = measure_checked(&spec).expect("measurement should succeed");
    let intrinsic = text_size(
        "WidePanelText",
        ThemeTokens::default().typography.text_scale,
    );
    assert_eq!(measured, intrinsic);
}

#[test]
fn explicit_max_still_caps_fixed_pixel_layout() {
    let spec = UiSpec::new(
        RootFrameSpec::new("root", panel("panel", textbox("VeryWideLabel")).pad_all(0))
            .padding(0)
            .layout(LayoutBox::fixed(1, 1).max(12, 12)),
    );
    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured.width, 12);
    assert_eq!(measured.height, 12);
}

#[test]
fn auto_absolute_layout_expands_to_positioned_child_bounds() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Absolute(
                AbsoluteSpec::new(vec![AbsoluteChild::new(
                    Point { x: 40, y: 30 },
                    spacer(Size {
                        width: 15,
                        height: 11,
                    }),
                )]),
            ),
        )
        .padding(0),
    );
    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(
        measured,
        Size {
            width: 55,
            height: 41,
        }
    );
}

#[test]
fn default_control_tokens_use_half_knob_diameter() {
    assert_eq!(ThemeTokens::default().controls.knob_diameter, 32);
}

#[test]
fn default_color_tokens_use_main_palette() {
    let palette = MainPalette::main();
    let tokens = ColorTokens::default();
    assert_eq!(tokens.background, palette.background_primary);
    assert_eq!(tokens.surface, palette.background_secondary);
    assert_eq!(tokens.border, palette.ui_secondary);
    assert_eq!(tokens.text, palette.text_primary);
    assert_eq!(tokens.accent, palette.accent_focus);
}

#[test]
fn theme_tokens_from_palette_uses_palette_for_color_roles() {
    let palette = MainPalette::main();
    let tokens = ThemeTokens::from_palette(palette);
    assert_eq!(tokens.colors.background, palette.background_primary);
    assert_eq!(tokens.colors.surface, palette.background_secondary);
    assert_eq!(tokens.colors.border, palette.ui_secondary);
    assert_eq!(tokens.colors.text, palette.text_primary);
    assert_eq!(tokens.colors.accent, palette.accent_focus);
}

#[test]
fn label_with_explicit_box_does_not_expand_root_width() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            panel(
                "panel",
                textbox("VERY LONG LABEL THAT MUST NOT WIDEN THE WINDOW")
                    .widget_layout(LayoutBox::fixed(64, 16).max(64, 16)),
            )
            .pad_all(0),
        )
        .padding(0),
    );
    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(
        measured,
        Size {
            width: 64,
            height: 16,
        }
    );
}

#[test]
fn container_layout_validator_rejects_absolute_constraints() {
    let error = validate_container_layout("Panel", LayoutBox::fixed(10, 10))
        .expect_err("container absolute constraints must hard-fail");
    assert!(matches!(
        error,
        DeclarativeError::InvalidContainerLayout { container_kind } if container_kind == "Panel"
    ));
}

#[test]
fn container_layout_validator_rejects_inverted_bounds() {
    let error = validate_container_layout("Panel", LayoutBox::auto().min(20, 10).max(8, 10))
        .expect_err("container inverted bounds must hard-fail");
    assert!(matches!(
        error,
        DeclarativeError::InvalidLayoutBounds {
            node_kind,
            axis,
            min,
            max
        } if node_kind == "Panel" && axis == "width" && min == 20 && max == 8
    ));
}

#[test]
fn text_size_saturates_huge_scale_without_overflow() {
    let measured = text_size("scale", u32::MAX);
    assert_eq!(
        measured,
        Size {
            width: u32::MAX,
            height: u32::MAX,
        }
    );
}
