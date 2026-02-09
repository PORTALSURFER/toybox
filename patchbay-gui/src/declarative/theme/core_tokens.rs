/// Core color token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorTokens {
    /// Window background.
    pub background: Color,
    /// Surface fill.
    pub surface: Color,
    /// Border color.
    pub border: Color,
    /// Primary text.
    pub text: Color,
    /// Accent color.
    pub accent: Color,
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self::main()
    }
}

impl ColorTokens {
    /// Build declarative color tokens from a semantic palette.
    pub const fn from_palette(palette: MainPalette) -> Self {
        Self {
            background: palette.background_primary,
            surface: palette.background_secondary,
            border: palette.ui_secondary,
            text: palette.text_primary,
            accent: palette.accent_focus,
        }
    }

    /// Return the canonical declarative color token set.
    pub const fn main() -> Self {
        Self::from_palette(MainPalette::main())
    }
}

/// Typography token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypographyTokens {
    /// Bitmap text scale.
    pub text_scale: u32,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self { text_scale: 2 }
    }
}

/// Spacing token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpacingTokens {
    /// Tiny spacing.
    pub xs: i32,
    /// Small spacing.
    pub sm: i32,
    /// Medium spacing.
    pub md: i32,
    /// Large spacing.
    pub lg: i32,
}
