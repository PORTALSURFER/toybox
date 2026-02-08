//! Strict declarative layout primitives for Patchbay GUI widgets.
//!
//! This module defines a pure-data UI specification and a renderer that emits
//! typed actions. UI state mutation is intentionally kept outside of the tree
//! via an explicit reducer step.

include!("errors_actions_root_render.rs");
include!("root_spec_and_node.rs");
include!("node_builders_and_sections.rs");
include!("builder_helpers_and_layout_box.rs");
include!("flex_and_grid_layout_types.rs");
include!("container_and_widget_specs_a.rs");
include!("widget_specs_b_and_tokens.rs");
include!("render_entry_and_debug_candidates.rs");
include!("debug_selection_and_validation.rs");
include!("measure_and_render_dispatch.rs");
include!("render_panel_and_flex.rs");
include!("render_grid_absolute_and_tracks.rs");
include!("render_widgets_and_regions.rs");
include!("layout_geometry_and_formatting.rs");

#[cfg(test)]
mod tests;
