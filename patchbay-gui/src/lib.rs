//! Patchbay GUI: a minimal, Vulkan-backed UI toolkit for CLAP plugin windows.
//!
//! The crate provides a declarative UI surface (containers + widgets) rendered
//! via a CPU canvas that is presented using wgpu. The focus is embedding into a
//! host-provided window handle on Windows.

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
    DropdownSpec, EdgeInsets, FlexSpec, GridSpec, GridTemplate, IndicatorSpec, Justify, KnobSpec,
    LabelSpec, LayoutBox, Length, Node, PanelSpec, RegionInteractionKind, RegionSpec, RenderResult,
    RootFrameSpec, SliderSpec, SpacingTokens, ThemeTokens, ToggleSpec, TrackSize, TypographyTokens,
    UiAction, UiSpec, measure_checked, render_checked,
};
#[cfg(not(target_os = "windows"))]
pub use crate::host::WindowHandle;
pub use crate::host::{GuiError, HostWindow, InputState, OpenParentedMode};
pub use crate::ui::{
    ButtonResponse, DropdownResponse, GridContext, GridResponse, GridSpec as ImmediateGridSpec,
    KnobResponse, Layout, PanelResponse, PanelStyle, RegionResponse, RootFrameResponse,
    RootFrameStyle, SliderResponse, Theme, ToggleResponse, Ui, UiState, WidgetId,
};
#[cfg(target_os = "windows")]
pub use crate::win32::WindowHandle;
