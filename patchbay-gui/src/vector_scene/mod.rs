//! Vello vector draw command encoding for text and knob primitives.
//!
//! The UI layer emits lightweight draw commands so widget interaction/layout can
//! stay independent from renderer details. The renderer consumes these commands
//! and appends high-quality vector primitives to the Vello scene.
#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

mod vector_scene_color_and_angle_helpers;
mod vector_scene_font_loading;
mod vector_scene_knob_rendering;
mod vector_scene_text_rendering;
mod vector_scene_types;

#[cfg(target_os = "windows")]
pub(crate) use vector_scene_types::VectorScenePainter;
pub(crate) use vector_scene_types::{KnobVisual, VectorCommand};
