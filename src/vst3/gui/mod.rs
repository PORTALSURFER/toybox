//! GUI helpers for VST3 plugin view implementations.

use std::ffi::{CStr, c_char};

#[cfg(any(feature = "gui", feature = "radiant-vst3", feature = "gpui-vst3"))]
use raw_window_handle::RawWindowHandle;
#[cfg(any(feature = "gui", feature = "radiant-vst3", feature = "gpui-vst3"))]
use std::cell::Cell;
#[cfg(any(feature = "gui", feature = "radiant-vst3", feature = "gpui-vst3"))]
use std::sync::Mutex;
#[cfg(any(feature = "gui", feature = "radiant-vst3", feature = "gpui-vst3"))]
use toybox_vst3_ffi::Class;
use toybox_vst3_ffi::Steinberg::Vst::TChar;
#[cfg(target_os = "macos")]
use toybox_vst3_ffi::Steinberg::kPlatformTypeNSView;
#[cfg(all(unix, not(target_os = "macos")))]
use toybox_vst3_ffi::Steinberg::kPlatformTypeX11EmbedWindowID;
use toybox_vst3_ffi::Steinberg::{
    FIDString, ViewRect, kPlatformTypeHWND, kResultFalse, kResultTrue, tresult,
};
#[cfg(any(feature = "gui", feature = "radiant-vst3", feature = "gpui-vst3"))]
use toybox_vst3_ffi::Steinberg::{
    IPlugFrame, IPlugView, IPlugViewTrait, TBool, kInvalidArgument, kResultOk,
};
#[cfg(any(feature = "gui", feature = "radiant-vst3", feature = "gpui-vst3"))]
use toybox_vst3_ffi::Steinberg::{char16, int16};

include!("string_conversion.rs");
include!("key_input.rs");
include!("platform_type.rs");
include!("hosted_view_types.rs");
include!("plug_view_impl.rs");
include!("view_rect_utils.rs");

#[cfg(all(
    feature = "radiant-vst3",
    any(target_os = "macos", target_os = "windows")
))]
pub use crate::radiant_gui::{RadiantVst3Editor, RadiantVst3HostedGui};

#[cfg(all(
    feature = "gpui-vst3",
    any(target_os = "macos", target_os = "windows")
))]
impl Vst3HostedGui for crate::gpui_gui::GpuiHostedGui {
    fn set_parent_raw(&mut self, parent: RawWindowHandle) {
        crate::gpui_gui::GpuiHostedGui::set_parent_raw(self, parent);
    }

    fn open(&mut self) -> bool {
        crate::gpui_gui::GpuiHostedGui::open(self)
    }

    fn close(&mut self) {
        crate::gpui_gui::GpuiHostedGui::close(self);
    }

    fn last_size(&self) -> Option<(u32, u32)> {
        crate::gpui_gui::GpuiHostedGui::last_size(self)
    }

    fn show(&self) -> bool {
        crate::gpui_gui::GpuiHostedGui::show(self)
    }

    fn set_callback_keyboard_mode(&mut self, callback_only: bool) {
        crate::gpui_gui::GpuiHostedGui::set_callback_keyboard_mode(self, callback_only);
    }

    fn request_resize(&self, width: u32, height: u32) {
        crate::gpui_gui::GpuiHostedGui::request_resize(self, width, height);
    }

    fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32) {
        crate::gpui_gui::GpuiHostedGui::host_size_from_logical(self, width, height)
    }

    fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32) {
        crate::gpui_gui::GpuiHostedGui::logical_size_from_host(self, width, height)
    }

    fn on_focus(&self, focused: bool) -> bool {
        crate::gpui_gui::GpuiHostedGui::on_focus(self, focused)
    }

    fn on_key_down(&self, key: char16, key_code: int16, modifiers: int16) -> bool {
        crate::gpui_gui::GpuiHostedGui::on_key_down(self, key, key_code, modifiers)
    }

    fn on_key_up(&self, key: char16, key_code: int16, modifiers: int16) -> bool {
        crate::gpui_gui::GpuiHostedGui::on_key_up(self, key, key_code, modifiers)
    }
}

/// Wrap a GPUI host facade in Toybox's reusable VST3 `IPlugView`.
#[cfg(all(
    feature = "gpui-vst3",
    any(target_os = "macos", target_os = "windows")
))]
pub fn create_gpui_view(
    gui: crate::gpui_gui::GpuiHostedGui,
    width: u32,
    height: u32,
    minimum: (u32, u32),
    maximum: (u32, u32),
) -> HostedVst3View<crate::gpui_gui::GpuiHostedGui> {
    HostedVst3View::new(gui, width, height).with_size_bounds(
        minimum.0,
        minimum.1,
        maximum.0,
        maximum.1,
    )
}

#[cfg(all(test, any(feature = "gui", feature = "radiant-vst3", feature = "gpui-vst3")))]
mod tests;
