//! Patchbay GUI re-exports for plugin UIs.
//!
//! The module mirrors the Patchbay GUI API so downstream plugins can depend on
//! `toybox` and keep GUI integrations consistent.

pub use patchbay_gui::*;

/// Declarative layout helpers for Patchbay GUI.
pub mod declarative {
    //! Declarative layout helpers re-exported from patchbay-gui.
    pub use patchbay_gui::{
        Align, FlexSpec, LabelSpec, Node, Padding, PanelSpec, RootFrameSpec, SizeSpec, SpacerSpec,
        UiSpec, WidgetSpec, DeclarativeGridSpec, measure, render,
    };
}
