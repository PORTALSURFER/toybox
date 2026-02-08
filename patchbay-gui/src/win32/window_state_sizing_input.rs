impl<State, Init, Build, Reduce> WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    fn on_resize(&mut self) {
        let width = (self.input.window_size.width.max(1)) as i32;
        let height = (self.input.window_size.height.max(1)) as i32;
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                width,
                height,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    fn sync_client_size_if_needed(&mut self) {
        let Some((width, height)) = self.current_client_size() else {
            return;
        };
        if self.should_apply_client_size(width, height) {
            self.apply_layout_size(Size { width, height }, false);
        }
    }

    fn current_client_size(&self) -> Option<(u32, u32)> {
        let mut rect = windows::Win32::Foundation::RECT::default();
        let ok = unsafe { GetClientRect(self.hwnd, &mut rect).is_ok() };
        if !ok {
            return None;
        }
        let width = (rect.right - rect.left).max(1) as u32;
        let height = (rect.bottom - rect.top).max(1) as u32;
        Some((width, height))
    }

    fn apply_child_size_request(&self, size: Size) {
        let width = size.width.max(1) as i32;
        let height = size.height.max(1) as i32;
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                width,
                height,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    fn sync_pointer_pos(&mut self) {
        let mut point = windows::Win32::Foundation::POINT::default();
        if unsafe { GetCursorPos(&mut point) }.is_err() {
            return;
        }
        let mut window_rect = windows::Win32::Foundation::RECT::default();
        if unsafe { GetWindowRect(self.hwnd, &mut window_rect) }.is_err() {
            return;
        }
        self.input.pointer_pos = Point {
            x: point.x - window_rect.left,
            y: point.y - window_rect.top,
        };
    }

    fn sync_mouse_buttons(&mut self) {
        let primary = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
        let secondary = unsafe { GetAsyncKeyState(VK_RBUTTON.0 as i32) } < 0;
        let shift = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } < 0;
        let alt = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0;

        if primary && !self.last_mouse_down {
            self.input.mouse_pressed = true;
        }
        if !primary && self.last_mouse_down {
            self.input.mouse_released = true;
        }
        if secondary && !self.last_mouse_secondary_down {
            self.input.mouse_secondary_pressed = true;
        }
        if !secondary && self.last_mouse_secondary_down {
            self.input.mouse_secondary_released = true;
        }

        self.input.mouse_down = primary;
        self.input.mouse_secondary_down = secondary;
        self.input.shift_down = shift;
        self.input.alt_down = alt;
        self.last_mouse_down = primary;
        self.last_mouse_secondary_down = secondary;
    }
}
