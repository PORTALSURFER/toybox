//! Shared process-local file logger for host crash diagnostics.
//!
//! The logger is intentionally dependency-free and flushes each write so logs
//! remain useful even when hosts terminate abruptly.

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default in-memory capacity for retained logging failures.
pub const DEFAULT_FAILURE_CAPACITY: usize = 64;

/// Errors produced by process file logging.
#[derive(Debug)]
pub enum LogError {
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

/// Process-local file logger with bounded in-memory failure retention.
#[derive(Debug)]
pub struct ProcessFileLogger {
    /// Prefix used to build the temp log filename (`{prefix}_{pid}.log`).
    file_prefix: &'static str,
    /// Maximum number of logging failures retained in memory.
    failure_capacity: usize,
    /// Global process log sink created on first write.
    file: OnceLock<Mutex<File>>,
    /// Bounded buffer of logging failures for best-effort diagnostics.
    errors: OnceLock<Mutex<VecDeque<String>>>,
}

impl ProcessFileLogger {
    /// Create a logger with a custom failure buffer capacity.
    pub const fn new(file_prefix: &'static str, failure_capacity: usize) -> Self {
        Self {
            file_prefix,
            failure_capacity,
            file: OnceLock::new(),
            errors: OnceLock::new(),
        }
    }

    /// Create a logger with [`DEFAULT_FAILURE_CAPACITY`].
    pub const fn with_default_capacity(file_prefix: &'static str) -> Self {
        Self::new(file_prefix, DEFAULT_FAILURE_CAPACITY)
    }

    /// Return the on-disk log file path for this process.
    pub fn log_path(&self) -> PathBuf {
        let pid = std::process::id();
        std::env::temp_dir().join(format!("{}_{pid}.log", self.file_prefix))
    }

    /// Initialize the log file if needed and return the file path.
    pub fn init(&self) -> Result<PathBuf, LogError> {
        let path = self.log_path();
        if self.file.get().is_none() {
            let file = open_log_file(&path)?;
            let _ = self.file.set(Mutex::new(file));
        }
        Ok(path)
    }

    /// Write a log line and flush it to disk for crash diagnostics.
    pub fn log_line(&self, message: &str) -> Result<(), LogError> {
        let _ = self.init()?;
        let file = self.file.get().ok_or(LogError::NotInitialized)?;
        let mut file = file.lock().map_err(|_| LogError::Poisoned)?;
        let stamp = timestamp_ms();
        writeln!(file, "[{stamp}] {message}")?;
        file.flush()?;
        Ok(())
    }

    /// Write a log line and convert failures into bounded in-memory diagnostics.
    pub fn log_line_safe(&self, context: &str, message: &str) {
        if let Err(err) = self.log_line(message) {
            self.record_failure(context, err);
        }
    }

    /// Record a logging failure without panicking callers.
    fn record_failure(&self, context: &str, err: LogError) {
        let entry = format!("{context}: {err}");
        let errors = self.errors.get_or_init(|| Mutex::new(VecDeque::new()));
        if let Ok(mut guard) = errors.lock() {
            push_failure_entry(&mut guard, entry, self.failure_capacity);
        }
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
