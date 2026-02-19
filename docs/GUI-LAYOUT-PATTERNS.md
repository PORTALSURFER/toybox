# GUI Layout Patterns

## Purpose
Repeatable recipes for strict container/slot/widget layouts.

## Canonical Root + Fractional Slots
```rust
let controls = row_slots(vec![
    weighted_slot(panel("knobs", knobs).pad_all(8), 70),
    weighted_slot(panel("dropdowns", dropdowns).pad_all(8), 30),
]);

let content = column_slots(vec![
    weighted_slot(panel("header", header).pad_all(0), 7),
    weighted_slot(panel("curve", curve).pad_all(0), 63),
    weighted_slot(panel("controls", controls).pad_all(0), 30),
]);

let root = root_frame_sized(
    "root",
    content,
    Size { width: 420, height: 258 },
    input.window_size,
)
.padding(0);
```

## Equal-Width Control Row
```rust
let row = row(vec![
    slider("mix", "Mix", 0.5, (0.0, 1.0)),
    slider("tone", "Tone", 0.3, (0.0, 1.0)),
    slider("drive", "Drive", 0.2, (0.0, 1.0)),
])
.justify_space_between()
.gap(8)
.fill_width();
```

## 4x4 Parameter Grid
```rust
let controls = grid(
    GridTemplate::columns_fr(4)
        .rows_fr(4)
        .gap_xy(10, 14)
        .pad_all(8),
    vec![
        knob("k1", "A", 0.4, (0.0, 1.0)),
        knob("k2", "B", 0.5, (0.0, 1.0)),
        knob("k3", "C", 0.6, (0.0, 1.0)),
        knob("k4", "D", 0.7, (0.0, 1.0)),
    ],
);
```

## Notes
- Prefer `root_frame_sized` with `column_slots`/`row_slots` for top-level composition.
- Use slot helpers (`weighted_slot`, `fraction_slot`, `fill_slot`) for slot-based splits.
- Slot tracks are fraction/fill only.
- Keep keys stable and descriptive (`slot/control` style).
