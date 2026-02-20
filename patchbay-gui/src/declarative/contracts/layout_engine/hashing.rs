/// Compute a stable hash from the debug representation of a value.
pub(crate) fn stable_debug_hash(value: &impl std::fmt::Debug) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hash as _;
    use std::hash::Hasher as _;
    struct HashSink<'a> {
        hasher: &'a mut std::collections::hash_map::DefaultHasher,
    }
    impl std::fmt::Write for HashSink<'_> {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            s.hash(self.hasher);
            Ok(())
        }
    }

    // Keep debug-shape semantics while avoiding one temporary allocation for
    // large trees and token payloads.
    let mut sink = HashSink {
        hasher: &mut hasher,
    };
    let _ = std::fmt::write(&mut sink, format_args!("{value:?}"));
    hasher.finish()
}
