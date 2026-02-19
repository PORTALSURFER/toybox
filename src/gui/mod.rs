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
        Align, ButtonSpec, ColorTokens, ControlTokens, DeclarativeError, DropdownSpec, EdgeInsets,
        FlexSpec, GridKind, GridSpec, GridTemplate, IndicatorSpec, Justify, KnobSpec, LabelSpec,
        LayoutBox, Length, MainPalette, Node, PanelSpec, RegionInteractionKind, RenderResult,
        RootFrameSpec, RootScaleMode, SliderSpec, Slot, SlotAlign, SlotSpec, SlotTrack,
        SpacingTokens, SurfaceCommand, ThemeTokens, ToggleSpec, TrackSize, TypographyTokens,
        UiAction, UiSpec, WeightedSlot, button, column, column_slots, dropdown, fill_slot,
        fraction_slot, grid, indicator, knob, label, measure_checked, panel, region,
        render_checked, root_frame_sized, row, row_slots, slider, slot, spacer, surface, toggle,
        weighted_slot, weighted_slot_lengths,
    };
}

/// Screenshot test helpers for declarative UIs.
///
/// This module is feature-gated so plugins can opt into screenshot testing
/// without pulling screenshot-only dependencies into release builds.
#[cfg(feature = "screenshot-test")]
pub mod screenshot_harness;
