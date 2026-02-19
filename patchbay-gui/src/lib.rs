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
    Align, ButtonSpec, ColorTokens, ControlTokens, DeclarativeError, DropdownSpec, EdgeInsets,
    FlexSpec, GridKind, GridSpec, GridTemplate, IndicatorSpec, Justify, KnobSpec, LabelSpec,
    LayoutBox, Length, Node, PanelSpec, RegionInteractionKind, RenderResult, RootFrameSpec,
    RootScaleMode, SliderSpec, Slot, SlotAlign, SlotSpec, SlotTrack, SpacingTokens, SurfaceCommand,
    ThemeTokens, ToggleSpec, TrackSize, TypographyTokens, UiAction, UiSpec, WeightedSlot, button,
    column, column_slots, dropdown, fill_slot, fraction_slot, grid, indicator, knob, label,
    measure_checked, panel, region, render_checked, root_frame_sized, row, row_slots, slider, slot,
    spacer, surface, toggle, weighted_slot, weighted_slot_lengths,
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
