/// Window-parent and geometry configuration for Win32 GUI creation.
pub(crate) struct SpawnWindowConfig {
    /// Host parent window handle.
    pub(crate) parent_hwnd: isize,
    /// Host parent module instance handle.
    pub(crate) parent_hinstance: isize,
    /// Child window title.
    pub(crate) title: String,
    /// Initial logical canvas size.
    pub(crate) size: Size,
}

/// Stateful UI callbacks and plugin state payload used during window creation.
pub(crate) struct SpawnCallbacks<State, Init, Build, Reduce> {
    /// Plugin-owned mutable UI state.
    pub(crate) state: State,
    /// One-time initialization callback.
    pub(crate) on_init: Init,
    /// Per-frame declarative UI builder callback.
    pub(crate) build: Build,
    /// Per-action reducer callback.
    pub(crate) reduce: Reduce,
}

/// Shared host/window synchronization state used by Win32 plumbing.
pub(crate) struct SpawnSharedState {
    /// Shared renderer device cache.
    pub(crate) device_cache: Arc<Mutex<Option<Arc<RendererDevice>>>>,
    /// Requested host-resize dimensions.
    pub(crate) resize_request: Arc<AtomicU64>,
    /// Last known actual size.
    pub(crate) last_size: Arc<AtomicU64>,
    /// Optional aspect ratio lock.
    pub(crate) aspect_ratio: Arc<AtomicU32>,
}

/// Initial UI container defaults applied to the window state.
pub(crate) struct SpawnUiConfig {
    /// Transient UI state machine storage.
    pub(crate) ui_state: UiState,
    /// Default layout cursor and spacing policy.
    pub(crate) layout: Layout,
    /// Initial theme palette.
    pub(crate) theme: Theme,
}

/// Complete request payload for creating a Win32 child GUI window.
pub(crate) struct SpawnWindowRequest<State, Init, Build, Reduce> {
    /// Parent handle, title, and size metadata.
    pub(crate) window: SpawnWindowConfig,
    /// UI state and callback hooks.
    pub(crate) callbacks: SpawnCallbacks<State, Init, Build, Reduce>,
    /// Shared synchronization handles.
    pub(crate) shared: SpawnSharedState,
    /// Initial layout/theme defaults.
    pub(crate) ui: SpawnUiConfig,
}

/// Spawn a GUI thread that owns the Win32 window and render loop.
pub fn spawn_window_thread<State, Init, Build, Reduce>(
    request: SpawnWindowRequest<State, Init, Build, Reduce>,
) -> Result<WindowHandle, GuiError>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    log_line_safe("win32: spawn_window_thread begin (using caller thread)");
    create_window_on_thread(request)
}

fn create_window_on_thread<State, Init, Build, Reduce>(
    request: SpawnWindowRequest<State, Init, Build, Reduce>,
) -> Result<WindowHandle, GuiError>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    let SpawnWindowRequest {
        window,
        callbacks,
        shared,
        ui,
    } = request;
    let SpawnWindowConfig {
        parent_hwnd: parent_hwnd_raw,
        parent_hinstance: parent_hinstance_raw,
        title,
        size,
    } = window;
    let SpawnCallbacks {
        state,
        on_init,
        build,
        reduce,
    } = callbacks;
    let SpawnSharedState {
        device_cache,
        resize_request,
        last_size,
        aspect_ratio,
    } = shared;
    let SpawnUiConfig {
        ui_state,
        layout,
        theme,
    } = ui;

    log_line_safe("win32: create_window_on_thread begin");
    let class_name = to_wide("PatchbayGuiWindow");
    let parent_hwnd = HWND(parent_hwnd_raw as *mut _);
    let parent_hinstance = HINSTANCE(parent_hinstance_raw as *mut _);
    let module_hinstance = if parent_hinstance.0.is_null() {
        let mut module = windows::Win32::Foundation::HMODULE::default();
        let proc_addr = window_proc::<State, Init, Build, Reduce> as *const () as *const u16;
        let got_module = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
                windows::core::PCWSTR(proc_addr),
                &mut module,
            )
        }
        .is_ok();
        if got_module {
            HINSTANCE(module.0)
        } else {
            unsafe { GetModuleHandleW(None).unwrap_or_default().into() }
        }
    } else {
        parent_hinstance
    };
    if parent_hinstance.0.is_null() {
        log_line_safe(&format!(
            "win32: parent hinstance was null, using module hinstance={:?}",
            module_hinstance
        ));
    }
    unsafe {
        if !windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(parent_hwnd)).as_bool() {
            log_line_safe(&format!(
                "win32: invalid parent hwnd={:?}; aborting CreateWindowExW",
                parent_hwnd
            ));
            return Err(GuiError::WindowCreateFailed);
        }
    }

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

    let title_w = to_wide(&title);
    log_line_safe(&format!(
        "win32: CreateWindowExW begin title=\"{}\" size={}x{} parent_hwnd={:?} parent_hinstance={:?} module_hinstance={:?}",
        title, size.width, size.height, parent_hwnd, parent_hinstance, module_hinstance
    ));
    let child_hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            size.width as i32,
            size.height as i32,
            Some(parent_hwnd),
            Some(HMENU(std::ptr::null_mut())),
            Some(module_hinstance),
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

    let window = SurfaceWindow {
        hwnd: child_hwnd,
        hinstance: module_hinstance,
    };
    log_line_safe("win32: creating renderer");
    let renderer_device = {
        let mut cache = device_cache
            .lock()
            .map_err(|_| GuiError::DeviceCachePoison)?;
        if let Some(device) = cache.as_ref() {
            Arc::clone(device)
        } else {
            let device = Arc::new(RendererDevice::new()?);
            *cache = Some(Arc::clone(&device));
            device
        }
    };
    let renderer = Renderer::new_with_device(renderer_device, window, size)?;
    log_line_safe("win32: renderer created");
    let canvas = Canvas::new(size.width, size.height);
    let background_brush = unsafe { CreateSolidBrush(colorref_from_theme(theme.background)) };

    let window_state = Box::new(WindowState {
        hwnd: child_hwnd,
        renderer,
        canvas,
        canonical_layout_size: size,
        input: InputState {
            window_size: size,
            ..InputState::default()
        },
        ui_state,
        layout,
        layout_origin: layout.cursor,
        theme,
        background_brush,
        state,
        on_init,
        build_spec: build,
        reduce_action: reduce,
        resize_request,
        last_size,
        aspect_ratio,
        initialized: false,
        shown: false,
        prewarm_frames: PREWARM_FRAMES,
        created_at: Instant::now(),
        last_mouse_down: false,
        last_mouse_secondary_down: false,
        debug_input: std::env::var_os("PATCHBAY_DEBUG_INPUT").is_some(),
        frame_counter: 0,
    });

    unsafe {
        let state_ptr = Box::into_raw(window_state);
        SetWindowLongPtrW(child_hwnd, GWLP_USERDATA, state_ptr as isize);
        SetTimer(Some(child_hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);
        DragAcceptFiles(child_hwnd, true);
        log_line_safe("win32: initial window hidden; waiting for show gate");
        let state = &mut *(state_ptr as *mut WindowState<State, Init, Build, Reduce>);
        // Synchronize to the actual client rect before the first frame.
        // Some hosts may constrain the child view at create-time without
        // emitting WM_SIZE immediately, which otherwise causes a one-frame (or
        // persistent) size mismatch and clipped content.
        state.on_resize();
        // Render once; on success it will reveal the window.
        state.render_frame();
    }

    let handle = WindowHandle { hwnd: child_hwnd };
    Ok(handle)
}
