//! Host-parented view implementation for the minimal VST3 example.

use std::cell::Cell;
use std::ffi::c_void;

use toybox::vst3::prelude::Steinberg::*;
use toybox::vst3::prelude::*;

/// Minimal host-parented view implementation.
pub(crate) struct GainView {
    /// Current view rectangle.
    rect: Cell<ViewRect>,
    /// Whether the view is currently attached to a host parent.
    attached: Cell<bool>,
}

impl GainView {
    /// Create a default view rectangle.
    pub(crate) fn new() -> Self {
        Self {
            rect: Cell::new(view_rect(420, 200)),
            attached: Cell::new(false),
        }
    }
}

impl Class for GainView {
    type Interfaces = (IPlugView,);
}

impl IPlugViewTrait for GainView {
    unsafe fn isPlatformTypeSupported(&self, r#type: FIDString) -> tresult {
        bool_to_tresult(unsafe { platform_type_matches(r#type, default_platform_type()) })
    }

    unsafe fn attached(&self, parent: *mut c_void, r#type: FIDString) -> tresult {
        if parent.is_null() {
            return kInvalidArgument;
        }
        if !unsafe { platform_type_matches(r#type, default_platform_type()) } {
            return kResultFalse;
        }

        self.attached.set(true);
        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        self.attached.set(false);
        kResultOk
    }

    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyDown(&self, _key: char16, _key_code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyUp(&self, _key: char16, _key_code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        unsafe { *size = self.rect.get() };
        kResultOk
    }

    unsafe fn onSize(&self, new_size: *mut ViewRect) -> tresult {
        if new_size.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        self.rect.set(unsafe { *new_size });
        kResultOk
    }

    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setFrame(&self, _frame: *mut IPlugFrame) -> tresult {
        kResultOk
    }

    unsafe fn canResize(&self) -> tresult {
        kResultTrue
    }

    unsafe fn checkSizeConstraint(&self, rect: *mut ViewRect) -> tresult {
        if rect.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        let rect = unsafe { &mut *rect };
        if rect.right - rect.left < 320 {
            rect.right = rect.left + 320;
        }
        if rect.bottom - rect.top < 160 {
            rect.bottom = rect.top + 160;
        }

        kResultOk
    }
}
