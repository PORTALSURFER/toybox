use super::*;

mod knob_updates_value_on_drag_tests {
    include!("00_knob_updates_value_on_drag_tests.rs");
}

mod toggle_labels_are_clamped_to_control_width_tests {
    include!("01_toggle_labels_are_clamped_to_control_width_tests.rs");
}

mod button_reports_click_tests {
    include!("02_button_reports_click_tests.rs");
}

mod dropdown_interaction_tests {
    include!("03_dropdown_interaction_tests.rs");
}

mod curve_editor_vector_quality_tests {
    include!("04_curve_editor_vector_quality_tests.rs");
}

mod curve_editor_snap_tests {
    include!("05_curve_editor_snap_tests.rs");
}

mod curve_editor_segment_move_tests {
    include!("06_curve_editor_segment_move_tests.rs");
}

mod curve_editor_point_constraint_tests {
    include!("07_curve_editor_point_constraint_tests.rs");
}
