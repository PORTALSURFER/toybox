//! CLAP entry helpers modeled after clack's `SinglePluginEntry`.

use clack_plugin::entry::SinglePluginEntry;

/// Convenience alias for CLAP entry points that expose a single plugin type.
///
/// This keeps the entry type stable across plugins, while still allowing
/// custom entries later when multiple plugin types are needed.
pub type PluginEntry<P> = SinglePluginEntry<P>;
