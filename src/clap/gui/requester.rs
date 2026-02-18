//! Host flush requester helpers for CLAP parameter automation.

use clack_extensions::params::HostParams;
use clack_plugin::prelude::HostSharedHandle;

/// Lightweight handle that allows GUI code to request a CLAP host parameter flush.
///
/// This wrapper captures the host shared handle and the host parameter extension
/// in a small value type that can be stored on GUI state and reused whenever
/// plugin controls need to notify the host.
#[derive(Clone, Copy)]
pub struct HostParamRequester {
    /// Shared host handle captured as `'static` for GUI runtime lifetimes.
    host: HostSharedHandle<'static>,
    /// CLAP parameter extension interface for flush requests.
    params: HostParams,
}

impl HostParamRequester {
    /// Ask the host to collect and apply queued parameter updates.
    pub fn request_flush(self) {
        self.params.request_flush(&self.host);
    }
}

/// Build a [`HostParamRequester`] from a CLAP shared host handle.
///
/// Returns `None` when the host does not provide the parameter extension.
///
/// # Safety
///
/// The returned value stores a `'static` handle internally so that GUI state can
/// outlive the temporary borrow of the current host handle safely within the
/// framework's parented window runtime.
pub fn host_param_requester(host: HostSharedHandle<'_>) -> Option<HostParamRequester> {
    let params = host.get_extension::<HostParams>()?;
    let host =
        unsafe { std::mem::transmute::<HostSharedHandle<'_>, HostSharedHandle<'static>>(host) };

    Some(HostParamRequester { host, params })
}
