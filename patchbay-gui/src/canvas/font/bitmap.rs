/// Built-in 5x7 bitmap font used by [`Canvas::draw_text`].
struct BitmapFont;

/// Glyph fallback for unsupported characters.
const UNKNOWN_GLYPH: [u8; 7] = [
    0b00000, 0b00100, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100,
];
/// Glyph table for numeric digits `0..=9`.
const DIGIT_GLYPHS: [[u8; 7]; 10] = [
    [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
    [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
    [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
    [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
    [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
    [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
    [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
];
/// Glyph table for uppercase letters `A..=Z`.
const LETTER_GLYPHS: [[u8; 7]; 26] = [
    [0b00100, 0b01010, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
    [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
    [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100],
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
    [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
    [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
    [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
    [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
    [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
    [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
    [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
    [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
    [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
    [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
    [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
    [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
    [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
    [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
];

impl BitmapFont {
    /// Return the 5x7 glyph bitmap for a character.
    fn glyph(ch: char) -> [u8; 7] {
        if ch == ' ' {
            return [0; 7];
        }
        if let Some(glyph) = Self::digit_glyph(ch) {
            return glyph;
        }
        if let Some(glyph) = Self::letter_glyph(ch) {
            return glyph;
        }
        Self::symbol_or_unknown(ch)
    }

    /// Resolve one decimal digit glyph.
    fn digit_glyph(ch: char) -> Option<[u8; 7]> {
        let index = ch.to_digit(10)? as usize;
        DIGIT_GLYPHS.get(index).copied()
    }

    /// Resolve one latin alphabet glyph using uppercase normalization.
    fn letter_glyph(ch: char) -> Option<[u8; 7]> {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_uppercase() {
            return None;
        }
        let index = (upper as u8 - b'A') as usize;
        LETTER_GLYPHS.get(index).copied()
    }

    /// Resolve punctuation glyphs with fallback to [`UNKNOWN_GLYPH`].
    fn symbol_or_unknown(ch: char) -> [u8; 7] {
        match ch {
            '-' => [
                0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
            ],
            '_' => [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
            ],
            '.' => [
                0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
            ],
            ':' => [
                0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
            ],
            '/' => [
                0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
            ],
            _ => UNKNOWN_GLYPH,
        }
    }
}

/// Return a 5x7 glyph bitmap for text measurement helpers.
pub(crate) fn glyph_bitmap_for_text(ch: char) -> [u8; 7] {
    BitmapFont::glyph(ch)
}
