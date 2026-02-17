//! Strict declarative Patchbay GUI re-exports for plugin UIs.
//!
//! The module exposes the strict declarative GUI surface so plugin crates can
//! build UI specs and reduce typed actions without depending on `patchbay-gui`
//! directly. Immediate-mode authoring APIs are intentionally not re-exported.

pub use patchbay_gui::{Canvas, Color, MainPalette, Point, Rect, Size};
pub use patchbay_gui::{render_spec_to_frame, RenderedFrame};

/// Strict declarative GUI types and rendering helpers.
pub mod declarative {
    pub use patchbay_gui::{
        AbsoluteChild, AbsoluteSpec, Align, ButtonSpec, ColorTokens, ControlTokens,
        DeclarativeError, DrawCommand, DropdownSpec, EdgeInsets, FlexSpec, GridKind, GridSpec,
        GridTemplate, IndicatorSpec, Justify, KnobSpec, LabelSpec, LayoutBox, Length, MainPalette,
        Node, PanelSpec, RegionInteractionKind, RegionSpec, RenderResult, RootFrameSpec,
        RootScaleMode, SectionChild, SectionSize, SliderSpec, SpacingTokens, ThemeTokens,
        ToggleSpec, TrackSize, TypographyTokens, UiAction, UiSpec, WeightedChild, button, column,
        column_sections, dropdown, fill_section, fraction, grid, indicator, knob, label,
        measure_checked, panel, region, render_checked, root_frame_sized, row, row_sections,
        slider, spacer, toggle, weighted, weighted_section_lengths,
    };
}
