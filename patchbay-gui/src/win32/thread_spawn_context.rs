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
