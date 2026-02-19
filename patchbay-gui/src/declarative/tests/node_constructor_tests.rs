use super::super::*;

#[test]
fn node_fluent_helpers_apply_container_and_style_fields() {
    let panel_node = panel("main", label("x"))
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

    let row_node = row(vec![label("a"), label("b")])
        .gap(6)
        .pad_xy(10, 8)
        .align_center()
        .justify_space_between();
    match row_node {
        Node::Row(flex) => {
            assert_eq!(flex.gap, 6);
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
    .gap_xy(3, 9)
    .pad_all(5);
    match grid_node {
        Node::Grid(grid) => {
            assert_eq!(grid.template.column_gap, 3);
            assert_eq!(grid.template.row_gap, 9);
            assert_eq!(grid.template.padding, EdgeInsets::all(5));
        }
        _ => panic!("expected grid node"),
    }

    let label_node = label("name").text_color(Color::rgb(200, 180, 90));
    match label_node {
        Node::Label(label) => assert_eq!(label.color, Some(Color::rgb(200, 180, 90))),
        _ => panic!("expected label node"),
    }

    let knob_node = knob("k", "Drive", 0.5, (0.0, 1.0))
        .value_label("50%")
        .text_scale(2);
    match knob_node {
        Node::Knob(knob) => {
            assert_eq!(knob.value_label.as_deref(), Some("50%"));
            assert_eq!(knob.text_scale, Some(2));
        }
        _ => panic!("expected knob node"),
    }

    let slider_node = slider("mix", "Mix", 0.3, (0.0, 1.0)).control_size(Size {
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

    let dropdown_node = dropdown(
        "mode",
        "Mode",
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
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
}

#[test]
fn helper_node_constructors_build_valid_spec() {
    let controls = row(vec![
        knob("drive", "Drive", 0.5, (0.0, 1.0)),
        slider("mix", "Mix", 0.25, (0.0, 1.0)),
        toggle("sync", "Sync", false),
        button("ping", "Ping"),
        dropdown("mode", "Mode", vec!["A".to_string(), "B".to_string()], 1),
    ]);
    let content = column(vec![
        label("Header"),
        controls,
        grid(
            GridTemplate::columns_fr(2).rows_fr(1).pad_all(4).gap(8),
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

    let knob = KnobSpec::new("k", "Drive", 0.5, (0.0, 1.0));
    let measured = measure_knob(&knob, &tokens);
    let expected = knob_block_size_for_diameter(90, 3);

    assert_eq!(measured, expected);
}

#[test]
fn measure_knob_uses_text_scale_override_when_present() {
    let mut tokens = ThemeTokens::default();
    tokens.controls.knob_diameter = 90;
    tokens.typography.text_scale = 3;

    let knob = KnobSpec::new("k", "Drive", 0.5, (0.0, 1.0)).text_scale(1);
    let measured = measure_knob(&knob, &tokens);
    let expected = knob_block_size_for_diameter(90, 1);

    assert_eq!(measured, expected);
}

#[test]
fn measure_knob_width_tracks_dial_hit_width_for_tight_tiling() {
    let mut tokens = ThemeTokens::default();
    tokens.controls.knob_diameter = 48;
    tokens.typography.text_scale = 3;

    let knob = KnobSpec::new("k", "Drive", 0.5, (0.0, 1.0));
    let measured = measure_knob(&knob, &tokens);
    let expected = knob_block_size_for_diameter(48, 3);

    assert_eq!(measured.width, expected.width);
}

#[test]
fn knob_constructor_defaults_to_auto_width_layout() {
    let knob = KnobSpec::new("k", "Drive", 0.5, (0.0, 1.0));

    assert_eq!(knob.layout.width, Length::Auto);
}
