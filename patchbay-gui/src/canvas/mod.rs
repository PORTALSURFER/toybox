//! CPU-side drawing surface used by the GUI renderer.

use std::cmp::{max, min};

include!("types/color_geometry.rs");
include!("storage/core.rs");
include!("raster/shape.rs");
include!("raster/arc.rs");
include!("raster/blit_and_text.rs");
include!("font/bitmap.rs");

#[cfg(test)]
mod tests;
