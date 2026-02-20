impl<'a> Ui<'a> {
    /// Width of dropdown scrollbar track in pixels.
    const DROPDOWN_SCROLLBAR_TRACK_WIDTH_PX: i32 = 4;
    /// Inset between menu edge and scrollbar track.
    const DROPDOWN_SCROLLBAR_EDGE_INSET_PX: i32 = 1;
    /// Minimum dropdown scrollbar thumb height in pixels.
    const DROPDOWN_SCROLLBAR_MIN_THUMB_HEIGHT_PX: i32 = 8;
    /// Extra text padding reserved when a scrollbar is visible.
    const DROPDOWN_SCROLLBAR_TEXT_GUTTER_PX: u32 = 2;

    /// Queue a dropdown overlay for deferred draw order.
    fn push_dropdown_overlay(
        &mut self,
        options: &[&str],
        hovered: Option<usize>,
        selected: usize,
        geometry: DropdownMenuGeometry,
        visual_style: DropdownVisualStyle,
    ) {
        let fill_color = visual_style.fill.unwrap_or(self.theme.knob_fill);
        let hover_fill_color = visual_style.hover_fill.unwrap_or(self.theme.knob_hover);
        let selected_fill_color = visual_style.selected_option_fill;
        let outline_color = visual_style.outline.unwrap_or(self.theme.knob_outline);
        let text_color = visual_style.text.unwrap_or(self.theme.text);
        let scrollbar = resolve_dropdown_scrollbar(
            geometry.menu_rect,
            geometry.control_height,
            options.len(),
            geometry.scroll_px,
            Self::DROPDOWN_SCROLLBAR_TRACK_WIDTH_PX,
            Self::DROPDOWN_SCROLLBAR_EDGE_INSET_PX,
            Self::DROPDOWN_SCROLLBAR_MIN_THUMB_HEIGHT_PX,
        );
        self.state.overlays.push(DropdownOverlay {
            base_rect: geometry.rect,
            menu_rect: geometry.menu_rect,
            options: options.iter().map(|option| (*option).to_string()).collect(),
            hovered,
            selected,
            open_up: geometry.open_up,
            scroll_px: geometry.scroll_px,
            row_height: geometry.control_height.max(1),
            scrollbar,
            fill_color,
            hover_fill_color,
            selected_fill_color,
            outline_color,
            text_color,
        });
    }

    /// Draw any deferred overlays (dropdown menus).
    pub fn draw_overlays(&mut self) {
        let overlays = std::mem::take(&mut self.state.overlays);
        for overlay in overlays.iter() {
            let height = overlay.row_height;
            let scrollbar_reserved_width = overlay
                .scrollbar
                .map(dropdown_scrollbar_reserved_width)
                .unwrap_or(0);
            for (index, option) in overlay.options.iter().enumerate() {
                let option_rect = Rect {
                    origin: Point {
                        x: overlay.menu_rect.origin.x,
                        y: if overlay.open_up {
                            overlay.base_rect.origin.y - height * (index as i32 + 1)
                                + overlay.scroll_px
                        } else {
                            overlay.base_rect.origin.y + height * (index as i32 + 1)
                                - overlay.scroll_px
                        },
                    },
                    size: Size {
                        width: overlay.menu_rect.size.width,
                        height: height as u32,
                    },
                };
                let Some(visible_rect) = rect_intersection(option_rect, overlay.menu_rect) else {
                    continue;
                };
                let option_fill = if overlay.hovered == Some(index) {
                    overlay.hover_fill_color
                } else if overlay.selected == index {
                    overlay.selected_fill_color.unwrap_or(overlay.fill_color)
                } else {
                    overlay.fill_color
                };
                self.canvas.fill_rect(visible_rect, option_fill);
                self.canvas
                    .stroke_rect(visible_rect, 1, overlay.outline_color);
                let option_text = Point {
                    x: visible_rect.origin.x + 4,
                    y: visible_rect.origin.y + (height - (7 * self.theme.text_scale as i32)) / 2,
                };
                let _ = self.draw_text_single_line_hard_clamped(
                    option_text,
                    option,
                    visible_rect
                        .size
                        .width
                        .saturating_sub(8)
                        .saturating_sub(scrollbar_reserved_width),
                    visible_rect.size.height,
                    overlay.text_color,
                    false,
                );
            }
            if let Some(scrollbar) = overlay.scrollbar {
                self.canvas.fill_rect(scrollbar.track_rect, overlay.fill_color);
                self.canvas
                    .stroke_rect(scrollbar.track_rect, 1, overlay.outline_color);
                self.canvas.fill_rect(scrollbar.thumb_rect, self.theme.knob_active);
                self.canvas
                    .stroke_rect(scrollbar.thumb_rect, 1, overlay.outline_color);
            }
        }
        self.state.overlays = overlays;
        if self.state.open_dropdown.is_some() && !self.state.open_dropdown_was_seen() {
            self.state.clear_open_dropdown();
        }
    }

    /// Clear any deferred overlay drawings for the next frame.
    pub fn clear_overlays(&mut self) {
        self.state.overlays.clear();
    }

    /// Reset per-frame input consumption flags.
    pub fn reset_input_consumption(&mut self) {
        self.state.consume_mouse_pressed = false;
        self.state.open_dropdown_seen_this_frame = false;
    }

    /// Return true when a primary-button press is still available this frame.
    fn mouse_pressed_available(&self) -> bool {
        self.input.mouse_pressed && !self.state.consume_mouse_pressed
    }

    /// Consume the frame's primary-button press.
    fn consume_mouse_pressed(&mut self) {
        self.state.consume_mouse_pressed = true;
    }

    /// Return true if this call successfully claimed the primary-button press.
    ///
    /// When a dropdown menu is open, non-dropdown controls must not claim the
    /// frame press. This prevents click-through behavior under floating menus.
    fn claim_mouse_pressed(&mut self) -> bool {
        if self.state.open_dropdown.is_some() {
            return false;
        }
        self.claim_mouse_pressed_raw()
    }

    /// Claim primary-button press for a dropdown control.
    ///
    /// If another dropdown is currently open, only that dropdown may claim.
    fn claim_mouse_pressed_for_dropdown(&mut self, id: WidgetId) -> bool {
        if let Some(open_id) = self.state.open_dropdown
            && open_id != id
        {
            return false;
        }
        self.claim_mouse_pressed_raw()
    }

    /// Claim primary-button press without widget-type gating.
    fn claim_mouse_pressed_raw(&mut self) -> bool {
        if !self.mouse_pressed_available() {
            return false;
        }
        self.consume_mouse_pressed();
        true
    }
}

/// Resolve dropdown scrollbar geometry for overflowing menu content.
fn resolve_dropdown_scrollbar(
    menu_rect: Rect,
    row_height: i32,
    option_count: usize,
    scroll_px: i32,
    track_width_px: i32,
    edge_inset_px: i32,
    min_thumb_height_px: i32,
) -> Option<DropdownOverlayScrollbar> {
    let content_height = row_height.saturating_mul(option_count as i32).max(0);
    let viewport_height = menu_rect.size.height as i32;
    if content_height <= viewport_height {
        return None;
    }
    let track_width = track_width_px.max(1);
    let edge_inset = edge_inset_px.max(0);
    let track_height = (viewport_height - edge_inset.saturating_mul(2)).max(1);
    let track_x = menu_rect.origin.x
        + menu_rect.size.width as i32
        - edge_inset
        - track_width;
    let track_rect = Rect {
        origin: Point {
            x: track_x,
            y: menu_rect.origin.y + edge_inset,
        },
        size: Size {
            width: track_width as u32,
            height: track_height as u32,
        },
    };
    let thumb_height = ((viewport_height as i64 * track_height as i64) / content_height as i64) as i32;
    let thumb_height = thumb_height.clamp(min_thumb_height_px.max(1), track_height);
    let max_scroll_px = (content_height - viewport_height).max(1);
    let max_thumb_offset = (track_height - thumb_height).max(0);
    let clamped_scroll = scroll_px.clamp(0, max_scroll_px);
    let thumb_offset = ((clamped_scroll as i64 * max_thumb_offset as i64) / max_scroll_px as i64) as i32;
    let thumb_rect = Rect {
        origin: Point {
            x: track_rect.origin.x,
            y: track_rect.origin.y + thumb_offset,
        },
        size: Size {
            width: track_rect.size.width,
            height: thumb_height as u32,
        },
    };
    Some(DropdownOverlayScrollbar {
        track_rect,
        thumb_rect,
    })
}

/// Return horizontal text space reserved by a visible dropdown scrollbar.
fn dropdown_scrollbar_reserved_width(scrollbar: DropdownOverlayScrollbar) -> u32 {
    scrollbar.track_rect.size.width + Ui::DROPDOWN_SCROLLBAR_TEXT_GUTTER_PX
}
