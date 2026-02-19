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
    Align, ButtonSpec, ColorTokens, ContainerLayout, ContainerLength, ControlTokens,
    DeclarativeError, DropdownSpec, EdgeInsets, FlexSpec, GridKind, GridSpec, GridTemplate,
    IndicatorSpec, Justify, KnobSpec, LabelSpec, LayoutBox, LayoutContainerKind, LayoutDiagnostic,
    LayoutDiagnosticLevel, LayoutDiagnosticsMode, LayoutEngineState, LayoutNodeDiagnostic,
    LayoutNodeDiagnosticReason, LayoutNodeKind, Length, MeasureCacheKey, MeasureCacheStats, Node,
    NodeId, OverflowPolicy, PanelSpec, RegionInteractionKind, RenderResult, RootFrameSpec,
    RootScaleMode, ScrollViewSpec, SliderSpec, Slot, SlotAlign, SlotCrossSize, SlotMainSize,
    SlotParams, SlotSpec, SpacingTokens, StackSpec, SurfaceCommand, SwitchCase, SwitchLayoutSpec,
    SwitchWidthRange, ThemeTokens, ToggleSpec, TrackSize, TypographyTokens, UiAction, UiSpec,
    WeightedSlot, WrapSpec, button, column, column_slots, dropdown, fill_slot, fraction_slot, grid,
    indicator, knob, label, measure_checked, panel, region, render_checked,
    render_checked_with_engine, root_frame_sized, row, row_slots, scroll_view, slider, slot,
    spacer, stack, surface, switch_layout, toggle, weighted_slot, weighted_slot_lengths,
    when_width_between, when_width_ge, when_width_lt, wrap,
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
