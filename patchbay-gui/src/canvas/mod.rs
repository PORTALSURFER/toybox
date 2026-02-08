//! CPU-side drawing surface used by the GUI renderer.

use std::cmp::{max, min};

include!("color_geometry_types.rs");
include!("canvas_storage_and_core_methods.rs");
include!("canvas_shape_raster_methods.rs");
include!("canvas_arc_raster_methods.rs");
include!("canvas_blit_and_text_methods.rs");
include!("bitmap_font.rs");

#[cfg(test)]
mod tests;
