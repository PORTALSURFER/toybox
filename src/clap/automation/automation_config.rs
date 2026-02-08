//! Automation parameter enable/disable configuration.

use std::collections::HashSet;

use clack_plugin::utils::ClapId;

/// Configuration for which parameters should emit automation events.
#[derive(Clone, Debug)]
pub struct AutomationConfig {
    /// Fallback enable state used when a parameter has no explicit override.
    default_enabled: bool,
    /// Parameter ids that are always automation-enabled.
    enabled: HashSet<ClapId>,
    /// Parameter ids that are always automation-disabled.
    disabled: HashSet<ClapId>,
}

impl AutomationConfig {
    /// Create a new automation config with a default enabled/disabled state.
    pub fn new(default_enabled: bool) -> Self {
        Self {
            default_enabled,
            enabled: HashSet::new(),
            disabled: HashSet::new(),
        }
    }

    /// Return true if the parameter should emit automation events.
    pub fn is_enabled(&self, param_id: ClapId) -> bool {
        if self.enabled.contains(&param_id) {
            return true;
        }
        if self.disabled.contains(&param_id) {
            return false;
        }
        self.default_enabled
    }

    /// Enable automation for a specific parameter.
    pub fn enable_param(&mut self, param_id: ClapId) {
        self.disabled.remove(&param_id);
        self.enabled.insert(param_id);
    }

    /// Disable automation for a specific parameter.
    pub fn disable_param(&mut self, param_id: ClapId) {
        self.enabled.remove(&param_id);
        self.disabled.insert(param_id);
    }
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self::new(true)
    }
}
