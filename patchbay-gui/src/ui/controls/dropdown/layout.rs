impl<'a> Ui<'a> {
    /// Resolve control placement and draw optional label text.
    pub(crate) fn resolve_dropdown_layout(
        &mut self,
        label: &str,
        width: i32,
        height: i32,
    ) -> DropdownLayout {
        let control_height = height.max(1);
        let control_size = Size {
            width: width.max(1) as u32,
            height: control_height as u32,
        };
        let block_size = self.dropdown_block_size(label, control_size);
        let rect = Rect {
            origin: self.draw_dropdown_label(label, control_size),
            size: control_size,
        };
        self.track_rect_internal(rect);

        DropdownLayout {
            block_size,
            rect,
            control_height,
        }
    }

    /// Draw dropdown label and return the control rectangle origin.
    pub(crate) fn draw_dropdown_label(&mut self, label: &str, control_size: Size) -> Point {
        let base = self.layout.cursor;
        if label.is_empty() {
            return base;
        }
        let _ = self.draw_text_single_line_clamped(
            base,
            label,
            control_size.width,
            self.theme.text,
            true,
        );
        Point {
            x: base.x,
            y: base.y + 8 * self.theme.text_scale as i32,
        }
    }

    /// Resolve menu placement relative to control bounds.
    pub(crate) fn resolve_dropdown_menu_geometry(
        &self,
        layout: DropdownLayout,
        option_count: usize,
        scroll_px: i32,
    ) -> DropdownMenuGeometry {
        let root = self.dropdown_root_bounds();
        let row_height = layout.control_height.max(1);
        let content_height = row_height.saturating_mul(option_count as i32);
        let (space_above, space_below) = self.dropdown_menu_available_space(layout.rect, row_height, root);
        let open_up = self.resolve_dropdown_open_up(content_height, space_above, space_below);
        let viewport_height = dropdown_viewport_height(content_height, space_above, space_below, open_up);
        let menu_origin = dropdown_menu_origin(layout.rect, root, viewport_height, row_height, open_up);
        let max_scroll_px = (content_height - viewport_height).max(0);
        DropdownMenuGeometry {
            rect: layout.rect,
            menu_rect: Rect {
                origin: menu_origin,
                size: dropdown_menu_size(layout.rect, viewport_height),
            },
            control_height: row_height,
            option_count,
            max_scroll_px,
            scroll_px: 0,
            open_up,
        }
        .with_scroll(scroll_px)
    }

    /// Resolve one option row rectangle for a given index.
    pub(crate) fn dropdown_option_rect(
        &self,
        geometry: DropdownMenuGeometry,
        index: usize,
        scroll_px: i32,
    ) -> Rect {
        let y = if geometry.open_up {
            geometry.rect.origin.y - geometry.control_height * (index as i32 + 1) + scroll_px
        } else {
            geometry.rect.origin.y + geometry.control_height * (index as i32 + 1) - scroll_px
        };
        Rect {
            origin: Point {
                x: geometry.menu_rect.origin.x,
                y,
            },
            size: Size {
                width: geometry.menu_rect.size.width,
                height: geometry.control_height as u32,
            },
        }
    }

    /// Resolve root viewport bounds used for floating dropdown menus.
    fn dropdown_root_bounds(&self) -> Rect {
        Rect {
            origin: Point { x: 0, y: 0 },
            size: self.canvas.size(),
        }
    }

    /// Resolve available menu space above/below the dropdown control.
    fn dropdown_menu_available_space(
        &self,
        rect: Rect,
        row_height: i32,
        root: Rect,
    ) -> (i32, i32) {
        let control_top = rect.origin.y;
        let control_bottom = rect.origin.y + row_height;
        let root_top = root.origin.y;
        let root_bottom = root.origin.y + root.size.height as i32;
        let space_above = (control_top - root_top).max(0);
        let space_below = (root_bottom - control_bottom).max(0);
        (space_above, space_below)
    }

    /// Resolve whether a dropdown menu should open upward.
    fn resolve_dropdown_open_up(
        &self,
        content_height: i32,
        space_above: i32,
        space_below: i32,
    ) -> bool {
        if content_height <= space_below {
            return false;
        }
        if content_height <= space_above {
            return true;
        }
        space_above > space_below
    }

    /// Advance the block layout cursor after dropdown rendering.
    pub(crate) fn advance_dropdown_layout_cursor(&mut self, layout: DropdownLayout) {
        self.layout.cursor.y = layout.rect.origin.y + layout.block_size.height as i32 + self.layout.spacing;
    }
}

impl DropdownMenuGeometry {
    /// Return this geometry with a scroll offset clamped to valid bounds.
    fn with_scroll(self, scroll_px: i32) -> Self {
        Self {
            scroll_px: scroll_px.clamp(0, self.max_scroll_px),
            ..self
        }
    }
}

/// Resolve clamped viewport height for a dropdown menu.
fn dropdown_viewport_height(content_height: i32, space_above: i32, space_below: i32, open_up: bool) -> i32 {
    let available = if open_up { space_above } else { space_below };
    content_height.max(1).min(available.max(1))
}

/// Resolve menu origin while clamping to root viewport bounds.
fn dropdown_menu_origin(
    rect: Rect,
    root: Rect,
    viewport_height: i32,
    row_height: i32,
    open_up: bool,
) -> Point {
    let root_bottom = root.origin.y + root.size.height as i32;
    let max_menu_x = root.origin.x + root.size.width as i32 - rect.size.width as i32;
    let menu_x = rect.origin.x.clamp(root.origin.x, max_menu_x.max(root.origin.x));
    let unclamped_y = if open_up {
        rect.origin.y - viewport_height
    } else {
        rect.origin.y + row_height
    };
    let max_menu_y = root_bottom - viewport_height;
    let menu_y = unclamped_y.clamp(root.origin.y, max_menu_y.max(root.origin.y));
    Point {
        x: menu_x,
        y: menu_y,
    }
}

/// Resolve menu viewport size from control width and resolved viewport height.
fn dropdown_menu_size(rect: Rect, viewport_height: i32) -> Size {
    Size {
        width: rect.size.width.max(1),
        height: viewport_height.max(1) as u32,
    }
}
