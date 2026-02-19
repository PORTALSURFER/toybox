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
        dropdown_popup_request: None,
        dropdown_popup_dispatch_queued: false,
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
