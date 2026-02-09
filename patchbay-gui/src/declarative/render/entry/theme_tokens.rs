impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xs: 4,
            sm: 8,
            md: 12,
            lg: 16,
        }
    }
}

/// Control-size token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlTokens {
    /// Default knob diameter.
    pub knob_diameter: u32,
    /// Default slider width.
    pub slider_width: u32,
    /// Default slider height.
    pub slider_height: u32,
    /// Default toggle width.
    pub toggle_width: u32,
    /// Default toggle height.
    pub toggle_height: u32,
    /// Default button width.
    pub button_width: u32,
    /// Default button height.
    pub button_height: u32,
    /// Default dropdown width.
    pub dropdown_width: u32,
    /// Default dropdown height.
    pub dropdown_height: u32,
}

impl Default for ControlTokens {
    fn default() -> Self {
        Self {
            knob_diameter: 32,
            slider_width: 180,
            slider_height: 28,
            toggle_width: 64,
            toggle_height: 28,
            button_width: 120,
            button_height: 28,
            dropdown_width: 180,
            dropdown_height: 28,
        }
    }
}

/// Root design tokens for declarative rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ThemeTokens {
    /// Color token set.
    pub colors: ColorTokens,
    /// Typography token set.
    pub typography: TypographyTokens,
    /// Spacing token set.
    pub spacing: SpacingTokens,
    /// Control token set.
    pub controls: ControlTokens,
}

impl ThemeTokens {
    /// Build declarative tokens from a semantic palette and default sizing.
    pub const fn from_palette(palette: MainPalette) -> Self {
        Self {
            colors: ColorTokens::from_palette(palette),
            typography: TypographyTokens { text_scale: 2 },
            spacing: SpacingTokens {
                xs: 4,
                sm: 8,
                md: 12,
                lg: 16,
            },
            controls: ControlTokens {
                knob_diameter: 32,
                slider_width: 180,
                slider_height: 28,
                toggle_width: 64,
                toggle_height: 28,
                button_width: 120,
                button_height: 28,
                dropdown_width: 180,
                dropdown_height: 28,
            },
        }
    }

    /// Return the canonical declarative token set.
    pub const fn main() -> Self {
        Self::from_palette(MainPalette::main())
    }
}

/// Measure the required size for a UI specification.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn measure_checked(spec: &UiSpec) -> Result<Size, DeclarativeError> {
    validate_spec(spec)?;
    let tokens = spec.root.tokens.unwrap_or_default();
    Ok(measure_root_frame(&spec.root, &tokens))
}
