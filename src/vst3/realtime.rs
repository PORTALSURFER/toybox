//! Realtime-safe control-to-audio runtime and coherent-state handoff.
//!
//! The runtime API keeps every raw ownership transition inside Toybox. Control
//! threads register a revision, fully construct a replacement, and publish it.
//! The serialized audio callback may adopt at most one candidate at a block
//! boundary. Rejected and displaced values are never destroyed there: they are
//! moved into a deferred batch for a control thread to reclaim.
//!
//! The state API is a bounded seqlock-style reader around caller-owned atomic
//! fields. Validation and writer serialization happen on control threads. The
//! audio callback performs at most two generation loads and one snapshot read,
//! keeping its previous coherent snapshot whenever a write overlaps the block
//! boundary.
//!
//! # Runtime replacement example
//!
//! ```
//! use toybox::vst3::{RuntimeAdoption, RuntimePublisher};
//!
//! struct Runtime {
//!     sample_rate: f32,
//! }
//!
//! let (control, mut audio) = RuntimePublisher::new(Runtime {
//!     sample_rate: 48_000.0,
//! });
//! let registration = control.register().expect("revision space");
//! registration.publish(Runtime {
//!     sample_rate: 96_000.0,
//! });
//!
//! let adoption = audio.try_adopt(|current, candidate| {
//!     current.sample_rate != candidate.sample_rate
//! });
//! assert!(matches!(adoption, RuntimeAdoption::Adopted { .. }));
//! assert_eq!(audio.current().sample_rate, 96_000.0);
//!
//! // Reclamation is a control-thread operation.
//! assert_eq!(control.reclaim(), 1);
//! ```

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering, fence};
use std::sync::{Arc, Mutex};

/// Monotonic identity assigned when a control-side runtime build is registered.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuntimeRevision(u64);

impl RuntimeRevision {
    /// Revision owned by the initial audio runtime.
    pub const INITIAL: Self = Self(0);

    /// Returns the numeric revision.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Error returned when no further monotonic runtime revision can be assigned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeRevisionExhausted;

impl fmt::Display for RuntimeRevisionExhausted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("realtime runtime revision space exhausted")
    }
}

impl Error for RuntimeRevisionExhausted {}

/// Reason a pending runtime was retained as deferred garbage instead of adopted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeRejection {
    /// A greater revision was registered before this candidate reached audio.
    Stale,
    /// The plugin's audio-side predicate chose to preserve the current runtime.
    Redundant,
}

/// Result of one bounded adoption attempt at an audio block boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdoption {
    /// No published candidate was waiting.
    NoCandidate,
    /// The candidate replaced the audio-owned current runtime.
    Adopted {
        /// Revision of the newly current runtime.
        revision: RuntimeRevision,
    },
    /// The candidate was moved to deferred retirement without becoming current.
    Rejected {
        /// Revision carried by the rejected candidate.
        revision: RuntimeRevision,
        /// Why the candidate was rejected.
        reason: RuntimeRejection,
    },
}

/// Intrusive node owned by exactly one runtime lifecycle state.
struct RuntimeNode<T> {
    /// Monotonic publication identity.
    revision: RuntimeRevision,
    /// Plugin-owned runtime value.
    value: T,
    /// Next node while this value belongs to a deferred-retire list.
    retired_next: *mut RuntimeNode<T>,
}

impl<T> RuntimeNode<T> {
    /// Allocates a node on a control thread or during initial construction.
    fn boxed(revision: RuntimeRevision, value: T) -> *mut Self {
        Box::into_raw(Box::new(Self {
            revision,
            value,
            retired_next: ptr::null_mut(),
        }))
    }
}

/// Atomics shared by all control publishers and the single audio owner.
struct RuntimeShared<T> {
    /// Latest candidate mailbox; ownership transfers through atomic exchange.
    pending: AtomicPtr<RuntimeNode<T>>,
    /// One published batch of values awaiting control-thread reclamation.
    retired: AtomicPtr<RuntimeNode<T>>,
    /// Greatest registered revision, including builds not yet published.
    latest_revision: AtomicU64,
}

// SAFETY: every `RuntimeNode<T>` has exclusive ownership in one of the
// control-owned, pending, audio-owned, audio-local-retired, or shared-retired
// states. Crossing threads moves ownership through an atomic operation. No
// thread shares a reference to `T`, so `T: Send` is sufficient.
unsafe impl<T: Send> Send for RuntimeShared<T> {}

// SAFETY: the same ownership-state invariant makes concurrent publisher,
// adoption, and reclamation operations safe. Only atomics are shared directly.
unsafe impl<T: Send> Sync for RuntimeShared<T> {}

impl<T> RuntimeShared<T> {
    /// Registers the next revision without wrapping the monotonic counter.
    fn register_revision(&self) -> Result<RuntimeRevision, RuntimeRevisionExhausted> {
        let mut observed = self.latest_revision.load(Ordering::Acquire);
        loop {
            let Some(next) = observed.checked_add(1) else {
                return Err(RuntimeRevisionExhausted);
            };
            match self.latest_revision.compare_exchange_weak(
                observed,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(RuntimeRevision(next)),
                Err(current) => observed = current,
            }
        }
    }

    /// Reconciles one fully built candidate with overlapping control publishers.
    fn publish(&self, revision: RuntimeRevision, value: T) {
        let mut candidate = RuntimeNode::boxed(revision, value);
        let mut candidate_revision = revision;

        loop {
            let displaced = self.pending.swap(candidate, Ordering::AcqRel);
            if displaced.is_null() {
                return;
            }

            // SAFETY: the exchange transferred exclusive ownership of
            // `displaced` to this control call. The currently published
            // `candidate` is not dereferenced here.
            let displaced_revision = unsafe { (*displaced).revision };
            if displaced_revision <= candidate_revision {
                // SAFETY: `displaced` is exclusively control-owned after the
                // exchange and was allocated by `RuntimeNode::boxed`.
                unsafe { drop_runtime_node(displaced) };
                return;
            }

            // A newer candidate was displaced by this older publication.
            // Re-publish it. If audio consumes the temporarily visible older
            // value, its revision check rejects it without destruction.
            candidate = displaced;
            candidate_revision = displaced_revision;
        }
    }

    /// Drops one chain after exclusive ownership reaches a control thread.
    fn reclaim_chain(&self, mut node: *mut RuntimeNode<T>) -> usize {
        let mut reclaimed = 0usize;
        while !node.is_null() {
            // SAFETY: the caller transferred exclusive ownership of the whole
            // list to this control call. Every link was written before the
            // list was released through `retired`.
            let next = unsafe { (*node).retired_next };
            // SAFETY: this node is exclusively owned and originated from
            // `RuntimeNode::boxed`.
            unsafe { drop_runtime_node(node) };
            reclaimed = reclaimed.saturating_add(1);
            node = next;
        }
        reclaimed
    }
}

impl<T> Drop for RuntimeShared<T> {
    fn drop(&mut self) {
        // `RuntimeShared` can be destroyed only after every publisher and the
        // audio owner released their `Arc`. Therefore no atomic peer remains,
        // and direct mutable access transfers exclusive ownership here.
        let pending = *self.pending.get_mut();
        let retired = *self.retired.get_mut();
        let _ = self.reclaim_chain(pending);
        let _ = self.reclaim_chain(retired);
    }
}

/// Drops one exclusively owned runtime node.
///
/// # Safety
///
/// `node` must be non-null, originate from [`RuntimeNode::boxed`], and no
/// longer be reachable by another ownership state.
unsafe fn drop_runtime_node<T>(node: *mut RuntimeNode<T>) {
    // SAFETY: the caller establishes the allocation and exclusive-ownership
    // requirements documented above.
    drop(unsafe { Box::from_raw(node) });
}

/// Cloneable control-side handle for runtime registration and reclamation.
pub struct RuntimePublisher<T>
where
    T: Send + 'static,
{
    /// Shared atomic lifecycle state.
    shared: Arc<RuntimeShared<T>>,
}

impl<T> Clone for RuntimePublisher<T>
where
    T: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> RuntimePublisher<T>
where
    T: Send + 'static,
{
    /// Creates paired control and audio owners around a fully built initial runtime.
    ///
    /// The returned [`AudioRuntime`] must not be dropped until host processing
    /// has stopped. Publishers may be dropped earlier because the audio owner
    /// retains the shared atomics.
    pub fn new(initial: T) -> (Self, AudioRuntime<T>) {
        let shared = Arc::new(RuntimeShared {
            pending: AtomicPtr::new(ptr::null_mut()),
            retired: AtomicPtr::new(ptr::null_mut()),
            latest_revision: AtomicU64::new(RuntimeRevision::INITIAL.get()),
        });
        let current = RuntimeNode::boxed(RuntimeRevision::INITIAL, initial);
        (
            Self {
                shared: Arc::clone(&shared),
            },
            AudioRuntime {
                shared,
                current,
                local_retired: ptr::null_mut(),
                not_sync: PhantomData,
            },
        )
    }

    /// Registers a monotonic revision before the caller constructs its runtime.
    ///
    /// Registering first makes overlapping setup calls latest-wins even when an
    /// older runtime takes longer to construct. Dropping the returned token does
    /// not roll the revision back because another publisher may already have
    /// observed it.
    pub fn register(&self) -> Result<RuntimeRegistration<T>, RuntimeRevisionExhausted> {
        Ok(RuntimeRegistration {
            shared: Arc::clone(&self.shared),
            revision: self.shared.register_revision()?,
        })
    }

    /// Returns the greatest revision registered by any publisher.
    pub fn latest_revision(&self) -> RuntimeRevision {
        RuntimeRevision(self.shared.latest_revision.load(Ordering::Acquire))
    }

    /// Reclaims the currently published deferred batch on this control thread.
    ///
    /// A value retained in the audio owner's local batch may require another
    /// audio boundary before it becomes visible here. Call this periodically
    /// from setup/control work and once more after processing has stopped.
    pub fn reclaim(&self) -> usize {
        let retired = self.shared.retired.swap(ptr::null_mut(), Ordering::AcqRel);
        self.shared.reclaim_chain(retired)
    }

    /// Discards the currently pending candidate on this control thread.
    ///
    /// This is useful during deactivation or teardown. It may safely race an
    /// audio adoption attempt: exactly one side receives ownership. Concurrent
    /// publishers should already be quiesced when teardown semantics require
    /// every pending build to be discarded.
    pub fn discard_pending(&self) -> bool {
        let pending = self.shared.pending.swap(ptr::null_mut(), Ordering::AcqRel);
        if pending.is_null() {
            false
        } else {
            // SAFETY: the exchange transferred exclusive ownership to this
            // control call, and pending nodes originate from `boxed`.
            unsafe { drop_runtime_node(pending) };
            true
        }
    }
}

/// Registered control-side build token tied to one runtime handoff.
#[must_use = "publish the completed runtime or intentionally abandon this registered revision"]
pub struct RuntimeRegistration<T>
where
    T: Send + 'static,
{
    /// Shared lifecycle state tied to this registration.
    shared: Arc<RuntimeShared<T>>,
    /// Revision reserved before runtime construction.
    revision: RuntimeRevision,
}

impl<T> RuntimeRegistration<T>
where
    T: Send + 'static,
{
    /// Returns the revision assigned before construction began.
    pub const fn revision(&self) -> RuntimeRevision {
        self.revision
    }

    /// Publishes a fully constructed runtime and consumes this registration.
    ///
    /// Allocation of Toybox's private ownership node occurs in this control-side
    /// call. No allocation is performed when audio later attempts adoption.
    pub fn publish(self, runtime: T) -> RuntimeRevision {
        let revision = self.revision;
        self.shared.publish(revision, runtime);
        revision
    }
}

/// Single-callback audio ownership of the current runtime and local retire list.
///
/// This type is `Send` but deliberately not `Sync`. Keep it behind the plugin's
/// serialized process-callback boundary and call [`Self::try_adopt`] once per
/// block. Dropping it destroys the current and locally retired runtimes, so its
/// owner must first ensure host processing has stopped. That post-processing
/// destruction is the only non-control path on which `T::drop` may run.
pub struct AudioRuntime<T>
where
    T: Send + 'static,
{
    /// Shared pending and deferred-retire atomics.
    shared: Arc<RuntimeShared<T>>,
    /// Non-null runtime exclusively owned by the serialized audio callback.
    current: *mut RuntimeNode<T>,
    /// Audio-local retired list awaiting a free shared retire slot.
    local_retired: *mut RuntimeNode<T>,
    /// Prevents accidental `Sync` while preserving movable ownership.
    not_sync: PhantomData<Cell<()>>,
}

// SAFETY: moving the whole audio owner to another thread moves exclusive
// ownership of its current and local-retired values. `T: Send` permits that.
unsafe impl<T: Send> Send for AudioRuntime<T> {}

impl<T> AudioRuntime<T>
where
    T: Send + 'static,
{
    /// Returns the current runtime revision.
    pub fn current_revision(&self) -> RuntimeRevision {
        // SAFETY: `current` is non-null and exclusively owned for the complete
        // lifetime of `AudioRuntime`.
        unsafe { (*self.current).revision }
    }

    /// Borrows the current runtime for audio processing.
    pub fn current(&self) -> &T {
        // SAFETY: `current` is non-null and owned by `self`; the shared borrow
        // prevents replacement for the returned reference's lifetime.
        unsafe { &(*self.current).value }
    }

    /// Mutably borrows the current runtime for audio processing.
    pub fn current_mut(&mut self) -> &mut T {
        // SAFETY: `current` is non-null and owned by `self`; `&mut self`
        // guarantees exclusive access to the value.
        unsafe { &mut (*self.current).value }
    }

    /// Attempts to adopt one pending runtime at a block boundary.
    ///
    /// Toybox performs a bounded sequence: one best-effort retire publication,
    /// one pending exchange, one revision load, and one final best-effort retire
    /// publication. No loop, lock, allocation, or `T::drop` occurs here. The
    /// predicate must itself remain realtime-safe and should return `false` for
    /// plugin-specific redundant replacements whose current tail must survive.
    pub fn try_adopt(&mut self, accept: impl FnOnce(&T, &T) -> bool) -> RuntimeAdoption {
        self.try_publish_retired();
        let candidate = self.shared.pending.swap(ptr::null_mut(), Ordering::AcqRel);
        if candidate.is_null() {
            return RuntimeAdoption::NoCandidate;
        }

        let mut candidate = AudioCandidate {
            owner: self,
            node: candidate,
        };
        let revision = candidate.revision();
        let latest = RuntimeRevision(
            candidate
                .owner
                .shared
                .latest_revision
                .load(Ordering::Acquire),
        );
        if revision != latest {
            return RuntimeAdoption::Rejected {
                revision,
                reason: RuntimeRejection::Stale,
            };
        }

        if !accept(candidate.owner.current(), candidate.value()) {
            return RuntimeAdoption::Rejected {
                revision,
                reason: RuntimeRejection::Redundant,
            };
        }

        let replacement = candidate.take();
        let displaced = std::mem::replace(&mut candidate.owner.current, replacement);
        candidate.owner.push_local_retired(displaced);
        candidate.owner.try_publish_retired();
        RuntimeAdoption::Adopted { revision }
    }

    /// Pushes one exclusively audio-owned node onto the local retire list.
    fn push_local_retired(&mut self, node: *mut RuntimeNode<T>) {
        debug_assert!(!node.is_null());
        // SAFETY: `node` is exclusively audio-owned, so its intrusive link may
        // be updated before the list is made visible to a control thread.
        unsafe { (*node).retired_next = self.local_retired };
        self.local_retired = node;
    }

    /// Publishes the local list with one bounded compare-exchange attempt.
    fn try_publish_retired(&mut self) {
        if self.local_retired.is_null() {
            return;
        }
        if self
            .shared
            .retired
            .compare_exchange(
                ptr::null_mut(),
                self.local_retired,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            // After the successful release, control may immediately reclaim
            // the list. Do not dereference the old local pointer again.
            self.local_retired = ptr::null_mut();
        }
    }
}

impl<T> Drop for AudioRuntime<T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        // Lifecycle contract: the owner stops processing before dropping this
        // type, so direct destruction here cannot occur in an audio callback.
        // SAFETY: `current` is non-null, exclusively owned, and boxed.
        unsafe { drop_runtime_node(self.current) };
        self.current = ptr::null_mut();

        let local_retired = std::mem::replace(&mut self.local_retired, ptr::null_mut());
        let _ = self.shared.reclaim_chain(local_retired);
    }
}

/// Panic-safe temporary ownership of a candidate taken by audio.
struct AudioCandidate<'a, T>
where
    T: Send + 'static,
{
    /// Audio owner that receives the node if adoption does not complete.
    owner: &'a mut AudioRuntime<T>,
    /// Candidate node, or null after ownership becomes current.
    node: *mut RuntimeNode<T>,
}

impl<T> AudioCandidate<'_, T>
where
    T: Send + 'static,
{
    /// Returns the candidate revision.
    fn revision(&self) -> RuntimeRevision {
        // SAFETY: the guard owns a non-null candidate until `take`.
        unsafe { (*self.node).revision }
    }

    /// Borrows the candidate value.
    fn value(&self) -> &T {
        // SAFETY: the guard owns a non-null candidate for this borrow.
        unsafe { &(*self.node).value }
    }

    /// Transfers candidate ownership out of the guard.
    fn take(&mut self) -> *mut RuntimeNode<T> {
        std::mem::replace(&mut self.node, ptr::null_mut())
    }
}

impl<T> Drop for AudioCandidate<'_, T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        if !self.node.is_null() {
            self.owner.push_local_retired(self.node);
            self.node = ptr::null_mut();
            self.owner.try_publish_retired();
        }
    }
}

/// Completed identity for one coherent state publication.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateGeneration(u64);

impl StateGeneration {
    /// Generation of the initial audio snapshot.
    pub const INITIAL: Self = Self(0);

    /// Returns the numeric completed generation.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Failure from validation or control-side state publication.
#[derive(Debug, Eq, PartialEq)]
pub enum StatePublishError<E> {
    /// Payload validation failed before the generation entered update state.
    Invalid(E),
    /// A previous writer panicked while applying fields; audio remains on its old snapshot.
    Poisoned,
    /// No later even generation can be represented without wrapping.
    GenerationExhausted,
}

impl<E> fmt::Display for StatePublishError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(error) => write!(formatter, "invalid state payload: {error}"),
            Self::Poisoned => formatter.write_str("coherent state publisher is poisoned"),
            Self::GenerationExhausted => {
                formatter.write_str("coherent state generation space exhausted")
            }
        }
    }
}

impl<E> Error for StatePublishError<E> where E: Error + 'static {}

/// Generation gate and control-writer serialization shared with audio.
struct CoherentStateShared {
    /// Even values are complete; odd values indicate an update in progress.
    generation: AtomicU64,
    /// Serializes control writers without ever being acquired by audio.
    writer: Mutex<()>,
}

/// Cloneable control-side publisher for coherent multi-field state updates.
#[derive(Clone)]
pub struct CoherentStatePublisher {
    /// Shared generation gate and writer lock.
    shared: Arc<CoherentStateShared>,
}

impl CoherentStatePublisher {
    /// Creates paired control and audio state endpoints from a coherent initial snapshot.
    ///
    /// `T: Copy` ensures replacing or returning snapshots cannot invoke a
    /// destructor on the audio thread.
    pub fn new<T>(initial: T) -> (Self, AudioStateSnapshot<T>)
    where
        T: Copy,
    {
        let shared = Arc::new(CoherentStateShared {
            generation: AtomicU64::new(StateGeneration::INITIAL.get()),
            writer: Mutex::new(()),
        });
        (
            Self {
                shared: Arc::clone(&shared),
            },
            AudioStateSnapshot {
                shared,
                current: initial,
                generation: StateGeneration::INITIAL,
            },
        )
    }

    /// Validates a payload, then serializes and publishes its multi-field update.
    ///
    /// Validation completes before the writer lock is acquired and before the
    /// generation becomes odd. `apply` should update caller-owned thread-safe
    /// fields and must not panic. If it does panic, the mutex is poisoned and
    /// the generation deliberately remains odd so audio never adopts a partial
    /// snapshot.
    pub fn validate_and_publish<P, V, E>(
        &self,
        payload: P,
        validate: impl FnOnce(P) -> Result<V, E>,
        apply: impl FnOnce(V),
    ) -> Result<StateGeneration, StatePublishError<E>> {
        let validated = validate(payload).map_err(StatePublishError::Invalid)?;
        let _writer = self
            .shared
            .writer
            .lock()
            .map_err(|_| StatePublishError::Poisoned)?;
        let before = self.shared.generation.load(Ordering::Acquire);
        if before & 1 != 0 {
            return Err(StatePublishError::Poisoned);
        }
        let Some(in_progress) = before.checked_add(1) else {
            return Err(StatePublishError::GenerationExhausted);
        };
        let Some(complete) = in_progress.checked_add(1) else {
            return Err(StatePublishError::GenerationExhausted);
        };

        // Acquire prevents the following field stores from moving before the
        // odd generation; release publishes the preceding writer-lock state.
        if self
            .shared
            .generation
            .compare_exchange(before, in_progress, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(StatePublishError::Poisoned);
        }

        apply(validated);
        self.shared.generation.store(complete, Ordering::Release);
        Ok(StateGeneration(complete))
    }

    /// Returns the completed generation, or `None` while a writer is active.
    pub fn completed_generation(&self) -> Option<StateGeneration> {
        let generation = self.shared.generation.load(Ordering::Acquire);
        (generation & 1 == 0).then_some(StateGeneration(generation))
    }
}

/// Cached coherent state snapshot owned by the serialized audio callback.
pub struct AudioStateSnapshot<T>
where
    T: Copy,
{
    /// Shared generation gate; audio never acquires its writer mutex.
    shared: Arc<CoherentStateShared>,
    /// Last snapshot proven coherent by equal even generation reads.
    current: T,
    /// Completed generation associated with `current`.
    generation: StateGeneration,
}

impl<T> AudioStateSnapshot<T>
where
    T: Copy,
{
    /// Observes state once at an audio block boundary without waiting or retrying.
    ///
    /// `read` should copy caller-owned atomic fields into `T`. If a writer is
    /// active before the read, `read` is skipped. If a writer overlaps the read,
    /// the candidate is discarded and the previous coherent snapshot is
    /// returned. The closure itself remains responsible for realtime safety.
    pub fn observe(&mut self, read: impl FnOnce() -> T) -> StateObservation<T> {
        let before = self.shared.generation.load(Ordering::Acquire);
        if before & 1 != 0 {
            return self.observation(false);
        }

        let candidate = read();
        // Keep every field read before the closing generation check. Together
        // with the opening acquire and writer's odd/even publication, this
        // full fence prevents a relaxed field load from escaping the checked
        // interval on weakly ordered targets.
        fence(Ordering::SeqCst);
        let after = self.shared.generation.load(Ordering::Acquire);
        if before != after || after & 1 != 0 {
            return self.observation(false);
        }

        let generation = StateGeneration(after);
        let changed = generation != self.generation;
        self.current = candidate;
        self.generation = generation;
        self.observation(changed)
    }

    /// Returns the current cached snapshot without reading caller-owned fields.
    pub const fn current(&self) -> T {
        self.current
    }

    /// Returns the completed generation associated with the cached snapshot.
    pub const fn generation(&self) -> StateGeneration {
        self.generation
    }

    /// Constructs an observation from the current cached state.
    fn observation(&self, changed: bool) -> StateObservation<T> {
        StateObservation {
            snapshot: self.current,
            generation: self.generation,
            changed,
        }
    }
}

/// Result of one bounded coherent-state observation at a block boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StateObservation<T>
where
    T: Copy,
{
    /// Previous or newly proven coherent snapshot.
    snapshot: T,
    /// Completed generation associated with the returned snapshot.
    generation: StateGeneration,
    /// Whether this observation first adopted a newly completed generation.
    changed: bool,
}

impl<T> StateObservation<T>
where
    T: Copy,
{
    /// Returns the coherent snapshot for this block.
    pub const fn snapshot(self) -> T {
        self.snapshot
    }

    /// Returns the completed generation associated with the snapshot.
    pub const fn generation(self) -> StateGeneration {
        self.generation
    }

    /// Returns whether a new completed generation was first observed now.
    ///
    /// Plugins can use this edge to apply their own DSP reset or tail policy.
    pub const fn changed(self) -> bool {
        self.changed
    }
}

#[cfg(test)]
mod tests {
    //! Deterministic ownership and concurrency coverage.

    use std::convert::Infallible;
    use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;

    use super::{
        CoherentStatePublisher, RuntimeAdoption, RuntimePublisher, RuntimeRejection,
        StateGeneration, StatePublishError,
    };
    use crate::test_alloc::assert_realtime_safe;

    /// Runtime that reports every destructor invocation.
    struct DropProbe {
        /// Test identity retained to make replacement assertions readable.
        id: u32,
        /// Shared destructor count.
        drops: Arc<AtomicUsize>,
    }

    impl DropProbe {
        /// Creates a counted runtime value.
        fn new(id: u32, drops: &Arc<AtomicUsize>) -> Self {
            Self {
                id,
                drops: Arc::clone(drops),
            }
        }
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Copy snapshot representing two fields that must change together.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Pair {
        /// First field.
        left: u32,
        /// Second field.
        right: u32,
    }

    #[test]
    fn adoption_and_rejection_never_drop_on_audio() {
        let drops = Arc::new(AtomicUsize::new(0));
        let (control, mut audio) = RuntimePublisher::new(DropProbe::new(0, &drops));

        control
            .register()
            .expect("revision")
            .publish(DropProbe::new(1, &drops));
        let adopted = assert_realtime_safe(|| audio.try_adopt(|_, _| true));
        assert!(matches!(adopted, RuntimeAdoption::Adopted { .. }));
        assert_eq!(audio.current().id, 1);
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        assert_eq!(control.reclaim(), 1);
        assert_eq!(drops.load(Ordering::Relaxed), 1);

        control
            .register()
            .expect("revision")
            .publish(DropProbe::new(2, &drops));
        let rejected = assert_realtime_safe(|| audio.try_adopt(|_, _| false));
        assert!(matches!(
            rejected,
            RuntimeAdoption::Rejected {
                reason: RuntimeRejection::Redundant,
                ..
            }
        ));
        assert_eq!(audio.current().id, 1);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert_eq!(control.reclaim(), 1);
        assert_eq!(drops.load(Ordering::Relaxed), 2);

        drop(audio);
        assert_eq!(drops.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn stale_overlapping_publishers_cannot_supersede_the_greatest_revision() {
        let drops = Arc::new(AtomicUsize::new(0));
        let (control, mut audio) = RuntimePublisher::new(DropProbe::new(0, &drops));
        let older = control.register().expect("older revision");
        let newer = control.register().expect("newer revision");
        let newer_revision = newer.revision();

        newer.publish(DropProbe::new(2, &drops));
        older.publish(DropProbe::new(1, &drops));
        assert_eq!(drops.load(Ordering::Relaxed), 1);

        assert_eq!(
            audio.try_adopt(|_, _| true),
            RuntimeAdoption::Adopted {
                revision: newer_revision
            }
        );
        assert_eq!(audio.current().id, 2);
        assert_eq!(control.latest_revision(), newer_revision);
        assert_eq!(control.reclaim(), 1);
    }

    #[test]
    fn registered_newer_build_makes_a_temporarily_visible_older_build_stale() {
        let drops = Arc::new(AtomicUsize::new(0));
        let (control, mut audio) = RuntimePublisher::new(DropProbe::new(0, &drops));
        let older = control.register().expect("older revision");
        let newer = control.register().expect("newer revision");
        let older_revision = older.revision();
        let newer_revision = newer.revision();

        older.publish(DropProbe::new(1, &drops));
        assert_eq!(
            audio.try_adopt(|_, _| true),
            RuntimeAdoption::Rejected {
                revision: older_revision,
                reason: RuntimeRejection::Stale,
            }
        );
        assert_eq!(audio.current().id, 0);

        newer.publish(DropProbe::new(2, &drops));
        assert_eq!(
            audio.try_adopt(|_, _| true),
            RuntimeAdoption::Adopted {
                revision: newer_revision,
            }
        );
        assert_eq!(audio.current().id, 2);
        assert_eq!(control.reclaim(), 1);
        assert_eq!(audio.try_adopt(|_, _| true), RuntimeAdoption::NoCandidate);
        assert_eq!(control.reclaim(), 1);
    }

    #[test]
    fn occupied_retire_slot_keeps_later_values_audio_local_until_next_boundary() {
        let drops = Arc::new(AtomicUsize::new(0));
        let (control, mut audio) = RuntimePublisher::new(DropProbe::new(0, &drops));

        control
            .register()
            .expect("first revision")
            .publish(DropProbe::new(1, &drops));
        let _ = audio.try_adopt(|_, _| true);
        control
            .register()
            .expect("second revision")
            .publish(DropProbe::new(2, &drops));
        let _ = audio.try_adopt(|_, _| true);

        assert_eq!(drops.load(Ordering::Relaxed), 0);
        assert_eq!(control.reclaim(), 1);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert_eq!(audio.try_adopt(|_, _| true), RuntimeAdoption::NoCandidate);
        assert_eq!(control.reclaim(), 1);
        assert_eq!(drops.load(Ordering::Relaxed), 2);
        assert_eq!(audio.current().id, 2);
    }

    #[test]
    fn deterministic_overlapping_control_threads_keep_the_newer_runtime() {
        let drops = Arc::new(AtomicUsize::new(0));
        let (control, mut audio) = RuntimePublisher::new(DropProbe::new(0, &drops));
        let older = control.register().expect("older revision");
        let newer = control.register().expect("newer revision");
        let newer_revision = newer.revision();
        let start = Arc::new(Barrier::new(3));
        let finish_newer = Arc::new(Barrier::new(2));

        let old_start = Arc::clone(&start);
        let old_finish_newer = Arc::clone(&finish_newer);
        let old_drops = Arc::clone(&drops);
        let old_thread = thread::spawn(move || {
            old_start.wait();
            old_finish_newer.wait();
            older.publish(DropProbe::new(1, &old_drops));
        });
        let new_start = Arc::clone(&start);
        let new_finish = Arc::clone(&finish_newer);
        let new_drops = Arc::clone(&drops);
        let new_thread = thread::spawn(move || {
            new_start.wait();
            newer.publish(DropProbe::new(2, &new_drops));
            new_finish.wait();
        });

        start.wait();
        old_thread.join().expect("older publisher");
        new_thread.join().expect("newer publisher");
        assert_eq!(
            audio.try_adopt(|_, _| true),
            RuntimeAdoption::Adopted {
                revision: newer_revision,
            }
        );
        assert_eq!(audio.current().id, 2);
    }

    #[test]
    fn teardown_reclaims_current_pending_and_deferred_values_after_processing() {
        let drops = Arc::new(AtomicUsize::new(0));
        let (control, mut audio) = RuntimePublisher::new(DropProbe::new(0, &drops));
        control
            .register()
            .expect("adopted revision")
            .publish(DropProbe::new(1, &drops));
        let _ = audio.try_adopt(|_, _| true);
        control
            .register()
            .expect("pending revision")
            .publish(DropProbe::new(2, &drops));

        drop(control);
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        drop(audio);
        assert_eq!(drops.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn invalid_state_never_enters_the_generation_gate() {
        let (control, mut audio) = CoherentStatePublisher::new(Pair { left: 1, right: 1 });
        let applied = AtomicBool::new(false);
        let result = control.validate_and_publish(
            Pair { left: 2, right: 3 },
            |_| Err::<Pair, _>("mismatch"),
            |_| applied.store(true, Ordering::Relaxed),
        );

        assert_eq!(result, Err(StatePublishError::Invalid("mismatch")));
        assert!(!applied.load(Ordering::Relaxed));
        assert_eq!(
            control.completed_generation(),
            Some(StateGeneration::INITIAL)
        );
        let observed = audio.observe(|| Pair { left: 9, right: 9 });
        assert_eq!(observed.snapshot(), Pair { left: 9, right: 9 });
        assert!(!observed.changed());
    }

    #[test]
    fn audio_keeps_the_previous_snapshot_while_state_update_is_in_progress() {
        let left = Arc::new(AtomicU32::new(1));
        let right = Arc::new(AtomicU32::new(1));
        let initial = Pair { left: 1, right: 1 };
        let (control, mut audio) = CoherentStatePublisher::new(initial);
        let started = Arc::new(Barrier::new(2));
        let finish = Arc::new(Barrier::new(2));

        let writer_control = control.clone();
        let writer_left = Arc::clone(&left);
        let writer_right = Arc::clone(&right);
        let writer_started = Arc::clone(&started);
        let writer_finish = Arc::clone(&finish);
        let writer = thread::spawn(move || {
            writer_control
                .validate_and_publish(Pair { left: 2, right: 2 }, Ok::<_, Infallible>, |pair| {
                    writer_left.store(pair.left, Ordering::Relaxed);
                    writer_started.wait();
                    writer_finish.wait();
                    writer_right.store(pair.right, Ordering::Relaxed);
                })
                .expect("state publication")
        });

        started.wait();
        let read_called = AtomicBool::new(false);
        let during = assert_realtime_safe(|| {
            audio.observe(|| {
                read_called.store(true, Ordering::Relaxed);
                Pair {
                    left: left.load(Ordering::Relaxed),
                    right: right.load(Ordering::Relaxed),
                }
            })
        });
        assert!(!read_called.load(Ordering::Relaxed));
        assert_eq!(during.snapshot(), initial);
        assert!(!during.changed());

        finish.wait();
        let completed = writer.join().expect("writer thread");
        let after = assert_realtime_safe(|| {
            audio.observe(|| Pair {
                left: left.load(Ordering::Relaxed),
                right: right.load(Ordering::Relaxed),
            })
        });
        assert_eq!(after.snapshot(), Pair { left: 2, right: 2 });
        assert_eq!(after.generation(), completed);
        assert!(after.changed());

        let stable = audio.observe(|| Pair {
            left: left.load(Ordering::Relaxed),
            right: right.load(Ordering::Relaxed),
        });
        assert_eq!(stable.snapshot(), Pair { left: 2, right: 2 });
        assert!(!stable.changed());
    }

    #[test]
    fn overlapping_read_discards_its_candidate_without_retrying() {
        let left = AtomicU32::new(1);
        let right = AtomicU32::new(1);
        let (control, mut audio) = CoherentStatePublisher::new(Pair { left: 1, right: 1 });
        let reads = AtomicUsize::new(0);

        let observed = audio.observe(|| {
            reads.fetch_add(1, Ordering::Relaxed);
            let candidate = Pair {
                left: left.load(Ordering::Relaxed),
                right: right.load(Ordering::Relaxed),
            };
            control
                .validate_and_publish(Pair { left: 2, right: 2 }, Ok::<_, Infallible>, |pair| {
                    left.store(pair.left, Ordering::Relaxed);
                    right.store(pair.right, Ordering::Relaxed);
                })
                .expect("overlapping state publication");
            candidate
        });

        assert_eq!(reads.load(Ordering::Relaxed), 1);
        assert_eq!(observed.snapshot(), Pair { left: 1, right: 1 });
        assert!(!observed.changed());
        let next = audio.observe(|| Pair {
            left: left.load(Ordering::Relaxed),
            right: right.load(Ordering::Relaxed),
        });
        assert_eq!(next.snapshot(), Pair { left: 2, right: 2 });
        assert!(next.changed());
    }
}
