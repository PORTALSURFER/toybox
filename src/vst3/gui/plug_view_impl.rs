#[cfg(feature = "gui")]
impl<G: Vst3HostedGui> IPlugViewTrait for HostedVst3View<G> {
    unsafe fn isPlatformTypeSupported(&self, r#type: FIDString) -> tresult {
        #[cfg(target_os = "windows")]
        {
            bool_to_tresult(unsafe { platform_type_matches(r#type, kPlatformTypeHWND) })
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = r#type;
            kResultFalse
        }
    }

    unsafe fn attached(&self, parent: *mut std::ffi::c_void, r#type: FIDString) -> tresult {
        if parent.is_null() {
            return kInvalidArgument;
        }

        let Some(parent_handle) = (unsafe { parent_to_raw_window_handle(parent, r#type) }) else {
            return kResultFalse;
        };

        let Ok(mut gui) = self.gui.lock() else {
            return kResultFalse;
        };
        gui.set_parent_raw(parent_handle);
        if !gui.open() {
            return kResultFalse;
        }
        let (min_width, min_height) = self.minimum_size();
        let (requested_width, requested_height) = if let Some((width, height)) = gui.last_size() {
            (width as i32, height as i32)
        } else {
            (min_width, min_height)
        };
        let (constrained_width, constrained_height) =
            self.constrain_uniform_size(requested_width, requested_height);
        if constrained_width != requested_width || constrained_height != requested_height {
            gui.request_resize(constrained_width as u32, constrained_height as u32);
        }
        self.rect.set(view_rect(constrained_width, constrained_height));

        self.attached.set(true);
        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        if let Ok(mut gui) = self.gui.lock() {
            gui.close();
        }
        self.attached.set(false);
        kResultOk
    }

    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyDown(&self, _key: char16, _key_code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyUp(&self, _key: char16, _key_code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kInvalidArgument;
        }
        self.sync_rect_from_gui();
        unsafe { *size = self.rect.get() };
        kResultOk
    }

    unsafe fn onSize(&self, new_size: *mut ViewRect) -> tresult {
        if new_size.is_null() {
            return kInvalidArgument;
        }

        let requested = unsafe { *new_size };
        let requested_width = (requested.right - requested.left).max(1);
        let requested_height = (requested.bottom - requested.top).max(1);
        let (constrained_width, constrained_height) =
            self.constrain_uniform_size(requested_width, requested_height);
        let constrained = view_rect(constrained_width, constrained_height);
        unsafe { *new_size = constrained };
        if let Ok(gui) = self.gui.lock() {
            gui.request_resize(constrained_width as u32, constrained_height as u32);
        }
        self.rect.set(constrained);
        kResultOk
    }

    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setFrame(&self, _frame: *mut IPlugFrame) -> tresult {
        kResultOk
    }

    unsafe fn canResize(&self) -> tresult {
        kResultTrue
    }

    unsafe fn checkSizeConstraint(&self, rect: *mut ViewRect) -> tresult {
        if rect.is_null() {
            return kInvalidArgument;
        }
        let rect = unsafe { &mut *rect };
        let requested_width = (rect.right - rect.left).max(1);
        let requested_height = (rect.bottom - rect.top).max(1);
        let (constrained_width, constrained_height) =
            self.constrain_uniform_size(requested_width, requested_height);
        rect.right = rect.left + constrained_width;
        rect.bottom = rect.top + constrained_height;
        kResultOk
    }
}
