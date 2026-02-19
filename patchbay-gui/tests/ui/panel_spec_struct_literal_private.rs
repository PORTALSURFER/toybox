use patchbay_gui::{label, Color, ContainerLayout, PanelSpec};

fn main() {
    let _ = PanelSpec {
        key: "panel".to_string(),
        title: None,
        padding: 0,
        background: Some(Color::rgb(0, 0, 0)),
        outline: None,
        header_height: None,
        layout: ContainerLayout::fill(),
        content: Box::new(label("x")),
    };
}
