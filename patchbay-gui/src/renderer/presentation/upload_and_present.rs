//! Canvas upload and presentation pass implementation.

use vello::kurbo::Affine;
use vello::peniko::Color as VelloColor;
use vello::{AaConfig, RenderParams};

use crate::canvas::Size;
use crate::host::GuiError;
use crate::logging::log_line_safe;

use super::{PresentationTransform, Renderer, should_reconfigure_surface};

include!("upload_and_present/upload.rs");
include!("upload_and_present/surface_acquire.rs");
include!("upload_and_present/render_pass.rs");
include!("upload_and_present/output.rs");
include!("upload_and_present/readback.rs");
