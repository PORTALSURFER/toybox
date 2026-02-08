impl<'a> Ui<'a> {

    /// Return the rendered footprint for a knob block at the current theme scale.
    ///
    /// The name and value labels are single-line and clamped to the knob block
    /// width, and the width includes ring/indicator padding so adjacent knobs in
    /// grid layouts do not overlap visually or interactively.
    pub fn knob_block_size(&self, _name_label: &str, _value_label: &str) -> Size {
        knob_block_size_for_diameter(self.layout.knob_size.max(1) as u32, self.theme.text_scale)
    }

    /// Return the rendered footprint for a slider block.
    ///
    /// The label is rendered above the control and clamped to control width.
    pub fn slider_block_size(&self, label: &str, control_size: Size) -> Size {
        let control = Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        };
        let label_height = if label.is_empty() {
            0
        } else {
            8 * self.theme.text_scale.max(1)
        };
        Size {
            width: control.width,
            height: control.height + label_height,
        }
    }

    /// Return the rendered footprint for a toggle block.
    ///
    /// The label is rendered above the control and clamped to control width.
    pub fn toggle_block_size(&self, label: &str, control_size: Size) -> Size {
        let control = Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        };
        let label_height = if label.is_empty() {
            0
        } else {
            8 * self.theme.text_scale.max(1)
        };
        Size {
            width: control.width,
            height: control.height + label_height,
        }
    }

    /// Return the rendered footprint for a dropdown block.
    ///
    /// The label is rendered above the control and clamped to control width.
    pub fn dropdown_block_size(&self, label: &str, control_size: Size) -> Size {
        let control = Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        };
        let label_height = if label.is_empty() {
            0
        } else {
            8 * self.theme.text_scale.max(1)
        };
        Size {
            width: control.width,
            height: control.height + label_height,
        }
    }

    /// Return the rendered footprint for a button block.
    ///
    /// Button labels are rendered inside the provided control size.
    pub fn button_block_size(&self, _label: &str, control_size: Size) -> Size {
        Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        }
    }

    /// Run a closure with a temporary layout origin.
    pub fn with_layout<F>(&mut self, origin: Point, mut f: F)
    where
        F: FnMut(&mut Ui<'_>),
    {
        let previous = *self.layout;
        self.layout_stack.push(previous);
        self.layout.cursor = origin;
        f(self);
        if let Some(restored) = self.layout_stack.pop() {
            *self.layout = restored;
        }
    }
}
