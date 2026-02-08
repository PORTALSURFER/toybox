/// Internal context used while constructing a Win32 child window and state.
struct ThreadSpawnContext<State, Init, Build, Reduce> {
    /// Class name used to register and create the child window.
    class_name: Vec<u16>,
    /// Parent host window handle.
    parent_hwnd: HWND,
    /// Parent module instance handle from the host.
    parent_hinstance: HINSTANCE,
    /// Module instance used for class registration and window creation.
    module_hinstance: HINSTANCE,
    /// Window title for the child view.
    title: String,
    /// Initial logical window size.
    size: Size,
    /// Plugin callbacks and mutable state payload.
    callbacks: SpawnCallbacks<State, Init, Build, Reduce>,
    /// Shared synchronization state across host and window thread.
    shared: SpawnSharedState,
    /// Initial UI theme/layout defaults.
    ui: SpawnUiConfig,
}

/// Parameters used to create the hidden Win32 child window.
struct ChildWindowCreateParams<'a> {
    /// Registered class name.
    class_name: &'a [u16],
    /// Window title.
    title: &'a str,
    /// Requested initial client size.
    size: Size,
    /// Parent host window handle.
    parent_hwnd: HWND,
    /// Parent module instance.
    parent_hinstance: HINSTANCE,
    /// Module instance used by the child window class.
    module_hinstance: HINSTANCE,
}

/// Transform a spawn request into normalized Win32 context values.
fn build_thread_spawn_context<State, Init, Build, Reduce>(
    request: SpawnWindowRequest<State, Init, Build, Reduce>,
) -> ThreadSpawnContext<State, Init, Build, Reduce>
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
        parent_hwnd,
        parent_hinstance,
        title,
        size,
    } = window;
    let parent_hwnd = HWND(parent_hwnd as *mut _);
    let parent_hinstance = HINSTANCE(parent_hinstance as *mut _);
    let module_hinstance =
        resolve_module_hinstance_for_window_proc::<State, Init, Build, Reduce>(parent_hinstance);
    log_module_fallback(parent_hinstance, module_hinstance);
    ThreadSpawnContext {
        class_name: to_wide("PatchbayGuiWindow"),
        parent_hwnd,
        parent_hinstance,
        module_hinstance,
        title,
        size,
        callbacks,
        shared,
        ui,
    }
}

/// Log when the host did not provide a parent module handle.
fn log_module_fallback(parent_hinstance: HINSTANCE, module_hinstance: HINSTANCE) {
    if parent_hinstance.0.is_null() {
        log_line_safe(&format!(
            "win32: parent hinstance was null, using module hinstance={:?}",
            module_hinstance
        ));
    }
}

/// Resolve the module handle to use when registering the window class.
fn resolve_module_hinstance_for_window_proc<State, Init, Build, Reduce>(
    parent_hinstance: HINSTANCE,
) -> HINSTANCE
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    if !parent_hinstance.0.is_null() {
        return parent_hinstance;
    }
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
}

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

/// Acquire or create a shared renderer device and construct a renderer.
fn create_renderer(
    device_cache: &Arc<Mutex<Option<Arc<RendererDevice>>>>,
    window: SurfaceWindow,
    size: Size,
) -> Result<Renderer, GuiError> {
    log_line_safe("win32: creating renderer");
    let renderer_device = load_renderer_device(device_cache)?;
    let renderer = Renderer::new_with_device(renderer_device, window, size)?;
    log_line_safe("win32: renderer created");
    Ok(renderer)
}

/// Load the renderer device from cache or create it when missing.
fn load_renderer_device(
    device_cache: &Arc<Mutex<Option<Arc<RendererDevice>>>>,
) -> Result<Arc<RendererDevice>, GuiError> {
    let mut cache = device_cache.lock().map_err(|_| GuiError::DeviceCachePoison)?;
    if let Some(device) = cache.as_ref() {
        return Ok(Arc::clone(device));
    }
    let device = Arc::new(RendererDevice::new()?);
    *cache = Some(Arc::clone(&device));
    Ok(device)
}

/// Build the full window state payload after the renderer has been created.
fn build_window_state<State, Init, Build, Reduce>(
    spawn_context: ThreadSpawnContext<State, Init, Build, Reduce>,
    child_hwnd: HWND,
    renderer: Renderer,
) -> Box<WindowState<State, Init, Build, Reduce>>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    let ThreadSpawnContext {
        size,
        callbacks,
        shared,
        ui,
        ..
    } = spawn_context;
    let SpawnCallbacks {
        state,
        on_init,
        build,
        reduce,
    } = callbacks;
    let SpawnSharedState {
        resize_request,
        last_size,
        aspect_ratio,
        ..
    } = shared;
    let SpawnUiConfig {
        ui_state,
        layout,
        theme,
    } = ui;
    Box::new(build_window_state_value(
        child_hwnd,
        renderer,
        size,
        WindowStateParts {
            state,
            on_init,
            build,
            reduce,
            resize_request,
            last_size,
            aspect_ratio,
            ui_state,
            layout,
            theme,
        },
    ))
}

/// All dynamic components needed to construct a [`WindowState`].
struct WindowStateParts<State, Init, Build, Reduce> {
    /// Plugin-owned mutable state.
    state: State,
    /// One-time UI initialization callback.
    on_init: Init,
    /// Declarative UI build callback.
    build: Build,
    /// UI action reducer callback.
    reduce: Reduce,
    /// Host-requested resize dimensions.
    resize_request: Arc<AtomicU64>,
    /// Last known actual size.
    last_size: Arc<AtomicU64>,
    /// Optional aspect ratio lock.
    aspect_ratio: Arc<AtomicU32>,
    /// UI interaction and transient state.
    ui_state: UiState,
    /// Current layout state.
    layout: Layout,
    /// Current UI theme.
    theme: Theme,
}

/// Create the concrete [`WindowState`] value from resolved parts.
fn build_window_state_value<State, Init, Build, Reduce>(
    child_hwnd: HWND,
    renderer: Renderer,
    size: Size,
    parts: WindowStateParts<State, Init, Build, Reduce>,
) -> WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    let canvas = Canvas::new(size.width, size.height);
    let background_brush = unsafe { CreateSolidBrush(colorref_from_theme(parts.theme.background)) };
    let debug_input = std::env::var_os("PATCHBAY_DEBUG_INPUT").is_some();
    WindowState {
        hwnd: child_hwnd,
        renderer,
        canvas,
        canonical_layout_size: size,
        input: InputState {
            window_size: size,
            ..InputState::default()
        },
        ui_state: parts.ui_state,
        layout: parts.layout,
        layout_origin: parts.layout.cursor,
        theme: parts.theme,
        background_brush,
        state: parts.state,
        on_init: parts.on_init,
        build_spec: parts.build,
        reduce_action: parts.reduce,
        resize_request: parts.resize_request,
        last_size: parts.last_size,
        aspect_ratio: parts.aspect_ratio,
        initialized: false,
        shown: false,
        prewarm_frames: PREWARM_FRAMES,
        created_at: Instant::now(),
        last_mouse_down: false,
        last_mouse_secondary_down: false,
        debug_input,
        frame_counter: 0,
    }
}

/// Install window state on the HWND and prime the first render frame.
fn install_window_state_and_prime<State, Init, Build, Reduce>(
    child_hwnd: HWND,
    window_state: Box<WindowState<State, Init, Build, Reduce>>,
) where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
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
}
