//! Convenience imports for VST3 plugin implementations.
//!
//! # Example
//!
//! ```
//! use toybox::vst3::prelude::*;
//! ```

pub use toybox_vst3_ffi::{
    Class, ComPtr, ComRef, ComWrapper, Interface, Steinberg, Steinberg::Vst::*, uid,
};

pub use crate::bundle::windows::{
    WindowsBundleFormat, WindowsBundlePaths, windows_bundle_name, windows_bundle_paths,
    windows_rustc_link_arg,
};
pub use crate::vst3::component::{
    CATEGORY_AUDIO_MODULE_CLASS, CATEGORY_COMPONENT_CONTROLLER_CLASS, copy_cstring,
    write_class_info, write_class_info_many, write_wide_name,
};
pub use crate::vst3::connection::{IToyboxSharedState, InstanceConnection, InstanceConnectionRole};
pub use crate::vst3::entry::PluginClassIds;
pub use crate::vst3::events::{Vst3EventTimeline, Vst3ParameterPoint, collect_vst3_timeline};
#[cfg(feature = "gui")]
pub use crate::vst3::gui::parent_to_raw_window_handle;
#[cfg(feature = "gui")]
pub use crate::vst3::gui::{HostedVst3View, Vst3HostedGui};
#[cfg(all(feature = "radiant-vst3", target_os = "macos"))]
pub use crate::vst3::gui::{RadiantVst3Editor, RadiantVst3HostedGui};
pub use crate::vst3::gui::{
    bool_to_tresult, copy_wstring, default_platform_type, platform_type_matches, tchar_len,
    view_rect,
};
pub use crate::vst3::params::{
    ParamRange, for_each_param_point, latest_param_point, parse_tchar_f64,
};
pub use crate::vst3::processor::{StereoAudioBuffers, process_ok, stereo_f32_buffers};
pub use crate::vst3::registration::{Vst3FactoryClass, create_plugin_factory};
pub use crate::vst3::state::{
    StreamError, VersionedPayload, decode_versioned_payload, encode_versioned_payload, read_exact,
    read_versioned_payload, try_encode_versioned_payload, write_all, write_versioned_payload,
};

pub use crate::events::{
    BlockEvent, BlockEventTimeline, ScheduledBlockEvent, TimelineBatch, TimelineIngestReport,
    TimelinePushStatus,
};
pub use crate::impl_vst3_instance_connection;
pub use crate::vst3_plugin_entry;
