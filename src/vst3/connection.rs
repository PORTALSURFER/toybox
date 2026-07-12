//! Identity-safe VST3 processor/controller instance connections.

use std::ffi::c_void;
use std::ptr;
use std::slice;
use std::sync::{Arc, Mutex, RwLock};

#[cfg(test)]
use toybox_vst3_ffi::Steinberg::Vst::IConnectionPointTrait;
use toybox_vst3_ffi::Steinberg::Vst::{IConnectionPoint, IMessage};
use toybox_vst3_ffi::Steinberg::{
    FUnknown, FUnknownVtbl, TUID, kInvalidArgument, kResultFalse, kResultOk, tresult,
};
use toybox_vst3_ffi::com_scrape_types::{
    Construct, Guid, Header, Inherits, InterfaceList, SmartPtr, Unknown, Wrapper,
};
use toybox_vst3_ffi::{Class, ComRef, Interface};

/// Identifies which side of a VST3 component/controller connection owns canonical state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstanceConnectionRole {
    /// The audio processor owns canonical shared state.
    Processor,
    /// The edit controller adopts state from its host-connected processor.
    Controller,
}

/// Per-object VST3 connection state shared through the host's exact `IConnectionPoint` peer.
///
/// Each processor and controller is constructed independently. When the host connects their
/// `IConnectionPoint` interfaces, the controller adopts an `Arc` clone of that exact processor's
/// state. No process-global instance registry or creation-order inference is involved.
pub struct InstanceConnection<T> {
    /// Determines whether this endpoint publishes or adopts state.
    role: InstanceConnectionRole,
    /// Current state used by the plugin object.
    shared: RwLock<Arc<T>>,
    /// Address of the peer passed to this endpoint's successful callback.
    peer: Mutex<Option<usize>>,
}

impl<T> InstanceConnection<T>
where
    T: Send + Sync + 'static,
{
    /// Creates an unconnected processor or controller endpoint with initial state.
    pub fn new(role: InstanceConnectionRole, shared: Arc<T>) -> Self {
        Self {
            role,
            shared: RwLock::new(shared),
            peer: Mutex::new(None),
        }
    }

    /// Returns the endpoint's current shared state.
    ///
    /// A controller's value changes to its processor's state after a successful host connection.
    pub fn shared(&self) -> Arc<T> {
        self.shared.read().map_or_else(
            |poisoned| Arc::clone(&poisoned.into_inner()),
            |shared| Arc::clone(&shared),
        )
    }

    /// Returns whether this endpoint has accepted a host connection.
    pub fn is_connected(&self) -> bool {
        self.peer.lock().map_or_else(
            |poisoned| poisoned.into_inner().is_some(),
            |peer| peer.is_some(),
        )
    }

    #[doc(hidden)]
    pub unsafe fn connect(&self, other: *mut IConnectionPoint) -> tresult {
        let Some(other_ref) = (unsafe { ComRef::from_raw(other) }) else {
            return kInvalidArgument;
        };
        let Some(bridge) = other_ref.cast::<IToyboxSharedState>() else {
            return kResultFalse;
        };

        let result = match self.role {
            InstanceConnectionRole::Processor => {
                let handle = self.export_shared();
                unsafe { bridge.adopt_shared(handle) }
            }
            InstanceConnectionRole::Controller => {
                let handle = bridge.export_shared();
                unsafe { self.adopt_shared(handle) }
            }
        };

        if result == kResultOk {
            self.set_peer(Some(other as usize));
        }
        result
    }

    #[doc(hidden)]
    pub unsafe fn disconnect(&self, other: *mut IConnectionPoint) -> tresult {
        if other.is_null() {
            return kInvalidArgument;
        }
        let other = other as usize;
        let mut peer = self
            .peer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if peer.is_some_and(|connected| connected == other) {
            *peer = None;
            kResultOk
        } else {
            kResultFalse
        }
    }

    #[doc(hidden)]
    pub unsafe fn notify(&self, _message: *mut IMessage) -> tresult {
        kResultFalse
    }

    #[doc(hidden)]
    pub fn export_shared(&self) -> *mut SharedStateHandle {
        let type_name = std::any::type_name::<T>().as_bytes();
        Box::into_raw(Box::new(SharedStateHandle {
            type_name: type_name.as_ptr(),
            type_name_len: type_name.len(),
            state: Arc::into_raw(self.shared()).cast::<c_void>(),
            release: release_arc::<T>,
        }))
    }

    #[doc(hidden)]
    pub unsafe fn adopt_shared(&self, handle: *mut SharedStateHandle) -> tresult {
        let Some(handle) = (unsafe { handle.as_ref() }) else {
            return kInvalidArgument;
        };
        let expected = std::any::type_name::<T>().as_bytes();
        let received = if handle.type_name.is_null() {
            &[][..]
        } else {
            unsafe { slice::from_raw_parts(handle.type_name, handle.type_name_len) }
        };
        let compatible = self.role == InstanceConnectionRole::Controller
            && received == expected
            && !handle.state.is_null();

        let handle =
            unsafe { Box::from_raw(handle as *const SharedStateHandle as *mut SharedStateHandle) };
        if !compatible {
            unsafe { (handle.release)(handle.state) };
            return kResultFalse;
        }

        let shared = unsafe { Arc::from_raw(handle.state.cast::<T>()) };
        let mut current = self
            .shared
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = shared;
        kResultOk
    }

    /// Records the peer without retaining its COM object.
    fn set_peer(&self, peer: Option<usize>) {
        *self
            .peer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = peer;
    }
}

/// Releases an exported `Arc<T>` reference after an incompatible transfer.
unsafe extern "system" fn release_arc<T>(state: *const c_void) {
    if !state.is_null() {
        drop(unsafe { Arc::from_raw(state.cast::<T>()) });
    }
}

/// Opaque owned state transfer used by Toybox's private COM bridge.
#[doc(hidden)]
#[repr(C)]
pub struct SharedStateHandle {
    type_name: *const u8,
    type_name_len: usize,
    state: *const c_void,
    release: unsafe extern "system" fn(*const c_void),
}

/// Private COM bridge queried from the exact `IConnectionPoint` supplied by the host.
#[doc(hidden)]
#[repr(C)]
pub struct IToyboxSharedState {
    vtbl: *const IToyboxSharedStateVtbl,
}

unsafe impl Send for IToyboxSharedState {}
unsafe impl Sync for IToyboxSharedState {}
unsafe impl Inherits<FUnknown> for IToyboxSharedState {}

impl Unknown for IToyboxSharedState {
    unsafe fn query_interface(this: *mut Self, iid: &Guid) -> Option<*mut c_void> {
        unsafe { <FUnknown as Unknown>::query_interface(this.cast::<FUnknown>(), iid) }
    }

    unsafe fn add_ref(this: *mut Self) -> usize {
        unsafe { <FUnknown as Unknown>::add_ref(this.cast::<FUnknown>()) }
    }

    unsafe fn release(this: *mut Self) -> usize {
        unsafe { <FUnknown as Unknown>::release(this.cast::<FUnknown>()) }
    }
}

unsafe impl Interface for IToyboxSharedState {
    type Vtbl = IToyboxSharedStateVtbl;

    const IID: Guid = *b"ToyboxStateV1___";

    fn inherits(iid: &Guid) -> bool {
        iid == &Self::IID || FUnknown::inherits(iid)
    }
}

/// Methods implemented by plugin classes that contain an [`InstanceConnection`].
#[doc(hidden)]
pub trait IToyboxSharedStateTrait {
    /// Exports one owned `Arc` reference.
    fn export_shared(&self) -> *mut SharedStateHandle;

    /// Consumes an exported state reference and adopts it when compatible.
    unsafe fn adopt_shared(&self, handle: *mut SharedStateHandle) -> tresult;
}

impl<P> IToyboxSharedStateTrait for P
where
    P: SmartPtr,
    P::Target: Inherits<IToyboxSharedState>,
{
    fn export_shared(&self) -> *mut SharedStateHandle {
        let pointer = self.ptr().cast::<IToyboxSharedState>();
        unsafe { ((*(*pointer).vtbl).export_shared)(pointer) }
    }

    unsafe fn adopt_shared(&self, handle: *mut SharedStateHandle) -> tresult {
        let pointer = self.ptr().cast::<IToyboxSharedState>();
        unsafe { ((*(*pointer).vtbl).adopt_shared)(pointer, handle) }
    }
}

#[doc(hidden)]
#[repr(C)]
pub struct IToyboxSharedStateVtbl {
    base: FUnknownVtbl,
    export_shared: unsafe extern "system" fn(*mut IToyboxSharedState) -> *mut SharedStateHandle,
    adopt_shared:
        unsafe extern "system" fn(*mut IToyboxSharedState, *mut SharedStateHandle) -> tresult,
}

impl IToyboxSharedState {
    /// Builds the custom bridge vtable for a plugin class.
    const fn make_vtbl<C, W, const OFFSET: isize>() -> IToyboxSharedStateVtbl
    where
        C: Class + IToyboxSharedStateTrait,
        W: Wrapper<C>,
    {
        unsafe extern "system" fn export_shared<C, W, const OFFSET: isize>(
            this: *mut IToyboxSharedState,
        ) -> *mut SharedStateHandle
        where
            C: Class + IToyboxSharedStateTrait,
            W: Wrapper<C>,
        {
            let header = unsafe { (this.cast::<u8>()).offset(-OFFSET).cast::<Header<C>>() };
            unsafe { (*W::data_from_header(header)).export_shared() }
        }

        unsafe extern "system" fn adopt_shared<C, W, const OFFSET: isize>(
            this: *mut IToyboxSharedState,
            handle: *mut SharedStateHandle,
        ) -> tresult
        where
            C: Class + IToyboxSharedStateTrait,
            W: Wrapper<C>,
        {
            let header = unsafe { (this.cast::<u8>()).offset(-OFFSET).cast::<Header<C>>() };
            unsafe { (*W::data_from_header(header)).adopt_shared(handle) }
        }

        IToyboxSharedStateVtbl {
            base: make_funknown_vtbl::<C, W, OFFSET>(),
            export_shared: export_shared::<C, W, OFFSET>,
            adopt_shared: adopt_shared::<C, W, OFFSET>,
        }
    }
}

unsafe impl<C, W, const OFFSET: isize> Construct<C, W, OFFSET> for IToyboxSharedState
where
    C: Class + IToyboxSharedStateTrait,
    W: Wrapper<C>,
{
    const OBJ: Self = Self {
        vtbl: &Self::make_vtbl::<C, W, OFFSET>(),
    };
}

/// Builds the `FUnknown` prefix for the custom bridge vtable.
const fn make_funknown_vtbl<C, W, const OFFSET: isize>() -> FUnknownVtbl
where
    C: Class,
    W: Wrapper<C>,
{
    unsafe extern "system" fn query_interface<C, W, const OFFSET: isize>(
        this: *mut FUnknown,
        iid: *const TUID,
        object: *mut *mut c_void,
    ) -> tresult
    where
        C: Class,
        W: Wrapper<C>,
    {
        if iid.is_null() || object.is_null() {
            return kInvalidArgument;
        }
        let header = unsafe { (this.cast::<u8>()).offset(-OFFSET).cast::<Header<C>>() };
        let guid = unsafe { &*iid.cast::<Guid>() };
        let Some(result) = C::Interfaces::query(guid) else {
            unsafe { *object = ptr::null_mut() };
            return kResultFalse;
        };
        let data = unsafe { W::data_from_header(header) };
        unsafe { W::add_ref(data) };
        unsafe { *object = (header.cast::<u8>()).offset(result).cast::<c_void>() };
        kResultOk
    }

    unsafe extern "system" fn add_ref<C, W, const OFFSET: isize>(this: *mut FUnknown) -> u32
    where
        C: Class,
        W: Wrapper<C>,
    {
        let header = unsafe { (this.cast::<u8>()).offset(-OFFSET).cast::<Header<C>>() };
        unsafe { W::add_ref(W::data_from_header(header)) as u32 }
    }

    unsafe extern "system" fn release<C, W, const OFFSET: isize>(this: *mut FUnknown) -> u32
    where
        C: Class,
        W: Wrapper<C>,
    {
        let header = unsafe { (this.cast::<u8>()).offset(-OFFSET).cast::<Header<C>>() };
        unsafe { W::release(W::data_from_header(header)) as u32 }
    }

    FUnknownVtbl {
        queryInterface: query_interface::<C, W, OFFSET>,
        addRef: add_ref::<C, W, OFFSET>,
        release: release::<C, W, OFFSET>,
    }
}

/// Implements VST3 connection interfaces by delegating to an [`InstanceConnection`] field.
#[macro_export]
macro_rules! impl_vst3_instance_connection {
    ($class:ty, $field:ident) => {
        impl $crate::vst3::prelude::IConnectionPointTrait for $class {
            unsafe fn connect(
                &self,
                other: *mut $crate::vst3::prelude::IConnectionPoint,
            ) -> $crate::vst3::prelude::Steinberg::tresult {
                unsafe { self.$field.connect(other) }
            }

            unsafe fn disconnect(
                &self,
                other: *mut $crate::vst3::prelude::IConnectionPoint,
            ) -> $crate::vst3::prelude::Steinberg::tresult {
                unsafe { self.$field.disconnect(other) }
            }

            unsafe fn notify(
                &self,
                message: *mut $crate::vst3::prelude::IMessage,
            ) -> $crate::vst3::prelude::Steinberg::tresult {
                unsafe { self.$field.notify(message) }
            }
        }

        impl $crate::vst3::connection::IToyboxSharedStateTrait for $class {
            fn export_shared(&self) -> *mut $crate::vst3::connection::SharedStateHandle {
                self.$field.export_shared()
            }

            unsafe fn adopt_shared(
                &self,
                handle: *mut $crate::vst3::connection::SharedStateHandle,
            ) -> $crate::vst3::prelude::Steinberg::tresult {
                unsafe { self.$field.adopt_shared(handle) }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use toybox_vst3_ffi::ComWrapper;

    #[derive(Debug)]
    struct State(u32);

    struct Endpoint {
        connection: InstanceConnection<State>,
    }

    impl Endpoint {
        fn new(role: InstanceConnectionRole, value: u32) -> Self {
            Self {
                connection: InstanceConnection::new(role, Arc::new(State(value))),
            }
        }
    }

    impl Class for Endpoint {
        type Interfaces = (IConnectionPoint, IToyboxSharedState);
    }

    crate::impl_vst3_instance_connection!(Endpoint, connection);

    fn connection_point(endpoint: &ComWrapper<Endpoint>) -> ComRef<'_, IConnectionPoint> {
        endpoint
            .as_com_ref::<IConnectionPoint>()
            .expect("endpoint connection point")
    }

    #[test]
    fn shuffled_creation_order_keeps_instances_independent() {
        let processor_one = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Processor, 11));
        let processor_two = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Processor, 22));
        let controller_two = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Controller, 0));
        let controller_one = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Controller, 0));

        let processor_one_point = connection_point(&processor_one);
        let processor_two_point = connection_point(&processor_two);
        let controller_one_point = connection_point(&controller_one);
        let controller_two_point = connection_point(&controller_two);
        assert_eq!(
            unsafe { processor_two_point.connect(controller_two_point.as_ptr()) },
            kResultOk
        );
        assert_eq!(
            unsafe { controller_one_point.connect(processor_one_point.as_ptr()) },
            kResultOk
        );

        assert_eq!(controller_one.connection.shared().0, 11);
        assert_eq!(controller_two.connection.shared().0, 22);
        assert!(controller_one.connection.is_connected());
        assert!(processor_two.connection.is_connected());
    }

    #[test]
    fn disconnect_destroy_and_reconnect_are_safe() {
        for cycle in 0..128 {
            let controller = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Controller, 0));
            let first_processor =
                ComWrapper::new(Endpoint::new(InstanceConnectionRole::Processor, cycle));
            let controller_point = connection_point(&controller);
            let first_point = connection_point(&first_processor);
            assert_eq!(
                unsafe { controller_point.connect(first_point.as_ptr()) },
                kResultOk
            );
            assert_eq!(controller.connection.shared().0, cycle);
            assert_eq!(
                unsafe { controller_point.disconnect(first_point.as_ptr()) },
                kResultOk
            );
            drop(first_processor);

            let second_processor = ComWrapper::new(Endpoint::new(
                InstanceConnectionRole::Processor,
                cycle + 1_000,
            ));
            let second_point = connection_point(&second_processor);
            assert_eq!(
                unsafe { second_point.connect(controller_point.as_ptr()) },
                kResultOk
            );
            assert_eq!(controller.connection.shared().0, cycle + 1_000);
        }
    }

    #[test]
    fn incompatible_or_unmatched_connections_do_not_replace_processor_state() {
        let first = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Processor, 1));
        let second = ComWrapper::new(Endpoint::new(InstanceConnectionRole::Processor, 2));
        let first_point = connection_point(&first);
        let second_point = connection_point(&second);

        assert_eq!(
            unsafe { first_point.connect(second_point.as_ptr()) },
            kResultFalse
        );
        assert_eq!(first.connection.shared().0, 1);
        assert_eq!(second.connection.shared().0, 2);
    }
}
