use std::collections::HashMap;
use std::path::PathBuf;

#[repr(C)]
pub struct PluginContext {
    pub package_name: String,
    pub version: String,
    pub path: PathBuf,
    pub metadata: HashMap<String, String>,
}

pub type PluginResult = Result<(), String>; 