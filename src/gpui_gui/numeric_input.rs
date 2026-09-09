//! A reusable numeric text field for embedded GPUI editors.

use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, DispatchPhase, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, Font, GlobalElementId, Hsla, InteractiveElement, KeyDownEvent, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Render, Rgba,
    ShapedLine, Style, Styled, TextAlign, TextRun, UTF16Selection, UnderlineStyle, Window, div,
    fill, font, point, px, relative, size,
};

const MAX_NUMERIC_TEXT_BYTES: usize = 64;

/// The inclusive range used when committing or stepping a numeric input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericInputRange {
    /// Inclusive lower bound.
    pub min: f32,
    /// Inclusive upper bound.
    pub max: f32,
}

impl NumericInputRange {
    /// Construct an inclusive numeric range.
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    fn normalized(self) -> Self {
        let min = if self.min.is_finite() { self.min } else { 0.0 };
        let max = if self.max.is_finite() { self.max } else { min };
        if min <= max {
            Self { min, max }
        } else {
            Self { min: max, max: min }
        }
    }

    fn clamp(self, value: f32) -> f32 {
        let range = self.normalized();
        if value.is_finite() {
            value.clamp(range.min, range.max)
        } else {
            range.min
        }
    }
}

/// Visual configuration for [`NumericInput`].
#[derive(Clone, Debug)]
pub struct NumericInputStyle {
    /// Font used to shape the draft text.
    pub font: Font,
    /// Text size in logical pixels.
    pub text_size: f32,
    /// Line height in logical pixels.
    pub line_height: f32,
    /// Text color.
    pub text_color: Hsla,
    /// Background used behind selected text.
    pub selection_color: Rgba,
    /// Caret color.
    pub cursor_color: Rgba,
    /// Horizontal alignment when the draft fits inside the field.
    pub text_align: TextAlign,
    /// Horizontal content inset in logical pixels.
    pub horizontal_padding: f32,
}

impl Default for NumericInputStyle {
    fn default() -> Self {
        Self {
            font: font(".SystemUIFont"),
            text_size: 13.0,
            line_height: 14.0,
            text_color: gpui::white(),
            selection_color: gpui::rgba(0x4f83cc80),
            cursor_color: gpui::rgba(0xffffffff),
            text_align: TextAlign::Left,
            horizontal_padding: 4.0,
        }
    }
}

/// Configuration for a [`NumericInput`].
#[derive(Clone)]
pub struct NumericInputConfig {
    /// Initial committed value.
    pub value: f32,
    /// Commit and stepping range.
    pub range: NumericInputRange,
    /// Regular keyboard increment.
    pub step: f32,
    /// Shift keyboard increment.
    pub fine_step: f32,
    /// Formatter used for committed values.
    pub formatter: Rc<dyn Fn(f32) -> String>,
    /// Visual configuration.
    pub style: NumericInputStyle,
}

impl NumericInputConfig {
    /// Construct a numeric input configuration.
    pub fn new(
        value: f32,
        range: NumericInputRange,
        step: f32,
        fine_step: f32,
        formatter: impl Fn(f32) -> String + 'static,
    ) -> Self {
        Self {
            value,
            range,
            step,
            fine_step,
            formatter: Rc::new(formatter),
            style: NumericInputStyle::default(),
        }
    }

    /// Set the visual configuration.
    pub fn with_style(mut self, style: NumericInputStyle) -> Self {
        self.style = style;
        self
    }
}

/// Emitted whenever the numeric draft changes through text or IME input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericInputChanged {
    /// The current numeric draft. It may be incomplete, such as `-` or `.`.
    pub text: String,
}

/// Emitted when the user submits the current draft with Enter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericInputSubmitted {
    /// The submitted numeric draft.
    pub text: String,
}

/// Emitted when the user cancels the current draft with Escape or blur.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NumericInputCanceled;

/// Emitted when the user steps the value with Up or Down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericInputStepped {
    /// Signed amount to add to the committed value.
    pub delta: f32,
}

/// A focused, editable numeric text field with selection, IME, clipboard, and caret scrolling.
pub struct NumericInput {
    config: NumericInputConfig,
    value: f32,
    focus_handle: FocusHandle,
    blur_subscription: Option<gpui::Subscription>,
    editing: bool,
    content: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
    selection_anchor: usize,
    marked_range: Option<Range<usize>>,
    dragging: bool,
    scroll_offset: f32,
    text_offset: f32,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl EventEmitter<NumericInputChanged> for NumericInput {}
impl EventEmitter<NumericInputSubmitted> for NumericInput {}
impl EventEmitter<NumericInputCanceled> for NumericInput {}
impl EventEmitter<NumericInputStepped> for NumericInput {}

impl NumericInput {
    /// Construct an input from a numeric configuration.
    pub fn new(config: NumericInputConfig, cx: &mut Context<Self>) -> Self {
        let range = config.range.normalized();
        let value = range.clamp(config.value);
        let content = (config.formatter)(value);
        let end = content.len();
        Self {
            config,
            value,
            focus_handle: cx.focus_handle(),
            blur_subscription: None,
            editing: false,
            content,
            selected_range: end..end,
            selection_reversed: false,
            selection_anchor: end,
            marked_range: None,
            dragging: false,
            scroll_offset: 0.0,
            text_offset: 0.0,
            last_layout: None,
            last_bounds: None,
        }
    }

    /// Return the focus handle owned by this input.
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    /// Return the current text draft.
    pub fn text(&self) -> &str {
        &self.content
    }

    /// Return the last committed numeric value known to the input.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Return whether the input currently owns an editing session.
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Replace the committed value and formatted display when the field is idle.
    pub fn set_value(&mut self, value: f32, cx: &mut Context<Self>) {
        self.value = self.config.range.clamp(value);
        if !self.editing {
            let text = (self.config.formatter)(self.value);
            self.set_text_inner(text, cx);
        }
    }

    /// Replace the visible draft while preserving the current editing session.
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.set_text_inner(text.into(), cx);
    }

    /// Finish or start an editing session.
    pub fn set_editing(&mut self, editing: bool, cx: &mut Context<Self>) {
        if self.editing == editing {
            return;
        }
        self.editing = editing;
        if !editing {
            self.marked_range = None;
            self.scroll_offset = 0.0;
            self.text_offset = 0.0;
        }
        cx.notify();
    }

    fn set_text_inner(&mut self, text: String, cx: &mut Context<Self>) {
        self.content = text;
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.selection_anchor = end;
        self.marked_range = None;
        self.scroll_offset = 0.0;
        self.text_offset = 0.0;
        self.last_layout = None;
        self.last_bounds = None;
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content[..offset]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content[offset..]
            .char_indices()
            .nth(1)
            .map(|(index, _)| offset + index)
            .unwrap_or(self.content.len())
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.content.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.selection_anchor = offset;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.content.len());
        let anchor = self.selection_anchor.min(self.content.len());
        if offset < anchor {
            self.selected_range = offset..anchor;
            self.selection_reversed = true;
        } else {
            self.selected_range = anchor..offset;
            self.selection_reversed = false;
        }
        cx.notify();
    }

    fn select_all(&mut self, cx: &mut Context<Self>) {
        self.selected_range = 0..self.content.len();
        self.selection_reversed = false;
        self.selection_anchor = 0;
        cx.notify();
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        offset_from_utf16(&self.content, offset)
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        self.content[..offset].chars().map(char::len_utf16).sum()
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    fn replace_selected_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let range = self.selected_range.clone();
        self.replace_text(range, text, window, cx);
    }

    fn valid_replacement(&self, range: &Range<usize>, text: &str) -> bool {
        let resulting_len = self
            .content
            .len()
            .saturating_sub(range.end.saturating_sub(range.start))
            .saturating_add(text.len());
        if resulting_len > MAX_NUMERIC_TEXT_BYTES || !is_numeric_draft(text) {
            return false;
        }
        let mut candidate = String::with_capacity(resulting_len);
        candidate.push_str(&self.content[..range.start]);
        candidate.push_str(text);
        candidate.push_str(&self.content[range.end..]);
        is_numeric_draft(&candidate)
    }

    fn replace_text(
        &mut self,
        range: Range<usize>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if range.start > range.end
            || range.end > self.content.len()
            || !self.content.is_char_boundary(range.start)
            || !self.content.is_char_boundary(range.end)
            || !self.valid_replacement(&range, text)
        {
            return;
        }
        self.editing = true;
        self.content.replace_range(range.clone(), text);
        let offset = range.start + text.len();
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.selection_anchor = offset;
        self.marked_range = None;
        cx.emit(NumericInputChanged {
            text: self.content.clone(),
        });
        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing = true;
        self.dragging = true;
        window.focus(&self.focus_handle, cx);
        let offset = self
            .last_layout
            .as_ref()
            .zip(self.last_bounds.as_ref())
            .map(|(line, bounds)| {
                let x = event.position.x - bounds.left() - px(self.text_offset)
                    + px(self.scroll_offset);
                line.closest_index_for_x(x)
            })
            .unwrap_or(self.content.len());
        if event.click_count >= 2 {
            self.select_all(cx);
        } else if event.modifiers.shift {
            self.select_to(offset, cx);
        } else {
            self.move_to(offset, cx);
        }
        cx.stop_propagation();
        cx.notify();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.dragging {
            return;
        }
        if event.pressed_button != Some(MouseButton::Left) {
            self.dragging = false;
            cx.notify();
            return;
        }
        let Some((line, bounds)) = self.last_layout.as_ref().zip(self.last_bounds.as_ref()) else {
            return;
        };
        let x = event.position.x - bounds.left() - px(self.text_offset) + px(self.scroll_offset);
        self.select_to(line.closest_index_for_x(x), cx);
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.dragging = false;
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        let modified = modifiers.platform || modifiers.control;
        if modified {
            match key {
                "a" => self.select_all(cx),
                "c" => {
                    if !self.selected_range.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(
                            self.content[self.selected_range.clone()].to_owned(),
                        ));
                    }
                }
                "x" => {
                    if !self.selected_range.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(
                            self.content[self.selected_range.clone()].to_owned(),
                        ));
                        self.replace_selected_text("", window, cx);
                    }
                }
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        self.replace_selected_text(&text.replace(['\r', '\n'], ""), window, cx);
                    }
                }
                _ => return,
            }
            cx.stop_propagation();
            return;
        }
        match key {
            "back" | "backspace" => {
                if self.selected_range.is_empty() {
                    self.select_to(self.previous_boundary(self.cursor_offset()), cx);
                }
                let range = self.selected_range.clone();
                self.replace_text(range, "", window, cx);
            }
            "delete" => {
                if self.selected_range.is_empty() {
                    self.select_to(self.next_boundary(self.cursor_offset()), cx);
                }
                let range = self.selected_range.clone();
                self.replace_text(range, "", window, cx);
            }
            "left" => {
                let offset = if modifiers.shift || self.selected_range.is_empty() {
                    self.previous_boundary(self.cursor_offset())
                } else {
                    self.selected_range.start
                };
                if modifiers.shift {
                    self.select_to(offset, cx);
                } else {
                    self.move_to(offset, cx);
                }
            }
            "right" => {
                let offset = if modifiers.shift || self.selected_range.is_empty() {
                    self.next_boundary(self.cursor_offset())
                } else {
                    self.selected_range.end
                };
                if modifiers.shift {
                    self.select_to(offset, cx);
                } else {
                    self.move_to(offset, cx);
                }
            }
            "home" => {
                if modifiers.shift {
                    self.select_to(0, cx);
                } else {
                    self.move_to(0, cx);
                }
            }
            "end" => {
                if modifiers.shift {
                    self.select_to(self.content.len(), cx);
                } else {
                    self.move_to(self.content.len(), cx);
                }
            }
            "up" | "down" => {
                let step = if modifiers.shift {
                    self.config.fine_step
                } else {
                    self.config.step
                }
                .abs();
                let direction = if key == "up" { 1.0 } else { -1.0 };
                cx.emit(NumericInputStepped {
                    delta: direction * step,
                });
            }
            "enter" => {
                self.editing = false;
                cx.emit(NumericInputSubmitted {
                    text: self.content.clone(),
                });
            }
            "escape" => {
                self.editing = false;
                cx.emit(NumericInputCanceled);
            }
            _ => return,
        }
        cx.stop_propagation();
    }
}

impl EntityInputHandler for NumericInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());
        self.replace_text(range, new_text, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());
        if range.start > range.end
            || range.end > self.content.len()
            || !self.content.is_char_boundary(range.start)
            || !self.content.is_char_boundary(range.end)
            || !self.valid_replacement(&range, new_text)
        {
            return;
        }
        self.editing = true;
        self.content.replace_range(range.clone(), new_text);
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|selection| {
                let start = offset_from_utf16(new_text, selection.start);
                let end = offset_from_utf16(new_text, selection.end).max(start);
                range.start + start..range.start + end
            })
            .unwrap_or_else(|| {
                let end = range.start + new_text.len();
                end..end
            });
        self.selection_reversed = false;
        self.selection_anchor = self.selected_range.start;
        cx.emit(NumericInputChanged {
            text: self.content.clone(),
        });
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + line.x_for_index(range.start) + px(self.text_offset)
                    - px(self.scroll_offset),
                bounds.top(),
            ),
            point(
                bounds.left() + line.x_for_index(range.end) + px(self.text_offset)
                    - px(self.scroll_offset),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        Some(self.offset_to_utf16(line.closest_index_for_x(
            point.x - bounds.left() - px(self.text_offset) + px(self.scroll_offset),
        )))
    }
}

impl Focusable for NumericInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct NumericTextElement {
    input: Entity<NumericInput>,
}

struct NumericTextPrepaint {
    line: Option<ShapedLine>,
    cursor: Option<gpui::PaintQuad>,
    selection: Option<gpui::PaintQuad>,
    scroll_offset: f32,
    text_offset: f32,
    line_height: f32,
}

impl gpui::IntoElement for NumericTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for NumericTextElement {
    type RequestLayoutState = ();
    type PrepaintState = NumericTextPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let line_height = self.input.read(cx).config.style.line_height;
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = px(line_height).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let style = input.config.style.clone();
        let text = input.content.clone();
        let run = TextRun {
            len: text.len(),
            font: style.font,
            color: style.text_color,
            background_color: None,
            underline: input.marked_range.as_ref().map(|_| UnderlineStyle {
                color: Some(style.text_color),
                thickness: px(1.0),
                wavy: false,
            }),
            strikethrough: None,
        };
        let line = window
            .text_system()
            .shape_line(text.into(), px(style.text_size), &[run], None);
        let cursor_x = f32::from(line.x_for_index(input.cursor_offset()));
        let width = f32::from(bounds.size.width);
        let line_width = f32::from(line.width);
        let text_offset = if line_width < width {
            match style.text_align {
                TextAlign::Left => 0.0,
                TextAlign::Center => (width - line_width) * 0.5,
                TextAlign::Right => width - line_width,
            }
        } else {
            0.0
        };
        let focused = input.focus_handle.is_focused(window);
        let active = input.editing || focused;
        let visible_width = (width - 1.0).max(0.0);
        let max_scroll = (f32::from(line.width) - visible_width).max(0.0);
        let mut scroll_offset = if active {
            input.scroll_offset.clamp(0.0, max_scroll)
        } else {
            0.0
        };
        if active {
            if cursor_x < scroll_offset {
                scroll_offset = cursor_x;
            } else if cursor_x >= scroll_offset + visible_width {
                scroll_offset = (cursor_x - visible_width).min(max_scroll);
            }
        }
        let _ = input;
        self.input.update(cx, |input, _| {
            input.scroll_offset = scroll_offset;
            input.text_offset = text_offset;
        });
        let input = self.input.read(cx);
        let cursor_x =
            f32::from(line.x_for_index(input.cursor_offset())) + text_offset - scroll_offset;
        let selection = if focused && !input.selected_range.is_empty() {
            Some(fill(
                Bounds::from_corners(
                    point(
                        bounds.left()
                            + line.x_for_index(input.selected_range.start)
                            + px(text_offset)
                            - px(scroll_offset),
                        bounds.top(),
                    ),
                    point(
                        bounds.left()
                            + line.x_for_index(input.selected_range.end)
                            + px(text_offset)
                            - px(scroll_offset),
                        bounds.bottom(),
                    ),
                ),
                input.config.style.selection_color,
            ))
        } else {
            None
        };
        let cursor = if focused && input.selected_range.is_empty() {
            Some(fill(
                Bounds::new(
                    point(bounds.left() + px(cursor_x), bounds.top()),
                    size(px(1.0), bounds.size.height),
                ),
                input.config.style.cursor_color,
            ))
        } else {
            None
        };
        NumericTextPrepaint {
            line: Some(line),
            cursor,
            selection,
            scroll_offset,
            text_offset,
            line_height: style.line_height,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        if focus_handle.is_focused(window) {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }
        // Install the capture listeners on every frame. A mouse down can be
        // followed by a move or release before GPUI paints another frame, so
        // registering them only while `dragging` is already true would miss
        // the first outside-field event.
        let input_for_move = self.input.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture {
                input_for_move.update(cx, |input, cx| {
                    if input.dragging {
                        input.on_mouse_move(event, window, cx);
                    }
                });
            }
        });
        let input_for_up = self.input.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture
                && event.button == MouseButton::Left
                && input_for_up.read(cx).dragging
            {
                input_for_up.update(cx, |input, cx| input.on_mouse_up(event, window, cx));
            }
        });
        let mask = Some(ContentMask { bounds });
        window.with_content_mask(mask, |window| {
            if let Some(selection) = prepaint.selection.take() {
                window.paint_quad(selection);
            }
            let line = prepaint.line.take().expect("numeric text line");
            let _ = line.paint(
                point(
                    bounds.left() + px(prepaint.text_offset) - px(prepaint.scroll_offset),
                    bounds.top(),
                ),
                px(prepaint.line_height),
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            );
            if focus_handle.is_focused(window)
                && let Some(cursor) = prepaint.cursor.take()
            {
                window.paint_quad(cursor);
            }
            self.input.update(cx, |input, _| {
                input.last_layout = Some(line);
                input.last_bounds = Some(bounds);
            });
        });
    }
}

impl Render for NumericInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        if self.blur_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.blur_subscription = Some(cx.on_blur(&focus_handle, window, |input, _, cx| {
                if input.editing {
                    input.editing = false;
                    input.scroll_offset = 0.0;
                    cx.emit(NumericInputCanceled);
                    cx.notify();
                }
            }));
        }
        div()
            .key_context("NumericInput")
            .track_focus(&self.focus_handle)
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_key_down(cx.listener(Self::on_key_down))
            .flex()
            .items_center()
            .size_full()
            .px(px(self.config.style.horizontal_padding))
            .overflow_hidden()
            .child(NumericTextElement { input: cx.entity() })
    }
}

fn offset_from_utf16(text: &str, offset: usize) -> usize {
    let mut utf16_offset = 0;
    for (index, ch) in text.char_indices() {
        if utf16_offset >= offset {
            return index;
        }
        let next_utf16_offset = utf16_offset + ch.len_utf16();
        if next_utf16_offset > offset {
            return index;
        }
        utf16_offset = next_utf16_offset;
    }
    text.len()
}

fn is_numeric_draft(text: &str) -> bool {
    let mut decimal_seen = false;
    for (index, byte) in text.bytes().enumerate() {
        match byte {
            b'-' if index == 0 => {}
            b'0'..=b'9' => {}
            b'.' if !decimal_seen => decimal_seen = true,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{is_numeric_draft, offset_from_utf16};

    #[test]
    fn accepts_incomplete_numeric_drafts() {
        for draft in ["", "-", ".", "-.", "-12", "12.5"] {
            assert!(is_numeric_draft(draft), "{draft}");
        }
    }

    #[test]
    fn rejects_non_numeric_drafts() {
        for draft in ["--1", "1.2.3", "1-2", "dB", "1 2", "∞"] {
            assert!(!is_numeric_draft(draft), "{draft}");
        }
    }

    #[test]
    fn maps_utf16_offsets_to_scalar_boundaries() {
        assert_eq!(offset_from_utf16("-12", 2), 2);
        assert_eq!(offset_from_utf16("😀2", 1), 0);
        assert_eq!(offset_from_utf16("😀2", 2), 4);
        assert_eq!(offset_from_utf16("😀2", 99), 5);
    }
}
