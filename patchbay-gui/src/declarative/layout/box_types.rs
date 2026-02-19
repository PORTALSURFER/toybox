/// Length value for constrained layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    /// Use measured content size.
    Auto,
    /// Fixed pixels.
    Px(u32),
    /// Fill available space with optional relative weight.
    Fill(u16),
}

impl Length {
    /// Return the fill weight.
    fn fill_weight(self) -> u32 {
        match self {
            Self::Fill(weight) => weight.max(1) as u32,
            _ => 0,
        }
    }
}

/// Container-axis length value for slot/container layout flows.
///
/// This type intentionally excludes absolute pixel sizing so non-root
/// container composition remains fully host-derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerLength {
    /// Use measured content size.
    Auto,
    /// Fill available space with optional relative weight.
    Fill(u16),
}

impl ContainerLength {
    /// Convert a container length into a generic layout length.
    const fn into_length(self) -> Length {
        match self {
            Self::Auto => Length::Auto,
            Self::Fill(weight) => Length::Fill(weight),
        }
    }
}

/// Box constraints for non-root containers.
///
/// Containers can only use host-derived fill/auto sizing. Absolute sizing and
/// explicit min/max constraints are intentionally unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerLayout {
    /// Width sizing mode.
    pub width: ContainerLength,
    /// Height sizing mode.
    pub height: ContainerLength,
}

impl ContainerLayout {
    /// Create unconstrained auto sizing.
    pub const fn auto() -> Self {
        Self {
            width: ContainerLength::Auto,
            height: ContainerLength::Auto,
        }
    }

    /// Create a box that fills available space.
    pub const fn fill() -> Self {
        Self {
            width: ContainerLength::Fill(1),
            height: ContainerLength::Fill(1),
        }
    }

    /// Set width behavior.
    pub const fn with_width(mut self, width: ContainerLength) -> Self {
        self.width = width;
        self
    }

    /// Set height behavior.
    pub const fn with_height(mut self, height: ContainerLength) -> Self {
        self.height = height;
        self
    }

    /// Set width to fill available space.
    pub const fn fill_width(mut self) -> Self {
        self.width = ContainerLength::Fill(1);
        self
    }

    /// Set height to fill available space.
    pub const fn fill_height(mut self) -> Self {
        self.height = ContainerLength::Fill(1);
        self
    }

    /// Convert container constraints into generic layout constraints.
    pub(crate) const fn to_layout_box(self) -> LayoutBox {
        LayoutBox {
            width: self.width.into_length(),
            height: self.height.into_length(),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }
}

/// Box constraints shared by all node types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutBox {
    /// Width sizing mode.
    pub width: Length,
    /// Height sizing mode.
    pub height: Length,
    /// Optional minimum width.
    pub min_width: Option<u32>,
    /// Optional minimum height.
    pub min_height: Option<u32>,
    /// Optional maximum width.
    pub max_width: Option<u32>,
    /// Optional maximum height.
    pub max_height: Option<u32>,
}

impl LayoutBox {
    /// Create unconstrained auto sizing.
    pub const fn auto() -> Self {
        Self {
            width: Length::Auto,
            height: Length::Auto,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }

    /// Create a box that fills available space.
    pub const fn fill() -> Self {
        Self {
            width: Length::Fill(1),
            height: Length::Fill(1),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }

    /// Create a fixed-size baseline box.
    ///
    /// The returned constraints use fixed pixel lengths as minimum floors.
    /// Content can still grow beyond these values when intrinsic measurement
    /// requires more space.
    pub const fn fixed(width: u32, height: u32) -> Self {
        Self {
            width: Length::Px(width),
            height: Length::Px(height),
            min_width: Some(width),
            min_height: Some(height),
            max_width: None,
            max_height: None,
        }
    }

    /// Set width behavior.
    pub const fn with_width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Set height behavior.
    pub const fn with_height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Set width to fill available space.
    pub const fn fill_width(mut self) -> Self {
        self.width = Length::Fill(1);
        self
    }

    /// Set height to fill available space.
    pub const fn fill_height(mut self) -> Self {
        self.height = Length::Fill(1);
        self
    }

    /// Set a fixed-width baseline while preserving current height behavior.
    ///
    /// The width acts as a minimum floor and may expand for larger intrinsic
    /// content unless an explicit max width is also applied.
    pub const fn fixed_width(mut self, width: u32) -> Self {
        self.width = Length::Px(width);
        self.min_width = Some(width);
        self.max_width = None;
        self
    }

    /// Set a fixed-height baseline while preserving current width behavior.
    ///
    /// The height acts as a minimum floor and may expand for larger intrinsic
    /// content unless an explicit max height is also applied.
    pub const fn fixed_height(mut self, height: u32) -> Self {
        self.height = Length::Px(height);
        self.min_height = Some(height);
        self.max_height = None;
        self
    }

    /// Set minimum size.
    pub const fn with_min(mut self, min_width: u32, min_height: u32) -> Self {
        self.min_width = Some(min_width);
        self.min_height = Some(min_height);
        self
    }

    /// Set minimum size constraints.
    pub const fn min(self, min_width: u32, min_height: u32) -> Self {
        self.with_min(min_width, min_height)
    }

    /// Set maximum size.
    pub const fn with_max(mut self, max_width: u32, max_height: u32) -> Self {
        self.max_width = Some(max_width);
        self.max_height = Some(max_height);
        self
    }

    /// Set maximum size constraints.
    pub const fn max(self, max_width: u32, max_height: u32) -> Self {
        self.with_max(max_width, max_height)
    }
}

/// Edge insets used by containers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeInsets {
    /// Left inset in pixels.
    pub left: i32,
    /// Right inset in pixels.
    pub right: i32,
    /// Top inset in pixels.
    pub top: i32,
    /// Bottom inset in pixels.
    pub bottom: i32,
}
