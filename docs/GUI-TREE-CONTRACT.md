# GUI Tree Contract

## Formal Grammar
- `UiSpec -> RootFrameSpec`
- `RootFrameSpec -> Slot`
- `Slot -> (Container | Widget)` and exactly one child
- `Container -> Vec<Slot>`
- `Widget -> leaf`

Container node kinds:
- `Panel`
- `Row`
- `Column`
- `Grid`
- `Absolute`

Widget node kinds:
- `Label`
- `Spacer`
- `Knob`
- `Slider`
- `Toggle`
- `Button`
- `Dropdown`
- `Region`
- `Indicator`

## Slot Track Policy
For canonical section-style layouts (`row_slots`, `column_slots`):
- allowed tracks: `Fraction`, `Fill`
- disallowed tracks: `Px`

## Hard Validation Rules
- root key is required and unique
- root content must be a `Slot`
- root slot child must be a `Container`
- container direct children must be `Slot`
- slot child may not be another `Slot`
- slot child must be `Container | Widget`
- slot fractions/fills must satisfy deterministic total rules

## Valid Example (4x4 Grid Nested in Slot Layout)
```rust
let matrix = grid(
    GridTemplate::columns_fr(4).rows_fr(4),
    vec![
        knob("k1", "A", 0.4, (0.0, 1.0)),
        knob("k2", "B", 0.5, (0.0, 1.0)),
    ],
);

let content = column_slots(vec![
    weighted_slot(panel("header", label("Header")), 20),
    weighted_slot(panel("matrix", matrix), 80),
]);

let spec = UiSpec::new(root_frame_sized(
    "root",
    content,
    Size { width: 420, height: 258 },
    input.window_size,
));
```

## Invalid Example (Container Child is a Widget, No Slot)
```rust
// Invalid: direct widget child under row container.
let bad = Node::Row(FlexSpec::row(vec![
    label("Direct child"),
]));
```

This fails with `InvalidContainerChild`.
