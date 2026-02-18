//! Patchbay GUI: a minimal, Vello-backed UI toolkit for CLAP plugin windows.
//!
//! The crate provides a strict declarative UI surface (containers + widgets)
//! rendered via a CPU canvas that is presented through Vello on top of wgpu.
//! The public API
//! exports only declarative authoring primitives and host integration helpers.
//! The focus is embedding into a host-provided window handle on Windows.

// Keep doc expectations visible at the crate boundary. The workspace lints also
// enforce this, but having it here makes the standard obvious to contributors.
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

mod canvas;
mod declarative;
#[cfg(target_os = "windows")]
mod host;
#[cfg(not(target_os = "windows"))]
#[path = "host_non_windows/mod.rs"]
mod host;
mod logging;
#[cfg(target_os = "windows")]
mod renderer;
mod screenshot;
mod ui;
mod vector;
#[cfg(target_os = "windows")]
mod win32;

pub use crate::canvas::{Canvas, Color, Point, Rect, Size};
pub use crate::declarative::{
    AbsoluteChild, AbsoluteSpec, Align, ButtonSpec, ColorTokens, ControlTokens, DeclarativeError,
    DrawCommand, DropdownSpec, EdgeInsets, FlexSpec, GridKind, GridSpec, GridTemplate,
    IndicatorSpec, Justify, KnobSpec, LabelSpec, LayoutBox, Length, Node, PanelSpec,
    RegionInteractionKind, RegionSpec, RenderResult, RootFrameSpec, RootScaleMode, SectionChild,
    SectionSize, SliderSpec, SpacingTokens, ThemeTokens, ToggleSpec, TrackSize, TypographyTokens,
    UiAction, UiSpec, WeightedChild, button, column, column_sections, dropdown, fill_section,
    fraction, grid, indicator, knob, label, measure_checked, panel, region, render_checked,
    root_frame_sized, row, row_sections, slider, spacer, toggle, weighted,
    weighted_section_lengths,
};
#[cfg(not(target_os = "windows"))]
pub use crate::host::WindowHandle;
pub use crate::host::{
    GuiError, HostWindow, InputState, OpenParentedCallbacks, OpenParentedMode, OpenParentedRequest,
};
pub use crate::screenshot::{RenderedFrame, render_spec_to_frame};
pub use crate::ui::MainPalette;
#[cfg(target_os = "windows")]
pub use crate::win32::WindowHandle;
