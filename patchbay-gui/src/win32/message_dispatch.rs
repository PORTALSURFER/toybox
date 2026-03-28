impl<State, Init, Build, Reduce> WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    fn handle_message(
        &mut self,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        self.handle_frame_messages(message, wparam, lparam)
            .or_else(|| self.handle_pointer_messages(message, wparam, lparam))
            .or_else(|| self.handle_input_messages(message, wparam, lparam))
            .or_else(|| self.handle_paint_timer_messages(message, wparam))
    }

    fn handle_frame_messages(
        &mut self,
        message: u32,
        _wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match message {
            WM_SIZE => {
                let raw_size = lparam.0 as u32;
                let width = raw_size & 0xFFFF;
                let height = (raw_size >> 16) & 0xFFFF;
                self.on_resize_from_message(width, height);
                Some(LRESULT(0))
            }
            WM_NCHITTEST => Some(LRESULT(HTCLIENT as isize)),
            WM_MOUSEACTIVATE => Some(LRESULT(MA_ACTIVATE as isize)),
            WM_DESTROY => Some(LRESULT(0)),
            _ => None,
        }
    }

    fn handle_pointer_messages(
        &mut self,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match message {
            WM_MOUSEMOVE => {
                self.arm_mouse_leave_tracking();
                self.input.pointer_pos = Point {
                    x: (lparam.0 & 0xFFFF) as i16 as i32,
                    y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
                };
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_MOUSELEAVE => self.handle_mouse_leave(),
            WM_LBUTTONDOWN => self.handle_primary_button_down(false),
            WM_LBUTTONDBLCLK => self.handle_primary_button_down(true),
            WM_LBUTTONUP => self.handle_primary_button_up(),
            WM_RBUTTONDOWN => self.handle_secondary_button_down(),
            WM_RBUTTONUP => self.handle_secondary_button_up(),
            WM_MOUSEWHEEL => {
                let delta = ((wparam.0 >> 16) & 0xFFFF) as i16 as f32 / 120.0;
                self.input.wheel_delta += delta;
                self.render_frame();
                Some(LRESULT(0))
            }
            _ => None,
        }
    }

    /// Register one-shot leave tracking so hover is cleared when the pointer
    /// exits the plugin client area.
    fn arm_mouse_leave_tracking(&mut self) {
        if self.mouse_leave_tracking_armed {
            return;
        }
        let mut tracking = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: self.hwnd,
            dwHoverTime: 0,
        };
        let ok = unsafe { TrackMouseEvent(&mut tracking).is_ok() };
        if ok {
            self.mouse_leave_tracking_armed = true;
        }
    }

    /// Drop pointer hover state immediately after Win32 reports cursor leave.
    fn handle_mouse_leave(&mut self) -> Option<LRESULT> {
        self.mouse_leave_tracking_armed = false;
        self.input.pointer_in_window = false;
        self.render_frame();
        Some(LRESULT(0))
    }

    fn handle_input_messages(
        &mut self,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match message {
            WM_GETDLGCODE => {
                if self.active_text_edit || self.has_registered_shortcuts() {
                    let flags = DLGC_WANTALLKEYS | DLGC_WANTCHARS;
                    Some(LRESULT(flags as isize))
                } else {
                    None
                }
            }
            WM_DROPFILES => {
                let hdrop = HDROP(wparam.0 as *mut _);
                self.input.dropped_files = collect_dropped_files(hdrop);
                unsafe {
                    DragFinish(hdrop);
                }
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_KEYDOWN => {
                let held_changed = self
                    .key_from_native_message(wparam)
                    .map(|ch| self.set_shortcut_key_down(ch, true))
                    .unwrap_or(false);
                let modifiers = self.current_shortcut_modifiers();
                if let Some(ch) = translate_virtual_key_to_input_char(wparam) {
                    if self.handle_key_char_input(ch, modifiers) {
                        return Some(LRESULT(0));
                    }
                }
                if held_changed {
                    self.render_frame();
                    return Some(LRESULT(0));
                }
                None
            }
            WM_KEYUP => {
                let held_changed = self
                    .key_from_native_message(wparam)
                    .map(|ch| self.set_shortcut_key_down(ch, false))
                    .unwrap_or(false);
                if held_changed {
                    self.render_frame();
                    return Some(LRESULT(0));
                }
                None
            }
            WM_CHAR => {
                let code = (wparam.0 & 0xFFFF) as u16;
                let Some(ch) = char::from_u32(code as u32) else {
                    return None;
                };
                if self.should_dedupe_native_char(ch) {
                    return Some(LRESULT(0));
                }
                let modifiers = self.current_shortcut_modifiers();
                if self.handle_key_char_input(ch, modifiers) {
                    Some(LRESULT(0))
                } else {
                    None
                }
            }
            PATCHBAY_MSG_INJECTED_CHAR => {
                let code = (wparam.0 & 0xFFFF) as u16;
                let Some(ch) = char::from_u32(code as u32) else {
                    return Some(LRESULT(0));
                };
                let modifiers = ShortcutModifiers::from_bits(lparam.0 as usize);
                self.recent_injected_char = Some((ch, Instant::now()));
                let held_changed = self.set_shortcut_key_down(ch, true);
                let handled = self.handle_key_char_input(ch, modifiers);
                if held_changed && !handled {
                    self.render_frame();
                }
                Some(LRESULT(0))
            }
            PATCHBAY_MSG_INJECTED_KEY_UP => {
                let code = (wparam.0 & 0xFFFF) as u16;
                let Some(ch) = char::from_u32(code as u32) else {
                    return Some(LRESULT(0));
                };
                if self.set_shortcut_key_down(ch, false) {
                    self.render_frame();
                }
                Some(LRESULT(0))
            }
            _ => None,
        }
    }

    fn has_registered_shortcuts(&self) -> bool {
        self.shortcut_bindings
            .lock()
            .map(|bindings| !bindings.is_empty())
            .unwrap_or(false)
    }

    fn resolve_shortcut_action(&self, ch: char, modifiers: ShortcutModifiers) -> Option<String> {
        let Ok(bindings) = self.shortcut_bindings.lock() else {
            return None;
        };
        bindings
            .iter()
            .find(|binding| binding.matches(ch, modifiers))
            .map(|binding| binding.action_key.clone())
    }

    fn handle_key_char_input(&mut self, ch: char, modifiers: ShortcutModifiers) -> bool {
        if self.active_text_edit {
            self.input.key_pressed = Some(ch);
            self.render_frame();
            return true;
        }
        let Some(action_key) = self.resolve_shortcut_action(ch, modifiers) else {
            return false;
        };
        let action = UiAction::ButtonPressed { key: action_key };
        invalidate_engine_for_action(&mut self.layout_engine, &action);
        (self.reduce_action)(&mut self.state, action);
        self.render_frame();
        true
    }

    fn should_dedupe_native_char(&mut self, ch: char) -> bool {
        let Some((recent_char, at)) = self.recent_injected_char else {
            return false;
        };
        if at.elapsed().as_millis() > DEDUPE_CHAR_WINDOW_MS {
            self.recent_injected_char = None;
            return false;
        }
        recent_char == ch
    }

    fn current_shortcut_modifiers(&self) -> ShortcutModifiers {
        let shift = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } < 0;
        let alt = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0;
        let ctrl = unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) } < 0;
        ShortcutModifiers::new(shift, alt, ctrl)
    }

    fn key_from_native_message(&self, wparam: WPARAM) -> Option<char> {
        let code = (wparam.0 & 0xFFFF) as u16;
        match code {
            0x30..=0x39 | 0x41..=0x5A => Some((code as u8 as char).to_ascii_lowercase()),
            _ => None,
        }
    }

    fn set_shortcut_key_down(&mut self, ch: char, down: bool) -> bool {
        let canonical = ch.to_ascii_lowercase();
        let keys = &mut self.input.held_shortcut_keys;
        if down {
            if keys.contains(&canonical) {
                return false;
            }
            keys.push(canonical);
            keys.sort_unstable();
            true
        } else if let Some(index) = keys.iter().position(|existing| *existing == canonical) {
            keys.remove(index);
            true
        } else {
            false
        }
    }

    fn handle_paint_timer_messages(&mut self, message: u32, wparam: WPARAM) -> Option<LRESULT> {
        match message {
            WM_PAINT => {
                self.paint_background();
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_TIMER if wparam.0 == TIMER_ID => {
                self.render_frame();
                Some(LRESULT(0))
            }
            #[cfg(feature = "frame-capture")]
            PATCHBAY_MSG_CAPTURE_FRAME => {
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_ERASEBKGND => {
                self.erase_background(wparam);
                Some(LRESULT(0))
            }
            _ => None,
        }
    }

    fn handle_primary_button_down(&mut self, double_clicked: bool) -> Option<LRESULT> {
        self.input.mouse_down = true;
        self.input.mouse_pressed = true;
        self.input.mouse_double_clicked = double_clicked;
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
            SetCapture(self.hwnd);
        };
        self.render_frame();
        Some(LRESULT(0))
    }

    fn handle_primary_button_up(&mut self) -> Option<LRESULT> {
        self.input.mouse_down = false;
        self.input.mouse_released = true;
        unsafe {
            let _ = ReleaseCapture();
        }
        self.render_frame();
        Some(LRESULT(0))
    }

    fn handle_secondary_button_down(&mut self) -> Option<LRESULT> {
        self.input.mouse_secondary_down = true;
        self.input.mouse_secondary_pressed = true;
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
            SetCapture(self.hwnd);
        };
        self.render_frame();
        Some(LRESULT(0))
    }

    fn handle_secondary_button_up(&mut self) -> Option<LRESULT> {
        self.input.mouse_secondary_down = false;
        self.input.mouse_secondary_released = true;
        unsafe {
            let _ = ReleaseCapture();
        }
        self.render_frame();
        Some(LRESULT(0))
    }

    fn paint_background(&self) {
        unsafe {
            let mut paint = PAINTSTRUCT::default();
            let hdc = BeginPaint(self.hwnd, &mut paint);
            FillRect(hdc, &paint.rcPaint, self.background_brush);
            let _ = EndPaint(self.hwnd, &paint);
        }
    }

    fn erase_background(&self, wparam: WPARAM) {
        let mut rect = windows::Win32::Foundation::RECT::default();
        let hdc = if wparam.0 == 0 {
            unsafe { GetDC(Some(self.hwnd)) }
        } else {
            HDC(wparam.0 as *mut _)
        };
        unsafe {
            if GetClientRect(self.hwnd, &mut rect).is_err() {
                log_line_safe("win32: GetClientRect failed in WM_ERASEBKGND");
            }
            FillRect(hdc, &rect, self.background_brush);
        }
        if wparam.0 == 0 {
            unsafe {
                let _ = ReleaseDC(Some(self.hwnd), hdc);
            }
        }
    }
}

/// Translate Win32 virtual keys into declarative text-edit control characters.
fn translate_virtual_key_to_input_char(wparam: WPARAM) -> Option<char> {
    match (wparam.0 & 0xFFFF) as u16 {
        key if key == VK_BACK.0 as u16 => Some('\u{8}'),
        key if key == VK_DELETE.0 as u16 => Some('\u{7f}'),
        key if key == VK_RETURN.0 as u16 => Some('\r'),
        key if key == VK_ESCAPE.0 as u16 => Some('\u{1b}'),
        key if key == VK_TAB.0 as u16 => Some('\t'),
        key if key == VK_SPACE.0 as u16 => Some(' '),
        key if key == VK_LEFT.0 as u16 => Some('\u{1c}'),
        key if key == VK_RIGHT.0 as u16 => Some('\u{1d}'),
        key if key == VK_HOME.0 as u16 => Some('\u{1e}'),
        key if key == VK_END.0 as u16 => Some('\u{1f}'),
        _ => None,
    }
}
