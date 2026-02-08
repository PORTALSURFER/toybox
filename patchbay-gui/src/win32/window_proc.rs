unsafe extern "system" fn window_proc<State, Init, Build, Reduce>(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    let ptr =
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr != 0 {
        let state = unsafe { &mut *(ptr as *mut WindowState<State, Init, Build, Reduce>) };
        if let Some(result) = state.handle_message(message, wparam, lparam) {
            return result;
        }
    }

    if message == WM_NCDESTROY {
        if ptr != 0 {
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(
                    ptr as *mut WindowState<State, Init, Build, Reduce>,
                ));
            }
        }
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}
