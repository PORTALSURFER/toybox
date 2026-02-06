//! Minimal file logger for Patchbay GUI crash diagnostics.
//!
//! This logger avoids external dependencies and flushes every write so we can
//! capture logs even when the host crashes.

use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors produced by the logger.
#[derive(Debug)]
pub(crate) enum LogError {
    /// The log file could not be opened or written.
    Io(std::io::Error),
    /// The log file mutex was poisoned by a panic.
    Poisoned,
    /// The log file was not initialized.
    NotInitialized,
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "log I/O error: {err}"),
            Self::Poisoned => write!(f, "log mutex poisoned"),
            Self::NotInitialized => write!(f, "log not initialized"),
        }
    }
}

impl std::error::Error for LogError {}

impl From<std::io::Error> for LogError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static LOG_ERRORS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

/// Return the on-disk log file path for this process.
pub(crate) fn log_path() -> PathBuf {
    let pid = std::process::id();
    std::env::temp_dir().join(format!("patchbay_gui_{pid}.log"))
}

/// Initialize the log file if needed and return the file path.
pub(crate) fn init() -> Result<PathBuf, LogError> {
    let path = log_path();
    if LOG_FILE.get().is_none() {
        let file = open_log_file(&path)?;
        let _ = LOG_FILE.set(Mutex::new(file));
    }
    Ok(path)
}

/// Write a log line and flush it to disk for crash diagnostics.
pub(crate) fn log_line(message: &str) -> Result<(), LogError> {
    let _ = init()?;
    let file = LOG_FILE.get().ok_or(LogError::NotInitialized)?;
    let mut file = file.lock().map_err(|_| LogError::Poisoned)?;
    let stamp = timestamp_ms();
    writeln!(file, "[{stamp}] {message}")?;
    file.flush()?;
    Ok(())
}

/// Write a log line and ignore any failures.
pub(crate) fn log_line_safe(message: &str) {
    if let Err(err) = log_line(message) {
        record_failure("patchbay-gui log", err);
    }
}

/// Record a logging failure without panicking the host.
pub(crate) fn record_failure(context: &str, err: LogError) {
    let entry = format!("{context}: {err}");
    let errors = LOG_ERRORS.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut guard) = errors.lock() {
        guard.push(entry);
    }
}

fn open_log_file(path: &Path) -> Result<File, LogError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
