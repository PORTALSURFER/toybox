/// Fixed frame step used for deterministic UI-side smoothing.
const EQ_SURFACE_FRAME_DT_SECONDS: f32 = 1.0 / 60.0;
/// Gaussian width used by attractor gravity blending.
const EQ_SURFACE_GRAVITY_SIGMA: f32 = 0.12;
/// Pointer hit radius for attractor handles in pixels.
const EQ_SURFACE_ATTRACTOR_HIT_RADIUS_PX: i32 = 10;

/// Local plotting geometry derived from the widget rectangle and padding.
#[derive(Clone, Copy, Debug)]
struct EqSurfaceGeometry {
    /// Inclusive local x coordinate for the left plot edge.
    left: i32,
    /// Inclusive local y coordinate for the top plot edge.
    top: i32,
    /// Inclusive local x coordinate for the right plot edge.
    right: i32,
    /// Inclusive local y coordinate for the bottom plot edge.
    bottom: i32,
    /// Plot width in local pixels.
    width: f32,
    /// Plot height in local pixels.
    height: f32,
}

/// Render an EQ attractor surface node and emit typed actions.
fn render_eq_attractor_surface(
    surface: &EqAttractorSurfaceSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let model = surface.model.normalized();
    let style = surface.style.normalized();
    let id = WidgetId::from_label(&surface.key);
    let mut runtime = ui.begin_eq_attractor_surface_runtime(id);

    let response = ui.region_with_key(
        &format!("eq-attractor-surface-{:016x}", id.as_u64()),
        rect,
    );

    let Some(geometry) = eq_surface_geometry(rect, style.padding_px) else {
        ui.fill_rect_visual(rect, tokens.colors.surface);
        ui.stroke_rect_visual(rect, 1.0, tokens.colors.border);
        ui.set_eq_attractor_surface_runtime(id, runtime);
        return;
    };

    sync_eq_surface_runtime(&model, style, &mut runtime);
    let target_bands = eq_target_band_values(&model, style, &runtime);
    if !runtime.initialized {
        runtime.smoothed_band_gains = target_bands.clone();
        runtime.initialized = true;
    } else {
        let band_coeff = eq_smoothing_coeff(style.band_gain_smoothing_seconds);
        if runtime.smoothed_band_gains.len() != target_bands.len() {
            runtime.smoothed_band_gains = target_bands.clone();
        } else {
            for (current, target) in runtime
                .smoothed_band_gains
                .iter_mut()
                .zip(target_bands.iter().copied())
            {
                *current = eq_smooth_value(*current, target, band_coeff);
            }
        }
    }

    render_eq_surface_background(rect, geometry, ui, tokens, style, &model);
    render_eq_surface_curve(rect, geometry, ui, tokens, &model, style, &runtime.smoothed_band_gains);
    render_eq_surface_nodes(rect, geometry, ui, tokens, &model, style, &runtime);

    let mut emitted = reduce_eq_surface_interaction(&model, geometry, response, &mut runtime);
    for action in emitted.drain(..) {
        actions.push(UiAction::EqAttractorSurfaceChanged {
            key: surface.key.clone(),
            action,
        });
    }

    ui.set_eq_attractor_surface_runtime(id, runtime);
}

/// Reduce one frame of EQ attractor surface interaction.
fn reduce_eq_surface_interaction(
    model: &EqAttractorSurfaceModel,
    geometry: EqSurfaceGeometry,
    response: RegionResponse,
    runtime: &mut crate::ui::EqAttractorSurfaceRuntimeState,
) -> Vec<EqAttractorSurfaceAction> {
    let mut emitted = Vec::with_capacity(2);
    let hit = eq_hit_attractor(model, geometry, response.local_pointer)
        .or_else(|| eq_hit_attractor(model, geometry, response.raw_local_pointer));

    // Windows can report a double-click as press + double-click in one frame.
    if response.secondary_clicked {
        if let Some(id) = hit {
            emitted.push(EqAttractorSurfaceAction::Remove { id });
            runtime.active_drag_id = None;
        }
        return emitted;
    }

    if response.double_clicked {
        if hit.is_none() {
            let (x, y) = eq_pointer_to_normalized(response.local_pointer, geometry);
            emitted.push(EqAttractorSurfaceAction::Add { x, y });
        }
        runtime.active_drag_id = None;
        return emitted;
    }

    if response.pressed {
        runtime.active_drag_id = hit;
        if let Some(id) = hit {
            emitted.push(EqAttractorSurfaceAction::Select { id });
        }
    }

    if response.dragged
        && let Some(id) = runtime.active_drag_id
    {
        let (x, y) = eq_pointer_to_normalized(response.raw_local_pointer, geometry);
        emitted.push(EqAttractorSurfaceAction::Move { id, x, y });
    }

    if response.released {
        runtime.active_drag_id = None;
    }

    emitted
}

/// Synchronize smoothed runtime state from model targets.
fn sync_eq_surface_runtime(
    model: &EqAttractorSurfaceModel,
    style: EqAttractorSurfaceStyle,
    runtime: &mut crate::ui::EqAttractorSurfaceRuntimeState,
) {
    let motion_coeff = eq_smoothing_coeff(style.motion_smoothing_seconds);
    let attractor_coeff = eq_smoothing_coeff(style.attractor_smoothing_seconds);

    if !runtime.initialized {
        runtime.smoothed_warp = model.warp;
        runtime.smoothed_pull_force = model.pull_force;
    } else {
        runtime.smoothed_warp = eq_smooth_value(runtime.smoothed_warp, model.warp, motion_coeff);
        runtime.smoothed_pull_force =
            eq_smooth_value(runtime.smoothed_pull_force, model.pull_force, motion_coeff);
    }

    for (index, attractor) in model.attractors.iter().enumerate() {
        let target = crate::ui::EqAttractorSurfaceSmoothedAttractorState {
            x: attractor.x,
            y: attractor.y,
            depth: model.depths.get(index).copied().unwrap_or(1.0),
            cycles: model.cycles.get(index).copied().unwrap_or(1.0),
            rate_hz: model.rates_hz.get(index).copied().unwrap_or(0.0),
        };
        let entry = runtime
            .smoothed_attractors
            .entry(attractor.id)
            .or_insert(target);
        if runtime.initialized {
            entry.x = eq_smooth_value(entry.x, target.x, attractor_coeff);
            entry.y = eq_smooth_value(entry.y, target.y, attractor_coeff);
            entry.depth = eq_smooth_value(entry.depth, target.depth, attractor_coeff);
            entry.cycles = eq_smooth_value(entry.cycles, target.cycles, attractor_coeff);
            entry.rate_hz = eq_smooth_value(entry.rate_hz, target.rate_hz, attractor_coeff);
        } else {
            *entry = target;
        }
    }

    runtime
        .smoothed_attractors
        .retain(|id, _| model.attractors.iter().any(|attractor| attractor.id == *id));

    if let Some(active) = runtime.active_drag_id
        && !model.attractors.iter().any(|attractor| attractor.id == active)
    {
        runtime.active_drag_id = None;
    }
}

/// Build target normalized band values from smoothed attractor runtime.
fn eq_target_band_values(
    model: &EqAttractorSurfaceModel,
    style: EqAttractorSurfaceStyle,
    runtime: &crate::ui::EqAttractorSurfaceRuntimeState,
) -> Vec<f32> {
    let mut values = Vec::with_capacity(model.eq_bands);

    for band in 0..model.eq_bands {
        let band_pos = if model.eq_bands > 1 {
            band as f32 / (model.eq_bands - 1) as f32
        } else {
            0.0
        };
        let mut centers = Vec::with_capacity(model.attractors.len());
        let mut strengths = Vec::with_capacity(model.attractors.len());
        let mut depths = Vec::with_capacity(model.attractors.len());
        let mut cycles = Vec::with_capacity(model.attractors.len());
        let mut phases = Vec::with_capacity(model.attractors.len());
        for attractor in &model.attractors {
            let Some(state) = runtime.smoothed_attractors.get(&attractor.id) else {
                continue;
            };
            let strength =
                eq_gravity_strength(state.y, runtime.smoothed_warp, runtime.smoothed_pull_force);
            if strength <= f32::EPSILON {
                continue;
            }
            centers.push(state.x);
            strengths.push(strength);
            depths.push(state.depth.clamp(0.0, 1.0));
            cycles.push(state.cycles.max(0.0));
            phases.push(state.rate_hz.max(0.0) * 0.25);
        }
        let sample_pos = if model.reverse_global {
            1.0 - band_pos
        } else {
            band_pos
        };
        let wave = eq_gravity_wave_sample(
            sample_pos,
            0.0,
            &centers,
            &strengths,
            &depths,
            &cycles,
            &phases,
            EQ_SURFACE_GRAVITY_SIGMA,
        );
        let gain_db = (wave * model.eq_depth_db).clamp(-model.eq_depth_db, model.eq_depth_db);
        let normalized = ((gain_db + style.db_range) / (2.0 * style.db_range)).clamp(0.0, 1.0);
        values.push(normalized);
    }

    values
}
