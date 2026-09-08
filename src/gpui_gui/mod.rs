//! Embedded GPUI editor hosting for native plugin children.

#![allow(clippy::missing_docs_in_private_items)]

mod platform;

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
    visibility_callback: platform::VisibilityCallback,
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
            visibility_callback: Rc::new(RefCell::new(None)),
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

    /// Observe effective native visibility changes on the host UI thread.
    pub fn with_visibility_callback(self, callback: impl FnMut(bool) + 'static) -> Self {
        *self.visibility_callback.borrow_mut() = Some(Box::new(callback));
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
            .map(|size| self.contract.constrain(size))
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
            .set(Some(self.contract.constrain((width, height))));
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
}

impl Drop for GpuiHostedGui {
    fn drop(&mut self) {
        self.close();
    }
}
