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
    /// Shared flag indicating active text-edit mode.
    pub(crate) active_text_edit: Arc<AtomicBool>,
    /// Registered shortcut bindings for focused key consumption.
    pub(crate) shortcut_bindings: Arc<Mutex<Vec<ShortcutBinding>>>,
    /// Shared frame-capture request/result state.
    #[cfg(feature = "frame-capture")]
    pub(crate) frame_capture: Arc<FrameCaptureState>,
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
    log_line_safe("win32: create_window_on_thread begin");
    let spawn_context = build_thread_spawn_context::<State, Init, Build, Reduce>(request);
    validate_parent_window(spawn_context.parent_hwnd)?;
    register_patchbay_window_class::<State, Init, Build, Reduce>(
        &spawn_context.class_name,
        spawn_context.module_hinstance,
    )?;
    let child_hwnd = create_hidden_child_window(&ChildWindowCreateParams {
        class_name: &spawn_context.class_name,
        title: &spawn_context.title,
        size: spawn_context.size,
        parent_hwnd: spawn_context.parent_hwnd,
        parent_hinstance: spawn_context.parent_hinstance,
        module_hinstance: spawn_context.module_hinstance,
    })?;
    let renderer = create_renderer(
        &spawn_context.shared.device_cache,
        SurfaceWindow {
            hwnd: child_hwnd,
            hinstance: spawn_context.module_hinstance,
        },
        spawn_context.size,
    )?;
    let window_state =
        build_window_state::<State, Init, Build, Reduce>(spawn_context, child_hwnd, renderer);
    install_window_state_and_prime::<State, Init, Build, Reduce>(child_hwnd, window_state);
    Ok(WindowHandle { hwnd: child_hwnd })
}
