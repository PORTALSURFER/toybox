//! Strict declarative Patchbay GUI re-exports for plugin UIs.
//!
//! The module exposes the strict declarative GUI surface so plugin crates can
//! build UI specs and reduce typed actions without depending on `patchbay-gui`
//! directly. Immediate-mode authoring APIs are intentionally not re-exported.

pub use patchbay_gui::{
    Canvas, Color, InputState, MainPalette, Point, Rect, RenderedFrame, Size, render_spec_to_frame,
};

/// Strict declarative GUI types and rendering helpers.
pub mod declarative {
    pub use patchbay_gui::{
        Align, ButtonSpec, ColorTokens, ContainerLayout, ContainerLength, ControlTokens,
        DeclarativeError, DropdownSpec, EdgeInsets, FlexSpec, GridKind, GridSpec, GridTemplate,
        IndicatorSpec, Justify, KnobSpec, LabelSpec, LayoutBox, LayoutContainerKind,
        LayoutDiagnostic, LayoutDiagnosticLevel, LayoutEngineState, Length, MainPalette,
        MeasureCacheKey, Node, OverflowPolicy, PanelSpec, RegionInteractionKind, RenderResult,
        RootFrameSpec, RootScaleMode, ScrollViewSpec, SliderSpec, Slot, SlotAlign, SlotCrossSize,
        SlotMainSize, SlotParams, SlotSpec, SpacingTokens, StackSpec, SurfaceCommand, ThemeTokens,
        ToggleSpec, TrackSize, TypographyTokens, UiAction, UiSpec, WeightedSlot, WrapSpec, button,
        column, column_slots, dropdown, fill_slot, fraction_slot, grid, indicator, knob, label,
        measure_checked, panel, region, render_checked, render_checked_with_engine,
        root_frame_sized, row, row_slots, scroll_view, slider, slot, spacer, stack, surface,
        toggle, weighted_slot, weighted_slot_lengths, wrap,
    };
}

/// Screenshot test helpers for declarative UIs.
///
/// This module is feature-gated so plugins can opt into screenshot testing
/// without pulling screenshot-only dependencies into release builds.
#[cfg(feature = "screenshot-test")]
pub mod screenshot_harness;
