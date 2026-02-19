pub fn assert_tracks_tile_parent_exactly(parent_extent: u32, gap: i32, tracks: &[u32]) {
    let gap_u32 = gap.max(0) as u32;
    let gap_total = gap_u32.saturating_mul(tracks.len().saturating_sub(1) as u32);
    let tracks_total = tracks.iter().copied().sum::<u32>();
    assert_eq!(
        tracks_total.saturating_add(gap_total),
        parent_extent,
        "tracks must exactly consume parent extent with configured gaps"
    );
    let mut cursor = 0u32;
    for track in tracks {
        cursor = cursor.saturating_add(*track);
        cursor = cursor.saturating_add(gap_u32);
    }
    let used = cursor.saturating_sub(gap_u32);
    assert_eq!(used, parent_extent, "no trailing slack is allowed");
}
