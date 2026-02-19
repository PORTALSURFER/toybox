//! Compile-fail guards for strict declarative API invariants.
//!
//! These tests lock in "impossible by API" guarantees so container layout
//! remains host-derived and internal slot/container structure cannot be mutated
//! externally.

#[test]
fn declarative_api_rejects_invalid_container_authoring_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/container_layout_rejects_layoutbox.rs");
    cases.compile_fail("tests/ui/grid_kind_field_is_private.rs");
    cases.compile_fail("tests/ui/layout_diagnostic_constraint_normalized_removed.rs");
    cases.compile_fail("tests/ui/layout_box_struct_literal_private.rs");
    cases.compile_fail("tests/ui/layout_engine_state_legacy_mark_dirty_removed.rs");
    cases.compile_fail("tests/ui/layout_engine_state_struct_literal_private.rs");
    cases.compile_fail("tests/ui/panel_spec_struct_literal_private.rs");
    cases.compile_fail("tests/ui/root_frame_spec_new_is_private.rs");
}
