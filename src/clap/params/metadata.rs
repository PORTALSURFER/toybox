use clack_extensions::params::{ParamInfo, ParamInfoFlags, ParamInfoWriter};
use clack_plugin::utils::ClapId;

/// Describes a CLAP parameter's metadata for registration with the host.
pub struct ParamSpec<'a> {
    /// Stable CLAP parameter identifier.
    pub id: ClapId,
    /// CLAP parameter flags (automation, stepped, etc.).
    pub flags: ParamInfoFlags,
    /// Parameter display name.
    pub name: &'a [u8],
    /// Parameter module/group name.
    pub module: &'a [u8],
    /// Minimum value.
    pub min_value: f64,
    /// Maximum value.
    pub max_value: f64,
    /// Default value.
    pub default_value: f64,
}

impl ParamSpec<'_> {
    /// Write this spec into the CLAP parameter info writer.
    pub fn write(&self, writer: &mut ParamInfoWriter) {
        writer.set(&ParamInfo {
            id: self.id,
            flags: self.flags,
            cookie: Default::default(),
            name: self.name,
            module: self.module,
            min_value: self.min_value,
            max_value: self.max_value,
            default_value: self.default_value,
        });
    }
}

/// Builder for CLAP parameter metadata.
///
/// This keeps param definitions compact while still producing a concrete
/// [`ParamSpec`].
pub struct ParamBuilder<'a> {
    /// Stable CLAP parameter identifier.
    id: ClapId,
    /// CLAP parameter flags accumulated by builder methods.
    flags: ParamInfoFlags,
    /// Parameter display name bytes.
    name: &'a [u8],
    /// Parameter module/group name bytes.
    module: &'a [u8],
    /// Minimum parameter value.
    min_value: f64,
    /// Maximum parameter value.
    max_value: f64,
    /// Default parameter value.
    default_value: f64,
}

impl<'a> ParamBuilder<'a> {
    /// Create a new builder with a required id, name, and module label.
    pub fn new(id: ClapId, name: &'a [u8], module: &'a [u8]) -> Self {
        Self {
            id,
            flags: ParamInfoFlags::empty(),
            name,
            module,
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        }
    }

    /// Mark the parameter as automatable.
    pub fn automatable(mut self) -> Self {
        self.flags |= ParamInfoFlags::IS_AUTOMATABLE;
        self
    }

    /// Mark the parameter as stepped (integer values only).
    pub fn stepped(mut self) -> Self {
        self.flags |= ParamInfoFlags::IS_STEPPED;
        self
    }

    /// Mark the parameter as an enum.
    pub fn enumerated(mut self) -> Self {
        self.flags |= ParamInfoFlags::IS_ENUM;
        self
    }

    /// Set the parameter's numeric range.
    pub fn range(mut self, min_value: f64, max_value: f64) -> Self {
        self.min_value = min_value;
        self.max_value = max_value;
        self
    }

    /// Set the parameter's default value.
    pub fn default(mut self, default_value: f64) -> Self {
        self.default_value = default_value;
        self
    }

    /// Convert this builder into a concrete [`ParamSpec`].
    pub fn build(self) -> ParamSpec<'a> {
        ParamSpec {
            id: self.id,
            flags: self.flags,
            name: self.name,
            module: self.module,
            min_value: self.min_value,
            max_value: self.max_value,
            default_value: self.default_value,
        }
    }
}
