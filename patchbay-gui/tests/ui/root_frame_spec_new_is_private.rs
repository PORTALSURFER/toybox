use patchbay_gui::{textbox, RootFrameSpec};

fn main() {
    let _ = RootFrameSpec::new("root", textbox("x"));
}
