# Strict Declarative GUI API

## Summary
`patchbay-gui` now exposes a strict declarative API:

- Build a pure-data `UiSpec` tree each frame.
- Render with `render_checked`.
- Consume emitted `UiAction` values in a reducer.

The UI tree does not contain callbacks or custom widget closures.

## Core Types
- `UiSpec`
- `RootFrameSpec`
- `Node`
- `LayoutBox` + `Length`
- `GridTemplate` + `TrackSize`
- `ThemeTokens`
- `UiAction`
- `RenderResult`

## Host Integration Pattern
Use the host window APIs with:
- a **build** closure: `FnMut(&InputState, &State) -> UiSpec`
- a **reduce** closure: `FnMut(&mut State, UiAction)`

This separates rendering intent from state mutation and keeps behavior deterministic.

## Example
```rust
use patchbay_gui::declarative::{
    ButtonSpec, Node, RootFrameSpec, UiAction, UiSpec,
};

#[derive(Default)]
struct GuiState {
    count: u32,
}

fn build(_input: &patchbay_gui::InputState, state: &GuiState) -> UiSpec {
    UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Button(ButtonSpec::new("inc", format!("Count {}", state.count))),
        )
    )
}

fn reduce(state: &mut GuiState, action: UiAction) {
    if let UiAction::ButtonPressed { key } = action {
        if key == "inc" {
            state.count = state.count.saturating_add(1);
        }
    }
}
```
