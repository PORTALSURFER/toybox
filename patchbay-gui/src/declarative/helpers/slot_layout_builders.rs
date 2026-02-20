/// Resolve weighted slot lengths that exactly consume the available space.
///
/// This uses the same deterministic largest-remainder distribution strategy as
/// Patchbay grid `Fr` track allocation so slot math stays consistent between
/// plugin-side sizing helpers and renderer-side layout.
///
/// Weights are clamped to at least `1` to match [`weighted_slot`].
pub fn weighted_slot_lengths(total: u32, weights: &[u16]) -> Vec<u32> {
    let total_weight: u32 = weights
        .iter()
        .map(|weight| u32::from((*weight).max(1)))
        .sum();
    if total_weight == 0 {
        return vec![0; weights.len()];
    }
    let normalized: Vec<u32> = weights
        .iter()
        .map(|weight| u32::from((*weight).max(1)))
        .collect();
    distribute_weighted_u32(total, &normalized)
}

/// Create a weighted full-size column slot layout.
///
/// Children fill width and split available height by relative `weight`.
pub fn column_slots(children: Vec<Slot>) -> Node {
    let rows: Vec<TrackSize> = children.iter().map(slot_track_to_grid_track).collect();
    let nodes: Vec<Node> = children
        .into_iter()
        .map(wrap_slot_child)
        .collect();
    let mut spec = GridSpec::new(
        GridTemplate::new(vec![TrackSize::Fr(1)])
            .rows(rows)
            .pad_all(0)
            .justify_start(),
        nodes,
    );
    spec.kind = GridKind::SlotColumn;
    Node::Grid(spec).container_layout(ContainerLayout::fill())
}

/// Create a weighted full-size row slot layout.
///
/// Children fill height and split available width by relative `weight`.
pub fn row_slots(children: Vec<Slot>) -> Node {
    let columns: Vec<TrackSize> = children.iter().map(slot_track_to_grid_track).collect();
    let nodes: Vec<Node> = children
        .into_iter()
        .map(wrap_slot_child)
        .collect();
    let mut spec = GridSpec::new(
        GridTemplate::new(columns)
            .rows(vec![TrackSize::Fr(1)])
            .pad_all(0)
            .justify_start(),
        nodes,
    );
    spec.kind = GridKind::SlotRow;
    Node::Grid(spec).container_layout(ContainerLayout::fill())
}

/// Convert high-level slot tracks into grid tracks.
fn slot_track_to_grid_track(child: &Slot) -> TrackSize {
    match child.params.size_main {
        SlotMainSize::Percent(percent) => TrackSize::Percent(percent),
        SlotMainSize::Fill(weight) => TrackSize::Fr(weight.max(1)),
        SlotMainSize::Intrinsic => TrackSize::Auto,
        SlotMainSize::Fixed(px) => TrackSize::Px(px),
    }
}

/// Wrap a slot child into a single-child container that applies strict
/// horizontal and vertical alignment.
fn wrap_slot_child(child: Slot) -> Node {
    let mut content = child.node;
    let min_width = child.params.min_width;
    let max_width = child.params.max_width;
    let min_height = child.params.min_height;
    let max_height = child.params.max_height;
    let align_x = child.params.align_x_override.unwrap_or(SlotAlign::Start);
    let align_y = child.params.align_y_override.unwrap_or(SlotAlign::Start);
    if matches!(align_x, SlotAlign::Stretch) || matches!(child.params.size_cross, SlotCrossSize::Fill) {
        content = content.fill_width();
    }
    if matches!(align_y, SlotAlign::Stretch) || matches!(child.params.size_cross, SlotCrossSize::Fill) {
        content = content.fill_height();
    }
    if is_widget_node(&content) {
        let mut layout = node_layout(&content);
        layout.min_width = min_width;
        layout.max_width = max_width;
        layout.min_height = min_height;
        layout.max_height = max_height;
        content = content.widget_layout(layout);
    }
    let justify = match align_x {
        SlotAlign::Start | SlotAlign::Stretch => Justify::Start,
        SlotAlign::Center => Justify::Center,
        SlotAlign::End => Justify::End,
    };
    let align = match align_y {
        SlotAlign::Start => Align::Start,
        SlotAlign::Center => Align::Center,
        SlotAlign::End => Align::End,
        SlotAlign::Stretch => Align::Stretch,
    };
    Node::Row(
        FlexSpec::row(vec![content])
            .padding(child.params.margin)
            .justify(justify)
            .align(align)
            .layout(ContainerLayout::fill()),
    )
}
