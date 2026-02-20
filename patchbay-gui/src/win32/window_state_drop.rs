impl<State, Init, Build, Reduce> Drop for WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    fn drop(&mut self) {
        self.active_text_edit_shared
            .store(false, std::sync::atomic::Ordering::Release);
        unsafe {
            let _ = DeleteObject(self.background_brush.into());
        }
    }
}
