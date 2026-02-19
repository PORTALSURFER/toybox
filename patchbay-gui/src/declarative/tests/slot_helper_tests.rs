use super::super::*;

#[test]
fn helper_layout_box_methods_apply_expected_constraints() {
    let layout = LayoutBox::auto()
        .fill_width()
        .fixed_height(24)
        .min(10, 20)
        .max(200, 30);
    assert_eq!(layout.width, Length::Fill(1));
    assert_eq!(layout.height, Length::Px(24));
    assert_eq!(layout.min_width, Some(10));
    assert_eq!(layout.min_height, Some(20));
    assert_eq!(layout.max_width, Some(200));
    assert_eq!(layout.max_height, Some(30));
}

#[test]
fn helper_justify_methods_apply_expected_distribution_modes() {
    let flex = FlexSpec::row(vec![textbox("A"), textbox("B")]).justify_space_between();
    assert_eq!(flex.justify, Justify::SpaceBetween);

    let flex = FlexSpec::row(vec![textbox("A"), textbox("B")]).justify_space_around();
    assert_eq!(flex.justify, Justify::SpaceAround);

    let flex = FlexSpec::row(vec![textbox("A"), textbox("B")]).justify_space_evenly();
    assert_eq!(flex.justify, Justify::SpaceEvenly);
}

#[test]
fn weighted_child_clamps_zero_weight_to_one() {
    let child = weighted_slot(textbox("x"), 0);
    assert_eq!(child.params.size_main, SlotMainSize::Fill(1));
}

#[test]
fn column_slots_apply_weighted_height_fill() {
    let node = column_slots(vec![weighted_slot(textbox("A"), 7), weighted_slot(textbox("B"), 30)]);
    let Node::Grid(grid) = node else {
        panic!("expected grid-backed column slot container");
    };
    assert_eq!(grid.layout, ContainerLayout::fill());
    assert_eq!(grid.template.columns, vec![TrackSize::Fr(1)]);
    assert_eq!(
        grid.template.rows,
        vec![TrackSize::Fr(7), TrackSize::Fr(30)]
    );
    assert_eq!(grid.template.column_gap, 0);
    assert_eq!(grid.template.row_gap, 0);
    assert_eq!(grid.template.padding, EdgeInsets::all(0));
    assert_eq!(grid.template.justify_x, Justify::Start);
    assert_eq!(grid.children.len(), 2);

    let first = node_layout(&grid.children[0]);
    assert_eq!(first.width, Length::Fill(1));
    assert_eq!(first.height, Length::Fill(1));

    let second = node_layout(&grid.children[1]);
    assert_eq!(second.width, Length::Fill(1));
    assert_eq!(second.height, Length::Fill(1));
}

#[test]
fn row_slots_apply_weighted_width_fill() {
    let node = row_slots(vec![weighted_slot(textbox("L"), 70), weighted_slot(textbox("R"), 30)]);
    let Node::Grid(grid) = node else {
        panic!("expected grid-backed row slot container");
    };
    assert_eq!(grid.layout, ContainerLayout::fill());
    assert_eq!(
        grid.template.columns,
        vec![TrackSize::Fr(70), TrackSize::Fr(30)]
    );
    assert_eq!(grid.template.rows, vec![TrackSize::Fr(1)]);
    assert_eq!(grid.template.column_gap, 0);
    assert_eq!(grid.template.row_gap, 0);
    assert_eq!(grid.template.padding, EdgeInsets::all(0));
    assert_eq!(grid.template.justify_x, Justify::Start);
    assert_eq!(grid.children.len(), 2);

    let left = node_layout(&grid.children[0]);
    assert_eq!(left.width, Length::Fill(1));
    assert_eq!(left.height, Length::Fill(1));

    let right = node_layout(&grid.children[1]);
    assert_eq!(right.width, Length::Fill(1));
    assert_eq!(right.height, Length::Fill(1));
}

#[test]
fn container_overflow_builder_sets_policy_without_resetting_fill_axes() {
    let node = row_slots(vec![weighted_slot(textbox("A"), 50), weighted_slot(textbox("B"), 50)])
        .container_overflow(OverflowPolicy::Compress)
        .fill();
    let Node::Grid(grid) = node else {
        panic!("expected grid-backed row slot container");
    };
    assert_eq!(grid.overflow_policy(), OverflowPolicy::Compress);
    assert_eq!(
        grid.container_layout(),
        ContainerLayout::fill().overflow(OverflowPolicy::Compress)
    );
}
