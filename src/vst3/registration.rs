//! Registration helpers to reduce VST3 export boilerplate.

use std::ptr;

use toybox_vst3_ffi::{
    Class, ComPtr, ComWrapper, Steinberg::IPluginFactory, Steinberg::IPluginFactoryTrait,
    com_scrape_types::MakeHeader,
};

/// Trait bound for VST3 plugin factory classes exported through toybox.
pub trait Vst3FactoryClass: Class + Default + IPluginFactoryTrait
where
    Self::Interfaces: MakeHeader<Self, ComWrapper<Self>>,
{
}

impl<T> Vst3FactoryClass for T
where
    T: Class + Default + IPluginFactoryTrait,
    T::Interfaces: MakeHeader<T, ComWrapper<T>>,
{
}

/// Create and leak a COM factory object for VST3 host discovery.
///
/// The returned pointer is owned by the host and follows VST3 COM lifetime
/// conventions.
pub fn create_plugin_factory<F>() -> *mut IPluginFactory
where
    F: Vst3FactoryClass + 'static,
    F::Interfaces: MakeHeader<F, ComWrapper<F>>,
{
    let Some(factory) = ComWrapper::new(F::default()).to_com_ptr::<IPluginFactory>() else {
        return ptr::null_mut();
    };
    ComPtr::into_raw(factory)
}

/// Export a VST3 plugin entrypoint using a default constructible factory type.
///
/// # Example
///
/// ```ignore
/// # use toybox::vst3::prelude::*;
/// # #[derive(Default)]
/// # struct Factory;
/// # impl Class for Factory { type Interfaces = (Steinberg::IPluginFactory,); }
/// # impl Steinberg::IPluginFactoryTrait for Factory {
/// #   unsafe fn getFactoryInfo(&self, _info: *mut Steinberg::PFactoryInfo) -> Steinberg::tresult { Steinberg::kResultOk }
/// #   unsafe fn countClasses(&self) -> i32 { 0 }
/// #   unsafe fn getClassInfo(&self, _index: i32, _info: *mut Steinberg::PClassInfo) -> Steinberg::tresult { Steinberg::kResultFalse }
/// #   unsafe fn createInstance(&self, _cid: Steinberg::FIDString, _iid: Steinberg::FIDString, _obj: *mut *mut std::ffi::c_void) -> Steinberg::tresult { Steinberg::kNoInterface }
/// # }
/// toybox::vst3_plugin_entry!(Factory);
/// ```
#[macro_export]
macro_rules! vst3_plugin_entry {
    ($factory:ty) => {
        const _: fn() = || {
            let _ = $crate::vst3::registration::create_plugin_factory::<$factory>;
        };

        #[cfg(target_os = "windows")]
        #[unsafe(no_mangle)]
        extern "system" fn InitDll() -> bool {
            true
        }

        #[cfg(target_os = "windows")]
        #[unsafe(no_mangle)]
        extern "system" fn ExitDll() -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[unsafe(no_mangle)]
        extern "system" fn BundleEntry(_bundle_ref: *mut core::ffi::c_void) -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[unsafe(no_mangle)]
        extern "system" fn BundleExit() -> bool {
            true
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        #[unsafe(no_mangle)]
        extern "system" fn ModuleEntry(_library_handle: *mut core::ffi::c_void) -> bool {
            true
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        #[unsafe(no_mangle)]
        extern "system" fn ModuleExit() -> bool {
            true
        }

        #[unsafe(no_mangle)]
        extern "system" fn GetPluginFactory()
        -> *mut $crate::vst3::prelude::Steinberg::IPluginFactory {
            $crate::vst3::registration::create_plugin_factory::<$factory>()
        }
    };
}
