//! Patchbay GUI re-exports for plugin UIs.
//!
//! The module mirrors the Patchbay GUI API so downstream plugins can depend on
//! `toybox` and keep GUI integrations consistent.

pub use patchbay_gui::{Canvas, Color, Point, Rect, Size, Theme};

/// Declarative layout helpers for Patchbay GUI.
pub mod declarative {
    pub use patchbay_gui::{
        AbsoluteChild, AbsoluteSpec, Align, ButtonEvent, ButtonSpec, DeclarativeError,
        DeclarativeGridSpec, DropdownEvent, DropdownSpec, FlexSpec, IndicatorSpec, KnobEvent,
        KnobSpec, LabelSpec, Node, Padding, PanelSpec, RegionEvent, RegionSpec, RootFrameSpec,
        SizeSpec, SliderEvent, SliderSpec, SpacerSpec, ToggleEvent, ToggleSpec, UiSpec, WidgetSpec,
        measure, measure_checked, render, render_checked,
    };
}
