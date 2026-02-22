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
#[cfg(feature = "frame-capture")]
mod frame_capture;
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
    AccentKey, Align, AlignBoxSpec, AspectBoxSpec, AspectRatio, ButtonSpec, ColorStateVariants,
    ColorTokens, ContainerLayout, ContainerLength, ControlTokens, CurveEditorSpec,
    CurveEditorStyle, CurveHighlightMode, CurveInteractionOptions, CurveModel, CurvePoint,
    CurveSegment, DeclarativeError, DefaultWidgetColorResolver, DropdownSpec, EdgeInsets,
    EndpointMode, EqAttractor, EqAttractorColorPolicy, EqAttractorSurfaceAction,
    EqAttractorSurfaceModel, EqAttractorSurfaceSpec, EqAttractorSurfaceStyle, FlexSpec, GridKind,
    GridSpec, GridTemplate, IndicatorSpec, Justify, KnobSpec, LayoutBox, LayoutContainerKind,
    LayoutDiagnostic, LayoutDiagnosticCode, LayoutDiagnosticLevel, LayoutDiagnosticsMode,
    LayoutEngineState, LayoutNodeDiagnostic, LayoutNodeDiagnosticReason, LayoutNodeKind,
    LayoutOverflowSummary, Length, MeasureCacheKey, MeasureCacheStats, Node, NodeId,
    OverflowPolicy, PaddingBoxSpec, PanelSpec, RegionInteractionKind, RenderResult, RootFrameSpec,
    RootScaleMode, ScrollViewSpec, SemanticColorToken, SliderSpec, Slot, SlotAlign, SlotCrossSize,
    SlotMainSize, SlotParams, SlotSpec, SpacingTokens, StackSpec, SurfaceCommand, SwitchCase,
    SwitchLayoutSpec, SwitchWidthRange, TabBarSpec, TextBoxSpec, ThemeTokens, ToggleSpec,
    TrackSize, TypographyTokens, UiAction, UiInvalidationScope, UiSpec, WeightedSlot,
    WidgetColorContext, WidgetColorResolver, WidgetColorRole, WidgetColorToken, WrapSpec,
    align_box, aspect_box, button, column, column_slots, curve_editor, dropdown,
    eq_attractor_surface, fill_slot, fraction_slot, grid, indicator, knob, measure_checked,
    padding_box, panel, region, render_checked, render_checked_with_engine, root_frame_sized, row,
    row_slots, scroll_view, slider, slot, spacer, stack, surface, switch_layout, tabbar, textbox,
    toggle, weighted_slot, weighted_slot_lengths, when_width_between, when_width_ge, when_width_lt,
    wrap,
};
#[cfg(feature = "frame-capture")]
pub use crate::frame_capture::CapturedWindowFrame;
#[cfg(not(target_os = "windows"))]
pub use crate::host::WindowHandle;
pub use crate::host::{
    GuiError, HostWindow, InputState, OpenParentedCallbacks, OpenParentedMode, OpenParentedRequest,
    ShortcutBinding, ShortcutModifiers,
};
pub use crate::screenshot::{RenderedFrame, render_spec_to_frame};
pub use crate::ui::MainPalette;
#[cfg(target_os = "windows")]
pub use crate::win32::WindowHandle;
