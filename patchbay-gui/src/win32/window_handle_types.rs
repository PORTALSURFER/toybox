/// Thin wrapper around an HWND for cross-thread use.
#[derive(Clone, Debug)]
pub struct WindowHandle {
    hwnd: HWND,
}

impl WindowHandle {
    /// Return the underlying HWND.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Show or hide the window.
    pub fn set_visible(&self, visible: bool) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }

    /// Return true if the HWND is still valid.
    pub fn is_valid(&self) -> bool {
        unsafe { windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(self.hwnd)).as_bool() }
    }

    /// Return true if the parent matches the provided HWND.
    pub fn parent_matches(&self, parent: isize) -> bool {
        unsafe { GetParent(self.hwnd).ok() == Some(HWND(parent as *mut _)) }
    }

    /// Destroy the underlying HWND.
    pub fn destroy(&self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }

    /// Post one Unicode character to the window's input queue.
    ///
    /// Returns `false` when the scalar value cannot be represented as one
    /// UTF-16 unit or when `PostMessageW` fails.
    pub fn post_text_char(&self, ch: char) -> bool {
        let mut units = [0u16; 2];
        let encoded = ch.encode_utf16(&mut units);
        if encoded.len() != 1 {
            return false;
        }
        unsafe {
            PostMessageW(
                Some(self.hwnd),
                WM_CHAR,
                WPARAM(encoded[0] as usize),
                LPARAM(0),
            )
            .is_ok()
        }
    }

    /// Post one injected Unicode character with explicit modifier flags.
    ///
    /// This uses a private Patchbay window message so input handling can
    /// distinguish host-injected keys (for example VST3 `onKeyDown`) from
    /// native `WM_CHAR` delivery and dedupe as needed.
    pub fn post_injected_text_char(&self, ch: char, modifiers: ShortcutModifiers) -> bool {
        let mut units = [0u16; 2];
        let encoded = ch.encode_utf16(&mut units);
        if encoded.len() != 1 {
            return false;
        }
        unsafe {
            PostMessageW(
                Some(self.hwnd),
                PATCHBAY_MSG_INJECTED_CHAR,
                WPARAM(encoded[0] as usize),
                LPARAM(modifiers.to_bits() as isize),
            )
            .is_ok()
        }
    }

    /// Post one injected key-up event with explicit modifier flags.
    pub fn post_injected_key_up(&self, ch: char, modifiers: ShortcutModifiers) -> bool {
        let mut units = [0u16; 2];
        let encoded = ch.encode_utf16(&mut units);
        if encoded.len() != 1 {
            return false;
        }
        unsafe {
            PostMessageW(
                Some(self.hwnd),
                PATCHBAY_MSG_INJECTED_KEY_UP,
                WPARAM(encoded[0] as usize),
                LPARAM(modifiers.to_bits() as isize),
            )
            .is_ok()
        }
    }

    /// Request one immediate render tick that fulfills a pending frame capture.
    #[cfg(feature = "frame-capture")]
    pub fn request_frame_capture(&self) -> bool {
        if !self.is_valid() {
            return false;
        }
        // Send synchronously so capture works even when tests do not pump a
        // normal Win32 message loop between API calls.
        unsafe {
            let _ = SendMessageW(
                self.hwnd,
                PATCHBAY_MSG_CAPTURE_FRAME,
                Some(WPARAM(0)),
                Some(LPARAM(0)),
            );
        }
        true
    }
}

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

/// A window type that exposes raw window handles for wgpu surfaces.
pub struct SurfaceWindow {
    hwnd: HWND,
    hinstance: HINSTANCE,
}

impl HasWindowHandle for SurfaceWindow {
    fn window_handle(&self) -> Result<WindowHandle06<'_>, raw_window_handle_06::HandleError> {
        let Some(hwnd) = NonZeroIsize::new(self.hwnd.0 as isize) else {
            return Err(HandleError::Unavailable);
        };
        let mut handle = Win32WindowHandle::new(hwnd);
        if let Some(hinstance) = NonZeroIsize::new(self.hinstance.0 as isize) {
            handle.hinstance = Some(hinstance);
        }
        Ok(unsafe { WindowHandle06::borrow_raw(RawWindowHandle06::Win32(handle)) })
    }
}

impl HasDisplayHandle for SurfaceWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle_06::HandleError> {
        let display = WindowsDisplayHandle::new();
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Windows(display)) })
    }
}

unsafe impl Send for SurfaceWindow {}
unsafe impl Sync for SurfaceWindow {}
