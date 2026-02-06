//! Patchbay GUI: a minimal, Vulkan-backed UI toolkit for CLAP plugin windows.
//!
//! The crate provides a declarative UI surface (containers + widgets) rendered
//! via a CPU canvas that is presented using wgpu. The focus is embedding into a
//! host-provided window handle on Windows.

#[cfg(target_os = "windows")]
mod canvas;
#[cfg(target_os = "windows")]
mod declarative;
#[cfg(target_os = "windows")]
mod host;
#[cfg(target_os = "windows")]
mod logging;
#[cfg(target_os = "windows")]
mod renderer;
#[cfg(target_os = "windows")]
mod ui;
#[cfg(target_os = "windows")]
mod win32;

#[cfg(not(target_os = "windows"))]
compile_error!("patchbay-gui currently supports Windows only.");

#[cfg(target_os = "windows")]
pub use crate::canvas::{Canvas, Color, Point, Rect, Size};
#[cfg(target_os = "windows")]
pub use crate::declarative::{
    measure, measure_checked, render, render_checked, AbsoluteChild, AbsoluteSpec, Align,
    ButtonEvent, ButtonSpec, DeclarativeError, DropdownEvent, DropdownSpec, FlexSpec,
    GridSpec as DeclarativeGridSpec, IndicatorSpec, KnobEvent, KnobSpec, LabelSpec, Node, Padding,
    PanelSpec, RegionEvent, RegionSpec, RootFrameSpec, SizeSpec, SliderEvent, SliderSpec,
    SpacerSpec, ToggleEvent, ToggleSpec, UiSpec, WidgetSpec,
};
#[cfg(target_os = "windows")]
pub use crate::host::{GuiError, HostWindow, InputState, OpenParentedMode};
#[cfg(target_os = "windows")]
pub use crate::ui::{
    ButtonResponse, DropdownResponse, GridContext, GridResponse, GridSpec, KnobResponse, Layout,
    PanelResponse, PanelStyle, RegionResponse, RootFrameResponse, RootFrameStyle, SliderResponse,
    Theme, ToggleResponse, Ui, WidgetId,
};
#[cfg(target_os = "windows")]
pub use crate::win32::WindowHandle;
