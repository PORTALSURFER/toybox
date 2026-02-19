use patchbay_gui::{LayoutBox, Length};

fn main() {
    let _ = LayoutBox {
        width: Length::Auto,
        height: Length::Auto,
        min_width: None,
        min_height: None,
        max_width: None,
        max_height: None,
    };
}
