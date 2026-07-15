impl<'a> Ui<'a> {
    /// Render and interact with a curve editor in a fixed rectangle.
    pub(crate) fn curve_editor_in_rect(
        &mut self,
        model: &mut crate::declarative::CurveModel,
        request: CurveEditorRectRenderRequest,
    ) -> CurveEditorResponse {
        model.normalize_in_place();
        let mut runtime = self.begin_curve_editor_runtime(request.id);
        let region_key = format!("curve-editor-{:016x}", request.id.as_u64());
        let region = self.region_with_key(&region_key, request.rect);
        let interaction = request.interaction.clone();
        let decorators = CurveEditorInteractionDecorators {
            segment_move: request.segment_move,
            point_horizontal_constraint: request.point_horizontal_constraint,
            point_vertical_constraint: request.point_vertical_constraint,
        };
        let changed = self.reduce_curve_editor_interaction(
            model,
            &mut runtime,
            interaction.clone(),
            decorators,
            region,
            request.rect,
        );
        if changed {
            model.normalize_in_place();
            enforce_endpoint_mode(model, interaction.endpoint_mode);
        }
        let visual_state = self.resolve_curve_editor_visual_state(
            model,
            &runtime,
            &interaction,
            decorators.segment_move,
            region,
            request.rect,
        );
        self.render_curve_editor_visuals(
            model,
            request.rect,
            visual_state,
            &request.style,
            &request.grid,
            request.playhead_x,
        );
        self.set_curve_editor_runtime(request.id, runtime);
        CurveEditorResponse { changed }
    }
}
