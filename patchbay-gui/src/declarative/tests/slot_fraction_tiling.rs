mod slot_fraction_axis_tests {
    include!("slot_fraction_axis_tests.rs");
}

mod slot_tiling_layout_tests {
    mod slot_tiling_layout_helpers {
        include!("slot_tiling_layout_helpers.rs");
    }
    mod slot_tiling_layout_assertions {
        include!("slot_tiling_layout_assertions.rs");
    }
}
