//! VST3-facing re-exports for shared Windows bundle helpers.
//!
//! Prefer importing these helpers from [`crate::bundle::windows`] for new code.

pub use crate::bundle::windows::{
    WindowsBundleFormat, WindowsBundlePaths, windows_bundle_name, windows_bundle_paths,
    windows_rustc_link_arg,
};
