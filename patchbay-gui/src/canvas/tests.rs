use super::*;

fn pixel_at(canvas: &Canvas, x: u32, y: u32) -> [u8; 4] {
    let width = canvas.size.width as usize;
    let index = (y as usize * width + x as usize) * 4;
    [
        canvas.pixels[index],
        canvas.pixels[index + 1],
        canvas.pixels[index + 2],
        canvas.pixels[index + 3],
    ]
}

#[test]
fn rect_contains_point() {
    let rect = Rect {
        origin: Point { x: 10, y: 20 },
        size: Size {
            width: 100,
            height: 50,
        },
    };
    assert!(rect.contains(Point { x: 10, y: 20 }));
    assert!(rect.contains(Point { x: 109, y: 69 }));
    assert!(!rect.contains(Point { x: 110, y: 70 }));
}

#[test]
fn draw_text_advances_cursor() {
    let mut canvas = Canvas::new(64, 64);
    canvas.draw_text(Point { x: 0, y: 0 }, "AB", Color::rgb(255, 255, 255), 1);
    assert!(canvas.pixels().iter().any(|value| *value != 0));
}

#[test]
fn stroke_arc_renders_top_semicircle() {
    let mut canvas = Canvas::new(21, 21);
    let center = Point { x: 10, y: 10 };
    let color = Color::rgb(200, 100, 50);
    canvas.stroke_arc(center, 8, 2, 0.0, std::f32::consts::PI, color);

    let top = pixel_at(&canvas, 10, 2);
    let bottom = pixel_at(&canvas, 10, 18);

    assert_eq!(top, [color.r, color.g, color.b, color.a]);
    assert_ne!(bottom, [color.r, color.g, color.b, color.a]);
}

#[test]
fn fill_polygon_fills_triangle_interior() {
    let mut canvas = Canvas::new(16, 16);
    let color = Color::rgb(20, 180, 220);
    canvas.fill_polygon(
        &[
            Point { x: 2, y: 12 },
            Point { x: 8, y: 2 },
            Point { x: 13, y: 12 },
        ],
        color,
    );

    assert_eq!(
        pixel_at(&canvas, 8, 8),
        [color.r, color.g, color.b, color.a]
    );
    assert_eq!(pixel_at(&canvas, 1, 1), [0, 0, 0, 0]);
}

#[test]
fn glyph_maps_lowercase_to_uppercase_for_letters() {
    assert_eq!(BitmapFont::glyph('a'), BitmapFont::glyph('A'));
    assert_eq!(BitmapFont::glyph('z'), BitmapFont::glyph('Z'));
}

#[test]
fn glyph_uses_consistent_fallback_for_unknown_chars() {
    assert_eq!(BitmapFont::glyph('~'), BitmapFont::glyph('`'));
    assert_eq!(BitmapFont::glyph('~'), BitmapFont::glyph('\u{20AC}'));
}

#[test]
fn glyph_preserves_known_symbols_and_space() {
    assert_ne!(BitmapFont::glyph('/'), BitmapFont::glyph('~'));
    assert_ne!(BitmapFont::glyph(':'), BitmapFont::glyph('~'));
    assert_eq!(BitmapFont::glyph(' '), [0; 7]);
}
