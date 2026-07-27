//! Portable fallback for platforms without a native Radiant child host yet.

use super::{RadiantEditor, Vst3HostedGui};
use radiant::runtime::Event;
use raw_window_handle::RawWindowHandle;
use std::sync::Mutex;

/// State-preserving host stub used to keep the host-neutral API portable.
pub struct RadiantVst3HostedGui {
    editor: Mutex<Option<Box<dyn RadiantEditor>>>,
    size: Mutex<(u32, u32)>,
    open: bool,
    visible: bool,
}

impl RadiantVst3HostedGui {
    /// Construct a portable editor host.
    pub fn new(
        _class_name: &'static str,
        editor: impl RadiantEditor,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            editor: Mutex::new(Some(Box::new(editor))),
            size: Mutex::new((width.max(1), height.max(1))),
            open: false,
            visible: false,
        }
    }

    /// Show the logical child host.
    pub fn show(&mut self) {
        self.visible = self.open;
    }

    /// Hide the logical child host.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Apply a DPI change to the logical editor.
    pub fn set_scale(&self, _scale: f64) {}
}

impl Vst3HostedGui for RadiantVst3HostedGui {
    fn set_parent_raw(&mut self, _parent: RawWindowHandle) {}

    fn open(&mut self) -> bool {
        if let (Ok(mut editor), Ok(size)) = (self.editor.lock(), self.size.lock()) {
            let Some(editor) = editor.as_mut() else {
                return false;
            };
            editor.resize(size.0, size.1);
            editor.dispatch_event(Event::resize(radiant::gui::types::Vector2::new(
                size.0 as f32,
                size.1 as f32,
            )));
            self.open = true;
            self.visible = true;
            true
        } else {
            false
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.open = false;
    }

    fn last_size(&self) -> Option<(u32, u32)> {
        self.size.lock().ok().map(|size| *size)
    }

    fn request_resize(&self, width: u32, height: u32) {
        if let Ok(mut size) = self.size.lock() {
            *size = (width.max(1), height.max(1));
        }
        if let Ok(mut editor) = self.editor.lock() {
            let Some(editor) = editor.as_mut() else {
                return;
            };
            editor.resize(width.max(1), height.max(1));
            editor.dispatch_event(Event::resize(radiant::gui::types::Vector2::new(
                width.max(1) as f32,
                height.max(1) as f32,
            )));
        }
    }

    fn on_key_down(&self, _key: u16, _key_code: i16, _modifiers: i16) -> bool {
        false
    }
}
