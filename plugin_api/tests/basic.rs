#[test]
fn test_plugin_context_fields() {
    use plugin_api::PluginContext;
    use std::collections::HashMap;
    let ctx = PluginContext {
        hook: "testhook".to_string(),
        package_name: "testpkg".to_string(),
        package_version: "0.1.0".to_string(),
        install_path: "/tmp/testpkg".to_string(),
        env: HashMap::new(),
    };
    assert_eq!(ctx.hook, "testhook");
    assert_eq!(ctx.package_name, "testpkg");
    assert_eq!(ctx.package_version, "0.1.0");
    assert_eq!(ctx.install_path, "/tmp/testpkg");
} 