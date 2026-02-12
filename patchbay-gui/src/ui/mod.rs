//! Widget rendering and interaction state for the Patchbay GUI.

use std::collections::HashMap;

use crate::canvas::{Canvas, Color, Point, Rect, Size};
use crate::host::InputState;
use crate::vector::scene::{KnobVisual, VectorCommand};

include!("theme_types.rs");
include!("core/state.rs");
include!("layout/types.rs");
include!("geometry_helpers.rs");
include!("text/helpers.rs");
include!("control_responses.rs");
include!("core/frame.rs");
include!("core/methods.rs");
include!("text/methods.rs");
include!("layout/utilities.rs");
include!("layout/container_frame.rs");
include!("layout/root_frame.rs");
include!("layout/panel_grid.rs");
include!("input/overlay.rs");
include!("core/region_and_misc.rs");
include!("controls/knob/types.rs");
include!("controls/knob/geometry.rs");
include!("controls/knob/interaction.rs");
include!("controls/knob/render.rs");
include!("controls/knob/api.rs");
include!("controls/slider/types.rs");
include!("controls/slider/layout.rs");
include!("controls/slider/interaction.rs");
include!("controls/slider/render.rs");
include!("controls/slider/api.rs");
include!("controls/toggle_button/types.rs");
include!("controls/toggle_button/layout.rs");
include!("controls/toggle_button/interaction.rs");
include!("controls/toggle_button/render.rs");
include!("controls/toggle_button/api.rs");
include!("controls/dropdown/types.rs");
include!("controls/dropdown/layout.rs");
include!("controls/dropdown/interaction.rs");
include!("controls/dropdown/render.rs");
include!("controls/dropdown/api.rs");
include!("controls/dropdown/keyed.rs");
include!("controls/dropdown/rect.rs");
include!("input/pointer.rs");
#[cfg(test)]
mod tests;
