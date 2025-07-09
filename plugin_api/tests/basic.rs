#[test]
fn test_plugin_api_builds() {
    use plugin_api::PluginContext;
    use std::collections::HashMap;
    use std::path::PathBuf;
    let _ctx = PluginContext {
        package_name: "test".to_string(),
        version: "0.1.0".to_string(),
        path: PathBuf::new(),
        metadata: HashMap::new(),
    };
} 