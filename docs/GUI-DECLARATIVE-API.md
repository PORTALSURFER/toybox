# Strict Declarative GUI API

## Summary
`toybox::gui::declarative` is the only supported UI authoring surface.

System-level layout target and gap tracking:

- `GUI-STRICT-HIERARCHICAL-LAYOUT-SPEC.md`

The API is strict and data-only:
- Build a pure `UiSpec` tree each frame.
- Render through host integration (`render_checked` internally).
- Handle emitted `UiAction` values in a reducer.

## Core Tree Model
- `UiSpec` / `RootFrameSpec`: root window definition.
- `Node`: typed tree node.
- `SlotSpec`: single-child slot node used as the required direct child type for containers.
- Container nodes: `Panel`, `Row`, `Column`, `Grid`, `Absolute`.
- Widget nodes: `Label`, `Spacer`, `Knob`, `Slider`, `Toggle`, `Button`, `Dropdown`, `Region`, `Indicator`.

Canonical grammar:
- Root is a special container with exactly one slot.
- Containers directly host slots.
- Slots host exactly one child (`Container | Widget`).
- Widgets are leaves.

See `GUI-TREE-CONTRACT.md` for the full contract and failure cases.

## Constructors
- Containers: `row(children)`, `column(children)`, `grid(template, children)`, `panel(key, content)`, `root_frame_sized(...)`
- Slots/helpers: `slot(child)`, `weighted_slot(node, weight)`, `fraction_slot(node, percent)`, `fill_slot(node)`, `column_slots(...)`, `row_slots(...)`
- Widgets: `label`, `knob`, `slider`, `toggle`, `button`, `dropdown`, `region`, `indicator`, `spacer`, `surface`
- Math helper: `weighted_slot_lengths(total, weights)`

## Fluent Helpers
- Layout:
  - containers: `.layout(ContainerLayout::...)`, `.container_layout(...)`, `.fill()`, `.fill_width()`, `.fill_height()`
  - widgets: `.widget_layout(LayoutBox::...)`, `.fill()`, `.fill_width()`, `.fill_height()`
- Spacing: `.gap(...)`, `.gap_xy(...)`, `.pad_all(...)`, `.pad_xy(...)`
- Flex alignment: `.align_*()`, `.justify_*()`
- Panel styling: `.title(...)`, `.background(...)`, `.outline(...)`
- Widget tuning: `.control_size(...)`, `.value_label(...)`, `.selected(...)`

## Validation Guarantees
`measure_checked` and `render_checked` hard-fail with `DeclarativeError` when invalid:
- root key is required and unique
- root content must be a slot
- root slot child must be a container
- containers may only contain slots
- slots must contain a non-slot child
- slot tracks must be `Fraction` or `Fill`
- slot fractions must satisfy total/fill constraints
- non-root containers must use host-derived `Auto`/`Fill` sizing only (no pixel/min/max constraints)
- widget semantic checks (ranges, selected index, control size, key uniqueness)

## Layout Migration Notes
Container and widget layout APIs are now explicitly separated:
- Use `ContainerLayout` for `Panel` / `Row` / `Column` / `Grid` / `Absolute`.
- Use `LayoutBox` only for widget sizing and root frame sizing.

Before:
```rust
let title = label("Pump").layout(LayoutBox::fixed(64, 16).max(64, 16));
let panel_node = panel("main", title).layout(LayoutBox::fill());
```

After:
```rust
use toybox::gui::declarative::{ContainerLayout, LayoutBox};

let title = label("Pump").widget_layout(LayoutBox::fixed(64, 16).max(64, 16));
let panel_node = panel("main", title).layout(ContainerLayout::fill());
```

## Example
```rust
use toybox::gui::declarative::{
    button, panel, row, root_frame_sized, LayoutBox, UiAction, UiSpec, Size,
};

fn build(input: &toybox::clap::gui::InputState) -> UiSpec {
    let controls = row(vec![
        button("inc", "Increment"),
        button("dec", "Decrement"),
    ])
    .justify_space_between();

    UiSpec::new(
        root_frame_sized(
            "root",
            panel("main", controls).fill(),
            Size { width: 420, height: 258 },
            input.window_size,
        )
        .layout(LayoutBox::fill()),
    )
}

fn reduce(_action: UiAction) {}
```
