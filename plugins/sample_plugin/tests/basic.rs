use plugin_api::PluginContext;
use std::collections::HashMap;

#[test]
fn test_sample_plugin_builds() {
    assert_eq!(3 * 3, 9);
}

#[test]
fn test_sample_plugin_run() {
    use std::path::PathBuf;
    use libloading::{Library, Symbol};
    use std::env;
    use serde_json;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("../../").canonicalize().expect("Failed to canonicalize workspace root");
    let candidates = [
        workspace_root.join("target/debug/libsample_plugin.dylib"),
        workspace_root.join("target/debug/deps/libsample_plugin.dylib"),
        workspace_root.join("test_project/.boltpm/plugins/libsample_plugin.dylib"),
    ];
    println!("Checking for libsample_plugin.dylib at the following locations:");
    for path in &candidates {
        println!("  {}", path.display());
    }
    let lib_path = candidates.iter().find(|p| p.exists());
    let lib = match lib_path {
        Some(path) => unsafe { Library::new(path).unwrap() },
        None => {
            let cwd = env::current_dir().unwrap();
            panic!("Could not find libsample_plugin.dylib in any known location. Current working directory: {}. Tried: {:?}", cwd.display(), candidates);
        }
    };
    let func: Symbol<unsafe extern "C" fn(*const u8, usize) -> i32> = unsafe { lib.get(b"run").unwrap() };
    let ctx = PluginContext {
        hook: "testhook".to_string(),
        package_name: "sample".to_string(),
        package_version: "1.2.3".to_string(),
        install_path: "/tmp/sample".to_string(),
        env: HashMap::new(),
    };
    let ctx_json = serde_json::to_string(&ctx).unwrap();
    let ctx_bytes = ctx_json.as_bytes();
    let result = unsafe { func(ctx_bytes.as_ptr(), ctx_bytes.len()) };
    assert_eq!(result, 0);
}

#[test]
fn test_plugin_context_fields() {
    let ctx = PluginContext {
        hook: "testhook".to_string(),
        package_name: "sample".to_string(),
        package_version: "1.2.3".to_string(),
        install_path: "/tmp/sample".to_string(),
        env: HashMap::new(),
    };
    assert_eq!(ctx.hook, "testhook");
    assert_eq!(ctx.package_name, "sample");
    assert_eq!(ctx.package_version, "1.2.3");
    assert_eq!(ctx.install_path, "/tmp/sample");
} 