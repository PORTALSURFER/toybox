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
            knob_size: 72,
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
        self.canvas.draw_text(position, text, self.theme.text, 1);
    }

    /// Draw a label at the current cursor and advance the cursor.
    pub fn label(&mut self, text: &str) {
        let pos = self.layout.cursor;
        self.canvas.draw_text(pos, text, self.theme.text, 1);
        self.layout.cursor.y += 10 + self.layout.spacing;
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
        let angle = (-0.75 + t * 1.5) * std::f32::consts::PI;
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
        self.canvas
            .draw_line(center, indicator, self.theme.knob_indicator);

        let label_pos = Point {
            x: knob_rect.origin.x,
            y: knob_rect.origin.y + knob_size + 6,
        };
        self.canvas.draw_text(label_pos, label, self.theme.text, 1);

        self.layout.cursor.y += knob_size + 16 + self.layout.spacing;
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

    /// Clear the background with the theme color.
    pub fn clear(&mut self) {
        self.canvas.clear(self.theme.background);
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
}
