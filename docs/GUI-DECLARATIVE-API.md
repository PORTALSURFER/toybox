# Strict Declarative GUI API

## Summary
`toybox::gui::declarative` is the only supported authoring surface for plugin UIs.

The API is strictly declarative:
- Build a pure-data `UiSpec` tree each frame.
- Render via host integration (`GuiHostWindow::open_parented*`) and `render_checked` internally.
- Apply emitted `UiAction` values in a reducer.

No callback-bearing widget nodes are supported.

## Core Model
- `UiSpec`: top-level UI tree.
- `RootFrameSpec`: root frame key, sizing, tokens, and content.
- `Node`: typed containers + controls.
- `LayoutBox` + `Length`: explicit sizing constraints.
- `GridTemplate` + `TrackSize`: grid track definitions.
- `ThemeTokens`: declarative token overrides.
- `UiAction`: typed interaction output consumed by reducer logic.

## Ergonomic Constructors
Use helper constructors for common nodes:
- `row(children)` / `column(children)`
- `grid(template, children)`
- `panel(key, content)`
- `label(text)`
- `knob(...)`, `slider(...)`, `toggle(...)`, `button(...)`, `dropdown(...)`
- `spacer(size)`, `region(key, size)`, `indicator(size, active)`

These map directly to `Node::*` variants and keep the tree callback-free.
Most node helpers also support `.layout(...)`, `.fill()`, `.fill_width()`, and `.fill_height()`.

## Layout Ergonomics
Use fluent helpers to reduce boilerplate:
- `FlexSpec`: `gap`, `pad_all`, `pad_xy`, `align_*`, `justify_*`
- `justify_*` covers `start`, `center`, `end`, `space_between`, `space_around`, and `space_evenly`.
- `GridTemplate`: `columns_fr`, `rows_fr`, `gap`, `gap_xy`, `pad_all`, `pad_xy`
- `LayoutBox`: `fill`, `fill_width`, `fill_height`, `fixed`, `fixed_width`, `fixed_height`, `min`, `max`

## Host Integration Pattern
Use the host window with:
- build: `FnMut(&InputState, &State) -> UiSpec`
- reduce: `FnMut(&mut State, UiAction)`

This keeps rendering deterministic and centralizes state mutation in one reducer step.

## Validation Guarantees
`measure_checked` and `render_checked` validate trees and return `DeclarativeError` on invalid specs.

Validation includes:
- non-empty root frame key
- non-empty keys for interactive keyed nodes
- unique keys across root + interactive keyed nodes
- non-empty grid columns
- finite, increasing control ranges (`min < max`) for knobs/sliders
- finite, in-range control values for knobs/sliders
- dropdown selected index in bounds
- non-zero explicit `control_size` overrides

## Example
```rust
use toybox::gui::declarative::{
    button, panel, FlexSpec, LayoutBox, Node, RootFrameSpec, UiAction, UiSpec,
};

#[derive(Default)]
struct GuiState {
    count: u32,
}

fn build(_input: &toybox::clap::gui::InputState, _state: &GuiState) -> UiSpec {
    let controls = Node::Row(
        FlexSpec::row(vec![
            button("inc", "Increment"),
            button("dec", "Decrement"),
        ])
        .justify_space_between(),
    );

    UiSpec::new(
        RootFrameSpec::new(
            "root",
            panel("main", controls).fill(),
        )
        .layout(LayoutBox::fill()),
    )
}

fn reduce(state: &mut GuiState, action: UiAction) {
    match action {
        UiAction::ButtonPressed { key } if key == "inc" => {
            state.count = state.count.saturating_add(1);
        }
        UiAction::ButtonPressed { key } if key == "dec" => {
            state.count = state.count.saturating_sub(1);
        }
        _ => {}
    }
}
```

## Migration Notes (Hard Break)
Legacy mixed-mode patterns are removed from the supported surface:
- no `UiSpec<'_, State>` callback-bearing widget nodes
- no `WidgetSpec` render closures
- no immediate-mode public authoring API for plugin UI composition

Migration steps:
1. Move all UI construction into pure `Node` trees.
2. Replace per-widget callbacks with reducer handling on `UiAction`.
3. Replace direct immediate layout code with `row/column/grid/panel` declarative nodes.
4. Use `measure_checked` in tests to catch key/range/selection/layout mistakes early.
