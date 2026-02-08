/// Layout result for a dropdown control render pass.
#[derive(Clone, Copy)]
struct DropdownLayout {
    /// Computed block footprint (label + control).
    block_size: Size,
    /// Control rectangle used for interaction and drawing.
    rect: Rect,
    /// Control height in pixels.
    control_height: i32,
}

/// Persistent interaction result for an open dropdown menu pass.
#[derive(Clone, Copy)]
struct DropdownMenuInteraction {
    /// Whether the dropdown remains open after processing input.
    open: bool,
    /// Whether a new option was selected.
    changed: bool,
    /// Hovered option index in the open menu, if any.
    hovered_index: Option<usize>,
    /// Whether the menu is rendered above the control.
    open_up: bool,
    /// Snapshot of mouse-pressed state for this pass.
    pressed: bool,
}

/// Geometry required to evaluate option hit-testing.
#[derive(Clone, Copy)]
struct DropdownMenuGeometry {
    /// Dropdown control rectangle.
    rect: Rect,
    /// Single-option row height in pixels.
    control_height: i32,
    /// Whether menu rows are placed upward.
    open_up: bool,
}

impl<'a> Ui<'a> {
    /// Draw a dropdown selector with the given label and options.
    pub fn dropdown(
        &mut self,
        id: WidgetId,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        width: i32,
        height: i32,
    ) -> DropdownResponse {
        let layout = self.prepare_dropdown_layout(label, width, height);
        self.track_rect_internal(layout.rect);

        let mut response = self.resolve_dropdown_state(id, layout.rect);
        self.draw_dropdown_control(layout, options, *selected, &response);

        if response.open {
            let menu = self.evaluate_open_dropdown_menu(layout, options, selected, response.hovered);
            response.open = menu.open;
            response.changed = menu.changed;
            if response.open {
                let clip_rect = self.clipped_rect(layout.rect).unwrap_or(layout.rect);
                self.push_dropdown_overlay(layout.rect, options, menu.hovered_index, menu.open_up, clip_rect);
            }
            if menu.pressed {
                self.consume_mouse_pressed();
            }
        }

        self.advance_dropdown_layout_cursor(layout);
        response
    }

    /// Resolve control placement and draw optional label text.
    fn prepare_dropdown_layout(&mut self, label: &str, width: i32, height: i32) -> DropdownLayout {
        let control_width = width.max(1);
        let control_height = height.max(1);
        let control_size = Size {
            width: control_width as u32,
            height: control_height as u32,
        };
        let block_size = self.dropdown_block_size(label, control_size);

        let mut rect_origin = self.layout.cursor;
        if !label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                self.layout.cursor,
                label,
                control_size.width,
                self.theme.text,
                true,
            );
            rect_origin.y += 8 * self.theme.text_scale as i32;
        }

        DropdownLayout {
            block_size,
            rect: Rect {
                origin: rect_origin,
                size: control_size,
            },
            control_height,
        }
    }

    /// Resolve hover/open state and apply click-to-toggle behavior.
    fn resolve_dropdown_state(&mut self, id: WidgetId, rect: Rect) -> DropdownResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = DropdownResponse {
            hovered,
            open: self.state.open_dropdown == Some(id),
            changed: false,
        };
        if hovered && self.mouse_pressed() {
            if response.open {
                self.state.open_dropdown = None;
                response.open = false;
            } else {
                self.state.open_dropdown = Some(id);
                response.open = true;
            }
        }
        response
    }

    /// Draw the closed-state dropdown control and selected option label.
    fn draw_dropdown_control(
        &mut self,
        layout: DropdownLayout,
        options: &[&str],
        selected: usize,
        response: &DropdownResponse,
    ) {
        let fill = if response.open {
            self.theme.knob_active
        } else if response.hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(layout.rect, fill);
        self.stroke_rect_clipped(layout.rect, 1, self.theme.knob_outline);

        let current = options.get(selected).copied().unwrap_or("-");
        let text_pos = Point {
            x: layout.rect.origin.x + 4,
            y: layout.rect.origin.y
                + (layout.control_height - (7 * self.theme.text_scale as i32)) / 2,
        };
        let _ = self.draw_text_single_line_clamped(
            text_pos,
            current,
            layout.rect.size.width.saturating_sub(8),
            self.theme.text,
            false,
        );
    }

    /// Evaluate option hover/selection while the dropdown menu is open.
    fn evaluate_open_dropdown_menu(
        &mut self,
        layout: DropdownLayout,
        options: &[&str],
        selected: &mut usize,
        hovered_control: bool,
    ) -> DropdownMenuInteraction {
        let geometry = self.resolve_dropdown_menu_geometry(layout, options.len());
        let pressed = self.mouse_pressed();
        let (hovered_index, any_hovered) = self.find_hovered_dropdown_option(geometry, options.len());
        let selection_change = self.apply_dropdown_selection(pressed, hovered_index, selected);
        let open = self.resolve_dropdown_open_after_press(
            pressed,
            hovered_control,
            any_hovered,
            selection_change,
        );

        DropdownMenuInteraction {
            open,
            changed: selection_change,
            hovered_index,
            open_up: geometry.open_up,
            pressed,
        }
    }

    /// Resolve menu placement relative to control bounds.
    fn resolve_dropdown_menu_geometry(
        &self,
        layout: DropdownLayout,
        option_count: usize,
    ) -> DropdownMenuGeometry {
        let menu_height = layout.control_height * option_count as i32;
        let canvas_height = self.canvas.size().height as i32;
        let open_up = layout.rect.origin.y + layout.control_height + menu_height > canvas_height
            && layout.rect.origin.y >= menu_height;
        DropdownMenuGeometry {
            rect: layout.rect,
            control_height: layout.control_height,
            open_up,
        }
    }

    /// Find hovered option index for the current pointer position.
    fn find_hovered_dropdown_option(
        &self,
        geometry: DropdownMenuGeometry,
        option_count: usize,
    ) -> (Option<usize>, bool) {
        let mut hovered_index = None;
        let mut any_hovered = false;
        for index in 0..option_count {
            let option_rect = self.dropdown_option_rect(geometry, index);
            if self.pointer_inside_clipped_rect(option_rect) {
                any_hovered = true;
                hovered_index = Some(index);
            }
        }
        (hovered_index, any_hovered)
    }

    /// Resolve one option row rectangle for a given index.
    fn dropdown_option_rect(&self, geometry: DropdownMenuGeometry, index: usize) -> Rect {
        let row_offset = geometry.control_height * (index as i32 + 1);
        let y = if geometry.open_up {
            geometry.rect.origin.y - row_offset
        } else {
            geometry.rect.origin.y + row_offset
        };
        Rect {
            origin: Point {
                x: geometry.rect.origin.x,
                y,
            },
            size: geometry.rect.size,
        }
    }

    /// Apply hovered-option selection on press and return whether value changed.
    fn apply_dropdown_selection(
        &mut self,
        pressed: bool,
        hovered_index: Option<usize>,
        selected: &mut usize,
    ) -> bool {
        let Some(index) = hovered_index else {
            return false;
        };
        if !pressed {
            return false;
        }
        *selected = index;
        self.state.open_dropdown = None;
        true
    }

    /// Resolve menu-open state after handling pointer press rules.
    fn resolve_dropdown_open_after_press(
        &mut self,
        pressed: bool,
        hovered_control: bool,
        any_hovered: bool,
        selection_change: bool,
    ) -> bool {
        if selection_change {
            return false;
        }
        if pressed && !hovered_control && !any_hovered {
            self.state.open_dropdown = None;
            return false;
        }
        true
    }

    /// Advance the block layout cursor after dropdown rendering.
    fn advance_dropdown_layout_cursor(&mut self, layout: DropdownLayout) {
        self.layout.cursor.y = layout.rect.origin.y + layout.block_size.height as i32 + self.layout.spacing;
    }
}
