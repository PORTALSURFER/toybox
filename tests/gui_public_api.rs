//! External compile coverage for the public GUI facade.

#![cfg(feature = "gui")]

use patchbay_gui::CurveEditorModifier as PatchbayCurveEditorModifier;
use toybox::gui::declarative::CurveEditorModifier as ToyboxCurveEditorModifier;

#[test]
fn curve_editor_modifier_is_nameable_through_supported_public_apis() {
    let direct = PatchbayCurveEditorModifier::Command;
    let facade = ToyboxCurveEditorModifier::Command;

    assert_eq!(direct, facade);
}
