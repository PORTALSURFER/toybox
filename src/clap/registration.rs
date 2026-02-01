//! Registration helpers to reduce CLAP export boilerplate.

pub use clack_plugin::prelude::clack_export_entry;

/// Export a single-plugin CLAP entry point with minimal boilerplate.
///
/// # Example
///
/// ```
/// use clack_plugin::prelude::*;
/// use toybox::clap::entry::PluginEntry;
///
/// pub struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     type AudioProcessor<'a> = ();
///     type Shared<'a> = ();
///     type MainThread<'a> = ();
/// }
///
/// impl DefaultPluginFactory for MyPlugin {
///     fn get_descriptor() -> PluginDescriptor {
///         PluginDescriptor::new("my.plugin", "My Plugin")
///     }
///
///     fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
///         Ok(())
///     }
///
///     fn new_main_thread<'a>(
///         _host: HostMainThreadHandle<'a>,
///         _shared: &'a Self::Shared<'a>,
///     ) -> Result<Self::MainThread<'a>, PluginError> {
///         Ok(())
///     }
/// }
///
/// toybox::clap_plugin_entry!(MyPlugin);
/// ```
#[macro_export]
macro_rules! clap_plugin_entry {
    ($plugin:ty) => {
        $crate::clap::registration::clack_export_entry!(
            $crate::clap::entry::PluginEntry::<$plugin>
        );
    };
}
