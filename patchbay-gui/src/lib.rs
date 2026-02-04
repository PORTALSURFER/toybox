//! Patchbay GUI: a minimal, Vulkan-backed UI toolkit for CLAP plugin windows.
//!
//! The crate provides a small immediate-mode UI surface (text + knobs) rendered
//! via a CPU canvas that is presented using wgpu. The focus is embedding into a
//! host-provided window handle on Windows.

#[cfg(not(target_os = "windows"))]
compile_error!("patchbay-gui currently supports Windows only.");

mod canvas;
mod host;
mod logging;
mod renderer;
mod ui;
#[cfg(target_os = "windows")]
mod win32;

pub use crate::canvas::{Canvas, Color, Point, Rect, Size};
pub use crate::host::{GuiError, HostWindow, InputState, OpenParentedMode};
#[cfg(target_os = "windows")]
pub use crate::win32::WindowHandle;
pub use crate::ui::{
    ButtonResponse, DropdownResponse, GridContext, GridResponse, GridSpec, KnobResponse, Layout,
    PanelResponse, PanelStyle, RegionResponse, SliderResponse, Theme, ToggleResponse, Ui, WidgetId,
};
