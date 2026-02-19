use patchbay_gui::{textbox, Color, ContainerLayout, PanelSpec};

fn main() {
    let _ = PanelSpec {
        key: "panel".to_string(),
        title: None,
        padding: 0,
        background: Some(Color::rgb(0, 0, 0)),
        outline: None,
        header_height: None,
        layout: ContainerLayout::fill(),
        content: Box::new(textbox("x")),
    };
}
