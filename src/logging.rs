//! Minimal file logger for GUI crash diagnostics in hosts.
//!
//! This logger is dependency-free and flushes on each write so logs survive
//! abrupt host crashes.

use std::collections::VecDeque;
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

/// Maximum number of in-memory logging failure entries retained per process.
const LOG_ERROR_CAPACITY: usize = 64;
/// Global process log sink created on first write.
static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
/// Best-effort bounded buffer of logging failures for later diagnostics.
static LOG_ERRORS: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

/// Return the on-disk log file path for this process.
pub(crate) fn log_path() -> PathBuf {
    let pid = std::process::id();
    std::env::temp_dir().join(format!("toybox_gui_{pid}.log"))
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
        record_failure("toybox-gui log", err);
    }
}

/// Record a logging failure without panicking the host.
pub(crate) fn record_failure(context: &str, err: LogError) {
    let entry = format!("{context}: {err}");
    let errors = LOG_ERRORS.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Ok(mut guard) = errors.lock() {
        push_failure_entry(&mut guard, entry, LOG_ERROR_CAPACITY);
    }
}

/// Push a failure entry while keeping the queue bounded.
fn push_failure_entry(entries: &mut VecDeque<String>, entry: String, capacity: usize) {
    if capacity == 0 {
        return;
    }
    if entries.len() >= capacity {
        let _ = entries.pop_front();
    }
    entries.push_back(entry);
}

/// Open or create the on-disk log file.
fn open_log_file(path: &Path) -> Result<File, LogError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

/// Return a unix timestamp in milliseconds for log entries.
fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::push_failure_entry;
    use std::collections::VecDeque;

    #[test]
    fn push_failure_entry_keeps_latest_entries() {
        let mut entries = VecDeque::new();
        push_failure_entry(&mut entries, "a".to_string(), 2);
        push_failure_entry(&mut entries, "b".to_string(), 2);
        push_failure_entry(&mut entries, "c".to_string(), 2);
        let retained: Vec<_> = entries.into_iter().collect();
        assert_eq!(retained, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn push_failure_entry_ignores_zero_capacity() {
        let mut entries = VecDeque::new();
        push_failure_entry(&mut entries, "a".to_string(), 0);
        assert!(entries.is_empty());
    }
}
