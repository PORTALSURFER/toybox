
/// Default rendered knob diameter in pixels for declarative and immediate UIs.
pub(crate) const DEFAULT_KNOB_DIAMETER: i32 = 32;
/// Horizontal padding reserved around knob circles and arc rings in block layouts.
pub(crate) const KNOB_BLOCK_SIDE_PADDING: i32 = 8;

/// Return the knob label height (in pixels) for a text scale.
pub(crate) fn knob_label_height(text_scale: u32) -> u32 {
    8 * text_scale.max(1)
}

/// Return the vertical gap (in pixels) around knob labels for a text scale.
pub(crate) fn knob_label_gap(text_scale: u32) -> u32 {
    4 * text_scale.max(1)
}

/// Return the full knob block footprint for a control diameter and text scale.
///
/// Declarative measurement and rendering both use this helper to keep
/// measured and rendered bounds identical.
pub(crate) fn knob_block_size_for_diameter(diameter: u32, text_scale: u32) -> Size {
    let knob_diameter = diameter.max(1);
    let label_height = knob_label_height(text_scale);
    let label_gap = knob_label_gap(text_scale);
    let dial_square_width = knob_diameter + (KNOB_BLOCK_SIDE_PADDING.max(0) * 2) as u32;
    let label_stack_height = knob_diameter + label_height * 2 + label_gap * 2;
    Size {
        width: dial_square_width,
        height: label_stack_height.max(dial_square_width),
    }
}

/// Unique identifier for widgets across frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create a widget id from a stable numeric seed.
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Hash a label into a widget id.
    ///
    /// The label must remain stable across frames for correct interaction
    /// tracking. If the label can change (for example, including formatted
    /// values), prefer using a stable key and hashing that instead.
    pub fn from_label(label: &str) -> Self {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in label.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self(hash)
    }
}

/// Semantic color palette shared across Patchbay-based GUIs.
///
/// The palette defines intent-level roles (focus, emphasis, text, and
/// backgrounds) so widget and declarative token defaults can stay visually
/// consistent without hardcoding unrelated RGB values throughout the codebase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainPalette {
    /// Primary focus accent for active elements and cursor-like indicators.
    pub accent_focus: Color,
    /// Secondary emphasis for highlighted syntax-like constructs.
    pub syntax_emphasis: Color,
    /// Identifier emphasis color for important names and symbols.
    pub identifiers: Color,
    /// Literal/constant emphasis color.
    pub literals: Color,
    /// Primary readable text color.
    pub text_primary: Color,
    /// De-emphasized text color for comments and inactive hints.
    pub text_muted: Color,
    /// Structural UI secondary color for borders, gutters, and separators.
    pub ui_secondary: Color,
    /// Primary background color.
    pub background_primary: Color,
    /// Secondary background color for panels and surfaces.
    pub background_secondary: Color,
}

impl MainPalette {
    /// Return the canonical main palette used by Patchbay GUI defaults.
    pub const fn main() -> Self {
        Self {
            accent_focus: Color::rgb(199, 240, 0),
            syntax_emphasis: Color::rgb(99, 214, 176),
            identifiers: Color::rgb(143, 191, 159),
            literals: Color::rgb(79, 175, 160),
            text_primary: Color::rgb(201, 201, 194),
            text_muted: Color::rgb(126, 138, 106),
            ui_secondary: Color::rgb(95, 102, 84),
            background_primary: Color::rgb(27, 30, 26),
            background_secondary: Color::rgb(35, 40, 35),
        }
    }
}

/// Theme colors for the GUI widgets.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Canvas background color.
    pub background: Color,
    /// Primary text color.
    pub text: Color,
    /// Text scale factor for the bitmap font.
    pub text_scale: u32,
    /// Knob fill color.
    pub knob_fill: Color,
    /// Knob outline color.
    pub knob_outline: Color,
    /// Knob active color.
    pub knob_active: Color,
    /// Knob hover color.
    pub knob_hover: Color,
    /// Knob indicator color.
    pub knob_indicator: Color,
}

impl Theme {
    /// Build widget theme defaults from a semantic palette.
    pub const fn from_palette(palette: MainPalette) -> Self {
        Self {
            background: palette.background_primary,
            text: palette.text_primary,
            text_scale: 2,
            knob_fill: palette.background_secondary,
            knob_outline: palette.ui_secondary,
            knob_active: palette.accent_focus,
            knob_hover: palette.syntax_emphasis,
            knob_indicator: palette.text_primary,
        }
    }

    /// Return the canonical main widget theme.
    pub const fn main() -> Self {
        Self::from_palette(MainPalette::main())
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::main()
    }
}
