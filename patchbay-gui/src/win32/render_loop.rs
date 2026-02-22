impl<State, Init, Build, Reduce> WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    fn render_frame(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_add(1);
        self.initialize_once();
        self.apply_pending_resize_request();
        self.prepare_frame_input();
        self.render_ui_frame();
        self.render_debug_overlay_if_enabled();
        let render_ok = self.present_canvas();
        self.maybe_show_window(render_ok);
        self.reset_transient_input_state();
    }

    fn initialize_once(&mut self) {
        if !self.initialized {
            (self.on_init)(&mut self.state);
            self.initialized = true;
        }
    }

    fn apply_pending_resize_request(&mut self) {
        let Some((width, height)) = unpack_size(self.resize_request.swap(0, Ordering::AcqRel))
        else {
            return;
        };
        let requested = Size {
            width: width.max(1),
            height: height.max(1),
        };
        let mut host_client_size = self.current_client_size();
        if host_client_size != Some((requested.width, requested.height)) {
            self.apply_child_size_request(requested);
            host_client_size = self.current_client_size();
        }
        let target = resolved_layout_size_for_resize_request(
            requested,
            host_client_size,
            self.configured_aspect_ratio(),
        );
        self.apply_layout_size(target, true);
    }

    fn prepare_frame_input(&mut self) {
        self.sync_client_size_if_needed();
        self.layout.cursor = self.layout_origin;
        self.sync_pointer_pos();
        self.sync_mouse_buttons();
    }

    fn render_ui_frame(&mut self) {
        self.ui_state.begin_frame();
        let initial_spec = (self.build_spec)(&self.input, &self.state);
        let initial_plan = plan_root_render(&initial_spec, self.input.window_size);
        let mut mapped_input = self.input.clone();
        mapped_input.window_size = initial_plan.layout_size;
        let drag_active = self.input.mouse_down || self.input.mouse_secondary_down;
        mapped_input.pointer_in_window = self.input.pointer_in_window || drag_active;
        mapped_input.pointer_pos = map_surface_pointer_to_design(
            &initial_plan.transform,
            self.input.pointer_pos,
            self.input.pointer_in_window,
            drag_active,
        );
        let spec = (self.build_spec)(&mapped_input, &self.state);
        self.active_text_edit = spec_has_active_text_edit(&spec);
        self.active_text_edit_shared
            .store(self.active_text_edit, Ordering::Release);
        let plan = plan_root_render(&spec, self.input.window_size);
        mapped_input.window_size = plan.layout_size;
        mapped_input.pointer_in_window = self.input.pointer_in_window || drag_active;
        mapped_input.pointer_pos = map_surface_pointer_to_design(
            &plan.transform,
            self.input.pointer_pos,
            self.input.pointer_in_window,
            drag_active,
        );
        if self.canvas.size() != plan.layout_size {
            self.canvas
                .resize(plan.layout_size.width, plan.layout_size.height);
        }
        self.canvas.clear(self.theme.background);
        self.renderer
            .set_presentation_transform(PresentationTransform {
                scale_x: plan.transform.scale_x,
                scale_y: plan.transform.scale_y,
                offset_x: plan.transform.offset_x,
                offset_y: plan.transform.offset_y,
            });
        let mut ui = Ui::new(
            &mut self.canvas,
            &mapped_input,
            &mut self.ui_state,
            &mut self.layout,
            &self.theme,
        );
        ui.set_vector_text_enabled(self.renderer.vector_text_available());
        ui.set_vector_shapes_enabled(true);
        ui.reset_input_consumption();
        ui.clear_overlays();
        match render_checked_with_engine(
            &spec,
            &mut ui,
            Point { x: 0, y: 0 },
            &mut self.layout_engine,
        ) {
            Ok(result) => {
                for action in result.actions {
                    update_text_edit_tracking_for_action(&mut self.active_text_edit, &action);
                    self.active_text_edit_shared
                        .store(self.active_text_edit, Ordering::Release);
                    invalidate_engine_for_action(&mut self.layout_engine, &action);
                    (self.reduce_action)(&mut self.state, action);
                }
            }
            Err(err) => {
                log_line_safe(&format!(
                    "win32: declarative render validation error: {err}"
                ));
            }
        }
        ui.draw_overlays();
        let vector_commands = ui.take_vector_commands();
        drop(ui);
        self.renderer.set_vector_commands(vector_commands);
        self.apply_measured_root_frame_resize_request(&spec);
    }

    fn apply_measured_root_frame_resize_request(&mut self, spec: &UiSpec) {
        let target = resolved_root_frame_resize_request(
            self.canonical_layout_size,
            self.ui_state.take_root_frame_size(),
            spec.root.design_size_value(),
        );
        let Some(target) = target else {
            return;
        };
        self.last_size
            .store(pack_size(target.width, target.height), Ordering::Release);
        self.resize_request
            .store(pack_size(target.width, target.height), Ordering::Release);
    }

    fn render_debug_overlay_if_enabled(&mut self) {
        if !self.debug_input {
            return;
        }
        let text = format!(
            "frame={} ptr=({}, {}) md={} mr={} rd={}",
            self.frame_counter,
            self.input.pointer_pos.x,
            self.input.pointer_pos.y,
            self.input.mouse_down as u8,
            self.input.mouse_released as u8,
            self.input.mouse_secondary_down as u8
        );
        self.canvas
            .draw_text(Point { x: 6, y: 6 }, &text, self.theme.text, 1);
    }

    fn present_canvas(&mut self) -> bool {
        self.renderer
            .upload(self.canvas.size(), self.canvas.pixels());
        let render_result = self.renderer.render();
        self.fulfill_frame_capture_if_requested(&render_result);
        render_result.is_ok()
    }

    #[cfg(feature = "frame-capture")]
    fn fulfill_frame_capture_if_requested(&self, render_result: &Result<(), GuiError>) {
        let Some(request_id) = self.frame_capture.pending_request_id() else {
            return;
        };
        let result = match render_result {
            Ok(()) => self
                .renderer
                .readback_render_target_rgba8()
                .map_err(|err| format!("readback failed: {err}")),
            Err(err) => Err(format!("render failed before readback: {err}")),
        };
        self.frame_capture.complete_request(request_id, result);
    }

    #[cfg(not(feature = "frame-capture"))]
    fn fulfill_frame_capture_if_requested(&self, _render_result: &Result<(), GuiError>) {}

    fn maybe_show_window(&mut self, render_ok: bool) {
        if self.shown || !render_ok {
            return;
        }
        if self.prewarm_frames > 0 {
            self.prewarm_frames = self.prewarm_frames.saturating_sub(1);
        }
        let elapsed_ms = self.created_at.elapsed().as_millis();
        if self.prewarm_frames == 0 && elapsed_ms >= MIN_SHOW_DELAY_MS {
            log_line_safe("win32: render ok, showing window");
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_SHOW);
            }
            self.shown = true;
            let _ = self.renderer.render();
        }
    }

    fn reset_transient_input_state(&mut self) {
        self.input.mouse_pressed = false;
        self.input.mouse_released = false;
        self.input.mouse_double_clicked = false;
        self.input.mouse_secondary_pressed = false;
        self.input.mouse_secondary_released = false;
        self.input.wheel_delta = 0.0;
        self.input.key_pressed = None;
        self.input.dropped_files.clear();
    }
}

fn map_surface_pointer_to_design(
    transform: &RootTransform,
    surface_pointer: Point,
    pointer_in_window: bool,
    drag_active: bool,
) -> Point {
    if drag_active {
        return transform.surface_to_design(surface_pointer);
    }
    if pointer_in_window {
        return transform.surface_to_design_clamped(surface_pointer);
    }
    transform.surface_to_design(surface_pointer)
}

/// Resolve a pending host resize request from measured root-frame output.
fn resolved_root_frame_resize_request(
    current_layout_size: Size,
    measured_root_size: Option<Size>,
    design_size: Option<Size>,
) -> Option<Size> {
    // Fixed design-size roots intentionally stay host-scaled and must not
    // request host size changes every frame.
    if design_size.is_some() {
        return None;
    }
    let measured = measured_root_size?;
    let target = Size {
        width: measured.width.max(1),
        height: measured.height.max(1),
    };
    if client_size_changed(current_layout_size, target) {
        Some(target)
    } else {
        None
    }
}

/// Invalidate declarative engine cache state for one emitted action.
///
/// Runtime invalidation is keyed by action source node when available. If an
/// action key cannot be resolved in the current registry, the engine falls back
/// to full-tree measure invalidation for safety.
fn invalidate_engine_for_action(engine: &mut LayoutEngineState, action: &UiAction) {
    let key = action_source_key(action);
    if let Some(node_id) = engine.node_id_for_key(key) {
        match action.invalidation_scope() {
            UiInvalidationScope::MeasureSubtree => engine.invalidate_measure_subtree(node_id),
            UiInvalidationScope::LayoutSubtree => engine.invalidate_layout_subtree(node_id),
        }
    } else {
        engine.invalidate_all_measure();
    }
}

/// Return the source widget key for an emitted UI action.
fn action_source_key(action: &UiAction) -> &str {
    match action {
        UiAction::KnobChanged { key, .. } => key,
        UiAction::SliderChanged { key, .. } => key,
        UiAction::ToggleChanged { key, .. } => key,
        UiAction::ButtonPressed { key } => key,
        UiAction::DropdownSelected { key, .. } => key,
        UiAction::TabSelected { key, .. } => key,
        UiAction::DropdownDoubleClicked { key } => key,
        UiAction::CurveEditorChanged { key, .. } => key,
        UiAction::TextBoxEditRequested { key } => key,
        UiAction::TextBoxEdited { key, .. } => key,
        UiAction::TextBoxEditCommitted { key, .. } => key,
        UiAction::TextBoxEditCanceled { key } => key,
        UiAction::RegionHover { key, .. } => key,
        UiAction::RegionInteracted { key, .. } => key,
    }
}

/// Update text-edit tracking from one emitted action.
fn update_text_edit_tracking_for_action(active: &mut bool, action: &UiAction) {
    match action {
        UiAction::TextBoxEditRequested { .. } => *active = true,
        UiAction::TextBoxEditCommitted { .. } | UiAction::TextBoxEditCanceled { .. } => {
            *active = false
        }
        _ => {}
    }
}

/// Return `true` when any text box in `spec` is currently in edit mode.
fn spec_has_active_text_edit(spec: &UiSpec) -> bool {
    node_has_active_text_edit(spec.root.content())
}

/// Return `true` when `node` or descendants include an editable active text box.
fn node_has_active_text_edit(node: &crate::declarative::Node) -> bool {
    use crate::declarative::Node;

    if let Node::TextBox(text_box) = node {
        return text_box
            .edit
            .as_ref()
            .map(|edit| edit.editing)
            .unwrap_or(false);
    }

    let mut active = false;
    node.for_each_child(|child| {
        if !active && node_has_active_text_edit(child) {
            active = true;
        }
    });
    active
}
