// Included by the shared macOS Radiant host.

#[link(name = "QuartzCore", kind = "framework")]
unsafe extern "C" {}

/// Owns a partially attached view until initialization commits.
struct PendingEditorView(Option<NonNull<Object>>);

impl Drop for PendingEditorView {
    fn drop(&mut self) {
        if let Some(view) = self.0.take() {
            unsafe {
                cleanup_editor_view(view.as_ptr());
            }
        }
    }
}

/// Attach a plain system Metal layer without registering a custom ObjC class.
unsafe fn create_metal_layer(view: *mut Object) -> Option<NonNull<Object>> {
    let backing: *mut Object = msg_send![view, layer];
    if backing.is_null() {
        return None;
    }
    let layer: *mut Object = msg_send![class!(CAMetalLayer), new];
    let layer = NonNull::new(layer)?;
    (*view).set_ivar("metal_layer", layer.as_ptr() as usize);
    let _: () = msg_send![backing, addSublayer: layer.as_ptr()];
    sync_metal_layer(view);
    Some(layer)
}

/// Keep the owned sublayer aligned without changing the NSView's backing layer.
unsafe fn sync_metal_layer(view: *mut Object) {
    let layer = *(*view).get_ivar::<usize>("metal_layer") as *mut Object;
    if layer.is_null() {
        return;
    }
    let backing: *mut Object = msg_send![view, layer];
    if backing.is_null() {
        return;
    }
    let bounds: NSRect = msg_send![backing, bounds];
    let scale: f64 = f64::from(view_dpi_scale(view).factor());
    let _: () = msg_send![class!(CATransaction), begin];
    let _: () = msg_send![class!(CATransaction), setDisableActions: YES];
    let _: () = msg_send![layer, setFrame: bounds];
    let _: () = msg_send![layer, setContentsScale: scale];
    let _: () = msg_send![class!(CATransaction), commit];
}

/// Release the retained sublayer after the renderer has been destroyed.
unsafe fn drop_metal_layer(view: *const Object) {
    let view = view.cast_mut();
    let layer = *(*view).get_ivar::<usize>("metal_layer") as *mut Object;
    (*view).set_ivar("metal_layer", 0_usize);
    if !layer.is_null() {
        let _: () = msg_send![layer, removeFromSuperlayer];
        let _: () = msg_send![layer, release];
    }
}

/// Read the native quarantine marker without borrowing editor state.
unsafe fn view_failed(view: *const Object) -> bool {
    view.as_ref()
        .is_some_and(|view| *view.get_ivar::<usize>("failed") != 0)
}

/// Stop callbacks without re-entering a potentially inconsistent editor.
unsafe fn quarantine_view(view: *mut Object) {
    (*view).set_ivar("failed", 1_usize);
    stop_redraw_driver(view);
}

/// Catch inside the Objective-C callback, before Rust reaches a non-unwinding ABI.
fn native_callback(view: &Object, operation: &str, callback: impl FnOnce()) {
    unsafe {
        if view_failed(view) {
            return;
        }
        if crate::gui_panic::contain(operation, callback).is_none() {
            quarantine_view(view as *const Object as *mut Object);
        }
    }
}

/// Clean each owned resource independently, including partially initialized views.
unsafe fn cleanup_editor_resources(view: *const Object) {
    let _ = crate::gui_panic::contain("stop redraw", || stop_redraw_driver(view));
    let _ = crate::gui_panic::contain("remove tracking", || remove_tracking_area(view));
    let _ = crate::gui_panic::contain("drop renderer", || drop_renderer(view));
    let _ = crate::gui_panic::contain("drop Metal layer", || drop_metal_layer(view));
    let _ = crate::gui_panic::contain("drop editor", || drop_runtime(view));
}

/// Detach and release our view after callbacks and GPU resources are stopped.
unsafe fn cleanup_editor_view(view: *mut Object) {
    cleanup_editor_resources(view);
    let _: () = msg_send![view, removeFromSuperview];
    let _: () = msg_send![view, release];
}
