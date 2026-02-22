/// Stable semantic key used to derive widget accent colors.
///
/// Accent keys intentionally avoid plugin-specific meaning so host/plugin UIs
/// can map any selected entity to a deterministic accent without adding custom
/// color systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccentKey {
    /// Stable key derived from a numeric entity identifier.
    Entity(u64),
    /// Stable key derived from a fixed semantic name.
    Named(&'static str),
}

/// Static widget color token for explicit non-accent styling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetColorToken {
    /// Surface/background-like control color.
    Surface,
    /// Border/outline color.
    Border,
    /// Foreground text color.
    Text,
    /// Theme accent color.
    Accent,
    /// Theme focus accent color.
    Focus,
}

/// Extended semantic color tokens for widget role resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticColorToken {
    /// Primary accent color.
    Accent,
    /// Focus-ring and keyboard-focus emphasis color.
    Focus,
    /// Neutral UI color.
    Neutral,
    /// De-emphasized semantic color.
    Muted,
    /// Positive/success semantic color.
    Success,
    /// Caution/warning semantic color.
    Warning,
    /// Error/danger semantic color.
    Danger,
    /// Informational semantic color.
    Info,
    /// Primary readable text color.
    TextPrimary,
    /// Muted secondary text color.
    TextMuted,
    /// Primary surface color.
    SurfacePrimary,
    /// Secondary surface color.
    SurfaceSecondary,
    /// Strong border color.
    BorderStrong,
    /// Subtle border color.
    BorderSubtle,
}

/// Widget color-role selection contract.
///
/// This additive role model keeps existing widget behavior unchanged unless a
/// caller opts in by setting a color role.
///
/// # Example
/// ```ignore
/// # use patchbay_gui::{AccentKey, WidgetColorRole, knob};
/// let selected_entity_id = 42_u64;
/// let node = knob("rate", 0.5, (0.0, 1.0))
///     .color_role(WidgetColorRole::Accent(AccentKey::Entity(selected_entity_id)));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetColorRole {
    /// Resolve from static widget color tokens.
    Static(WidgetColorToken),
    /// Resolve from a deterministic accent key.
    Accent(AccentKey),
    /// Resolve from semantic theme-like tokens.
    Semantic(SemanticColorToken),
}

/// State-aware resolved widget colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorStateVariants {
    /// Base/default color.
    pub base: Color,
    /// Hover color.
    pub hover: Color,
    /// Active/pressed color.
    pub active: Color,
    /// Disabled color.
    pub disabled: Color,
    /// Focus-ring color.
    pub focus_ring: Color,
}

/// Resolution context used by widget color resolvers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetColorContext {
    /// Theme token set available for semantic fallback and contrast clamping.
    pub tokens: ThemeTokens,
    /// Whether the target widget is currently disabled.
    pub disabled: bool,
    /// Whether the target widget is currently focused.
    pub focused: bool,
}

/// Trait for resolving a widget color role into state variants.
pub trait WidgetColorResolver {
    /// Resolve role-derived colors for a widget in the given context.
    fn resolve(
        &self,
        role: WidgetColorRole,
        context: WidgetColorContext,
    ) -> ColorStateVariants;
}

/// Default deterministic resolver used by Toybox declarative widgets.
///
/// The accent path maps keys to a precomputed, perceptually balanced hue table
/// authored from evenly distributed OKLCH hue samples and clamped into sRGB.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultWidgetColorResolver;

impl DefaultWidgetColorResolver {
    /// Build the default widget color resolver.
    pub const fn new() -> Self {
        Self
    }
}

impl WidgetColorResolver for DefaultWidgetColorResolver {
    fn resolve(
        &self,
        role: WidgetColorRole,
        context: WidgetColorContext,
    ) -> ColorStateVariants {
        let background = context.tokens.colors.background;
        let role_base = match role {
            WidgetColorRole::Static(token) => resolve_static_token(token, context.tokens),
            WidgetColorRole::Accent(key) => resolve_accent_key_color(key),
            WidgetColorRole::Semantic(token) => resolve_semantic_token(token, context.tokens),
        };

        let base = clamp_legibility(role_base, background);
        let hover = clamp_legibility(lighten(base, 0.12), background);
        let active = clamp_legibility(lighten(base, 0.22), background);
        let disabled_base = desaturate(base, 0.62);
        let disabled = with_alpha(clamp_legibility(disabled_base, background), 150);
        let focus_base = if context.focused {
            active
        } else {
            lighten(base, 0.08)
        };
        let focus_ring = ensure_min_contrast(focus_base, background, 3.0);

        if context.disabled {
            return ColorStateVariants {
                base: disabled,
                hover: disabled,
                active: disabled,
                disabled,
                focus_ring: disabled,
            };
        }

        ColorStateVariants {
            base,
            hover,
            active,
            disabled,
            focus_ring,
        }
    }
}

/// Resolve static widget color tokens from the current theme token set.
fn resolve_static_token(token: WidgetColorToken, tokens: ThemeTokens) -> Color {
    match token {
        WidgetColorToken::Surface => tokens.colors.surface,
        WidgetColorToken::Border => tokens.colors.border,
        WidgetColorToken::Text => tokens.colors.text,
        WidgetColorToken::Accent => tokens.colors.accent,
        WidgetColorToken::Focus => ensure_min_contrast(tokens.colors.accent, tokens.colors.background, 3.0),
    }
}

/// Resolve semantic widget color tokens from the current theme token set.
fn resolve_semantic_token(token: SemanticColorToken, tokens: ThemeTokens) -> Color {
    match token {
        SemanticColorToken::Accent => tokens.colors.accent,
        SemanticColorToken::Focus => ensure_min_contrast(tokens.colors.accent, tokens.colors.background, 3.0),
        SemanticColorToken::Neutral => tokens.colors.surface,
        SemanticColorToken::Muted => blend(tokens.colors.text, tokens.colors.background, 0.52),
        SemanticColorToken::Success => Color::rgb(84, 206, 136),
        SemanticColorToken::Warning => Color::rgb(227, 178, 74),
        SemanticColorToken::Danger => Color::rgb(226, 109, 112),
        SemanticColorToken::Info => Color::rgb(98, 183, 230),
        SemanticColorToken::TextPrimary => tokens.colors.text,
        SemanticColorToken::TextMuted => blend(tokens.colors.text, tokens.colors.background, 0.40),
        SemanticColorToken::SurfacePrimary => tokens.colors.background,
        SemanticColorToken::SurfaceSecondary => tokens.colors.surface,
        SemanticColorToken::BorderStrong => tokens.colors.border,
        SemanticColorToken::BorderSubtle => blend(tokens.colors.border, tokens.colors.surface, 0.45),
    }
}

/// Resolve an accent key into a stable palette color.
fn resolve_accent_key_color(key: AccentKey) -> Color {
    let hash = match key {
        AccentKey::Entity(id) => splitmix64(id),
        AccentKey::Named(name) => splitmix64(fnv1a64(name.as_bytes())),
    };
    let index = (hash as usize) % ACCENT_PALETTE_OKLCH.len();
    ACCENT_PALETTE_OKLCH[index]
}

/// Deterministic accent palette sampled from evenly spaced OKLCH hue bands.
const ACCENT_PALETTE_OKLCH: [Color; 16] = [
    Color::rgb(233, 95, 98),
    Color::rgb(236, 126, 79),
    Color::rgb(226, 157, 68),
    Color::rgb(208, 184, 65),
    Color::rgb(176, 202, 69),
    Color::rgb(133, 212, 80),
    Color::rgb(90, 214, 112),
    Color::rgb(70, 210, 153),
    Color::rgb(63, 202, 185),
    Color::rgb(73, 188, 214),
    Color::rgb(92, 170, 232),
    Color::rgb(120, 150, 236),
    Color::rgb(149, 133, 233),
    Color::rgb(179, 120, 224),
    Color::rgb(206, 111, 202),
    Color::rgb(226, 106, 171),
];

/// Mix a 64-bit input into a well-distributed hash value.
fn splitmix64(value: u64) -> u64 {
    let mut z = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Compute a deterministic FNV-1a 64-bit hash for byte slices.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Apply luminance/saturation and contrast guardrails for legible UI colors.
fn clamp_legibility(color: Color, background: Color) -> Color {
    let lum_clamped = clamp_luminance(color, 0.22, 0.82);
    let sat_clamped = clamp_saturation(lum_clamped, 0.22, 0.88);
    ensure_min_contrast(sat_clamped, background, 2.2)
}

/// Clamp relative luminance into a target range by blending toward white/black.
fn clamp_luminance(color: Color, min_luminance: f32, max_luminance: f32) -> Color {
    let l = relative_luminance(color);
    if l < min_luminance {
        let t = ((min_luminance - l) / (1.0 - l + 1.0e-6)).clamp(0.0, 1.0);
        return blend(color, Color::rgb(255, 255, 255), t);
    }
    if l > max_luminance {
        let t = ((l - max_luminance) / (l + 1.0e-6)).clamp(0.0, 1.0);
        return blend(color, Color::rgb(0, 0, 0), t);
    }
    color
}

/// Clamp saturation proxy values into the target range.
fn clamp_saturation(color: Color, min_sat: f32, max_sat: f32) -> Color {
    let sat = saturation(color);
    if sat < min_sat {
        let factor = ((min_sat - sat) / min_sat.max(1.0e-6)).clamp(0.0, 1.0);
        return boost_saturation(color, factor);
    }
    if sat > max_sat {
        let factor = ((sat - max_sat) / sat.max(1.0e-6)).clamp(0.0, 1.0);
        return desaturate(color, factor);
    }
    color
}

/// Return a simple saturation proxy from sRGB channel spread.
fn saturation(color: Color) -> f32 {
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;
    r.max(g).max(b) - r.min(g).min(b)
}

/// Increase saturation by moving channels away from gray luminance.
fn boost_saturation(color: Color, amount: f32) -> Color {
    let gray = (relative_luminance(color) * 255.0).round();
    let r = gray + (color.r as f32 - gray) * (1.0 + amount);
    let g = gray + (color.g as f32 - gray) * (1.0 + amount);
    let b = gray + (color.b as f32 - gray) * (1.0 + amount);
    Color::rgba(clamp_channel(r), clamp_channel(g), clamp_channel(b), color.a)
}

/// Decrease saturation by blending channels toward gray luminance.
fn desaturate(color: Color, amount: f32) -> Color {
    let gray = (relative_luminance(color) * 255.0).round();
    let r = color.r as f32 + (gray - color.r as f32) * amount;
    let g = color.g as f32 + (gray - color.g as f32) * amount;
    let b = color.b as f32 + (gray - color.b as f32) * amount;
    Color::rgba(clamp_channel(r), clamp_channel(g), clamp_channel(b), color.a)
}

/// Lighten a color by blending toward white.
fn lighten(color: Color, amount: f32) -> Color {
    blend(color, Color::rgb(255, 255, 255), amount)
}

/// Linearly blend two colors with factor `t` in the range `[0, 1]`.
fn blend(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mix = |lhs: u8, rhs: u8| -> u8 {
        clamp_channel(lhs as f32 + (rhs as f32 - lhs as f32) * t)
    };
    Color::rgba(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), mix(a.a, b.a))
}

/// Copy a color while overriding alpha.
fn with_alpha(color: Color, alpha: u8) -> Color {
    Color::rgba(color.r, color.g, color.b, alpha)
}

/// Ensure `color` reaches the minimum contrast ratio against `background`.
fn ensure_min_contrast(color: Color, background: Color, min_ratio: f32) -> Color {
    if contrast_ratio(color, background) >= min_ratio {
        return color;
    }

    let white = Color::rgb(255, 255, 255);
    let black = Color::rgb(0, 0, 0);
    let white_ratio = contrast_ratio(white, background);
    let black_ratio = contrast_ratio(black, background);
    let fallback = if white_ratio >= black_ratio { white } else { black };
    if contrast_ratio(fallback, background) >= min_ratio {
        return fallback;
    }
    fallback
}

/// Compute WCAG-style contrast ratio between two colors.
fn contrast_ratio(a: Color, b: Color) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let high = l1.max(l2);
    let low = l1.min(l2);
    (high + 0.05) / (low + 0.05)
}

/// Compute relative luminance for an sRGB color.
fn relative_luminance(color: Color) -> f32 {
    let to_linear = |channel: u8| -> f32 {
        let value = channel as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let r = to_linear(color.r);
    let g = to_linear(color.g);
    let b = to_linear(color.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Clamp a floating-point channel value into `[0, 255]`.
fn clamp_channel(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod widget_color_resolver_tests {
    use super::*;

    #[test]
    fn accent_resolution_is_deterministic() {
        let resolver = DefaultWidgetColorResolver::new();
        let context = WidgetColorContext {
            tokens: ThemeTokens::default(),
            disabled: false,
            focused: false,
        };
        let first = resolver.resolve(
            WidgetColorRole::Accent(AccentKey::Entity(7)),
            context,
        );
        let second = resolver.resolve(
            WidgetColorRole::Accent(AccentKey::Entity(7)),
            context,
        );
        assert_eq!(first, second);
    }

    #[test]
    fn active_and_hover_are_not_weaker_than_base() {
        let resolver = DefaultWidgetColorResolver::new();
        let context = WidgetColorContext {
            tokens: ThemeTokens::default(),
            disabled: false,
            focused: false,
        };
        let variants = resolver.resolve(
            WidgetColorRole::Accent(AccentKey::Entity(42)),
            context,
        );
        let base_l = relative_luminance(variants.base);
        let hover_l = relative_luminance(variants.hover);
        let active_l = relative_luminance(variants.active);
        assert!(hover_l >= base_l || variants.hover == variants.base);
        assert!(active_l >= hover_l || variants.active == variants.hover);
    }

    #[test]
    fn contrast_guardrail_applies_for_focus_ring() {
        let resolver = DefaultWidgetColorResolver::new();
        let context = WidgetColorContext {
            tokens: ThemeTokens::default(),
            disabled: false,
            focused: true,
        };
        let variants = resolver.resolve(
            WidgetColorRole::Accent(AccentKey::Entity(2)),
            context,
        );
        assert!(
            contrast_ratio(variants.focus_ring, context.tokens.colors.background) >= 3.0,
            "focus ring contrast should satisfy minimum guardrail"
        );
    }
}
