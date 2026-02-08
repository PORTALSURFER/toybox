struct WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    hwnd: HWND,
    renderer: Renderer,
    canvas: Canvas,
    canonical_layout_size: Size,
    input: InputState,
    ui_state: UiState,
    layout: Layout,
    layout_origin: Point,
    theme: Theme,
    background_brush: HBRUSH,
    state: State,
    on_init: Init,
    build_spec: Build,
    reduce_action: Reduce,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    initialized: bool,
    shown: bool,
    prewarm_frames: u8,
    created_at: Instant,
    last_mouse_down: bool,
    last_mouse_secondary_down: bool,
    debug_input: bool,
    frame_counter: u64,
}

impl<State, Init, Build, Reduce> WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    fn configured_aspect_ratio(&self) -> Option<f32> {
        let bits = self.aspect_ratio.load(Ordering::Relaxed);
        if bits == 0 {
            return None;
        }
        let ratio = f32::from_bits(bits);
        if ratio.is_finite() && ratio > 0.0 {
            Some(ratio)
        } else {
            None
        }
    }

    /// Return true when an observed client size should be applied to layout.
    ///
    /// Resize handling intentionally consumes all intermediate host sizes,
    /// including transient non-aspect values. Filtering those samples causes
    /// visible slide/snap artifacts while users drag-resize plugin windows.
    fn should_apply_client_size(&self, width: u32, height: u32) -> bool {
        // Keep the configured ratio plumbed for host negotiation paths, but do
        // not gate live client-size adoption on ratio matching.
        let _ = self.configured_aspect_ratio();
        client_size_changed(
            self.canonical_layout_size,
            Size {
                width: width.max(1),
                height: height.max(1),
            },
        )
    }

    fn apply_layout_size(&mut self, size: Size, sync_pointer: bool) {
        let size = Size {
            width: size.width.max(1),
            height: size.height.max(1),
        };
        self.canonical_layout_size = size;
        self.last_size
            .store(pack_size(size.width, size.height), Ordering::Release);
        self.input.window_size = size;
        self.renderer.resize(size);
        if sync_pointer {
            self.sync_pointer_pos();
        }
    }
}
