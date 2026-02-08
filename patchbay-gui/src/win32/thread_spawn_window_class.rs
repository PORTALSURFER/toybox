/// Validate that the host parent HWND is a live window.
fn validate_parent_window(parent_hwnd: HWND) -> Result<(), GuiError> {
    unsafe {
        if windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(parent_hwnd)).as_bool() {
            return Ok(());
        }
    }
    log_line_safe(&format!(
        "win32: invalid parent hwnd={:?}; aborting CreateWindowExW",
        parent_hwnd
    ));
    Err(GuiError::WindowCreateFailed)
}

/// Register the Win32 class used for Patchbay child windows.
fn register_patchbay_window_class<State, Init, Build, Reduce>(
    class_name: &[u16],
    module_hinstance: HINSTANCE,
) -> Result<(), GuiError>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    let cursor = unsafe { LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW) }
        .map_err(|err| {
            log_line_safe(&format!("win32: LoadCursorW error: {err:?}"));
            GuiError::WindowCreateFailed
        })?;
    unsafe {
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: Some(window_proc::<State, Init, Build, Reduce>),
            hInstance: module_hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: cursor,
            hbrBackground: HBRUSH(std::ptr::null_mut()),
            ..Default::default()
        };
        RegisterClassW(&wnd_class);
    }
    log_line_safe("win32: RegisterClassW completed");
    Ok(())
}

/// Create the hidden child window attached to the host parent.
fn create_hidden_child_window(params: &ChildWindowCreateParams<'_>) -> Result<HWND, GuiError> {
    let title_w = to_wide(params.title);
    log_line_safe(&format!(
        "win32: CreateWindowExW begin title=\"{}\" size={}x{} parent_hwnd={:?} parent_hinstance={:?} module_hinstance={:?}",
        params.title,
        params.size.width,
        params.size.height,
        params.parent_hwnd,
        params.parent_hinstance,
        params.module_hinstance
    ));
    let child_hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(params.class_name.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            params.size.width as i32,
            params.size.height as i32,
            Some(params.parent_hwnd),
            Some(HMENU(std::ptr::null_mut())),
            Some(params.module_hinstance),
            None,
        )
    }
    .map_err(|err| {
        log_line_safe(&format!("win32: CreateWindowExW error: {err:?}"));
        GuiError::WindowCreateFailed
    })?;
    log_line_safe(&format!("win32: CreateWindowExW ok hwnd={:?}", child_hwnd));
    unsafe {
        let _ = ShowWindow(child_hwnd, SW_HIDE);
    }
    Ok(child_hwnd)
}
