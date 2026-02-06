//! Minimal file logger wrapper for toybox GUI diagnostics.
//!
//! The shared implementation lives in `toybox-process-log`; this module keeps
//! local call sites unchanged.

use toybox_process_log::ProcessFileLogger;

/// Process-local logger used for toybox GUI diagnostics.
static LOGGER: ProcessFileLogger = ProcessFileLogger::with_default_capacity("toybox_gui");

/// Write a log line and ignore any failures.
pub(crate) fn log_line_safe(message: &str) {
    LOGGER.log_line_safe("toybox-gui log", message);
}
