pub mod wac_read;
pub mod wac_write;
pub mod wasi_support;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShimParametersConfig {
    pub component_to_shim: Option<String>,
    pub interface_to_shim: Option<String>,
    pub package_shim: String,
}

pub struct ShimParametersExplicit {
    pub component_to_shim: String,
    pub interface_to_shim: String,
    pub package_shim: String,
}

pub struct ShimParametersImplicit {
    pub package_shim: String,
}
