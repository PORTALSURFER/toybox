//! Immediate-mode widgets for the Patchbay GUI.

use crate::canvas::{Canvas, Color, Point, Rect, Size};
use crate::host::InputState;

/// Unique identifier for widgets across frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create a widget id from a stable numeric seed.
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Hash a label into a widget id.
    ///
    /// The label must remain stable across frames for correct interaction
    /// tracking. If the label can change (for example, including formatted
    /// values), prefer using a stable key and hashing that instead.
    pub fn from_label(label: &str) -> Self {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in label.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self(hash)
    }
}

/// Theme colors for the GUI widgets.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Canvas background color.
    pub background: Color,
    /// Primary text color.
    pub text: Color,
    /// Text scale factor for the bitmap font.
    pub text_scale: u32,
    /// Knob fill color.
    pub knob_fill: Color,
    /// Knob outline color.
    pub knob_outline: Color,
    /// Knob active color.
    pub knob_active: Color,
    /// Knob hover color.
    pub knob_hover: Color,
    /// Knob indicator color.
    pub knob_indicator: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::rgb(18, 19, 22),
            text: Color::rgb(238, 239, 242),
            text_scale: 2,
            knob_fill: Color::rgb(52, 57, 66),
            knob_outline: Color::rgb(88, 94, 104),
            knob_active: Color::rgb(90, 140, 220),
            knob_hover: Color::rgb(110, 130, 170),
            knob_indicator: Color::rgb(240, 240, 240),
        }
    }
}

/// Input and stateful data for widget interaction.
#[derive(Debug, Default)]
pub struct UiState {
    active: Option<WidgetId>,
    hot: Option<WidgetId>,
    drag_start: Option<Point>,
    drag_value: f32,
    open_dropdown: Option<WidgetId>,
}

/// Layout state for sequential widgets.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Current cursor position in pixels.
    pub cursor: Point,
    /// Width of the layout column.
    pub column_width: i32,
    /// Vertical spacing between widgets.
    pub spacing: i32,
    /// Default knob size in pixels.
    pub knob_size: i32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            cursor: Point { x: 16, y: 16 },
            column_width: 180,
            spacing: 18,
            knob_size: 64,
        }
    }
}

/// Response metadata from knob widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct KnobResponse {
    /// The knob value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the knob.
    pub hovered: bool,
    /// The knob is actively being dragged.
    pub active: bool,
}

/// Response metadata from slider widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct SliderResponse {
    /// The slider value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the slider.
    pub hovered: bool,
    /// The slider is actively being dragged.
    pub active: bool,
}

/// Response metadata from toggle widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToggleResponse {
    /// The toggle value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the toggle.
    pub hovered: bool,
}

/// Response metadata from button widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonResponse {
    /// The button was clicked this frame.
    pub clicked: bool,
    /// The pointer is hovering the button.
    pub hovered: bool,
}

/// Response metadata from custom region widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegionResponse {
    /// The pointer is hovering the region.
    pub hovered: bool,
    /// The region is actively being dragged.
    pub active: bool,
    /// The primary button was pressed on the region.
    pub pressed: bool,
    /// The primary button was released on the region.
    pub released: bool,
    /// The pointer is being dragged while active.
    pub dragged: bool,
    /// The secondary button was clicked on the region.
    pub secondary_clicked: bool,
    /// The primary button was double-clicked on the region.
    pub double_clicked: bool,
}

/// Response metadata from dropdown widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct DropdownResponse {
    /// The selection changed this frame.
    pub changed: bool,
    /// The dropdown is open this frame.
    pub open: bool,
    /// The pointer is hovering the dropdown control.
    pub hovered: bool,
}

/// UI frame context used to draw widgets and handle input.
pub struct Ui<'a> {
    canvas: &'a mut Canvas,
    input: &'a InputState,
    state: &'a mut UiState,
    layout: &'a mut Layout,
    theme: &'a Theme,
}

impl<'a> Ui<'a> {
    /// Create a UI frame tied to the given canvas and input snapshot.
    pub fn new(
        canvas: &'a mut Canvas,
        input: &'a InputState,
        state: &'a mut UiState,
        layout: &'a mut Layout,
        theme: &'a Theme,
    ) -> Self {
        Self {
            canvas,
            input,
            state,
            layout,
            theme,
        }
    }

    /// Access the current layout cursor.
    pub fn cursor(&self) -> Point {
        self.layout.cursor
    }

    /// Set the layout cursor position.
    pub fn set_cursor(&mut self, cursor: Point) {
        self.layout.cursor = cursor;
    }

    /// Advance the cursor vertically.
    pub fn advance_y(&mut self, amount: i32) {
        self.layout.cursor.y += amount;
    }

    /// Draw a label at the given position.
    pub fn text(&mut self, position: Point, text: &str) {
        self.canvas
            .draw_text(position, text, self.theme.text, self.theme.text_scale);
    }

    /// Access the input snapshot for this frame.
    pub fn input(&self) -> &InputState {
        self.input
    }

    /// Access the canvas for custom drawing.
    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    /// Access the layout for custom sizing.
    pub fn layout_mut(&mut self) -> &mut Layout {
        self.layout
    }

    /// Draw a label at the current cursor and advance the cursor.
    pub fn label(&mut self, text: &str) {
        let pos = self.layout.cursor;
        let line_height = 8 * self.theme.text_scale as i32;
        self.canvas
            .draw_text(pos, text, self.theme.text, self.theme.text_scale);
        self.layout.cursor.y += line_height + self.layout.spacing;
    }

    /// Create an interactive region for custom drawing.
    ///
    /// Use this to capture hover/drag interactions over arbitrary rectangles.
    /// The `key` must be stable across frames.
    pub fn region_with_key(&mut self, key: &str, rect: Rect) -> RegionResponse {
        let id = WidgetId::from_label(key);
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = RegionResponse {
            hovered,
            active: self.state.active == Some(id),
            pressed: false,
            released: false,
            dragged: false,
            secondary_clicked: false,
            double_clicked: false,
        };

        if hovered && self.input.mouse_pressed {
            self.state.active = Some(id);
            self.state.drag_start = Some(self.input.pointer_pos);
            response.active = true;
            response.pressed = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            self.state.drag_start = None;
            response.active = false;
            response.released = true;
        }

        if self.state.active == Some(id) && self.input.mouse_down {
            response.dragged = true;
        }

        if hovered && self.input.mouse_secondary_pressed {
            response.secondary_clicked = true;
        }

        if hovered && self.input.mouse_double_clicked {
            response.double_clicked = true;
        }

        response
    }

    /// Draw a knob with the given label and value.
    ///
    /// The provided `id` must be stable across frames. If the label changes
    /// while dragging, use [`Ui::knob_with_key`] to provide a separate stable
    /// identifier.
    pub fn knob(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let knob_size = self.layout.knob_size;
        let label_height = 8 * self.theme.text_scale as i32;
        let label_gap = 4 * self.theme.text_scale as i32;
        let knob_rect = Rect {
            origin: self.layout.cursor,
            size: Size {
                width: knob_size as u32,
                height: knob_size as u32,
            },
        };
        let center = Point {
            x: knob_rect.origin.x + knob_size / 2,
            y: knob_rect.origin.y + knob_size / 2,
        };
        let radius = knob_size / 2 - 4;
        let hovered = knob_rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = KnobResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };

        if hovered && self.input.mouse_pressed {
            self.state.active = Some(id);
            self.state.drag_start = Some(self.input.pointer_pos);
            self.state.drag_value = *value;
            response.active = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            self.state.drag_start = None;
            response.active = false;
        }

        if self.state.active == Some(id) && self.input.mouse_down {
            if let Some(start) = self.state.drag_start {
                let dy = (self.input.pointer_pos.y - start.y) as f32;
                let delta = -dy * 0.005 * (range.1 - range.0);
                let new_value = (self.state.drag_value + delta).clamp(range.0, range.1);
                if (*value - new_value).abs() > f32::EPSILON {
                    *value = new_value;
                    response.changed = true;
                }
            }
        }

        if hovered && self.input.wheel_delta != 0.0 {
            let step = 0.02 * (range.1 - range.0);
            let new_value = (*value + step * self.input.wheel_delta.signum())
                .clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        let t = (*value - range.0) / (range.1 - range.0).max(1.0e-6);
        let arc_start = 7.0 * std::f32::consts::PI / 4.0 + std::f32::consts::PI;
        let arc_end = 5.0 * std::f32::consts::PI / 4.0 + std::f32::consts::PI;
        let arc_span = if arc_end < arc_start {
            arc_end + std::f32::consts::TAU - arc_start
        } else {
            arc_end - arc_start
        };
        let angle = arc_start + t * arc_span;
        let indicator = Point {
            x: center.x + (angle.cos() * (radius as f32 * 0.7)) as i32,
            y: center.y + (angle.sin() * (radius as f32 * 0.7)) as i32,
        };

        let fill = if response.active {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };

        self.canvas.fill_circle(center, radius, fill);
        self.canvas
            .stroke_circle(center, radius, 2, self.theme.knob_outline);
        let arc_radius = radius + 6;
        let arc_thickness = 3;
        self.canvas.stroke_arc(
            center,
            arc_radius,
            arc_thickness,
            arc_start,
            arc_end,
            self.theme.knob_outline,
        );
        self.canvas.stroke_arc(
            center,
            arc_radius,
            arc_thickness,
            arc_start,
            angle,
            self.theme.knob_indicator,
        );
        self.canvas
            .draw_line(center, indicator, self.theme.knob_indicator);

        let mut extra_height = 0;
        if !label.is_empty() {
            let label_pos = Point {
                x: knob_rect.origin.x,
                y: knob_rect.origin.y + knob_size + label_gap,
            };
            self.canvas
                .draw_text(label_pos, label, self.theme.text, self.theme.text_scale);
            extra_height = label_gap + label_height;
        }

        self.layout.cursor.y += knob_size + extra_height + self.layout.spacing;
        response
    }

    /// Draw a knob with a stable key and a potentially dynamic label.
    ///
    /// The `key` should be a stable identifier across frames (for example,
    /// `"attack"`). The `label` can change to include formatted values without
    /// breaking drag tracking.
    ///
    /// # Example
    /// ```
    /// # use patchbay_gui::Ui;
    /// # use patchbay_gui::canvas::Canvas;
    /// # use patchbay_gui::host::InputState;
    /// # use patchbay_gui::ui::{Layout, Theme, UiState};
    /// let mut canvas = Canvas::new(200, 200);
    /// let mut layout = Layout::default();
    /// let theme = Theme::default();
    /// let mut ui_state = UiState::default();
    /// let input = InputState::default();
    /// let mut value = 0.5;
    /// let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    /// ui.knob_with_key("attack", "Attack 0.50s", &mut value, (0.0, 1.0));
    /// ```
    pub fn knob_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let id = WidgetId::from_label(key);
        self.knob(id, label, value, range)
    }

    /// Draw a horizontal slider with the given label and value.
    ///
    /// The provided `id` must be stable across frames. If the label changes
    /// while dragging, use [`Ui::slider_with_key`] to provide a separate stable
    /// identifier.
    pub fn slider(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        width: i32,
        height: i32,
    ) -> SliderResponse {
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            self.canvas
                .draw_text(base, label, self.theme.text, self.theme.text_scale);
            rect_origin.y += label_height;
        }

        let rect = Rect {
            origin: rect_origin,
            size: Size {
                width: width.max(1) as u32,
                height: height.max(1) as u32,
            },
        };
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = SliderResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };

        if hovered && self.input.mouse_pressed {
            self.state.active = Some(id);
            response.active = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            response.active = false;
        }

        if self.state.active == Some(id) && self.input.mouse_down {
            let span = (range.1 - range.0).max(1.0e-6);
            let x = (self.input.pointer_pos.x - rect.origin.x) as f32;
            let t = (x / rect.size.width.max(1) as f32).clamp(0.0, 1.0);
            let new_value = range.0 + t * span;
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        if hovered && self.input.wheel_delta != 0.0 {
            let span = (range.1 - range.0).max(1.0e-6);
            let step = 0.02 * span;
            let new_value = (*value + step * self.input.wheel_delta.signum())
                .clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        let span = (range.1 - range.0).max(1.0e-6);
        let t = ((*value - range.0) / span).clamp(0.0, 1.0);
        let track_height = (height / 4).max(4);
        let track_y = rect.origin.y + (height - track_height) / 2;
        let track_rect = Rect {
            origin: Point {
                x: rect.origin.x,
                y: track_y,
            },
            size: Size {
                width: rect.size.width,
                height: track_height as u32,
            },
        };
        let fill_width = ((rect.size.width as f32) * t).round() as u32;
        let fill_rect = Rect {
            origin: track_rect.origin,
            size: Size {
                width: fill_width,
                height: track_rect.size.height,
            },
        };
        let fill = if response.active {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(track_rect, fill);
        self.canvas
            .stroke_rect(track_rect, 1, self.theme.knob_outline);
        self.canvas
            .fill_rect(fill_rect, self.theme.knob_indicator);

        let handle_x = rect.origin.x + (rect.size.width as f32 * t) as i32;
        let handle_center = Point {
            x: handle_x,
            y: rect.origin.y + height / 2,
        };
        let handle_radius = (height / 2).max(3);
        self.canvas
            .fill_circle(handle_center, handle_radius, self.theme.knob_indicator);

        self.layout.cursor.y = rect.origin.y + height + self.layout.spacing;
        response
    }

    /// Draw a horizontal slider with a stable key and a dynamic label.
    pub fn slider_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        width: i32,
        height: i32,
    ) -> SliderResponse {
        let id = WidgetId::from_label(key);
        self.slider(id, label, value, range, width, height)
    }

    /// Draw a toggle switch with the given label and value.
    pub fn toggle(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut bool,
        width: i32,
        height: i32,
    ) -> ToggleResponse {
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            self.canvas
                .draw_text(base, label, self.theme.text, self.theme.text_scale);
            rect_origin.y += label_height;
        }
        let rect = Rect {
            origin: rect_origin,
            size: Size {
                width: width.max(1) as u32,
                height: height.max(1) as u32,
            },
        };
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = ToggleResponse {
            hovered,
            changed: false,
        };
        if hovered && self.input.mouse_pressed {
            *value = !*value;
            response.changed = true;
        }
        let fill = if *value {
            self.theme.knob_indicator
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas
            .stroke_rect(rect, 1, self.theme.knob_outline);

        let thumb_radius = (height / 2).max(3);
        let thumb_x = if *value {
            rect.origin.x + width - thumb_radius
        } else {
            rect.origin.x + thumb_radius
        };
        let thumb_center = Point {
            x: thumb_x,
            y: rect.origin.y + height / 2,
        };
        self.canvas
            .fill_circle(thumb_center, thumb_radius, self.theme.knob_outline);

        self.layout.cursor.y = rect.origin.y + height + self.layout.spacing;
        response
    }

    /// Draw a toggle switch with a stable key and a dynamic label.
    pub fn toggle_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut bool,
        width: i32,
        height: i32,
    ) -> ToggleResponse {
        let id = WidgetId::from_label(key);
        self.toggle(id, label, value, width, height)
    }

    /// Draw a button with the given label.
    pub fn button(
        &mut self,
        id: WidgetId,
        label: &str,
        width: i32,
        height: i32,
    ) -> ButtonResponse {
        let rect = Rect {
            origin: self.layout.cursor,
            size: Size {
                width: width.max(1) as u32,
                height: height.max(1) as u32,
            },
        };
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = ButtonResponse {
            hovered,
            clicked: false,
        };
        if hovered && self.input.mouse_pressed {
            response.clicked = true;
        }
        let fill = if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas
            .stroke_rect(rect, 1, self.theme.knob_outline);
        let text_pos = Point {
            x: rect.origin.x + 4,
            y: rect.origin.y
                + (height - (7 * self.theme.text_scale as i32)) / 2,
        };
        self.canvas
            .draw_text(text_pos, label, self.theme.text, self.theme.text_scale);

        self.layout.cursor.y = rect.origin.y + height + self.layout.spacing;
        response
    }

    /// Draw a button with a stable key and a dynamic label.
    pub fn button_with_key(
        &mut self,
        key: &str,
        label: &str,
        width: i32,
        height: i32,
    ) -> ButtonResponse {
        let id = WidgetId::from_label(key);
        self.button(id, label, width, height)
    }

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
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            self.canvas
                .draw_text(base, label, self.theme.text, self.theme.text_scale);
            rect_origin.y += label_height;
        }

        let rect = Rect {
            origin: rect_origin,
            size: Size {
                width: width.max(1) as u32,
                height: height.max(1) as u32,
            },
        };
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = DropdownResponse {
            hovered,
            open: self.state.open_dropdown == Some(id),
            changed: false,
        };

        if hovered && self.input.mouse_pressed {
            if response.open {
                self.state.open_dropdown = None;
                response.open = false;
            } else {
                self.state.open_dropdown = Some(id);
                response.open = true;
            }
        }

        let fill = if response.open {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas
            .stroke_rect(rect, 1, self.theme.knob_outline);
        let current = options.get(*selected).copied().unwrap_or("-");
        let text_pos = Point {
            x: rect.origin.x + 4,
            y: rect.origin.y
                + (height - (7 * self.theme.text_scale as i32)) / 2,
        };
        self.canvas
            .draw_text(text_pos, current, self.theme.text, self.theme.text_scale);

        if response.open {
            let mut any_hovered = false;
            for (index, option) in options.iter().enumerate() {
                let option_rect = Rect {
                    origin: Point {
                        x: rect.origin.x,
                        y: rect.origin.y + height * (index as i32 + 1),
                    },
                    size: rect.size,
                };
                let option_hovered = option_rect.contains(self.input.pointer_pos);
                if option_hovered {
                    any_hovered = true;
                }
                let option_fill = if option_hovered {
                    self.theme.knob_hover
                } else {
                    self.theme.knob_fill
                };
                self.canvas.fill_rect(option_rect, option_fill);
                self.canvas
                    .stroke_rect(option_rect, 1, self.theme.knob_outline);
                let option_text = Point {
                    x: option_rect.origin.x + 4,
                    y: option_rect.origin.y
                        + (height - (7 * self.theme.text_scale as i32)) / 2,
                };
                self.canvas
                    .draw_text(option_text, option, self.theme.text, self.theme.text_scale);
                if option_hovered && self.input.mouse_pressed {
                    *selected = index;
                    response.changed = true;
                    self.state.open_dropdown = None;
                    response.open = false;
                }
            }

            if self.input.mouse_pressed && !hovered && !any_hovered {
                self.state.open_dropdown = None;
                response.open = false;
            }
        }

        self.layout.cursor.y = rect.origin.y + height + self.layout.spacing;
        response
    }

    /// Draw a dropdown selector with a stable key and a dynamic label.
    pub fn dropdown_with_key(
        &mut self,
        key: &str,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        width: i32,
        height: i32,
    ) -> DropdownResponse {
        let id = WidgetId::from_label(key);
        self.dropdown(id, label, options, selected, width, height)
    }

    /// Clear the background with the theme color.
    pub fn clear(&mut self) {
        self.canvas.clear(self.theme.background);
    }

    /// Draw a non-interactive indicator cell.
    ///
    /// This is useful for sequencer step lights or other simple state displays.
    pub fn indicator(&mut self, rect: Rect, active: bool) {
        let fill = if active {
            self.theme.knob_indicator
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas.stroke_rect(rect, 1, self.theme.knob_outline);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;

    #[test]
    fn knob_updates_value_on_drag() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.5;
        let mut input = InputState::default();
        input.pointer_pos = Point { x: 30, y: 30 };
        input.mouse_pressed = true;
        input.mouse_down = true;

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.knob(WidgetId::new(1), "GAIN", &mut value, (0.0, 1.0));
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 30, y: 10 };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response = ui.knob(WidgetId::new(1), "GAIN", &mut value, (0.0, 1.0));
            assert!(response.changed);
        }
    }

    #[test]
    fn knob_with_key_allows_dynamic_labels() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.5;
        let mut input = InputState::default();
        input.pointer_pos = Point { x: 30, y: 30 };
        input.mouse_pressed = true;
        input.mouse_down = true;

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.knob_with_key("attack", "Attack 0.50s", &mut value, (0.0, 1.0));
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 30, y: 10 };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response =
                ui.knob_with_key("attack", "Attack 0.60s", &mut value, (0.0, 1.0));
            assert!(response.changed);
        }
    }

    #[test]
    fn slider_updates_value_on_drag() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.0;
        let mut input = InputState::default();
        input.pointer_pos = Point { x: 20, y: 40 };
        input.mouse_pressed = true;
        input.mouse_down = true;

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.slider(WidgetId::new(2), "GAIN", &mut value, (0.0, 1.0), 100, 16);
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 80, y: 40 };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response =
                ui.slider(WidgetId::new(2), "GAIN", &mut value, (0.0, 1.0), 100, 16);
            assert!(response.changed);
        }
    }

    #[test]
    fn toggle_flips_on_click() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = false;
        let mut input = InputState::default();
        input.pointer_pos = Point { x: 20, y: 40 };
        input.mouse_pressed = true;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.toggle(WidgetId::new(3), "Toggle", &mut value, 40, 16);
        assert!(response.changed);
        assert!(value);
    }

    #[test]
    fn button_reports_click() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut input = InputState::default();
        input.pointer_pos = Point { x: 20, y: 40 };
        input.mouse_pressed = true;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.button(WidgetId::new(4), "OK", 40, 16);
        assert!(response.clicked);
    }

    #[test]
    fn dropdown_selects_option() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut input = InputState::default();
        let options = ["Off", "Mono", "Poly"];
        let mut selected = 0;

        input.pointer_pos = Point { x: 20, y: 40 };
        input.mouse_pressed = true;
        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.dropdown(WidgetId::new(5), "Mode", &options, &mut selected, 80, 16);
        }

        input.mouse_pressed = true;
        input.pointer_pos = Point { x: 20, y: 70 };
        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response =
                ui.dropdown(WidgetId::new(5), "Mode", &options, &mut selected, 80, 16);
            assert!(response.changed);
            assert_eq!(selected, 1);
        }
    }
}
