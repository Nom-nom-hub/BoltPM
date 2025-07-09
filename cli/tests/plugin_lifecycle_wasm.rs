#[cfg(feature = "wasm_plugins")]
#[test]
fn test_wasm_plugin_lifecycle_success() {
    use assert_cmd::Command;
    use std::fs;
    use std::path::Path;

    // Clean plugins dir
    let plugin_dir = Path::new(".boltpm/plugins");
    if plugin_dir.exists() {
        for entry in fs::read_dir(plugin_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map(|e| e == "dylib" || e == "so" || e == "dll" || e == "wasm").unwrap_or(false) {
                let _ = fs::remove_file(&path);
            }
        }
    }
    // Clean plugins_output dir
    let output_dir = Path::new(".boltpm/plugins_output");
    if output_dir.exists() {
        for entry in fs::read_dir(output_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let _ = fs::remove_file(&path);
        }
    }
    // Setup
    let plugin_path = plugin_dir.join("test_plugin.wasm");
    fs::create_dir_all(plugin_path.parent().unwrap()).unwrap();
    fs::copy("tests/fixtures/plugins/success.wasm", &plugin_path).unwrap();
    // Write package.json with a dummy dependency
    fs::write("package.json", r#"{"name":"boltpm-demo","version":"1.0.0","dependencies":{"foo":"1.0.0"}}"#).unwrap();

    // Run
    let mut cmd = Command::cargo_bin("boltpm").unwrap();
    cmd.arg("install")
        .assert()
        .success(); // assert exit code == 0
    // Assert plugin side effect
    let marker = Path::new(".boltpm/plugins_output/wasm_test_hook");
    assert!(marker.exists(), "WASM plugin did not write marker file");
    let contents = fs::read_to_string(marker).unwrap();
    assert_eq!(contents, "wasm plugin ran");
}

#[cfg(feature = "wasm_plugins")]
#[test]
fn test_wasm_plugin_lifecycle_failure() {
    use assert_cmd::Command;
    use std::fs;
    use std::path::Path;

    // Clean plugins dir
    let plugin_dir = Path::new(".boltpm/plugins");
    if plugin_dir.exists() {
        for entry in fs::read_dir(plugin_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map(|e| e == "dylib" || e == "so" || e == "dll" || e == "wasm").unwrap_or(false) {
                let _ = fs::remove_file(&path);
            }
        }
    }
    // Clean plugins_output dir
    let output_dir = Path::new(".boltpm/plugins_output");
    if output_dir.exists() {
        for entry in fs::read_dir(output_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let _ = fs::remove_file(&path);
        }
    }
    // Setup
    let plugin_path = plugin_dir.join("test_plugin.wasm");
    fs::create_dir_all(plugin_path.parent().unwrap()).unwrap();
    fs::copy("tests/fixtures/plugins/failure.wasm", &plugin_path).unwrap();
    // Write package.json with a dummy dependency
    fs::write("package.json", r#"{"name":"boltpm-demo","version":"1.0.0","dependencies":{"foo":"1.0.0"}}"#).unwrap();

    // Run
    let mut cmd = Command::cargo_bin("boltpm").unwrap();
    cmd.arg("install")
        .assert()
        .failure(); // assert exit code != 0
    // Assert plugin side effect
    let marker = Path::new(".boltpm/plugins_output/wasm_test_hook");
    assert!(marker.exists(), "WASM plugin did not write marker file");
    let contents = fs::read_to_string(marker).unwrap();
    assert_eq!(contents, "wasm plugin ran");
} 