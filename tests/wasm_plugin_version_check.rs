use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use cli::plugin::{run_plugins, PluginContext};

#[test]
fn test_wasm_plugin_version_check() {
    // Setup temp project dir
    let temp = tempdir().unwrap();
    let plugins_dir = temp.path().join(".boltpm/plugins");
    fs::create_dir_all(&plugins_dir).unwrap();
    // Copy a WASM plugin missing _boltpm_plugin_v1 (simulate by using failure.wasm)
    let wasm_src = PathBuf::from("../wasm_plugins/failure/target/wasm32-unknown-unknown/release/failure.wasm");
    let wasm_dst = plugins_dir.join("test_plugin.wasm");
    fs::copy(&wasm_src, &wasm_dst).expect("Failed to copy test WASM plugin");
    // Prepare PluginContext
    let ctx = PluginContext {
        hook: "preinstall".to_string(),
        package_name: "testpkg".to_string(),
        package_version: "1.0.0".to_string(),
        install_path: temp.path().to_string_lossy().to_string(),
        env: std::env::vars().collect(),
    };
    // Run plugins (should fail)
    let result = run_plugins("preinstall", &ctx);
    assert!(result.is_err(), "WASM plugin without _boltpm_plugin_v1 should fail");
} 