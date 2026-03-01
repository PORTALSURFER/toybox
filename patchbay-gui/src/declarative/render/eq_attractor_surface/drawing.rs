/// Render EQ attractor background fill and optional grid lines.
fn render_eq_surface_background(
    rect: Rect,
    geometry: EqSurfaceGeometry,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    style: EqAttractorSurfaceStyle,
    model: &EqAttractorSurfaceModel,
) {
    ui.fill_rect_visual(rect, eq_scale_alpha(tokens.colors.surface, 220));
    ui.stroke_rect_visual(rect, 1.0, tokens.colors.border);

    if !style.show_grid {
        return;
    }

    let grid_color = eq_scale_alpha(tokens.colors.border, 120);
    for freq_hz in [20.0_f32, 50.0, 100.0, 200.0, 500.0, 1_000.0, 2_000.0, 5_000.0, 10_000.0, 20_000.0]
    {
        if freq_hz < model.freq_min_hz || freq_hz > model.freq_max_hz {
            continue;
        }
        let t = eq_freq_to_t(freq_hz, model.freq_min_hz, model.freq_max_hz);
        let local_x = geometry.left + (geometry.width * t).round() as i32;
        let start = Point {
            x: rect.origin.x + local_x,
            y: rect.origin.y + geometry.top,
        };
        let end = Point {
            x: rect.origin.x + local_x,
            y: rect.origin.y + geometry.bottom,
        };
        ui.draw_line_visual(start, end, 1.0, grid_color);
    }

    for normalized in [0.25_f32, 0.5, 0.75] {
        let local_y = geometry.bottom - (geometry.height * normalized).round() as i32;
        let start = Point {
            x: rect.origin.x + geometry.left,
            y: rect.origin.y + local_y,
        };
        let end = Point {
            x: rect.origin.x + geometry.right,
            y: rect.origin.y + local_y,
        };
        ui.draw_line_visual(start, end, 1.0, eq_scale_alpha(tokens.colors.border, 96));
    }
}

/// Render filled EQ curve and antialiased outline.
fn render_eq_surface_curve(
    rect: Rect,
    geometry: EqSurfaceGeometry,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    model: &EqAttractorSurfaceModel,
    style: EqAttractorSurfaceStyle,
    band_values: &[f32],
) {
    if band_values.is_empty() {
        return;
    }

    let points = eq_curve_points(rect, geometry, model, style, band_values);
    if points.len() < 2 {
        return;
    }

    let fill_color = eq_scale_alpha(tokens.colors.accent, 46);
    let bottom_y = rect.origin.y + geometry.bottom;
    for point in &points {
        ui.draw_line_visual(
            Point {
                x: point.x,
                y: bottom_y,
            },
            *point,
            1.0,
            fill_color,
        );
    }

    let outline_glow = eq_scale_alpha(tokens.colors.accent, 112);
    let outline_core = eq_scale_alpha(tokens.colors.accent, 220);
    for segment in points.windows(2) {
        if let [start, end] = segment {
            ui.draw_line_visual(*start, *end, 2.2, outline_glow);
            ui.draw_line_visual(*start, *end, 1.1, outline_core);
        }
    }
}

/// Render attractor handle circles.
fn render_eq_surface_nodes(
    rect: Rect,
    geometry: EqSurfaceGeometry,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    model: &EqAttractorSurfaceModel,
    style: EqAttractorSurfaceStyle,
    runtime: &crate::ui::EqAttractorSurfaceRuntimeState,
) {
    if !style.show_nodes {
        return;
    }

    let resolver = DefaultWidgetColorResolver::new();
    for attractor in &model.attractors {
        let smoothed = runtime
            .smoothed_attractors
            .get(&attractor.id)
            .copied()
            .unwrap_or(crate::ui::EqAttractorSurfaceSmoothedAttractorState {
                x: attractor.x,
                y: attractor.y,
                depth: 1.0,
                cycles: 1.0,
                rate_hz: 0.0,
            });
        let local = eq_normalized_to_local(smoothed.x, smoothed.y, geometry);
        let center = Point {
            x: rect.origin.x + local.x,
            y: rect.origin.y + local.y,
        };
        let role = match style.color_policy {
            EqAttractorColorPolicy::PerAttractorAccent => {
                WidgetColorRole::Accent(AccentKey::Entity(attractor.id))
            }
            EqAttractorColorPolicy::SingleAccent(key) => WidgetColorRole::Accent(key),
        };
        let variants = resolver.resolve(
            role,
            WidgetColorContext {
                tokens: *tokens,
                disabled: false,
                focused: attractor.selected,
            },
        );
        let stroke = eq_scale_alpha(variants.active, 255);
        // Use a bright core and fully opaque outline so attractors remain clear on dark surfaces.
        ui.canvas()
            .fill_circle(center, 7, eq_scale_alpha(tokens.colors.text, 255));
        ui.canvas().stroke_circle(center, 10, 4, stroke);
        if attractor.selected {
            ui.canvas()
                .stroke_circle(center, 13, 3, eq_scale_alpha(variants.focus_ring, 255));
        }
    }
}

/// Build smoothed curve polyline points in absolute coordinates.
fn eq_curve_points(
    rect: Rect,
    geometry: EqSurfaceGeometry,
    model: &EqAttractorSurfaceModel,
    style: EqAttractorSurfaceStyle,
    band_values: &[f32],
) -> Vec<Point> {
    let mut points = Vec::with_capacity(style.curve_samples + 1);
    let log_min = model.freq_min_hz.ln();
    let log_span = (model.freq_max_hz.ln() - log_min).max(1.0e-6);

    for sample in 0..=style.curve_samples {
        let t = sample as f32 / style.curve_samples as f32;
        let freq = (log_min + log_span * t).exp();
        let band_t = eq_freq_to_t(freq, model.freq_min_hz, model.freq_max_hz);
        let y = eq_sample_band_value(band_values, band_t);
        let local = eq_normalized_to_local(t, y, geometry);
        points.push(Point {
            x: rect.origin.x + local.x,
            y: rect.origin.y + local.y,
        });
    }

    points
}
