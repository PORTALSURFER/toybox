use patchbay_gui::LayoutEngineState;

fn main() {
    let _ = LayoutEngineState {
        measure_cache_hits: 0,
        ..LayoutEngineState::default()
    };
}
