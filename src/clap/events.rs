//! CLAP event batching and range helpers.

use std::ops::Bound;

use clack_plugin::events::io::{EventBatch, EventBatcher, InputEvents};

/// Provides CLAP input event batching and common range conversions.
pub struct EventRouter<'a> {
    input: &'a InputEvents<'a>,
}

impl<'a> EventRouter<'a> {
    /// Create a new event router for a CLAP input event list.
    pub fn new(input: &'a InputEvents<'a>) -> Self {
        Self { input }
    }

    /// Returns an iterator that batches input events by sample time.
    pub fn batches(&self) -> EventBatcher<'a> {
        self.input.batch()
    }

    /// Iterate all event batches and invoke the callback with a concrete sample range.
    ///
    /// The returned boolean is `true` if any batch was processed.
    pub fn for_each_batch<F>(&self, buffer_len: usize, mut f: F) -> bool
    where
        F: FnMut(EventBatch<'a>, Option<(usize, usize)>),
    {
        let mut processed_any = false;
        for batch in self.input.batch() {
            let range = bounds_to_range(batch.sample_bounds(), buffer_len);
            f(batch, range);
            processed_any = true;
        }
        processed_any
    }
}

/// Convert CLAP sample bounds into a concrete processing range.
pub fn bounds_to_range(
    bounds: (Bound<usize>, Bound<usize>),
    buffer_len: usize,
) -> Option<(usize, usize)> {
    if buffer_len == 0 {
        return None;
    }
    let (start, end) = match bounds {
        (Bound::Included(start), Bound::Excluded(end)) => (start, end),
        (Bound::Included(start), Bound::Included(end)) => (start, end.saturating_add(1)),
        (Bound::Unbounded, Bound::Excluded(end)) => (0, end),
        (Bound::Unbounded, Bound::Included(end)) => (0, end.saturating_add(1)),
        (Bound::Excluded(start), Bound::Excluded(end)) => {
            (start.saturating_add(1), end)
        }
        (Bound::Excluded(start), Bound::Included(end)) => {
            (start.saturating_add(1), end.saturating_add(1))
        }
        (Bound::Included(start), Bound::Unbounded) => (start, buffer_len),
        (Bound::Excluded(start), Bound::Unbounded) => (start.saturating_add(1), buffer_len),
        (Bound::Unbounded, Bound::Unbounded) => (0, buffer_len),
    };

    if start >= end || start >= buffer_len {
        None
    } else {
        Some((start, end.min(buffer_len)))
    }
}

#[cfg(test)]
mod tests {
    use super::bounds_to_range;
    use std::ops::Bound;

    #[test]
    fn bounds_to_range_handles_empty_buffer() {
        assert_eq!(None, bounds_to_range((Bound::Unbounded, Bound::Unbounded), 0));
    }

    #[test]
    fn bounds_to_range_clamps_to_buffer() {
        assert_eq!(
            Some((0, 4)),
            bounds_to_range((Bound::Unbounded, Bound::Unbounded), 4)
        );
        assert_eq!(
            Some((1, 4)),
            bounds_to_range((Bound::Included(1), Bound::Unbounded), 4)
        );
        assert_eq!(
            Some((1, 3)),
            bounds_to_range((Bound::Included(1), Bound::Excluded(10)), 3)
        );
    }

    #[test]
    fn bounds_to_range_skips_invalid() {
        assert_eq!(
            None,
            bounds_to_range((Bound::Included(4), Bound::Unbounded), 4)
        );
        assert_eq!(
            None,
            bounds_to_range((Bound::Included(3), Bound::Included(2)), 4)
        );
    }
}
