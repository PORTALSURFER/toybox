/// Fixed frame step used for deterministic UI-side smoothing.
const EQ_SURFACE_FRAME_DT_SECONDS: f32 = 1.0 / 60.0;
/// Pointer hit radius for attractor handles in pixels.
const EQ_SURFACE_ATTRACTOR_HIT_RADIUS_PX: i32 = 10;
/// Pointer travel required before a selected attractor starts moving.
const EQ_SURFACE_DRAG_THRESHOLD_PX: i32 = 3;

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
        }
        runtime.active_drag_id = None;
        runtime.drag_origin = None;
        runtime.drag_started = false;
        return emitted;
    }

    if response.double_clicked {
        if hit.is_none() {
            let (x, y) = eq_pointer_to_normalized(response.local_pointer, geometry);
            emitted.push(EqAttractorSurfaceAction::Add { x, y });
        }
        runtime.active_drag_id = None;
        runtime.drag_origin = None;
        runtime.drag_started = false;
        return emitted;
    }

    if response.pressed {
        runtime.active_drag_id = hit;
        runtime.drag_origin = Some(response.raw_local_pointer);
        runtime.drag_started = false;
        if let Some(id) = hit {
            emitted.push(EqAttractorSurfaceAction::Select { id });
        }
    }

    if response.dragged && let Some(id) = runtime.active_drag_id {
        if !runtime.drag_started
            && let Some(origin) = runtime.drag_origin
        {
            let dx = response.raw_local_pointer.x - origin.x;
            let dy = response.raw_local_pointer.y - origin.y;
            let distance2 = dx * dx + dy * dy;
            if distance2 >= EQ_SURFACE_DRAG_THRESHOLD_PX.pow(2) {
                runtime.drag_started = true;
            }
        }
        if runtime.drag_started {
            let (x, y) = eq_pointer_to_normalized(response.raw_local_pointer, geometry);
            emitted.push(EqAttractorSurfaceAction::Move { id, x, y });
        }
    }

    if response.released {
        runtime.active_drag_id = None;
        runtime.drag_origin = None;
        runtime.drag_started = false;
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
        runtime.smoothed_wave_depth = model.wave_depth;
        runtime.smoothed_wave_cycles = model.wave_cycles;
        runtime.smoothed_wave_phase_token = model.wave_phase_token;
    } else {
        runtime.smoothed_wave_depth = eq_smooth_value(
            runtime.smoothed_wave_depth,
            model.wave_depth,
            motion_coeff,
        );
        runtime.smoothed_wave_cycles = eq_smooth_value(
            runtime.smoothed_wave_cycles,
            model.wave_cycles,
            motion_coeff,
        );
        runtime.smoothed_wave_phase_token = eq_smooth_value(
            runtime.smoothed_wave_phase_token,
            model.wave_phase_token,
            motion_coeff,
        );
    }

    for (index, attractor) in model.attractors.iter().enumerate() {
        let target = crate::ui::EqAttractorSurfaceSmoothedAttractorState {
            x: attractor.x,
            y: attractor.y,
            pull: model.attractor_pulls.get(index).copied().unwrap_or(1.0),
            radius: model.attractor_radii.get(index).copied().unwrap_or(0.12),
        };
        let entry = runtime
            .smoothed_attractors
            .entry(attractor.id)
            .or_insert(target);
        if runtime.initialized {
            entry.x = eq_smooth_value(entry.x, target.x, attractor_coeff);
            entry.y = eq_smooth_value(entry.y, target.y, attractor_coeff);
            entry.pull = eq_smooth_value(entry.pull, target.pull, attractor_coeff);
            entry.radius = eq_smooth_value(entry.radius, target.radius, attractor_coeff);
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
        runtime.drag_origin = None;
        runtime.drag_started = false;
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
        let mut targets = Vec::with_capacity(model.attractors.len());
        let mut pulls = Vec::with_capacity(model.attractors.len());
        let mut radii = Vec::with_capacity(model.attractors.len());
        for attractor in &model.attractors {
            let Some(state) = runtime.smoothed_attractors.get(&attractor.id) else {
                continue;
            };
            if state.pull <= f32::EPSILON {
                continue;
            }
            centers.push(state.x);
            targets.push((state.y * 2.0 - 1.0) * runtime.smoothed_wave_depth.clamp(0.0, 1.0));
            pulls.push(state.pull.max(0.0));
            radii.push(state.radius.max(0.01));
        }
        let sample_pos = if model.reverse_global {
            1.0 - band_pos
        } else {
            band_pos
        };
        let wave = eq_wave_sample(
            sample_pos,
            runtime.smoothed_wave_phase_token,
            runtime.smoothed_wave_cycles,
            runtime.smoothed_wave_depth,
            &centers,
            &targets,
            &pulls,
            &radii,
        );
        let gain_db = (wave * model.eq_depth_db).clamp(-model.eq_depth_db, model.eq_depth_db);
        let normalized = ((gain_db + style.db_range) / (2.0 * style.db_range)).clamp(0.0, 1.0);
        values.push(normalized);
    }

    values
}
