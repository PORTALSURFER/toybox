use patchbay_gui::{label, GridKind, GridSpec, GridTemplate, TrackSize};

fn main() {
    let mut grid = GridSpec::new(
        GridTemplate::new(vec![TrackSize::Fr(1)]).rows(vec![TrackSize::Fr(1)]),
        vec![label("cell")],
    );
    grid.kind = GridKind::SlotRow;
}
