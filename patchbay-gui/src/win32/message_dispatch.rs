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
            .or_else(|| self.handle_input_messages(message, wparam))
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
                self.input.pointer_pos = Point {
                    x: (lparam.0 & 0xFFFF) as i16 as i32,
                    y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i32,
                };
                self.render_frame();
                Some(LRESULT(0))
            }
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

    fn handle_input_messages(&mut self, message: u32, wparam: WPARAM) -> Option<LRESULT> {
        match message {
            WM_GETDLGCODE => {
                let flags = DLGC_WANTALLKEYS | DLGC_WANTCHARS;
                Some(LRESULT(flags as isize))
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
                if let Some(ch) = translate_virtual_key_to_input_char(wparam) {
                    self.input.key_pressed = Some(ch);
                    self.render_frame();
                }
                Some(LRESULT(0))
            }
            WM_CHAR => {
                let code = (wparam.0 & 0xFFFF) as u16;
                if let Some(ch) = char::from_u32(code as u32) {
                    self.input.key_pressed = Some(ch);
                }
                self.render_frame();
                Some(LRESULT(0))
            }
            _ => None,
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
        key if key == VK_RETURN.0 as u16 => Some('\r'),
        key if key == VK_ESCAPE.0 as u16 => Some('\u{1b}'),
        key if key == VK_TAB.0 as u16 => Some('\t'),
        key if key == VK_SPACE.0 as u16 => Some(' '),
        _ => None,
    }
}
