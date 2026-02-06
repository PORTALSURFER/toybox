//! Entry metadata helpers for VST3 plugins.

use toybox_vst3_ffi::Steinberg::TUID;

/// Declares the class identifiers used by a VST3 plugin pair.
///
/// VST3 plugins typically expose at least one audio component class and one
/// edit controller class. Keeping the IDs grouped helps keep factory wiring
/// explicit and deterministic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginClassIds {
    /// Class ID for the audio component implementation.
    pub component: TUID,
    /// Class ID for the edit controller implementation.
    pub controller: TUID,
}

impl PluginClassIds {
    /// Build a new class-id group.
    pub const fn new(component: TUID, controller: TUID) -> Self {
        Self {
            component,
            controller,
        }
    }
}
