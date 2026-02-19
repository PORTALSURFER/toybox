use patchbay_gui::{label, LayoutBox, PanelSpec};

fn main() {
    let _ = PanelSpec::new("panel", label("x")).layout(LayoutBox::fixed(200, 100));
}
