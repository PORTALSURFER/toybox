/// Grid track sizing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackSize {
    /// Fixed track size.
    Px(u32),
    /// Track size from intrinsic content.
    Auto,
    /// Track size as a percentage of parent axis space.
    Percent(u8),
    /// Track that receives equal shares of remaining axis space.
    Fill,
    /// Fractional track fill weight.
    Fr(u16),
}

impl TrackSize {
    /// Return fractional weight.
    fn fr_weight(self) -> u32 {
        match self {
            Self::Fr(weight) => weight.max(1) as u32,
            _ => 0,
        }
    }
}
