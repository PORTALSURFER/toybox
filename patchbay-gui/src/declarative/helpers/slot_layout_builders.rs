/// Resolve weighted slot lengths that exactly consume the available space.
///
/// This uses the same deterministic largest-remainder distribution strategy as
/// Patchbay grid `Fr` track allocation so slot math stays consistent between
/// plugin-side sizing helpers and renderer-side layout.
///
/// Weights are clamped to at least `1` to match [`weighted_slot`].
pub fn weighted_slot_lengths(total: u32, weights: &[u16]) -> Vec<u32> {
    let total_percent: u16 = weights.iter().copied().sum();
    if total_percent == 0 {
        return vec![0; weights.len()];
    }
    let target_total = total
        .saturating_mul(total_percent as u32)
        .saturating_div(100);
    let normalized: Vec<u32> = weights.iter().map(|weight| u32::from(*weight)).collect();
    distribute_weighted_u32(target_total, &normalized)
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
            .gap(0)
            .pad_all(0)
            .justify_start(),
        nodes,
    );
    spec.kind = GridKind::SectionColumn;
    Node::Grid(spec).layout(LayoutBox::fill())
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
            .gap(0)
            .pad_all(0)
            .justify_start(),
        nodes,
    );
    spec.kind = GridKind::SectionRow;
    Node::Grid(spec).layout(LayoutBox::fill())
}

/// Convert high-level slot tracks into grid tracks.
fn slot_track_to_grid_track(child: &Slot) -> TrackSize {
    match child.size {
        SlotTrack::Fraction(percent) => TrackSize::Percent(percent),
        SlotTrack::Fill => TrackSize::Fill,
    }
}

/// Wrap a slot child into a single-child container that applies strict
/// horizontal and vertical alignment.
fn wrap_slot_child(child: Slot) -> Node {
    let mut content = child.node;
    if matches!(child.align_x, SlotAlign::Stretch) {
        content = content.layout(LayoutBox::auto().fill_width());
    }
    if matches!(child.align_y, SlotAlign::Stretch) {
        content = content.layout(LayoutBox::auto().fill_height());
    }
    let justify = match child.align_x {
        SlotAlign::Start | SlotAlign::Stretch => Justify::Start,
        SlotAlign::Center => Justify::Center,
        SlotAlign::End => Justify::End,
    };
    let align = match child.align_y {
        SlotAlign::Start => Align::Start,
        SlotAlign::Center => Align::Center,
        SlotAlign::End => Align::End,
        SlotAlign::Stretch => Align::Stretch,
    };
    Node::Row(
        FlexSpec::row(vec![content])
            .gap(0)
            .pad_all(0)
            .justify(justify)
            .align(align)
            .layout(LayoutBox::fill()),
    )
}
