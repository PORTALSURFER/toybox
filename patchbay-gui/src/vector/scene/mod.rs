//! Vello vector draw command encoding for text and knob primitives.
//!
//! The UI layer emits lightweight draw commands so widget interaction/layout can
//! stay independent from renderer details. The renderer consumes these commands
//! and appends high-quality vector primitives to the Vello scene.
#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

mod color_and_angle_helpers;
mod font_loading;
mod knob_rendering;
mod shapes_rendering;
mod text_rendering;
mod types;

#[cfg(target_os = "windows")]
pub(crate) use types::VectorScenePainter;
pub(crate) use types::{
    CircleStrokeVisual, CircleVisual, KnobVisual, LineVisual, PolygonVisual, PolylineVisual,
    RectStrokeVisual, RectVisual, VectorCommand,
};
