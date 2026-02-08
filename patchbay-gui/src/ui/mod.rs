//! Widget rendering and interaction state for the Patchbay GUI.

use std::collections::HashMap;

use crate::canvas::{Canvas, Color, Point, Rect, Size};
use crate::host::InputState;
use crate::vector_scene::{KnobVisual, VectorCommand};

include!("theme_types.rs");
include!("ui_state_types.rs");
include!("layout_types.rs");
include!("geometry_helpers.rs");
include!("text_helpers.rs");
include!("control_responses.rs");
include!("ui_frame_struct.rs");
include!("ui_core_methods.rs");
include!("ui_text_methods.rs");
include!("ui_layout_utility_methods.rs");
include!("ui_root_frame_methods.rs");
include!("ui_panel_grid_methods.rs");
include!("ui_overlay_input_methods.rs");
include!("ui_region_and_misc_methods.rs");
include!("ui_knob_types.rs");
include!("ui_knob_geometry_methods.rs");
include!("ui_knob_interaction_methods.rs");
include!("ui_knob_render_methods.rs");
include!("ui_knob_methods.rs");
include!("ui_slider_methods.rs");
include!("ui_toggle_button_methods.rs");
include!("ui_dropdown_methods.rs");
include!("ui_dropdown_keyed_methods.rs");
include!("ui_dropdown_rect_methods.rs");
include!("ui_pointer_helpers.rs");
#[cfg(test)]
mod tests;
