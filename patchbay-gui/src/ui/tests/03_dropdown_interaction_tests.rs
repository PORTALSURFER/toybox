use super::*;
use crate::canvas::Canvas;
use crate::host::InputState;

fn canvas_pixel(canvas: &Canvas, x: u32, y: u32) -> Color {
    let width = canvas.size().width as usize;
    let idx = ((y as usize) * width + (x as usize)) * 4;
    let pixels = canvas.pixels();
    Color::rgba(pixels[idx], pixels[idx + 1], pixels[idx + 2], pixels[idx + 3])
}

#[test]
fn dropdown_opens_and_closes_on_primary_press() {
    let mut canvas = Canvas::new(200, 200);
    let mut layout = Layout::default();
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 0;
    let id = WidgetId::new(17);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
        assert_eq!(ui.state.open_dropdown, Some(id));
    }

    layout.cursor = layout_origin;
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(!response.open);
        assert_eq!(ui.state.open_dropdown, None);
    }
}

#[test]
fn dropdown_closes_on_outside_press_when_open() {
    let mut canvas = Canvas::new(200, 200);
    let mut layout = Layout::default();
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 0;
    let id = WidgetId::new(18);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let _ = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
    }

    layout.cursor = layout_origin;
    let outside_input = InputState {
        pointer_pos: Point { x: 180, y: 180 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &outside_input, &mut ui_state, &mut layout, &theme);
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(!response.open);
        assert!(!response.changed);
        assert_eq!(ui.state.open_dropdown, None);
    }
}

#[test]
fn open_dropdown_blocks_click_through_to_prior_widgets() {
    let mut canvas = Canvas::new(220, 220);
    let mut layout = Layout::default();
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 0;
    let dropdown_id = WidgetId::new(181);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 70 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let _ = ui.button(WidgetId::new(4), "Armed", 70, 16);
        let response = ui.dropdown(dropdown_id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
    }

    layout.cursor = layout_origin;
    let outside_press = InputState {
        pointer_pos: Point { x: 20, y: 20 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &outside_press, &mut ui_state, &mut layout, &theme);
    ui.reset_input_consumption();
    let button = ui.button(WidgetId::new(4), "Armed", 70, 16);
    let response = ui.dropdown(dropdown_id, "Mode", &options, &mut selected, 80, 16);
    assert!(
        !button.clicked,
        "outside press while menu is open should not click through to prior widgets"
    );
    assert!(!response.open, "outside press should close the open dropdown");
    assert_eq!(ui.state.open_dropdown, None);
}

#[test]
fn dropdown_open_up_selection_uses_rows_above_control() {
    let mut canvas = Canvas::new(120, 80);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 50 },
        ..Layout::default()
    };
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 2;
    let id = WidgetId::new(19);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 70 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
    }

    layout.cursor = layout_origin;
    let select_first_above_input = InputState {
        pointer_pos: Point { x: 20, y: 58 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(
            &mut canvas,
            &select_first_above_input,
            &mut ui_state,
            &mut layout,
            &theme,
        );
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.changed);
        assert!(!response.open);
        assert_eq!(selected, 0);
        assert_eq!(ui.state.open_dropdown, None);
    }
}

#[test]
fn dropdown_near_top_prefers_downward_open_direction() {
    let mut canvas = Canvas::new(120, 90);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 4 },
        ..Layout::default()
    };
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 0;
    let id = WidgetId::new(20);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 24 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
    assert!(response.open);
    let overlay = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued");
    assert!(
        !overlay.open_up,
        "dropdown near top should open downward when there is more space below"
    );
}

#[test]
fn dropdown_clamped_menu_allows_wheel_scroll() {
    let mut canvas = Canvas::new(120, 70);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 8 },
        ..Layout::default()
    };
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["0", "1", "2", "3", "4", "5"];
    let mut selected = 0;
    let id = WidgetId::new(21);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 30 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
    }

    layout.cursor = layout_origin;
    let scroll_input = InputState {
        pointer_pos: Point { x: 20, y: 50 },
        wheel_delta: -1.0,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &scroll_input, &mut ui_state, &mut layout, &theme);
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
        assert!(
            ui.state.open_dropdown_scroll_px > 0,
            "wheel input over open menu should advance dropdown scroll"
        );
    }
}

#[test]
fn dropdown_menu_ignores_pointer_hits_outside_window() {
    let mut canvas = Canvas::new(120, 70);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 8 },
        ..Layout::default()
    };
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["0", "1", "2", "3", "4", "5"];
    let mut selected = 0;
    let id = WidgetId::new(221);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 30 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
    }

    layout.cursor = layout_origin;
    let outside_window_input = InputState {
        pointer_pos: Point { x: 20, y: 50 },
        pointer_in_window: false,
        wheel_delta: -1.0,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(
            &mut canvas,
            &outside_window_input,
            &mut ui_state,
            &mut layout,
            &theme,
        );
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
        assert_eq!(
            ui.state.open_dropdown_scroll_px, 0,
            "wheel input outside the plugin window must not scroll the open menu"
        );
        let overlay = ui
            .state
            .overlays
            .last()
            .expect("dropdown overlay should remain visible");
        assert_eq!(
            overlay.hovered, None,
            "outside-window pointer must not mark any dropdown row as hovered"
        );
    }
}

#[test]
fn dropdown_overlay_text_scrolls_with_option_rows_in_vector_mode() {
    let mut canvas = Canvas::new(120, 69);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 8 },
        ..Layout::default()
    };
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["OPT0", "OPT1", "OPT2", "OPT3", "OPT4", "OPT5"];
    let mut selected = 0;
    let id = WidgetId::new(28);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 12 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        ui.set_vector_text_enabled(true);
        let response = ui.dropdown(id, "", &options, &mut selected, 80, 16);
        assert!(response.open);
    }

    layout.cursor = layout_origin;
    let scroll_input = InputState {
        pointer_pos: Point { x: 20, y: 50 },
        wheel_delta: -1.0,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &scroll_input, &mut ui_state, &mut layout, &theme);
    ui.set_vector_text_enabled(true);
    ui.reset_input_consumption();
    let response = ui.dropdown(id, "", &options, &mut selected, 80, 16);
    assert!(response.open);
    let overlay = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued");
    let menu_origin_y = overlay.menu_rect.origin.y;
    let row_top = if overlay.open_up {
        overlay.base_rect.origin.y - overlay.row_height + overlay.scroll_px
    } else {
        overlay.base_rect.origin.y + overlay.row_height - overlay.scroll_px
    };
    let expected_text_y =
        row_top + (overlay.row_height - (7 * theme.text_scale as i32)) / 2;
    ui.draw_overlays();
    let commands = ui.take_vector_commands();
    let found = commands.iter().any(|command| {
        matches!(
            command,
            VectorCommand::Text {
                origin,
                clip_rect: Some(clip_rect),
                text,
                ..
            } if text == "OPT0"
                && origin.y == expected_text_y
                && clip_rect.origin.y >= menu_origin_y
        )
    });
    assert!(
        found,
        "expected first overlay option text to track row origin while scrolling"
    );
}

#[test]
fn dropdown_overlay_emits_vector_rects_when_vector_shapes_enabled() {
    let mut canvas = Canvas::new(120, 90);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Init", "Verse", "Hook"];
    let mut selected = 0;
    let id = WidgetId::new(29);
    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };

    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    ui.set_vector_shapes_enabled(true);
    let response = ui.dropdown(id, "Preset", &options, &mut selected, 80, 16);
    assert!(response.open);
    let menu_rect = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued")
        .menu_rect;
    ui.draw_overlays();
    let commands = ui.take_vector_commands();
    let menu_right = menu_rect.origin.x + menu_rect.size.width as i32;
    let menu_bottom = menu_rect.origin.y + menu_rect.size.height as i32;
    let has_menu_fill = commands.iter().any(|command| match command {
        VectorCommand::RectFill(fill) => {
            let right = fill.rect.origin.x + fill.rect.size.width as i32;
            let bottom = fill.rect.origin.y + fill.rect.size.height as i32;
            fill.rect.origin.x >= menu_rect.origin.x
                && fill.rect.origin.y >= menu_rect.origin.y
                && right <= menu_right
                && bottom <= menu_bottom
        }
        _ => false,
    });
    let has_menu_stroke = commands.iter().any(|command| match command {
        VectorCommand::RectStroke(stroke) => {
            let right = stroke.rect.origin.x + stroke.rect.size.width as i32;
            let bottom = stroke.rect.origin.y + stroke.rect.size.height as i32;
            stroke.rect.origin.x >= menu_rect.origin.x
                && stroke.rect.origin.y >= menu_rect.origin.y
                && right <= menu_right
                && bottom <= menu_bottom
        }
        _ => false,
    });
    assert!(has_menu_fill, "expected dropdown menu fill in vector commands");
    assert!(has_menu_stroke, "expected dropdown menu stroke in vector commands");
}

#[test]
fn dropdown_overlay_vector_commands_append_after_existing_vector_content() {
    let mut canvas = Canvas::new(120, 90);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Init", "Verse", "Hook"];
    let mut selected = 0;
    let id = WidgetId::new(30);
    let marker_color = Color::rgb(2, 199, 233);
    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };

    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    ui.set_vector_shapes_enabled(true);
    ui.draw_line_visual(
        Point { x: 5, y: 5 },
        Point { x: 35, y: 11 },
        1.0,
        marker_color,
    );
    let response = ui.dropdown(id, "Preset", &options, &mut selected, 80, 16);
    assert!(response.open);
    let menu_rect = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued")
        .menu_rect;
    ui.draw_overlays();
    let commands = ui.take_vector_commands();
    let marker_index = commands
        .iter()
        .position(|command| matches!(command, VectorCommand::Line(line) if line.color == marker_color))
        .expect("expected marker line command");
    let menu_right = menu_rect.origin.x + menu_rect.size.width as i32;
    let menu_bottom = menu_rect.origin.y + menu_rect.size.height as i32;
    let first_overlay_index = commands
        .iter()
        .position(|command| match command {
            VectorCommand::RectFill(fill) => {
                let right = fill.rect.origin.x + fill.rect.size.width as i32;
                let bottom = fill.rect.origin.y + fill.rect.size.height as i32;
                fill.rect.origin.x >= menu_rect.origin.x
                    && fill.rect.origin.y >= menu_rect.origin.y
                    && right <= menu_right
                    && bottom <= menu_bottom
            }
            VectorCommand::RectStroke(stroke) => {
                let right = stroke.rect.origin.x + stroke.rect.size.width as i32;
                let bottom = stroke.rect.origin.y + stroke.rect.size.height as i32;
                stroke.rect.origin.x >= menu_rect.origin.x
                    && stroke.rect.origin.y >= menu_rect.origin.y
                    && right <= menu_right
                    && bottom <= menu_bottom
            }
            _ => false,
        })
        .expect("expected dropdown overlay vector command");
    assert!(
        first_overlay_index > marker_index,
        "dropdown overlay vector commands must be appended after regular vector content"
    );
}

#[test]
fn dropdown_clamped_menu_uses_whole_row_viewport_height() {
    let mut canvas = Canvas::new(120, 69);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 8 },
        ..Layout::default()
    };
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["0", "1", "2", "3", "4", "5"];
    let mut selected = 0;
    let id = WidgetId::new(26);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 12 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    let response = ui.dropdown(id, "", &options, &mut selected, 80, 16);
    assert!(response.open);
    let overlay = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued");
    assert_eq!(
        overlay.menu_rect.size.height % overlay.row_height as u32,
        0,
        "clamped menu viewport should snap to full option rows when space allows"
    );
}

#[test]
fn dropdown_menu_keeps_root_edge_inset() {
    let mut canvas = Canvas::new(120, 90);
    let mut layout = Layout {
        cursor: Point { x: 92, y: 2 },
        ..Layout::default()
    };
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["0", "1", "2", "3", "4", "5"];
    let mut selected = 0;
    let id = WidgetId::new(24);

    let open_input = InputState {
        pointer_pos: Point { x: 95, y: 8 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    let response = ui.dropdown(id, "", &options, &mut selected, 40, 16);
    assert!(response.open);
    let overlay = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued");
    let menu_right = overlay.menu_rect.origin.x + overlay.menu_rect.size.width as i32;
    let menu_bottom = overlay.menu_rect.origin.y + overlay.menu_rect.size.height as i32;
    assert!(overlay.menu_rect.origin.x >= 2);
    assert!(overlay.menu_rect.origin.y >= 2);
    assert!(menu_right <= 118);
    assert!(menu_bottom <= 88);
}

#[test]
fn dropdown_overlay_draws_scrollbar_when_menu_overflows() {
    let mut canvas = Canvas::new(120, 70);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 8 },
        ..Layout::default()
    };
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["0", "1", "2", "3", "4", "5", "6", "7"];
    let mut selected = 0;
    let id = WidgetId::new(25);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 30 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
    assert!(response.open);
    let overlay = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued");
    let scrollbar = overlay
        .scrollbar
        .expect("overflowing dropdown should resolve scrollbar geometry");
    assert!(scrollbar.thumb_rect.size.height < scrollbar.track_rect.size.height);
    ui.draw_overlays();
    let thumb_center = Point {
        x: scrollbar.thumb_rect.origin.x + scrollbar.thumb_rect.size.width as i32 / 2,
        y: scrollbar.thumb_rect.origin.y + scrollbar.thumb_rect.size.height as i32 / 2,
    };
    assert_eq!(
        canvas_pixel(&canvas, thumb_center.x as u32, thumb_center.y as u32),
        theme.knob_active
    );
}

#[test]
fn dropdown_visual_style_overrides_apply_to_open_menu_overlay() {
    let mut canvas = Canvas::new(120, 90);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Init", "Verse", "Hook"];
    let mut selected = 0;
    let id = WidgetId::new(22);
    let style = DropdownVisualStyle {
        fill: Some(Color::rgb(150, 44, 44)),
        hover_fill: Some(Color::rgb(150, 44, 44)),
        active_fill: Some(Color::rgb(170, 66, 66)),
        outline: Some(Color::rgb(95, 31, 31)),
        text: Some(Color::rgb(240, 220, 220)),
        selected_option_fill: Some(Color::rgb(200, 88, 88)),
    };
    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };

    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown_with_visual_style(
            id,
            "Preset",
            &options,
            &mut selected,
            Size {
                width: 80,
                height: 16,
            },
            style,
        );
        assert!(response.open);
        let overlay = ui
            .state
            .overlays
            .last()
            .expect("dropdown overlay should be queued");
        assert_eq!(overlay.fill_color, style.fill.expect("fill color should exist"));
        assert_eq!(
            overlay.hover_fill_color,
            style.hover_fill.expect("hover fill color should exist")
        );
        assert_eq!(
            overlay.outline_color,
            style.outline.expect("outline color should exist")
        );
        assert_eq!(overlay.text_color, style.text.expect("text color should exist"));
        assert_eq!(
            overlay.selected_fill_color,
            style.selected_option_fill,
            "selected option fill override should propagate to overlay state"
        );
    }
    let fill_color = canvas_pixel(&canvas, 92, 38);
    assert_eq!(
        fill_color,
        style.active_fill.expect("active fill color should exist"),
        "open dropdown control should use active fill override"
    );
}

#[test]
fn dropdown_selected_option_fill_applies_to_selected_menu_row() {
    let mut canvas = Canvas::new(120, 90);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Init", "Verse", "Hook"];
    let mut selected = 1;
    let id = WidgetId::new(27);
    let selected_fill = Color::rgb(188, 66, 66);
    let style = DropdownVisualStyle {
        selected_option_fill: Some(selected_fill),
        ..DropdownVisualStyle::default()
    };
    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };

    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    let response = ui.dropdown_with_visual_style(
        id,
        "Preset",
        &options,
        &mut selected,
        Size {
            width: 80,
            height: 16,
        },
        style,
    );
    assert!(response.open);
    let overlay = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued");
    let sample_x = (overlay.menu_rect.origin.x + 2).max(0) as u32;
    let row_top = if overlay.open_up {
        overlay.base_rect.origin.y
            - overlay.row_height * (overlay.selected as i32 + 1)
            + overlay.scroll_px
    } else {
        overlay.base_rect.origin.y
            + overlay.row_height * (overlay.selected as i32 + 1)
            - overlay.scroll_px
    };
    let selected_row_y = row_top + overlay.row_height / 2;
    ui.draw_overlays();
    assert_eq!(
        canvas_pixel(&canvas, sample_x, selected_row_y.max(0) as u32),
        selected_fill
    );
}

#[test]
fn dropdown_selected_text_is_clipped_in_vector_mode() {
    let mut canvas = Canvas::new(200, 120);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState::default();
    let options = ["A VERY LONG PRESET NAME THAT SHOULD CLIP"];
    let mut selected = 0usize;

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    ui.set_vector_text_enabled(true);
    let _ = ui.dropdown(WidgetId::new(23), "", &options, &mut selected, 80, 16);

    let commands = ui.take_vector_commands();
    let clipped_command = commands.iter().find_map(|command| match command {
        VectorCommand::Text {
            clip_rect: Some(rect),
            ..
        } => Some(*rect),
        _ => None,
    });
    let clip_rect = clipped_command.expect("dropdown should emit clipped text command");
    assert_eq!(clip_rect.origin.x, 20);
    assert_eq!(clip_rect.origin.y, 17);
    assert_eq!(clip_rect.size.width, 72);
    assert_eq!(clip_rect.size.height, 16);
}
