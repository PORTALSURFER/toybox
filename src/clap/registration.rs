//! Registration helpers to reduce CLAP export boilerplate.

use clack_extensions::audio_ports::{PluginAudioPorts, PluginAudioPortsImpl};
#[cfg(feature = "gui")]
use clack_extensions::gui::{PluginGui, PluginGuiImpl};
use clack_extensions::params::{PluginAudioProcessorParams, PluginMainThreadParams, PluginParams};
use clack_extensions::state::{PluginState, PluginStateImpl};
use clack_plugin::extensions::PluginExtensions;
use clack_plugin::plugin::Plugin;

pub use clack_plugin::prelude::clack_export_entry;

/// Register the default CLAP extensions used by toybox plugins.
///
/// This always registers audio ports, params, and plugin state extensions.
pub fn register_default_extensions<P>(builder: &mut PluginExtensions<P>)
where
    P: Plugin,
    for<'a> P::MainThread<'a>: PluginAudioPortsImpl + PluginMainThreadParams + PluginStateImpl,
    for<'a> P::AudioProcessor<'a>: PluginAudioProcessorParams,
{
    builder
        .register::<PluginAudioPorts>()
        .register::<PluginParams>()
        .register::<PluginState>();
}

/// Register the default CLAP extensions plus GUI support.
#[cfg(feature = "gui")]
pub fn register_default_extensions_with_gui<P>(builder: &mut PluginExtensions<P>)
where
    P: Plugin,
    for<'a> P::MainThread<'a>:
        PluginAudioPortsImpl + PluginMainThreadParams + PluginStateImpl + PluginGuiImpl,
    for<'a> P::AudioProcessor<'a>: PluginAudioProcessorParams,
{
    register_default_extensions(builder);
    builder.register::<PluginGui>();
}

/// Export a single-plugin CLAP entry point with minimal boilerplate.
///
/// This macro enforces that the plugin main-thread type implements
/// [`PluginStateImpl`]. If state support is missing, compilation fails.
///
/// # Example
///
/// ```
/// use clack_plugin::prelude::*;
/// use toybox::clap::entry::PluginEntry;
/// use toybox::clack_plugin::stream::{InputStream, OutputStream};
///
/// pub struct MyPlugin;
/// pub struct MyMainThread;
///
/// impl<'a> PluginMainThread<'a, ()> for MyMainThread {}
///
/// impl Plugin for MyPlugin {
///     type AudioProcessor<'a> = ();
///     type Shared<'a> = ();
///     type MainThread<'a> = MyMainThread;
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
///         Ok(MyMainThread)
///     }
/// }
///
/// impl toybox::clack_extensions::state::PluginStateImpl for MyMainThread {
///     fn save(&mut self, _output: &mut OutputStream) -> Result<(), PluginError> {
///         Ok(())
///     }
///
///     fn load(&mut self, _input: &mut InputStream) -> Result<(), PluginError> {
///         Ok(())
///     }
/// }
///
/// toybox::clap_plugin_entry!(MyPlugin);
/// ```
///
/// # Compile-Fail Example
/// ```compile_fail
/// use toybox::clap::prelude::*;
///
/// pub struct NoStatePlugin;
///
/// impl Plugin for NoStatePlugin {
///     type AudioProcessor<'a> = ();
///     type Shared<'a> = ();
///     type MainThread<'a> = ();
/// }
///
/// impl DefaultPluginFactory for NoStatePlugin {
///     fn get_descriptor() -> PluginDescriptor {
///         PluginDescriptor::new("example.no-state", "No State")
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
/// toybox::clap_plugin_entry!(NoStatePlugin);
/// ```
#[macro_export]
macro_rules! clap_plugin_entry {
    ($plugin:ty) => {
        const _: () = {
            fn assert_plugin_state_impl<P>()
            where
                P: $crate::clack_plugin::prelude::Plugin,
                for<'a> <P as $crate::clack_plugin::prelude::Plugin>::MainThread<'a>:
                    $crate::clack_extensions::state::PluginStateImpl,
            {
            }

            let _ = assert_plugin_state_impl::<$plugin>;
        };

        $crate::clap::registration::clack_export_entry!(
            $crate::clap::entry::PluginEntry::<$plugin>
        );
    };
}
