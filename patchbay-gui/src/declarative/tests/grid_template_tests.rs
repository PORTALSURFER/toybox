use super::super::*;

#[test]
fn grid_gap_sets_both_axes() {
    let template = GridTemplate::columns_fr(2).gap(9);
    assert_eq!(template.column_gap, 9);
    assert_eq!(template.row_gap, 9);
}

#[test]
fn grid_template_defaults_to_tight_left_packing() {
    let template = GridTemplate::columns_fr(3);
    assert_eq!(template.column_gap, 0);
    assert_eq!(template.row_gap, 0);
    assert_eq!(template.justify_x, Justify::Start);
}

#[test]
fn grid_template_justify_helpers_set_horizontal_distribution() {
    assert_eq!(
        GridTemplate::columns_fr(2).justify_center().justify_x,
        Justify::Center
    );
    assert_eq!(
        GridTemplate::columns_fr(2).justify_end().justify_x,
        Justify::End
    );
    assert_eq!(
        GridTemplate::columns_fr(2).justify_space_between().justify_x,
        Justify::SpaceBetween
    );
    assert_eq!(
        GridTemplate::columns_fr(2).justify_space_around().justify_x,
        Justify::SpaceAround
    );
    assert_eq!(
        GridTemplate::columns_fr(2).justify_space_evenly().justify_x,
        Justify::SpaceEvenly
    );
}
