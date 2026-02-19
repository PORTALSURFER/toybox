use patchbay_gui::LayoutEngineState;

fn main() {
    let mut engine = LayoutEngineState::default();
    engine.mark_measure_dirty();
}
