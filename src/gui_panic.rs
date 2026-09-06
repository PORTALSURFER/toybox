//! Unwind containment for plugin GUI callbacks; never used by the audio thread.

use std::io::Write;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Keep Rust unwinds inside the plugin and retain the original hook diagnostics.
pub(crate) fn contain<T>(operation: &str, callback: impl FnOnce() -> T) -> Option<T> {
    match catch_unwind(AssertUnwindSafe(callback)) {
        Ok(value) => Some(value),
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("non-string panic payload");
            let _ = writeln!(
                std::io::stderr().lock(),
                "Toybox GUI {operation} failed: {message}"
            );
            // A user-defined panic payload can itself panic in Drop.
            if let Err(secondary) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
                std::mem::forget(secondary);
            }
            None
        }
    }
}
