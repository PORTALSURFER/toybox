//! External compile coverage for the public GUI facade.

#![cfg(feature = "gui")]

use patchbay_gui::{
    CurveEditorModifier as PatchbayCurveEditorModifier,
    CurvePointHorizontalConstraintModifier as PatchbayCurvePointHorizontalConstraintModifier,
    CurvePointVerticalConstraintModifier as PatchbayCurvePointVerticalConstraintModifier,
};
use toybox::gui::declarative::{
    CurveEditorModifier as ToyboxCurveEditorModifier,
    CurvePointHorizontalConstraintModifier as ToyboxCurvePointHorizontalConstraintModifier,
    CurvePointVerticalConstraintModifier as ToyboxCurvePointVerticalConstraintModifier,
    CurveSegmentMoveOptions as ToyboxCurveSegmentMoveOptions,
};

#[test]
fn curve_editor_modifier_is_nameable_through_supported_public_apis() {
    let direct = PatchbayCurveEditorModifier::Command;
    let facade = ToyboxCurveEditorModifier::Command;
    let direct_shift: PatchbayCurvePointHorizontalConstraintModifier =
        PatchbayCurveEditorModifier::Shift;
    let facade_shift: ToyboxCurvePointHorizontalConstraintModifier =
        ToyboxCurveEditorModifier::Shift;
    let direct_shift_option: PatchbayCurvePointVerticalConstraintModifier =
        PatchbayCurveEditorModifier::ShiftOption;
    let facade_shift_option: ToyboxCurvePointVerticalConstraintModifier =
        ToyboxCurveEditorModifier::ShiftOption;

    assert_eq!(direct, facade);
    assert_eq!(direct_shift, facade_shift);
    assert_eq!(direct_shift_option, facade_shift_option);
}

#[test]
fn legacy_curve_editor_modifier_matches_remain_exhaustive() {
    fn direct_name(modifier: PatchbayCurveEditorModifier) -> &'static str {
        match modifier {
            PatchbayCurveEditorModifier::Command => "command",
        }
    }

    fn facade_name(modifier: ToyboxCurveEditorModifier) -> &'static str {
        match modifier {
            ToyboxCurveEditorModifier::Command => "command",
        }
    }

    assert_eq!(direct_name(PatchbayCurveEditorModifier::Command), "command");
    assert_eq!(facade_name(ToyboxCurveEditorModifier::Command), "command");
}

#[test]
fn legacy_curve_option_struct_literals_remain_exhaustive() {
    let interaction = patchbay_gui::CurveInteractionOptions {
        max_points: 64,
        min_point_spacing_x: 1.0e-4,
        drag_start_threshold_px: 3,
        push_through_threshold_px: 2,
        endpoint_mode: patchbay_gui::EndpointMode::Independent,
        double_click_delete_interior: true,
        whole_curve_offset: false,
        snap: patchbay_gui::CurveSnapConfig::default(),
    };
    let style = patchbay_gui::CurveEditorStyle {
        background: patchbay_gui::Color::rgb(20, 22, 22),
        border: patchbay_gui::Color::rgb(80, 85, 80),
        grid_vertical: patchbay_gui::Color::rgb(39, 43, 40),
        grid_vertical_emphasis: patchbay_gui::Color::rgb(69, 76, 71),
        grid_horizontal: patchbay_gui::Color::rgb(53, 58, 53),
        line: patchbay_gui::Color::rgb(140, 230, 220),
        line_highlight: patchbay_gui::Color::rgb(199, 250, 242),
        node_fill: patchbay_gui::Color::rgb(170, 180, 170),
        node_stroke: patchbay_gui::Color::rgb(110, 120, 110),
        node_hover_fill: patchbay_gui::Color::rgb(220, 236, 220),
        node_hover_stroke: patchbay_gui::Color::rgb(125, 140, 125),
        node_selected_fill: patchbay_gui::Color::rgb(240, 250, 240),
        node_selected_stroke: patchbay_gui::Color::rgb(130, 145, 130),
        preview_fill: patchbay_gui::Color::rgba(170, 240, 232, 96),
        preview_stroke: patchbay_gui::Color::rgb(160, 230, 222),
        playhead_core: patchbay_gui::Color::rgb(220, 230, 220),
        playhead_stroke: patchbay_gui::Color::rgb(124, 136, 124),
        highlight_mode: patchbay_gui::CurveHighlightMode::BrightCircle,
    };

    assert_eq!(
        interaction,
        patchbay_gui::CurveInteractionOptions::default()
    );
    assert_eq!(style, patchbay_gui::CurveEditorStyle::default());
}

#[test]
fn segment_move_is_opted_into_without_extending_legacy_struct_literals() {
    let options = patchbay_gui::CurveSegmentMoveOptions::new(
        PatchbayCurveEditorModifier::Command,
        patchbay_gui::Color::rgb(4, 5, 6),
    );
    let facade_options: ToyboxCurveSegmentMoveOptions = options;
    let model = patchbay_gui::CurveModel::new(
        vec![
            patchbay_gui::CurvePoint::new(0.0, 0.0),
            patchbay_gui::CurvePoint::new(1.0, 1.0),
        ],
        vec![patchbay_gui::CurveSegment::new(0.0)],
    );
    let node = patchbay_gui::curve_editor("curve", model).curve_segment_move(facade_options);

    assert!(matches!(node, patchbay_gui::Node::Slot(_)));
}

#[test]
fn point_horizontal_constraint_is_an_opt_in_declarative_decorator() {
    let model = patchbay_gui::CurveModel::new(
        vec![
            patchbay_gui::CurvePoint::new(0.0, 0.0),
            patchbay_gui::CurvePoint::new(1.0, 1.0),
        ],
        vec![patchbay_gui::CurveSegment::new(0.0)],
    );
    let node = patchbay_gui::curve_editor("curve", model)
        .curve_point_horizontal_constraint(ToyboxCurveEditorModifier::Shift);

    assert!(matches!(node, patchbay_gui::Node::Slot(_)));
}

#[test]
fn point_vertical_constraint_is_an_opt_in_declarative_decorator() {
    let model = patchbay_gui::CurveModel::new(
        vec![
            patchbay_gui::CurvePoint::new(0.0, 0.0),
            patchbay_gui::CurvePoint::new(1.0, 1.0),
        ],
        vec![patchbay_gui::CurveSegment::new(0.0)],
    );
    let node = patchbay_gui::curve_editor("curve", model)
        .curve_point_vertical_constraint(ToyboxCurveEditorModifier::ShiftOption);

    assert!(matches!(node, patchbay_gui::Node::Slot(_)));
}
