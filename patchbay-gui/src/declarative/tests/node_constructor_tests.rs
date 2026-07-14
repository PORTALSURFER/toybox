use super::super::*;

#[test]
fn node_fluent_helpers_apply_container_and_style_fields() {
    let panel_node = panel("main", textbox("x"))
        .title("Main")
        .pad_all(14)
        .background(Color::rgb(12, 16, 22))
        .outline(Color::rgb(44, 52, 68));
    match panel_node {
        Node::Panel(panel) => {
            assert_eq!(panel.title.as_deref(), Some("Main"));
            assert_eq!(panel.padding, 14);
            assert_eq!(panel.background, Some(Color::rgb(12, 16, 22)));
            assert_eq!(panel.outline, Some(Color::rgb(44, 52, 68)));
        }
        _ => panic!("expected panel node"),
    }

    let row_node = row(vec![textbox("a"), textbox("b")])
        .pad_xy(10, 8)
        .align_center()
        .justify_space_between();
    match row_node {
        Node::Row(flex) => {
            assert_eq!(flex.padding, EdgeInsets::symmetric(10, 8));
            assert_eq!(flex.align, Align::Center);
            assert_eq!(flex.justify, Justify::SpaceBetween);
        }
        _ => panic!("expected row node"),
    }

    let grid_node = grid(
        GridTemplate::columns_fr(2),
        vec![spacer(Size {
            width: 8,
            height: 8,
        })],
    )
    .pad_all(5);
    match grid_node {
        Node::Grid(grid) => {
            assert_eq!(grid.template.padding, EdgeInsets::all(5));
        }
        _ => panic!("expected grid node"),
    }

    let text_box_node = textbox("name")
        .text_color(Color::rgb(200, 180, 90))
        .text_align_center();
    match text_box_node {
        Node::TextBox(text_box) => {
            assert_eq!(text_box.color, Some(Color::rgb(200, 180, 90)));
            assert_eq!(text_box.align, TextBoxAlign::Center);
        }
        _ => panic!("expected text box node"),
    }

    let editable_text_box = textbox("Init")
        .text_editable("preset-title", true)
        .text_edit_max_chars(24);
    match editable_text_box {
        Node::TextBox(text_box) => {
            let edit = text_box.edit.expect("editable contract should exist");
            assert_eq!(edit.key, "preset-title");
            assert!(edit.editing);
            assert_eq!(edit.max_chars, 24);
        }
        _ => panic!("expected text box node"),
    }

    let knob_node = knob("k", 0.5, (0.0, 1.0));
    match knob_node {
        Node::Knob(knob) => {
            assert_eq!(knob.key, "k");
            assert_eq!(knob.value, 0.5);
            assert_eq!(knob.range, (0.0, 1.0));
            assert_eq!(knob.color_role, None);
            assert!(!knob.disabled);
            assert!(!knob.focused);
        }
        _ => panic!("expected knob node"),
    }

    let slider_node = slider("mix", 0.3, (0.0, 1.0)).control_size(Size {
        width: 140,
        height: 24,
    });
    match slider_node {
        Node::Slider(slider) => assert_eq!(
            slider.control_size,
            Some(Size {
                width: 140,
                height: 24
            })
        ),
        _ => panic!("expected slider node"),
    }

    let button_node = button("ping")
        .button_label("Ping")
        .control_size(Size {
            width: 88,
            height: 24,
        });
    match button_node {
        Node::Button(button) => {
            assert_eq!(button.label.as_deref(), Some("Ping"));
            assert_eq!(
                button.control_size,
                Some(Size {
                    width: 88,
                    height: 24
                })
            );
            assert_eq!(button.color_role, None);
            assert!(!button.disabled);
            assert!(!button.focused);
        }
        _ => panic!("expected button node"),
    }

    let dropdown_node = dropdown(
        "mode",
        3,
        0,
    )
    .selected(2)
    .control_size(Size {
        width: 160,
        height: 24,
    });
    match dropdown_node {
        Node::Dropdown(dropdown) => {
            assert_eq!(dropdown.selected, 2);
            assert_eq!(
                dropdown.control_size,
                Some(Size {
                    width: 160,
                    height: 24
                })
            );
        }
        _ => panic!("expected dropdown node"),
    }

    let tab_bar_node = tabbar("sound", 2, 0)
        .tab_labels(vec!["Kick".into(), "Ride".into()])
        .control_size(Size {
            width: 180,
            height: 28,
        })
        .color_role(WidgetColorRole::Accent(AccentKey::Entity(11)))
        .disabled(true)
        .focused(true);
    match tab_bar_node {
        Node::TabBar(tab_bar) => {
            assert_eq!(tab_bar.tab_count, 2);
            assert_eq!(tab_bar.selected, 0);
            assert_eq!(
                tab_bar.tab_labels,
                Some(vec!["Kick".to_string(), "Ride".to_string()])
            );
            assert_eq!(
                tab_bar.control_size,
                Some(Size {
                    width: 180,
                    height: 28
                })
            );
            assert_eq!(
                tab_bar.color_role,
                Some(WidgetColorRole::Accent(AccentKey::Entity(11)))
            );
            assert!(tab_bar.disabled);
            assert!(tab_bar.focused);
        }
        _ => panic!("expected tab bar node"),
    }

    let focused_toggle = toggle("sync", true)
        .color_role(WidgetColorRole::Accent(AccentKey::Entity(7)))
        .disabled(true)
        .focused(true);
    match focused_toggle {
        Node::Toggle(toggle) => {
            assert_eq!(
                toggle.color_role,
                Some(WidgetColorRole::Accent(AccentKey::Entity(7)))
            );
            assert!(toggle.disabled);
            assert!(toggle.focused);
        }
        _ => panic!("expected toggle node"),
    }
}

#[test]
fn curve_segment_move_decorator_preserves_following_curve_builders() {
    let options = CurveSegmentMoveOptions::new(
        CurveEditorModifier::Command,
        Color::rgb(4, 5, 6),
    );
    let style = CurveEditorStyle {
        line: Color::rgb(7, 8, 9),
        ..CurveEditorStyle::default()
    };
    let model = CurveModel::new(
        vec![CurvePoint::new(0.0, 0.0), CurvePoint::new(1.0, 1.0)],
        vec![CurveSegment::new(0.0)],
    );
    let node = curve_editor("curve", model)
        .curve_segment_move(options)
        .curve_style(style.clone())
        .fill();

    let Node::Slot(slot) = node else {
        panic!("segment-move opt-in should use the existing opaque slot wrapper");
    };
    assert_eq!(slot.curve_segment_move(), Some(options));
    let Node::CurveEditor(curve_editor) = slot.child() else {
        panic!("segment-move slot should preserve the curve-editor child");
    };
    assert_eq!(curve_editor.style, style);
    assert_eq!(curve_editor.layout, LayoutBox::fill());
}

#[test]
fn curve_point_constraint_composes_with_segment_move_and_following_builders() {
    let segment_move = CurveSegmentMoveOptions::new(
        CurveEditorModifier::Command,
        Color::rgb(4, 5, 6),
    );
    let model = CurveModel::new(
        vec![CurvePoint::new(0.0, 0.0), CurvePoint::new(1.0, 1.0)],
        vec![CurveSegment::new(0.0)],
    );
    let node = curve_editor("curve", model)
        .curve_point_horizontal_constraint(CurveEditorModifier::Shift)
        .curve_segment_move(segment_move)
        .curve_grid(CurveGridConfig {
            emphasized_verticals: vec![0.5],
        })
        .fill();

    let Node::Slot(slot) = node else {
        panic!("curve interaction opt-ins should share the opaque slot wrapper");
    };
    assert_eq!(slot.curve_segment_move(), Some(segment_move));
    assert_eq!(
        slot.curve_point_horizontal_constraint(),
        Some(CurveEditorModifier::Shift)
    );
    let Node::CurveEditor(curve_editor) = slot.child() else {
        panic!("decorated slot should preserve the curve-editor child");
    };
    assert_eq!(curve_editor.grid.emphasized_verticals, vec![0.5]);
    assert_eq!(curve_editor.layout, LayoutBox::fill());
}

#[test]
fn helper_node_constructors_build_valid_spec() {
    let controls = row(vec![
        knob("drive", 0.5, (0.0, 1.0)),
        slider("mix", 0.25, (0.0, 1.0)),
        toggle("sync", false),
        button("ping"),
        dropdown("mode", 2, 1),
        tabbar("family", 2, 0),
    ]);
    let content = column(vec![
        textbox("Header"),
        controls,
        grid(
            GridTemplate::columns_fr(2).rows_fr(1).pad_all(4),
            vec![
                spacer(Size {
                    width: 8,
                    height: 8,
                }),
                indicator(
                    Size {
                        width: 8,
                        height: 8,
                    },
                    true,
                ),
            ],
        ),
        switch_layout(
            vec![
                when_width_lt(480, panel("compact-mode", textbox("Compact"))),
                when_width_ge(480, panel("wide-mode", textbox("Wide"))),
            ],
            panel("fallback-mode", textbox("Fallback")),
        ),
        region(
            "plot",
            Size {
                width: 120,
                height: 40,
            },
        ),
    ]);
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel("main", content).fill(),
    ));
    let measured = measure_checked(&spec).expect("helper-composed tree should validate");
    assert!(measured.width > 0);
    assert!(measured.height > 0);
}

#[test]
fn measure_knob_matches_shared_block_metrics() {
    let mut tokens = ThemeTokens::default();
    tokens.controls.knob_diameter = 90;
    tokens.typography.text_scale = 3;

    let knob = KnobSpec::new("k", 0.5, (0.0, 1.0));
    let measured = measure_knob(&knob, &tokens);
    let expected = knob_block_size_for_diameter(90, 3);

    assert_eq!(measured, expected);
}

#[test]
fn measure_knob_uses_theme_text_scale() {
    let mut tokens = ThemeTokens::default();
    tokens.controls.knob_diameter = 90;
    tokens.typography.text_scale = 1;

    let knob = KnobSpec::new("k", 0.5, (0.0, 1.0));
    let measured = measure_knob(&knob, &tokens);
    let expected = knob_block_size_for_diameter(90, 1);

    assert_eq!(measured, expected);
}

#[test]
fn measure_knob_width_tracks_dial_hit_width_for_tight_tiling() {
    let mut tokens = ThemeTokens::default();
    tokens.controls.knob_diameter = 48;
    tokens.typography.text_scale = 3;

    let knob = KnobSpec::new("k", 0.5, (0.0, 1.0));
    let measured = measure_knob(&knob, &tokens);
    let expected = knob_block_size_for_diameter(48, 3);

    assert_eq!(measured.width, expected.width);
}

#[test]
fn knob_constructor_defaults_to_auto_width_layout() {
    let knob = KnobSpec::new("k", 0.5, (0.0, 1.0));

    assert_eq!(knob.layout.width, Length::Auto);
}

#[test]
fn measure_tab_bar_is_deterministic_with_or_without_labels() {
    let tokens = ThemeTokens::default();
    let without_labels = TabBarSpec::new("family", 3, 0);
    let with_labels = TabBarSpec::new("family", 3, 0).tab_labels(vec![
        "Kick".into(),
        "Ride".into(),
        "Snare".into(),
    ]);

    assert_eq!(
        measure_tab_bar(&without_labels, &tokens),
        measure_tab_bar(&with_labels, &tokens)
    );
}

#[test]
fn measure_tab_bar_uses_explicit_control_size() {
    let tokens = ThemeTokens::default();
    let tab_bar = TabBarSpec::new("family", 2, 0).control_size(Size {
        width: 222,
        height: 31,
    });

    assert_eq!(
        measure_tab_bar(&tab_bar, &tokens),
        Size {
            width: 222,
            height: 31
        }
    );
}

#[test]
fn eq_attractor_surface_constructor_sets_key_model_and_style() {
    let model = EqAttractorSurfaceModel::new(vec![EqAttractor::new(9, 0.25, 0.75)]);
    let style = EqAttractorSurfaceStyle::default();
    let node = eq_attractor_surface("eq", model.clone(), style);

    match node {
        Node::EqAttractorSurface(surface) => {
            assert_eq!(surface.key, "eq");
            assert_eq!(surface.model, model);
            assert_eq!(surface.style, style);
        }
        other => panic!("unexpected node: {other:?}"),
    }
}
