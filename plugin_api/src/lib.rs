use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginContext {
    pub hook: String,
    pub package_name: String,
    pub package_version: String,
    pub install_path: String,
    pub env: HashMap<String, String>,
}

pub type PluginResult = Result<(), String>;
