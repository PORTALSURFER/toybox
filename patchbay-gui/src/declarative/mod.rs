//! Strict declarative layout primitives for Patchbay GUI widgets.
//!
//! This module defines a pure-data UI specification and a renderer that emits
//! typed actions. UI state mutation is intentionally kept outside of the tree
//! via an explicit reducer step.

include!("contracts/errors.rs");
include!("contracts/ui_actions.rs");
include!("contracts/render_result.rs");
include!("contracts/root_transform.rs");
include!("root_spec_and_node.rs");
include!("node_builders_and_sections.rs");
include!("helpers/section_layout_builders.rs");
include!("helpers/node_constructors.rs");
include!("layout/box_types.rs");
include!("flex_and_grid_layout_types.rs");
include!("container_and_widget_specs_a.rs");
include!("specs/knob_slider.rs");
include!("specs/toggle_button_dropdown_region_indicator.rs");
include!("theme/core_tokens.rs");
include!("render/entry/theme_tokens.rs");
include!("render/entry/root_render_plan.rs");
include!("render/entry/debug_border_candidates.rs");
include!("render/entry/render_checked.rs");
include!("debug/border_selection.rs");
include!("validation/node_tree.rs");
include!("validation/rules.rs");
include!("measurement/root_frame.rs");
include!("measurement/containers.rs");
include!("measurement/grid.rs");
include!("measurement/widgets.rs");
include!("render/panel.rs");
include!("render/flex/types.rs");
include!("render/flex/main_axis.rs");
include!("render/flex/render.rs");
include!("render/grid/axis.rs");
include!("render/grid/layout.rs");
include!("render/absolute.rs");
include!("render/label.rs");
include!("render/dispatch.rs");
include!("render/widgets.rs");
include!("render/region.rs");
include!("layout_geometry_and_formatting.rs");

#[cfg(test)]
mod tests;
