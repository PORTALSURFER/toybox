use super::super::*;

#[test]
fn slot_tracks_allocate_percentages_before_fill_remainder() {
    let widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &[
            TrackSize::Percent(25),
            TrackSize::Percent(35),
            TrackSize::Fill,
        ],
        columns: 3,
        rows: 1,
        gap: 0,
        available: 200,
        is_columns: true,
        intrinsic: &[Size {
            width: 0,
            height: 0,
        }; 3],
    });
    assert_eq!(widths, vec![50, 70, 80]);
    assert_eq!(widths.iter().sum::<u32>(), 200);
}

#[test]
fn slot_grid_allows_weighted_fill_tracks_without_exact_percent_total() {
    let spec = UiSpec::new(root_frame_sized(
        "root",
        column_slots(vec![
            weighted_slot(panel("top", textbox("Top")).pad_all(0), 10),
            weighted_slot(panel("bottom", textbox("Bottom")).pad_all(0), 20),
        ]),
        Size {
            width: 300,
            height: 180,
        },
    ));
    let measured = measure_checked(&spec).expect("fill-track weights are normalized to parent bounds");
    assert_eq!(
        measured,
        Size {
            width: 300,
            height: 180,
        }
    );
}

#[test]
fn slot_grid_allows_sub_hundred_percent_when_fill_present() {
    let spec = UiSpec::new(root_frame_sized(
        "root",
        column_slots(vec![
            fraction_slot(panel("fixed", textbox("Fixed")).pad_all(0), 30),
            fill_slot(panel("fill", textbox("Fill")).pad_all(0)),
        ]),
        Size {
            width: 300,
            height: 180,
        },
    ));
    let measured = measure_checked(&spec).expect("fill should absorb remaining slot space");
    assert_eq!(
        measured,
        Size {
            width: 300,
            height: 180,
        }
    );
}

#[test]
fn weighted_slot_lengths_consume_total_without_gaps() {
    let heights = weighted_slot_lengths(259, &[7, 63, 30]);
    assert_eq!(heights, vec![18, 163, 78]);
    assert_eq!(heights.iter().sum::<u32>(), 259);

    let widths = weighted_slot_lengths(799, &[70, 30]);
    assert_eq!(widths, vec![559, 240]);
    assert_eq!(widths.iter().sum::<u32>(), 799);
}
