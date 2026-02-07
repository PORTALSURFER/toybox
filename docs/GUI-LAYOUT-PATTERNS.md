# GUI Layout Patterns

## Purpose
This document provides repeatable layout recipes for strict declarative plugin UIs.

## Root + Section Stack
Use a root frame that fills host space, then stack panels in a column.

```rust
let root = RootFrameSpec::new(
    "root",
    column(vec![
        panel("header", label("Header")).fill_width(),
        panel("body", body_node).fill(),
    ])
    .gap(12)
    .pad_xy(16, 12),
)
.layout(LayoutBox::fill());
```

## Equal-Width Control Row
Use `Fill` widths and `justify_space_between` when parent width is larger than content.

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

## Parameter Grid
Use fractional columns for adaptive control grids.

```rust
let controls = grid(
    GridTemplate::columns_fr(4).gap_xy(10, 14).pad_all(8),
    vec![
        knob("k1", "A", 0.4, (0.0, 1.0)),
        knob("k2", "B", 0.5, (0.0, 1.0)),
        knob("k3", "C", 0.6, (0.0, 1.0)),
        knob("k4", "D", 0.7, (0.0, 1.0)),
    ],
);
```

## Label + Control Group
Bundle related text and controls in a small column with explicit spacing.

```rust
let group = column(vec![
    label("Envelope"),
    row(vec![
        knob("attack", "Attack", 0.2, (0.0, 1.0)),
        knob("decay", "Decay", 0.5, (0.0, 1.0)),
        knob("sustain", "Sustain", 0.8, (0.0, 1.0)),
        knob("release", "Release", 0.4, (0.0, 1.0)),
    ]),
]);
```

## Interaction Region + Controls
Use a region for pointer interaction and a control rail beneath it.

```rust
let graph_section = column(vec![
    region("graph", Size { width: 520, height: 240 }),
    row(vec![
        toggle("snap", "Snap", true),
        button("reset", "Reset"),
    ]).gap(8),
]);
```

## Notes
- Prefer `LayoutBox::fill_width()` for panels in stacked columns.
- `LayoutBox::fixed(w, h)` defines a minimum baseline, not a hard cap; content can grow beyond it.
- Use `LayoutBox::auto().max(w, h)` when you need strict clipping/capping behavior.
- Declarative `Label` rendering is single-line and hard-clamped to its assigned rect (no ellipsis), so provide explicit label boxes when overflow must be prevented.
- Keep keys stable and descriptive (`section/control`).
- Start with `justify_start` and add `justify_space_*` only when you need distribution across slack space.
