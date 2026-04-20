/// Shared two-pass frame planning output for live and headless Patchbay scenes.
///
/// Both rendering entrypoints build specs from the same surface input, remap
/// pointer coordinates through the same root transform contract, then execute
/// the same declarative scene with backend-specific feature toggles.
#[derive(Clone, Debug)]
pub(crate) struct PlannedSceneFrame {
    /// Original host/surface input snapshot.
    pub surface_input: crate::host::InputState,
    /// Final design-space input snapshot used for scene execution.
    pub frame_input: crate::host::InputState,
    /// Fully resolved declarative spec for the frame.
    pub spec: UiSpec,
    /// Final root render plan for the frame.
    pub plan: RootRenderPlan,
}

/// Backend feature toggles applied while executing one planned scene frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneRenderFeatures {
    /// Whether vector text commands should be emitted.
    pub vector_text: bool,
    /// Whether vector shape commands should be emitted.
    pub vector_shapes: bool,
}

/// Result of executing one planned scene frame.
#[derive(Debug)]
pub(crate) struct ExecutedSceneFrame {
    /// Declarative render metadata for the executed frame.
    pub render_result: RenderResult,
    /// Vector overlay commands collected during the frame.
    pub vector_commands: Vec<crate::vector::scene::VectorCommand>,
}

/// Build the shared two-pass scene plan used by live and headless consumers.
///
/// The callback intentionally runs twice:
///
/// - first against surface input so root sizing can resolve the initial transform
/// - then against design-space input remapped through that transform
///
/// The second pass becomes the canonical scene contract for the frame, and any
/// backend-specific work should happen only after this function returns.
pub(crate) fn plan_scene_frame<Build>(
    surface_input: &crate::host::InputState,
    mut build_spec: Build,
) -> PlannedSceneFrame
where
    Build: FnMut(&crate::host::InputState) -> UiSpec,
{
    let initial_spec = build_spec(surface_input);
    let initial_plan = plan_root_render(&initial_spec, surface_input.window_size);
    let remapped_input = remap_surface_input(surface_input, initial_plan);
    let spec = build_spec(&remapped_input);
    let plan = plan_root_render(&spec, surface_input.window_size);
    let frame_input = remap_surface_input(surface_input, plan);

    PlannedSceneFrame {
        surface_input: surface_input.clone(),
        frame_input,
        spec,
        plan,
    }
}

/// Execute one planned scene frame with the provided UI/runtime state.
///
/// Callers remain responsible for backend-specific setup such as canvas resize,
/// surface presentation transforms, and GPU upload/readback. This helper owns
/// only the shared declarative scene execution contract.
pub(crate) fn execute_scene_frame(
    frame: &PlannedSceneFrame,
    canvas: &mut crate::Canvas,
    ui_state: &mut crate::ui::UiState,
    layout: &mut crate::ui::Layout,
    theme: &crate::ui::Theme,
    engine: &mut LayoutEngineState,
    features: SceneRenderFeatures,
) -> Result<ExecutedSceneFrame, DeclarativeError> {
    let mut ui = crate::ui::Ui::new(
        canvas,
        &frame.frame_input,
        ui_state,
        layout,
        theme,
    );
    ui.set_vector_text_enabled(features.vector_text);
    ui.set_vector_shapes_enabled(features.vector_shapes);
    ui.reset_input_consumption();
    ui.clear_overlays();
    let render_result = render_checked_with_engine(
        &frame.spec,
        &mut ui,
        crate::canvas::Point { x: 0, y: 0 },
        engine,
    )?;
    ui.draw_overlays();
    let vector_commands = ui.take_vector_commands();

    Ok(ExecutedSceneFrame {
        render_result,
        vector_commands,
    })
}

/// Remap one surface input snapshot into design space for a resolved root plan.
fn remap_surface_input(
    surface_input: &crate::host::InputState,
    plan: RootRenderPlan,
) -> crate::host::InputState {
    let drag_active = surface_input.mouse_down || surface_input.mouse_secondary_down;
    let mut mapped = surface_input.clone();
    mapped.window_size = plan.layout_size;
    mapped.pointer_in_window = surface_input.pointer_in_window || drag_active;
    mapped.pointer_pos = map_surface_pointer_to_design(
        &plan.transform,
        surface_input.pointer_pos,
        surface_input.pointer_in_window,
        drag_active,
    );
    mapped
}

/// Map a host surface-space pointer into root design-space coordinates.
fn map_surface_pointer_to_design(
    transform: &RootTransform,
    surface_pointer: crate::canvas::Point,
    pointer_in_window: bool,
    drag_active: bool,
) -> crate::canvas::Point {
    if drag_active {
        return transform.surface_to_design(surface_pointer);
    }
    if pointer_in_window {
        return transform.surface_to_design_clamped(surface_pointer);
    }
    transform.surface_to_design(surface_pointer)
}
