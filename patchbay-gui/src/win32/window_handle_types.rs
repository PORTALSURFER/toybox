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
