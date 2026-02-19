use patchbay_gui::{textbox, GridKind, GridSpec, GridTemplate, TrackSize};

fn main() {
    let mut grid = GridSpec::new(
        GridTemplate::new(vec![TrackSize::Fr(1)]).rows(vec![TrackSize::Fr(1)]),
        vec![textbox("cell")],
    );
    grid.kind = GridKind::SlotRow;
}
