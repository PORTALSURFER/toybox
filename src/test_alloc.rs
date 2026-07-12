//! Thread-local allocator auditing for realtime regression tests.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    /// Whether the current test thread is auditing allocator calls.
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    /// Allocation/reallocation count on the current test thread.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    /// Deallocation count on the current test thread.
    static DEALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

/// Test allocator that can audit one thread without serializing the suite.
struct TrackingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(&ALLOCATIONS);
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(&ALLOCATIONS);
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record(&ALLOCATIONS);
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record(&DEALLOCATIONS);
        unsafe { System.dealloc(ptr, layout) }
    }
}

/// Record one allocator operation when tracking is enabled.
fn record(counter: &'static std::thread::LocalKey<Cell<usize>>) {
    TRACKING.with(|tracking| {
        if tracking.get() {
            counter.with(|count| count.set(count.get().saturating_add(1)));
        }
    });
}

/// Disable allocator tracking even if the audited operation panics.
struct TrackingGuard;

impl Drop for TrackingGuard {
    fn drop(&mut self) {
        TRACKING.with(|tracking| tracking.set(false));
    }
}

/// Assert that an operation neither allocates nor deallocates.
pub(crate) fn assert_realtime_safe<T>(operation: impl FnOnce() -> T) -> T {
    ALLOCATIONS.with(|count| count.set(0));
    DEALLOCATIONS.with(|count| count.set(0));
    TRACKING.with(|tracking| tracking.set(true));
    let guard = TrackingGuard;
    let result = operation();
    drop(guard);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0, "operation allocated");
    assert_eq!(DEALLOCATIONS.with(Cell::get), 0, "operation deallocated");
    result
}
