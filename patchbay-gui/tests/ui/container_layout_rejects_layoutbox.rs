use patchbay_gui::{textbox, LayoutBox, PanelSpec};

fn main() {
    let _ = PanelSpec::new("panel", textbox("x")).layout(LayoutBox::fixed(200, 100));
}
