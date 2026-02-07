//! Patchbay GUI: a minimal, Vello-backed UI toolkit for CLAP plugin windows.
//!
//! The crate provides a strict declarative UI surface (containers + widgets)
//! rendered via a CPU canvas that is presented through Vello on top of wgpu.
//! The public API
//! exports only declarative authoring primitives and host integration helpers.
//! The focus is embedding into a host-provided window handle on Windows.

mod canvas;
mod declarative;
#[cfg(target_os = "windows")]
mod host;
#[cfg(not(target_os = "windows"))]
#[path = "host_non_windows.rs"]
mod host;
mod logging;
#[cfg(target_os = "windows")]
mod renderer;
mod ui;
#[cfg(target_os = "windows")]
mod win32;

pub use crate::canvas::{Canvas, Color, Point, Rect, Size};
pub use crate::declarative::{
    AbsoluteChild, AbsoluteSpec, Align, ButtonSpec, ColorTokens, ControlTokens, DeclarativeError,
    DrawCommand, DropdownSpec, EdgeInsets, FlexSpec, GridSpec, GridTemplate, IndicatorSpec,
    Justify, KnobSpec, LabelSpec, LayoutBox, Length, Node, PanelSpec, RegionInteractionKind,
    RegionSpec, RenderResult, RootFrameSpec, SliderSpec, SpacingTokens, ThemeTokens, ToggleSpec,
    TrackSize, TypographyTokens, UiAction, UiSpec, button, column, dropdown, grid, indicator, knob,
    label, measure_checked, panel, region, render_checked, row, slider, spacer, toggle,
};
#[cfg(not(target_os = "windows"))]
pub use crate::host::WindowHandle;
pub use crate::host::{GuiError, HostWindow, InputState, OpenParentedMode};
#[cfg(target_os = "windows")]
pub use crate::win32::WindowHandle;
