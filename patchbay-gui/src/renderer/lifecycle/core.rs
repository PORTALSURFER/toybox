//! Renderer lifecycle operations (create, resize, command updates).

use std::sync::Arc;

use crate::canvas::Size;
use crate::host::GuiError;
use crate::logging::log_line_safe;
use crate::vector::scene::VectorCommand;
use crate::win32::SurfaceWindow;
use vello::RendererOptions;

use super::{PresentationTransform, Renderer, RendererDevice, map_vello_init_error};

include!("init.rs");
include!("frame.rs");
include!("resize.rs");
include!("canvas_image.rs");
