# GUI Migration Guide (Legacy to Strict Declarative)

## Purpose
This guide describes how to migrate plugin UIs from legacy callback-style declarative APIs to the strict declarative `toybox::gui::declarative` surface.

## Required End State
- Build a pure-data `UiSpec` each frame.
- Handle interactions only through `UiAction` in a reducer.
- Avoid callback-bearing widget specs and immediate-mode authoring types.

## API Mapping
| Legacy pattern | Strict declarative replacement |
| --- | --- |
| `UiSpec<'static, State>` | `UiSpec` |
| `RootFrameSpec { size: SizeSpec::... }` | `RootFrameSpec::new(...).layout(LayoutBox::...)` |
| `PanelSpec { size: SizeSpec::... }` | `PanelSpec::new(...).layout(LayoutBox::...)` |
| `FlexSpec { size: SizeSpec::..., padding: Padding::... }` | `FlexSpec::row/column(...).layout(...).pad_all/pad_xy(...)` |
| `Node::Widget(WidgetSpec { render: ... })` | Compose built-in nodes (`Label`, `Knob`, `Slider`, `Toggle`, `Button`, `Dropdown`, `Region`, `Grid`, `Panel`) and use `RegionSpec::draw_commands(...)` for custom drawing |
| `ButtonSpec::on_interaction` | `UiAction::ButtonPressed` in reducer |
| `DropdownSpec::on_interaction` | `UiAction::DropdownSelected` in reducer |
| `KnobSpec::on_interaction` | `UiAction::KnobChanged` in reducer |
| `ToggleSpec::on_interaction` | `UiAction::ToggleChanged` in reducer |
| `measure(...)` | `measure_checked(...)` |

## Migration Workflow
1. Replace legacy declarative imports with strict declarative imports.
2. Convert `SizeSpec` to `LayoutBox` constraints.
3. Remove node callbacks and move behavior into the reducer.
4. Keep stable keys per control (`section/control` naming is recommended).
5. Run `measure_checked` in tests for structural and semantic validation.

## Interaction Migration Pattern
1. Build function emits controls keyed by stable IDs.
2. Reducer matches `UiAction` variants and updates state or automation queues.
3. Any side effects (host parameter flush, preset load) happen in reducer branches.

## Validation Rules You Must Satisfy
- Root key is non-empty and must not collide with keyed descendants.
- All keyed interactive nodes must be unique and non-empty.
- Knob/slider range must be finite and increasing (`min < max`).
- Knob/slider current value must be finite and inside the declared range.
- Dropdown `selected` index must be within options bounds.
- Explicit `control_size` values must be non-zero.

## Recommended Rollout Order
1. Migrate static panels and text sections.
2. Migrate standard controls (button/toggle/slider/knob/dropdown).
3. Migrate custom interactive regions.
4. Add layout polish (`justify_space_*`, `pad_xy`, `columns_fr/rows_fr`) after behavior parity is restored.
