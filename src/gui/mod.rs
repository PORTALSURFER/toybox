//! Strict declarative Patchbay GUI re-exports for plugin UIs.
//!
//! The module exposes the strict declarative GUI surface so plugin crates can
//! build UI specs and reduce typed actions without depending on `patchbay-gui`
//! directly. Immediate-mode authoring APIs are intentionally not re-exported.

pub use patchbay_gui::{Canvas, Color, Point, Rect, Size};

/// Strict declarative GUI types and rendering helpers.
pub mod declarative {
    pub use patchbay_gui::{
        AbsoluteChild, AbsoluteSpec, Align, ButtonSpec, ColorTokens, ControlTokens,
        DeclarativeError, DropdownSpec, EdgeInsets, FlexSpec, GridSpec, GridTemplate,
        IndicatorSpec, Justify, KnobSpec, LabelSpec, LayoutBox, Length, Node, PanelSpec,
        RegionInteractionKind, RegionSpec, RenderResult, RootFrameSpec, SliderSpec, SpacingTokens,
        ThemeTokens, ToggleSpec, TrackSize, TypographyTokens, UiAction, UiSpec, button, column,
        dropdown, grid, indicator, knob, label, measure_checked, panel, region, render_checked,
        row, slider, spacer, toggle,
    };
}
