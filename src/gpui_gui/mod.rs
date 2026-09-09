//! Embedded GPUI editor hosting for native plugin children.

#![allow(clippy::missing_docs_in_private_items)]

mod numeric_input;
mod platform;

pub use numeric_input::{
    NumericInput, NumericInputCanceled, NumericInputChanged, NumericInputConfig, NumericInputRange,
    NumericInputStepped, NumericInputStyle, NumericInputSubmitted,
};

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{AnyView, App, AppContext, Application, ApplicationHandle, IntoElement, Render, Window};
use raw_window_handle::RawWindowHandle;

use self::platform::EmbeddedPlatform;

type ViewFactory = dyn Fn(&mut gpui::Window, &mut App) -> AnyView;

/// A logical editor size contract used by GPUI host resize negotiation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuiSizeContract {
    /// Inclusive minimum logical size.
    pub minimum: (u32, u32),
    /// Preferred logical size used when opening the editor.
    pub preferred: (u32, u32),
    /// Inclusive maximum logical size.
    pub maximum: (u32, u32),
}

impl GpuiSizeContract {
    fn new(preferred: (u32, u32), minimum: (u32, u32), maximum: (u32, u32)) -> Self {
        let preferred = (preferred.0.max(1), preferred.1.max(1));
        Self {
            minimum: (
                minimum.0.max(1).min(preferred.0),
                minimum.1.max(1).min(preferred.1),
            ),
            preferred,
            maximum: (maximum.0.max(preferred.0), maximum.1.max(preferred.1)),
        }
    }

    fn constrain(self, requested: (u32, u32)) -> (u32, u32) {
        (
            requested.0.clamp(self.minimum.0, self.maximum.0).max(1),
            requested.1.clamp(self.minimum.1, self.maximum.1).max(1),
        )
    }

    fn constrain_fixed_aspect(self, requested: (u32, u32)) -> (u32, u32) {
        let common = gcd(self.preferred.0, self.preferred.1);
        let unit_width = self.preferred.0 / common;
        let unit_height = self.preferred.1 / common;
        let minimum_multiplier =
            ceil_div(self.minimum.0, unit_width).max(ceil_div(self.minimum.1, unit_height));
        let maximum_multiplier =
            u64::from(self.maximum.0 / unit_width).min(u64::from(self.maximum.1 / unit_height));
        let requested_multiplier = u64::from(requested.0.max(1) / unit_width)
            .min(u64::from(requested.1.max(1) / unit_height));
        let multiplier = requested_multiplier.clamp(minimum_multiplier, maximum_multiplier);
        (
            (u64::from(unit_width) * multiplier) as u32,
            (u64::from(unit_height) * multiplier) as u32,
        )
    }
}

fn gcd(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

fn ceil_div(value: u32, divisor: u32) -> u64 {
    u64::from(value).div_ceil(u64::from(divisor))
}

fn max_logical_dimension_for_host(host_size: u32, scale: f32) -> u32 {
    let fits = |logical_size| platform::logical_to_host_size(logical_size, 1, scale).0 <= host_size;
    if !fits(1) {
        return 1;
    }
    let mut lower = 1_u32;
    let mut upper = u32::MAX;
    while lower < upper {
        let candidate = lower + (upper - lower).div_ceil(2);
        if fits(candidate) {
            lower = candidate;
        } else {
            upper = candidate - 1;
        }
    }
    lower
}

fn constrain_host_size_at_scale(
    contract: GpuiSizeContract,
    fixed_aspect_ratio: bool,
    width: u32,
    height: u32,
    scale: f32,
) -> (u32, u32) {
    let logical_budget = (
        max_logical_dimension_for_host(width, scale),
        max_logical_dimension_for_host(height, scale),
    );
    let logical_size = if fixed_aspect_ratio {
        contract.constrain_fixed_aspect(logical_budget)
    } else {
        contract.constrain(logical_budget)
    };
    platform::logical_to_host_size(logical_size.0, logical_size.1, scale)
}

struct AnyRoot {
    view: AnyView,
}

impl Render for AnyRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        self.view.clone()
    }
}

struct AppDriver {
    handle: RefCell<Option<ApplicationHandle>>,
    failed: Cell<bool>,
}

impl AppDriver {
    fn set_handle(&self, handle: ApplicationHandle) {
        self.handle.replace(Some(handle));
    }

    fn take_handle(&self) -> Option<ApplicationHandle> {
        self.handle.borrow_mut().take()
    }
}

struct Runtime {
    platform: Rc<EmbeddedPlatform>,
    driver: Rc<AppDriver>,
    _window: gpui::AnyWindowHandle,
}

/// A GPUI view hosted in a foreign native child window.
///
/// The host owns the native run loop. Call [`Self::pump`] from that loop when
/// the host gives the plugin UI time to process queued GPUI work.
pub struct GpuiHostedGui {
    class_name: &'static str,
    factory: Rc<ViewFactory>,
    parent: Option<RawWindowHandle>,
    size: Cell<Option<(u32, u32)>>,
    contract: GpuiSizeContract,
    fixed_aspect_ratio: bool,
    visibility_callback: platform::VisibilityCallback,
    pointer_cancel_callback: platform::PointerCancelCallback,
    runtime: Option<Runtime>,
    callback_keyboard_only: bool,
}

impl GpuiHostedGui {
    /// Create a hostable GPUI view factory with its preferred logical size.
    pub fn new(
        class_name: &'static str,
        factory: impl Fn(&mut gpui::Window, &mut App) -> AnyView + 'static,
        width: u32,
        height: u32,
    ) -> Self {
        let preferred = (width.max(1), height.max(1));
        Self {
            class_name,
            factory: Rc::new(factory),
            parent: None,
            size: Cell::new(Some(preferred)),
            contract: GpuiSizeContract::new(preferred, (1, 1), (u32::MAX, u32::MAX)),
            fixed_aspect_ratio: false,
            visibility_callback: Rc::new(RefCell::new(None)),
            pointer_cancel_callback: Rc::new(RefCell::new(None)),
            runtime: None,
            callback_keyboard_only: false,
        }
    }

    /// Apply an inclusive logical minimum, preferred, and maximum size.
    pub fn with_size_contract(
        mut self,
        minimum: (u32, u32),
        preferred: (u32, u32),
        maximum: (u32, u32),
    ) -> Self {
        self.contract = GpuiSizeContract::new(preferred, minimum, maximum);
        self.size.set(Some(self.contract.preferred));
        self
    }

    /// Preserve the preferred width-to-height ratio during host resizing.
    pub fn with_fixed_aspect_ratio(mut self) -> Self {
        self.fixed_aspect_ratio = true;
        self
    }

    /// Observe effective native visibility changes on the host UI thread.
    pub fn with_visibility_callback(self, callback: impl FnMut(bool) + 'static) -> Self {
        *self.visibility_callback.borrow_mut() = Some(Box::new(callback));
        self
    }

    /// Observe native pointer cancellation on the host UI thread.
    ///
    /// The callback is deferred through the embedded GPUI gateway and runs
    /// when the native child loses pointer capture or focus. No synthetic
    /// mouse-up or keyboard event is emitted for the cancellation.
    pub fn with_pointer_cancel_callback(self, callback: impl FnMut() + 'static) -> Self {
        *self.pointer_cancel_callback.borrow_mut() = Some(Box::new(callback));
        self
    }

    /// Attach a VST3 or CLAP parent window handle.
    pub fn set_parent_raw(&mut self, parent: RawWindowHandle) {
        self.parent = Some(parent);
    }

    /// Attach the parent supplied by a CLAP GUI callback.
    pub fn set_parent(&mut self, _window: clack_extensions::gui::Window<'_>) {
        #[cfg(target_os = "macos")]
        if let Some(ns_view) = _window.as_cocoa_nsview() {
            let mut handle = raw_window_handle::AppKitWindowHandle::empty();
            handle.ns_view = ns_view;
            self.parent = Some(RawWindowHandle::AppKit(handle));
        }

        #[cfg(target_os = "windows")]
        if let Some(hwnd) = _window.as_win32_hwnd() {
            let mut handle = raw_window_handle::Win32WindowHandle::empty();
            handle.hwnd = hwnd;
            handle.hinstance = std::ptr::null_mut();
            self.parent = Some(RawWindowHandle::Win32(handle));
        }
    }

    /// Open the GPUI application and host its root view.
    pub fn open(&mut self) -> bool {
        let _ = self.class_name;
        if self.runtime.is_some() || self.parent.is_none() {
            return self.runtime.is_some();
        }
        let parent = self.parent.expect("parent checked above");
        let Ok(platform) = EmbeddedPlatform::new(
            parent,
            self.class_name,
            self.callback_keyboard_only,
            self.visibility_callback.clone(),
            self.pointer_cancel_callback.clone(),
        ) else {
            return false;
        };
        let driver = Rc::new(AppDriver {
            handle: RefCell::new(None),
            failed: Cell::new(false),
        });
        let opened_window = Rc::new(Cell::new(None));
        let opened_window_for_callback = opened_window.clone();
        let factory = self.factory.clone();
        let (width, height) = self
            .size
            .get()
            .map(|size| self.constrain_logical_size(size))
            .unwrap_or(self.contract.preferred);
        let application = Application::with_platform(platform.clone());
        let Some(handle) = crate::gui_panic::contain("GPUI open", || {
            application.run_embedded(move |cx| {
                let options = gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::new(
                        gpui::point(gpui::px(0.0), gpui::px(0.0)),
                        gpui::size(gpui::px(width as f32), gpui::px(height as f32)),
                    ))),
                    show: true,
                    focus: true,
                    kind: gpui::WindowKind::Normal,
                    is_movable: false,
                    is_resizable: true,
                    is_minimizable: false,
                    ..Default::default()
                };
                let Ok(window) = cx.open_window(options, move |window, cx| {
                    let view = factory(window, cx);
                    cx.new(|_| AnyRoot { view })
                }) else {
                    return;
                };
                opened_window_for_callback.set(Some(window.into()));
            })
        }) else {
            driver.failed.set(true);
            platform.stop();
            return false;
        };
        let Some(window) = opened_window.get() else {
            driver.failed.set(true);
            platform.stop();
            return false;
        };
        driver.set_handle(handle);
        self.size.set(Some((width, height)));
        self.runtime = Some(Runtime {
            platform,
            driver,
            _window: window,
        });
        self.notify_visibility(true);
        true
    }

    /// Stop native callbacks and release the GPUI application handle.
    pub fn close(&mut self) {
        let Some(runtime) = self.runtime.take() else {
            return;
        };
        self.notify_visibility(false);
        runtime.platform.stop();
        let _ = runtime.driver.take_handle();
    }

    /// Process queued foreground work on the host UI thread.
    pub fn pump(&self) {
        if let Some(runtime) = &self.runtime {
            runtime.platform.pump();
            runtime.platform.sync_visibility();
            if runtime.platform.failed() {
                runtime.driver.failed.set(true);
            }
        }
    }

    /// Render the current GPUI scene into tightly packed physical RGBA8 pixels.
    ///
    /// The host remains responsible for pumping its native event loop. The
    /// capture requests one GPUI frame and waits only through the bounded
    /// embedded pump, so it cannot retain a task or renderer after close.
    pub fn capture_rgba(&self) -> anyhow::Result<(u32, u32, Vec<u8>)> {
        self.runtime
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("GPUI editor is not open"))?
            .platform
            .capture_rgba()
    }

    /// Return the latest host-facing logical size.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        let (width, height) = self.size.get()?;
        Some(self.host_size_from_logical(width, height))
    }

    /// Constrain a host-facing size to the logical contract and convert it
    /// back to the host's units.
    pub fn constrain_host_size(&self, width: u32, height: u32) -> (u32, u32) {
        constrain_host_size_at_scale(
            self.contract,
            self.fixed_aspect_ratio,
            width,
            height,
            self.host_scale_factor(),
        )
    }

    /// Return CLAP resize hints for this editor's resize policy.
    pub fn resize_hints(&self) -> clack_extensions::gui::GuiResizeHints {
        let strategy = if self.fixed_aspect_ratio {
            clack_extensions::gui::AspectRatioStrategy::Preserve {
                width: self.contract.preferred.0,
                height: self.contract.preferred.1,
            }
        } else {
            clack_extensions::gui::AspectRatioStrategy::Disregard
        };
        clack_extensions::gui::GuiResizeHints {
            can_resize_horizontally: true,
            can_resize_vertically: true,
            strategy,
        }
    }

    /// Show the already-open view.
    pub fn show(&self) -> bool {
        let open = self.runtime.is_some();
        if open {
            if let Some(runtime) = &self.runtime {
                runtime.platform.show(true);
            }
            self.notify_visibility(true);
        }
        open
    }

    /// Select callback-only VST3 keyboard delivery.
    pub fn set_callback_keyboard_mode(&mut self, callback_only: bool) {
        self.callback_keyboard_only = callback_only;
    }

    /// Convert logical editor dimensions to host dimensions.
    pub fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32) {
        self.runtime.as_ref().map_or_else(
            || platform::logical_to_host_size(width, height, self.host_scale_factor()),
            |runtime| runtime.platform.host_size_from_logical(width, height),
        )
    }

    /// Convert host dimensions to logical editor dimensions.
    pub fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32) {
        self.runtime.as_ref().map_or_else(
            || platform::host_to_logical_size(width, height, self.host_scale_factor()),
            |runtime| runtime.platform.logical_size_from_host(width, height),
        )
    }

    /// Apply a host-driven resize request.
    pub fn request_resize(&self, width: u32, height: u32) {
        let (width, height) = self.logical_size_from_host(width, height);
        self.size
            .set(Some(self.constrain_logical_size((width, height))));
        if let Some(runtime) = &self.runtime {
            let (width, height) = self.size.get().expect("size was just set");
            runtime
                .platform
                .resize(gpui::size(gpui::px(width as f32), gpui::px(height as f32)));
        }
    }

    /// Forward one VST3 key-down callback to GPUI.
    pub fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        self.runtime.as_ref().is_some_and(|runtime| {
            runtime
                .platform
                .dispatch_vst3_key(key, key_code, modifiers, true)
        })
    }

    /// Forward one VST3 key-up callback to GPUI.
    pub fn on_key_up(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        self.runtime.as_ref().is_some_and(|runtime| {
            runtime
                .platform
                .dispatch_vst3_key(key, key_code, modifiers, false)
        })
    }

    /// Apply a host focus request.
    pub fn on_focus(&self, focused: bool) -> bool {
        self.runtime
            .as_ref()
            .is_some_and(|runtime| runtime.platform.focus(focused))
    }

    fn notify_visibility(&self, visible: bool) {
        if let Ok(mut callback) = self.visibility_callback.try_borrow_mut()
            && let Some(callback) = callback.as_mut()
        {
            let _ = crate::gui_panic::contain("GPUI visibility callback", || callback(visible));
        }
    }

    fn host_scale_factor(&self) -> f32 {
        self.runtime.as_ref().map_or_else(
            || self.parent.map_or(1.0, platform::parent_scale_factor),
            |runtime| runtime.platform.scale_factor(),
        )
    }

    fn constrain_logical_size(&self, size: (u32, u32)) -> (u32, u32) {
        if self.fixed_aspect_ratio {
            self.contract.constrain_fixed_aspect(size)
        } else {
            self.contract.constrain(size)
        }
    }
}

impl Drop for GpuiHostedGui {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    use super::GpuiHostedGui;
    use super::{
        GpuiSizeContract, constrain_host_size_at_scale, max_logical_dimension_for_host, platform,
    };

    #[test]
    fn size_contract_keeps_free_aspect_requests_independent() {
        let contract = GpuiSizeContract::new((800, 500), (400, 250), (1200, 750));
        assert_eq!(contract.constrain((900, 700)), (900, 700));
        assert_eq!(contract.constrain((200, 1)), (400, 250));
        assert_eq!(contract.constrain((2000, 2000)), (1200, 750));
    }

    #[test]
    fn fixed_aspect_contract_uses_preferred_ratio_inside_bounds() {
        let contract = GpuiSizeContract::new((800, 500), (400, 250), (1200, 750));
        assert_eq!(contract.constrain_fixed_aspect((800, 700)), (800, 500));
        assert_eq!(contract.constrain_fixed_aspect((300, 200)), (400, 250));
        assert_eq!(contract.constrain_fixed_aspect((2000, 2000)), (1200, 750));
        let adjusted = contract.constrain_fixed_aspect((407, 255));
        assert_eq!(adjusted, (400, 250));
        assert_eq!(contract.constrain_fixed_aspect(adjusted), adjusted);
    }

    #[test]
    fn host_budget_uses_largest_roundtrip_safe_logical_dimension() {
        assert_eq!(max_logical_dimension_for_host(1002, 1.25), 801);
        assert_eq!(platform::logical_to_host_size(801, 1, 1.25).0, 1001);
        assert!(platform::logical_to_host_size(802, 1, 1.25).0 > 1002);

        let contract = GpuiSizeContract::new((800, 500), (400, 250), (1200, 750));
        for (width, height) in [(1001, 625), (1002, 627), (1252, 782), (1600, 1000)] {
            let adjusted = constrain_host_size_at_scale(contract, true, width, height, 1.25);
            assert!(adjusted.0 <= width && adjusted.1 <= height);
            assert_eq!(
                constrain_host_size_at_scale(contract, true, adjusted.0, adjusted.1, 1.25,),
                adjusted
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_host_size_conversion_stays_in_logical_points() {
        let gui = GpuiHostedGui::new(
            "toybox-gpui-size-test",
            |_window, _cx| panic!("size tests never open a native editor"),
            800,
            500,
        );
        assert_eq!(gui.host_size_from_logical(800, 500), (800, 500));
        assert_eq!(gui.logical_size_from_host(800, 500), (800, 500));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_host_size_helpers_apply_fractional_dpi_once() {
        assert_eq!(
            super::platform::logical_to_host_size(800, 500, 1.25),
            (1000, 625)
        );
        assert_eq!(
            super::platform::host_to_logical_size(1000, 625, 1.25),
            (800, 500)
        );
    }
}
