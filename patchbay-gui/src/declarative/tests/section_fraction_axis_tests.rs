use super::super::*;

#[test]
fn section_tracks_allocate_percentages_before_fill_remainder() {
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
fn section_grid_requires_exact_hundred_percent_without_fill() {
    let spec = UiSpec::new(root_frame_sized(
        "root",
        column_sections(vec![
            weighted(panel("top", label("Top")).pad_all(0), 10),
            weighted(panel("bottom", label("Bottom")).pad_all(0), 20),
        ]),
        Size {
            width: 300,
            height: 180,
        },
        Size {
            width: 300,
            height: 180,
        },
    ));
    let error = measure_checked(&spec).expect_err("fractions below 100% must be rejected");
    assert!(matches!(
        error,
        DeclarativeError::InvalidSectionFractions {
            total_percent: 30,
            fill_count: 0
        }
    ));
}

#[test]
fn section_grid_allows_sub_hundred_percent_when_fill_present() {
    let spec = UiSpec::new(root_frame_sized(
        "root",
        column_sections(vec![
            fraction(panel("fixed", label("Fixed")).pad_all(0), 30),
            fill_section(panel("fill", label("Fill")).pad_all(0)),
        ]),
        Size {
            width: 300,
            height: 180,
        },
        Size {
            width: 300,
            height: 180,
        },
    ));
    let measured = measure_checked(&spec).expect("fill should absorb remaining section space");
    assert_eq!(
        measured,
        Size {
            width: 300,
            height: 180,
        }
    );
}

#[test]
fn weighted_section_lengths_consume_total_without_gaps() {
    let heights = weighted_section_lengths(259, &[7, 63, 30]);
    assert_eq!(heights, vec![18, 163, 78]);
    assert_eq!(heights.iter().sum::<u32>(), 259);

    let widths = weighted_section_lengths(799, &[70, 30]);
    assert_eq!(widths, vec![559, 240]);
    assert_eq!(widths.iter().sum::<u32>(), 799);
}
