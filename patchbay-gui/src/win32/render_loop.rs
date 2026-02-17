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
        let target = resolved_layout_size_for_resize_request(requested, host_client_size);
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
        mapped_input.pointer_pos = initial_plan
            .transform
            .surface_to_design_clamped(self.input.pointer_pos);
        let spec = (self.build_spec)(&mapped_input, &self.state);
        let plan = plan_root_render(&spec, self.input.window_size);
        mapped_input.pointer_pos = plan.transform.surface_to_design_clamped(self.input.pointer_pos);
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
        ui.reset_input_consumption();
        ui.clear_overlays();
        match render_checked(&spec, &mut ui, Point { x: 0, y: 0 }) {
            Ok(result) => {
                for action in result.actions {
                    (self.reduce_action)(&mut self.state, action);
                }
            }
            Err(err) => {
                log_line_safe(&format!("win32: declarative render validation error: {err}"));
            }
        }
        ui.draw_overlays();
        self.renderer.set_vector_commands(ui.take_vector_commands());
        let _ = self.ui_state.take_root_frame_size();
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
        self.renderer.upload(self.canvas.size(), self.canvas.pixels());
        self.renderer.render().is_ok()
    }

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
