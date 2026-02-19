const WM_PATCHBAY_OPEN_DROPDOWN_POPUP: u32 = WM_APP + 0x310;

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
        self.handle_dropdown_popup_messages(message)
            .or_else(|| self.handle_frame_messages(message, wparam, lparam))
            .or_else(|| self.handle_pointer_messages(message, wparam, lparam))
            .or_else(|| self.handle_input_messages(message, wparam))
            .or_else(|| self.handle_paint_timer_messages(message, wparam))
    }

    fn handle_dropdown_popup_messages(&mut self, message: u32) -> Option<LRESULT> {
        if message != WM_PATCHBAY_OPEN_DROPDOWN_POPUP {
            return None;
        }
        self.dropdown_popup_dispatch_queued = false;
        let Some(request) = self.dropdown_popup_request.take() else {
            return Some(LRESULT(0));
        };
        let result = self.show_native_dropdown_popup(&request);
        let popup_result = match result {
            Some(index) => DropdownPopupResult::Selected {
                dropdown_id: request.dropdown_id,
                index,
            },
            None => DropdownPopupResult::Closed {
                dropdown_id: request.dropdown_id,
            },
        };
        self.ui_state.set_dropdown_popup_result(popup_result);
        self.ui_state.clear_open_dropdown();
        self.render_frame();
        Some(LRESULT(0))
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
            WM_DROPFILES => {
                let hdrop = HDROP(wparam.0 as *mut _);
                self.input.dropped_files = collect_dropped_files(hdrop);
                unsafe {
                    DragFinish(hdrop);
                }
                self.render_frame();
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
        unsafe { SetCapture(self.hwnd) };
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
        unsafe { SetCapture(self.hwnd) };
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

    fn show_native_dropdown_popup(&self, request: &DropdownPopupRequest) -> Option<usize> {
        if request.options.is_empty() {
            return None;
        }

        let menu = unsafe { CreatePopupMenu() }.ok()?;
        for (index, option) in request.options.iter().enumerate() {
            let mut flags = MF_STRING;
            if index == request.selected {
                flags |= MF_CHECKED;
            }
            let wide = to_wide(option);
            if unsafe { AppendMenuW(menu, flags, index + 1, PCWSTR(wide.as_ptr())) }.is_err() {
                unsafe {
                    let _ = DestroyMenu(menu);
                }
                return None;
            }
        }

        let mut anchor = windows::Win32::Foundation::POINT {
            x: request.control_rect_surface.origin.x,
            y: request.control_rect_surface.origin.y,
        };
        if !unsafe { ClientToScreen(self.hwnd, &mut anchor).as_bool() } {
            unsafe {
                let _ = DestroyMenu(menu);
            }
            return None;
        }

        let control_height = request.control_rect_surface.size.height.max(1) as i32;
        let mut popup_x = anchor.x;
        let mut popup_y = if request.open_up {
            anchor.y
        } else {
            anchor.y + control_height
        };

        let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let popup_width = request.control_rect_surface.size.width.max(1) as i32;
        if popup_x + popup_width > screen_width {
            popup_x = (screen_width - popup_width).max(0);
        }
        if popup_x < 0 {
            popup_x = 0;
        }
        popup_y = popup_y.clamp(0, screen_height.max(1));

        let align_flag = if request.open_up {
            TPM_BOTTOMALIGN
        } else {
            TPM_TOPALIGN
        };
        let selected = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_NONOTIFY | TPM_LEFTALIGN | TPM_RIGHTBUTTON | align_flag,
                popup_x,
                popup_y,
                None,
                self.hwnd,
                None,
            )
        };
        unsafe {
            let _ = DestroyMenu(menu);
        }
        let command = selected.0;
        if command <= 0 {
            return None;
        }
        Some((command as usize).saturating_sub(1))
    }
}
