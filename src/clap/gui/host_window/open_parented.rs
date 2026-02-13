impl GuiHostWindow {
    /// Open a parented Patchbay GUI window.
    ///
    /// # Deprecated
    ///
    /// Prefer [`Self::open_parented_with`] with a [`GuiOpenRequest`].
    /// This adapter preserves compatibility while keeping wide callback wiring in one
    /// request value.
    #[deprecated(
        since = "0.1.0",
        note = "Use open_parented_with(GuiOpenRequest::with_callbacks(...))"
    )]
    ///
    /// The caller supplies the initial state, a declarative UI builder, and an
    /// action reducer. The helper handles resize requests and stores the last
    /// logical size.
    pub fn open_parented<State, Init, Build, Reduce>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&patchbay_gui::InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(GuiOpenRequest::with_callbacks(
            title,
            size,
            GuiOpenCallbacks::new(state, on_init, build, reduce),
        ))
    }

    /// Open a parented window, reusing it if it is already open.
    ///
    /// # Deprecated
    ///
    /// Prefer [`Self::open_parented_with`] with a [`GuiOpenRequest`].
    #[deprecated(
        since = "0.1.0",
        note = "Use open_parented_with(GuiOpenRequest::with_callbacks(...).with_mode(ReuseIfOpen))"
    )]
    ///
    /// This mirrors Patchbay's default behavior: if a window is already open
    /// and attached to the same parent, the new state is ignored and the
    /// existing window is shown.
    pub fn open_parented_reuse<State, Init, Build, Reduce>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&patchbay_gui::InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(
            GuiOpenRequest::with_callbacks(
                title,
                size,
                GuiOpenCallbacks::new(state, on_init, build, reduce),
            )
            .with_mode(patchbay_gui::OpenParentedMode::ReuseIfOpen),
        )
    }

    /// Open a parented window with an explicit reuse policy.
    ///
    /// The `size` argument is used as the initial window size.
    pub fn open_parented_with<State, Init, Build, Reduce>(
        &mut self,
        request: GuiOpenRequest<State, Init, Build, Reduce>,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&patchbay_gui::InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        let GuiOpenRequest {
            title,
            size,
            callbacks,
            mode,
        } = request;
        let GuiOpenCallbacks {
            state,
            on_init,
            build,
            reduce,
        } = callbacks;
        log_line_safe(&format!(
            "toybox/gui: open_parented title=\"{}\" requested_size={}x{} mode={mode:?}",
            title, size.0, size.1
        ));
        self.inner
            .open_parented_with(
                PatchbayOpenParentedRequest::with_callbacks(
                    title,
                    Size {
                        width: size.0.max(1),
                        height: size.1.max(1),
                    },
                    OpenParentedCallbacks::new(state, on_init, build, reduce),
                )
                .with_mode(mode),
            )
            .map_err(map_gui_error)
    }
}
