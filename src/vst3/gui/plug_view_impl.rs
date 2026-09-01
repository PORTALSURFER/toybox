#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
impl<G: Vst3HostedGui> IPlugViewTrait for HostedVst3View<G> {
    unsafe fn isPlatformTypeSupported(&self, r#type: FIDString) -> tresult {
        #[cfg(target_os = "windows")]
        {
            bool_to_tresult(unsafe { platform_type_matches(r#type, kPlatformTypeHWND) })
        }

        #[cfg(target_os = "macos")]
        {
            bool_to_tresult(unsafe { platform_type_matches(r#type, kPlatformTypeNSView) })
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
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
        gui.set_callback_keyboard_mode(true);
        gui.set_parent_raw(parent_handle);
        if !gui.open() {
            return kResultFalse;
        }
        let (requested_host_width, requested_host_height) =
            if let Some((width, height)) = gui.last_size() {
                (width, height)
            } else {
                gui.host_size_from_logical(self.default_size.0 as u32, self.default_size.1 as u32)
            };
        let (requested_width, requested_height) =
            gui.logical_size_from_host(requested_host_width, requested_host_height);
        let current = self.current_logical_size_for_gui(&gui);
        let (constrained_width, constrained_height) = self.constrain_uniform_size_from_current(
            logical_dimension(requested_width),
            logical_dimension(requested_height),
            current,
        );
        let (constrained_host_width, constrained_host_height) = gui.host_size_from_logical(
            constrained_width as u32,
            constrained_height as u32,
        );
        if constrained_host_width != requested_host_width
            || constrained_host_height != requested_host_height
        {
            gui.request_resize(constrained_host_width, constrained_host_height);
        }
        self.rect.set(view_rect(
            logical_dimension(constrained_host_width),
            logical_dimension(constrained_host_height),
        ));
        if !gui.show() {
            gui.close();
            return kResultFalse;
        }

        self.attached.set(true);
        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        if let Ok(mut gui) = self.gui.lock() {
            gui.close();
            gui.set_callback_keyboard_mode(false);
        }
        self.attached.set(false);
        kResultOk
    }

    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyDown(&self, key: char16, key_code: int16, modifiers: int16) -> tresult {
        let Ok(gui) = self.gui.lock() else {
            return kResultFalse;
        };
        bool_to_tresult(gui.on_key_down(key, key_code, modifiers))
    }

    unsafe fn onKeyUp(&self, key: char16, key_code: int16, modifiers: int16) -> tresult {
        let Ok(gui) = self.gui.lock() else {
            return kResultFalse;
        };
        bool_to_tresult(gui.on_key_up(key, key_code, modifiers))
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
        let requested_host_width = requested.right.saturating_sub(requested.left).max(1);
        let requested_host_height = requested.bottom.saturating_sub(requested.top).max(1);
        let Ok(gui) = self.gui.lock() else {
            return kResultFalse;
        };
        let (requested_width, requested_height) =
            gui.logical_size_from_host(requested_host_width as u32, requested_host_height as u32);
        let current = self.current_logical_size_for_gui(&gui);
        let (constrained_width, constrained_height) = self.constrain_uniform_size_from_current(
            logical_dimension(requested_width),
            logical_dimension(requested_height),
            current,
        );
        let (constrained_host_width, constrained_host_height) = gui.host_size_from_logical(
            constrained_width as u32,
            constrained_height as u32,
        );
        let Some(constrained) = view_rect_with_origin(
            requested.left,
            requested.top,
            logical_dimension(constrained_host_width),
            logical_dimension(constrained_host_height),
        ) else {
            return kResultFalse;
        };
        unsafe { *new_size = constrained };
        gui.request_resize(constrained_host_width, constrained_host_height);
        self.rect.set(constrained);
        kResultOk
    }

    unsafe fn onFocus(&self, state: TBool) -> tresult {
        let Ok(gui) = self.gui.lock() else {
            return kResultFalse;
        };
        if gui.on_focus(state != 0) {
            kResultOk
        } else {
            kResultFalse
        }
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
        let requested_host_width = rect.right.saturating_sub(rect.left).max(1);
        let requested_host_height = rect.bottom.saturating_sub(rect.top).max(1);
        let Ok(gui) = self.gui.lock() else {
            return kResultFalse;
        };
        let (requested_width, requested_height) =
            gui.logical_size_from_host(requested_host_width as u32, requested_host_height as u32);
        let current = self.current_logical_size_for_gui(&gui);
        let (constrained_width, constrained_height) = self.constrain_uniform_size_from_current(
            logical_dimension(requested_width),
            logical_dimension(requested_height),
            current,
        );
        let (constrained_host_width, constrained_host_height) = gui.host_size_from_logical(
            constrained_width as u32,
            constrained_height as u32,
        );
        let Some(constrained) = view_rect_with_origin(
            rect.left,
            rect.top,
            logical_dimension(constrained_host_width),
            logical_dimension(constrained_host_height),
        ) else {
            return kResultFalse;
        };
        *rect = constrained;
        kResultOk
    }
}
