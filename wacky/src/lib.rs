pub mod wac_read;
pub mod wac_write;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShimParameters {
    pub component_to_shim: String,
    pub interface_to_shim: String,
    pub package_shim: String,
}
