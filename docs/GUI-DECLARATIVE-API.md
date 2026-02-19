# Strict Declarative GUI API

## Summary
`toybox::gui::declarative` is the only supported UI authoring surface.

System-level layout target and gap tracking:

- `GUI-STRICT-HIERARCHICAL-LAYOUT-SPEC.md`

The API is strict and data-only:
- Build a pure `UiSpec` tree each frame.
- Render through host integration (`render_checked_with_engine` internally on Win32 runtime path).
- Handle emitted `UiAction` values in a reducer.

## Core Tree Model
- `UiSpec` / `RootFrameSpec`: root window definition.
- `Node`: typed tree node.
- `SlotSpec`: single-child slot node used as the required direct child type for containers.
- Container nodes: `Panel`, `PaddingBox`, `AlignBox`, `AspectBox`, `Row`, `Column`, `Grid`, `Absolute`, `Stack`, `ScrollView`, `Wrap`, `SwitchLayout`.
- Widget nodes: `Label`, `Spacer`, `Knob`, `Slider`, `Toggle`, `Button`, `Dropdown`, `Region`, `Indicator`.

Canonical grammar:
- Root is a special container with exactly one slot.
- Containers directly host slots.
- Slots host exactly one child (`Container | Widget`).
- Widgets are leaves.

See `GUI-TREE-CONTRACT.md` for the full contract and failure cases.

## Constructors
- Containers: `row(children)`, `column(children)`, `grid(template, children)`, `panel(key, content)`, `padding_box(content)`, `align_box(content)`, `aspect_box(content, ratio)`, `stack(children)`, `scroll_view(content)`, `wrap(children)`, `switch_layout(cases, fallback)`, `root_frame_sized(...)`
- Switch cases: `when_width_lt(max, child)`, `when_width_between(min, max, child)`, `when_width_ge(min, child)`
- Slots/helpers: `slot(child)`, `weighted_slot(node, weight)`, `fraction_slot(node, percent)`, `fill_slot(node)`, `column_slots(...)`, `row_slots(...)`, `SlotParams`, `SlotMainSize`, `SlotCrossSize`
- Widgets: `label`, `knob`, `slider`, `toggle`, `button`, `dropdown`, `region`, `indicator`, `spacer`, `surface`
- Math helper: `weighted_slot_lengths(total, weights)`
- Engine API: `LayoutEngineState`, `render_checked_with_engine(...)`
- Engine invalidation API: `NodeId`, `node_id_for_key(...)`, `invalidate_layout_subtree(...)`, `invalidate_measure_subtree(...)`, `invalidate_all_layout(...)`, `invalidate_all_measure(...)`, `measure_cache_stats(...)`

## Fluent Helpers
- Layout:
  - containers: `.layout(ContainerLayout::...)`, `.overflow(OverflowPolicy::...)`, `.container_layout(...)`, `.container_overflow(...)`, `.fill()`, `.fill_width()`, `.fill_height()`
  - widgets: `.widget_layout(LayoutBox::...)`, `.fill()`, `.fill_width()`, `.fill_height()`
- Spacing: `.gap(...)`, `.gap_xy(...)`, `.pad_all(...)`, `.pad_xy(...)`
- Slot alignment for single-slot containers: `.slot_align(SlotAlign::..., SlotAlign::...)`
- Aspect-box ratio updates: `.aspect_ratio(width, height)`
- Flex alignment: `.align_*()`, `.justify_*()`
- Panel styling: `.title(...)`, `.background(...)`, `.outline(...)`
- Widget tuning: `.control_size(...)`, `.value_label(...)`, `.selected(...)`

## Validation Guarantees
`measure_checked` and `render_checked` hard-fail with `DeclarativeError` when invalid:
- root key is required and unique
- root content must be a slot
- root slot child must be a container
- declarative tree depth must stay within the fail-fast validation limit
- containers may only contain slots
- slots must contain a non-slot child
- slot percentages must satisfy deterministic total/fill constraints
- non-root containers must use host-derived `Auto`/`Fill` sizing only (no pixel/min/max constraints)
- widget semantic checks (ranges, selected index, control size, key uniqueness)

`RenderResult` includes:
- `layout_diagnostics` with stable messages and typed `LayoutDiagnosticCode` values
- `overflow` (`LayoutOverflowSummary`) with deterministic `clipped` / `compressed` / `skipped` / `total` counters
For detailed per-node geometry diagnostics, set `RootFrameSpec::layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode)` and read `RenderResult.node_layout_diagnostics`.

`LayoutEngineState` no longer exposes mutable root dirty flags. Use explicit invalidation:
- `node_id_for_key("widget-key")` to resolve deterministic node identity.
- `invalidate_layout_subtree(node_id)` for geometry-only subtree changes.
- `invalidate_measure_subtree(node_id)` for intrinsic/content changes (also marks layout dirty).
- `invalidate_all_layout()` / `invalidate_all_measure()` for full-tree invalidation.
- `measure_cache_stats()` and `last_registry_version()` for read-only diagnostics.

Win32 host runtime keeps one persistent `LayoutEngineState` per window and
uses keyed subtree invalidation after reducer-applied UI actions:
- measure-subtree invalidation for control/value actions
- layout-subtree invalidation for interaction-only region actions
- full-tree measure fallback only when a key cannot be resolved

## Layout Migration Notes
Container and widget layout APIs are now explicitly separated:
- Use `ContainerLayout` for `Panel` / `PaddingBox` / `AlignBox` / `AspectBox` / `Row` / `Column` / `Grid` / `Absolute` / `Stack` / `ScrollView` / `Wrap` / `SwitchLayout`.
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
