/// Resolved layout geometry for a toggle draw pass.
#[derive(Clone, Copy)]
struct ToggleLayoutResolved {
    /// Total vertical block consumed by toggle + optional label.
    block_size: Size,
    /// Toggle control rectangle.
    rect: Rect,
}
