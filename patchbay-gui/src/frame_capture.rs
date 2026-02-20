//! Test-oriented frame capture primitives for hosted windows.
//!
//! The capture flow is request/response based:
//! - host-side code requests a specific next frame id
//! - the render loop fulfills that id with either pixels or an error
//! - waiters block on a condition variable until completion or timeout

#[cfg(target_os = "windows")]
use std::sync::{Condvar, Mutex};
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

/// RGBA8 pixels captured from one rendered hosted-window frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedWindowFrame {
    /// Captured width in pixels.
    pub width: u32,
    /// Captured height in pixels.
    pub height: u32,
    /// RGBA pixels in row-major order with top-left origin.
    pub pixels: Vec<u8>,
}

/// Shared frame-capture request/result state.
#[cfg(target_os = "windows")]
#[derive(Debug, Default)]
pub(crate) struct FrameCaptureState {
    /// Protected request/result payload.
    inner: Mutex<FrameCaptureInner>,
    /// Condition variable notified when a request completes.
    ready: Condvar,
}

/// Internal request/result payload protected by one mutex.
#[cfg(target_os = "windows")]
#[derive(Debug, Default)]
struct FrameCaptureInner {
    /// Monotonic id assigned to the most recent request.
    next_request_id: u64,
    /// Monotonic id of the most recently completed request.
    completed_request_id: u64,
    /// Completion payload for the most recently completed request.
    completed_result: Option<Result<CapturedWindowFrame, String>>,
}

/// Result of waiting for one requested capture completion.
#[cfg(target_os = "windows")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FrameCaptureWaitError {
    /// Capture did not complete before timeout.
    Timeout,
    /// Internal synchronization was poisoned.
    Poisoned,
    /// Capture completed without producing a payload.
    MissingResult,
}

#[cfg(target_os = "windows")]
impl FrameCaptureState {
    /// Register a new capture request and return its request id.
    pub(crate) fn begin_request(&self) -> Result<u64, FrameCaptureWaitError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| FrameCaptureWaitError::Poisoned)?;
        inner.next_request_id = inner.next_request_id.saturating_add(1);
        inner.completed_result = None;
        Ok(inner.next_request_id)
    }

    /// Return the pending request id when a request is awaiting completion.
    pub(crate) fn pending_request_id(&self) -> Option<u64> {
        let Ok(inner) = self.inner.lock() else {
            return None;
        };
        if inner.next_request_id > inner.completed_request_id {
            Some(inner.next_request_id)
        } else {
            None
        }
    }

    /// Mark one request as completed and publish its result.
    pub(crate) fn complete_request(
        &self,
        request_id: u64,
        result: Result<CapturedWindowFrame, String>,
    ) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        if request_id < inner.completed_request_id {
            return;
        }
        inner.completed_request_id = request_id;
        inner.completed_result = Some(result);
        self.ready.notify_all();
    }

    /// Block until `request_id` completes or `timeout` elapses.
    pub(crate) fn wait_for_request(
        &self,
        request_id: u64,
        timeout: Duration,
    ) -> Result<Result<CapturedWindowFrame, String>, FrameCaptureWaitError> {
        let deadline = Instant::now() + timeout;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| FrameCaptureWaitError::Poisoned)?;
        while inner.completed_request_id < request_id {
            let now = Instant::now();
            if now >= deadline {
                return Err(FrameCaptureWaitError::Timeout);
            }
            let remaining = deadline.saturating_duration_since(now);
            let (guard, timed_out) = self
                .ready
                .wait_timeout(inner, remaining)
                .map_err(|_| FrameCaptureWaitError::Poisoned)
                .map(|pair| (pair.0, pair.1.timed_out()))?;
            inner = guard;
            if timed_out && inner.completed_request_id < request_id {
                return Err(FrameCaptureWaitError::Timeout);
            }
        }
        inner
            .completed_result
            .clone()
            .ok_or(FrameCaptureWaitError::MissingResult)
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::{CapturedWindowFrame, FrameCaptureState, FrameCaptureWaitError};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn request_wait_roundtrips_success() {
        let state = Arc::new(FrameCaptureState::default());
        let request = state.begin_request().expect("request id");
        let worker = Arc::clone(&state);
        let handle = thread::spawn(move || {
            worker.complete_request(
                request,
                Ok(CapturedWindowFrame {
                    width: 2,
                    height: 1,
                    pixels: vec![1, 2, 3, 4, 5, 6, 7, 8],
                }),
            );
        });
        let got = state
            .wait_for_request(request, Duration::from_millis(500))
            .expect("wait should not fail")
            .expect("capture should succeed");
        handle.join().expect("worker should join");
        assert_eq!(got.width, 2);
        assert_eq!(got.height, 1);
        assert_eq!(got.pixels.len(), 8);
    }

    #[test]
    fn request_wait_times_out_without_completion() {
        let state = FrameCaptureState::default();
        let request = state.begin_request().expect("request id");
        let err = state
            .wait_for_request(request, Duration::from_millis(10))
            .expect_err("wait should time out");
        assert_eq!(err, FrameCaptureWaitError::Timeout);
    }
}
