//! Egui/baseview GUI helpers for CLAP plugins.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use baseview::{Size, WindowHandle, WindowOpenOptions, WindowScalePolicy};
use clack_plugin::plugin::PluginError;
use egui_baseview::{egui::Context, EguiWindow, GraphicsConfig, Queue};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// Packed size stored as (width << 32) | height.
fn pack_size(width: u32, height: u32) -> u64 {
    ((width as u64) << 32) | (height as u64)
}

fn unpack_size(value: u64) -> Option<(u32, u32)> {
    if value == 0 {
        return None;
    }
    let width = (value >> 32) as u32;
    let height = (value & 0xFFFF_FFFF) as u32;
    Some((width, height))
}

/// Wrapper around a baseview window for an egui-based CLAP editor.
pub struct EguiHostWindow {
    /// Raw handle to the parent window provided by the host.
    parent: Option<RawWindowHandle>,
    /// Handle to the created editor window.
    handle: Option<WindowHandle>,
    /// Pending logical resize request from the host.
    resize_request: Arc<AtomicU64>,
    /// Most recent logical size reported by the host.
    last_size: Arc<AtomicU64>,
}

impl Default for EguiHostWindow {
    fn default() -> Self {
        Self {
            parent: None,
            handle: None,
            resize_request: Arc::new(AtomicU64::new(0)),
            last_size: Arc::new(AtomicU64::new(0)),
        }
    }
}

unsafe impl HasRawWindowHandle for EguiHostWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.parent.unwrap()
    }
}

impl EguiHostWindow {
    /// Set the raw parent handle provided by the host.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
        self.parent = Some(parent);
    }

    /// Return the most recently observed logical size, if any.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        unpack_size(self.last_size.load(Ordering::Acquire))
    }

    /// Request a logical resize from the GUI thread.
    pub fn request_resize(&self, width: u32, height: u32) {
        self.resize_request
            .store(pack_size(width, height), Ordering::Release);
    }

    /// Open a parented egui/baseview window.
    ///
    /// The caller supplies the initial state and the egui update callback. The
    /// helper handles resize requests and stores the last logical size.
    pub fn open_parented<State, Init, Frame>(
        &mut self,
        title: String,
        size: (f64, f64),
        graphics: GraphicsConfig,
        state: State,
        mut on_init: Init,
        mut on_frame: Frame,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&Context, &mut Queue, &mut State) + 'static,
        Frame: FnMut(&Context, &mut Queue, &mut State) + 'static,
        State: 'static,
    {
        if self.parent.is_none() {
            return Err(PluginError::Message("No parent window provided"));
        }

        let settings = WindowOpenOptions {
            title,
            size: Size::new(size.0, size.1),
            scale: WindowScalePolicy::SystemScaleFactor,
            gl_config: Some(Default::default()),
        };

        let resize_request = self.resize_request.clone();
        let last_size = self.last_size.clone();

        self.handle = Some(EguiWindow::open_parented(
            self,
            settings,
            graphics,
            state,
            move |ctx: &Context, queue: &mut Queue, state: &mut State| {
                on_init(ctx, queue, state);
            },
            move |ctx: &Context, queue: &mut Queue, state: &mut State| {
                if let Some((width, height)) = unpack_size(resize_request.swap(0, Ordering::AcqRel))
                {
                    ctx.send_viewport_cmd(egui_baseview::egui::ViewportCommand::InnerSize(
                        egui_baseview::egui::vec2(width as f32, height as f32),
                    ));
                    last_size.store(pack_size(width, height), Ordering::Release);
                }

                let content_rect = ctx.input(|input| input.content_rect());
                let logical_width = content_rect.width().round().max(1.0) as u32;
                let logical_height = content_rect.height().round().max(1.0) as u32;
                last_size.store(pack_size(logical_width, logical_height), Ordering::Release);

                on_frame(ctx, queue, state);
            },
        ));

        Ok(())
    }
}
