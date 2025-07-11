use serial_test::serial;
use std::env;
use std::fs;
use std::process::Command;

fn workspace_path(rel: &str) -> std::path::PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(rel)
}

fn clean_plugins_dir() {
    let plugins_dir = workspace_path(".boltpm/plugins");
    let _ = fs::remove_dir_all(&plugins_dir);
    fs::create_dir_all(&plugins_dir).unwrap();
}

fn setup_test_plugin(success: bool) {
    let plugins_dir = workspace_path(".boltpm/plugins");
    // Print current plugins for debug
    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        println!("[DEBUG] Plugins before setup:");
        for entry in entries.flatten() {
            println!("[DEBUG] - {:?}", entry.path());
        }
    }
    // Determine plugin file name based on OS
    let (src, dest_name) = {
        #[cfg(target_os = "windows")]
        {
            if success {
                ("../target/debug/test_plugin.dll", "test_plugin.dll")
            } else {
                ("../target/debug/test_plugin_fail.dll", "test_plugin.dll")
            }
        }
        #[cfg(target_os = "macos")]
        {
            if success {
                ("../target/debug/libtest_plugin.dylib", "test_plugin.dylib")
            } else {
                (
                    "../target/debug/libtest_plugin_fail.dylib",
                    "test_plugin.dylib",
                )
            }
        }
        #[cfg(target_os = "linux")]
        {
            if success {
                ("../target/debug/libtest_plugin.so", "test_plugin.so")
            } else {
                ("../target/debug/libtest_plugin_fail.so", "test_plugin.so")
            }
        }
    };
    println!("[DEBUG] Copying plugin from: {}", src);
    fs::create_dir_all(&plugins_dir).unwrap();
    fs::copy(src, plugins_dir.join(dest_name)).expect(&format!(
        "Failed to copy plugin from {} to {}",
        src,
        plugins_dir.join(dest_name).display()
    ));
    // Print plugins after copy for debug
    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        println!("[DEBUG] Plugins after setup:");
        for entry in entries.flatten() {
            println!("[DEBUG] - {:?}", entry.path());
        }
    }
}

fn cleanup_plugin_output() {
    let pre = workspace_path(".boltpm/plugins_output/preinstall");
    let post = workspace_path(".boltpm/plugins_output/postinstall");
    let _ = fs::remove_file(pre);
    let _ = fs::remove_file(post);
}

#[test]
#[ignore]
#[serial]
fn test_plugin_lifecycle_success() {
    clean_plugins_dir();
    cleanup_plugin_output();
    setup_test_plugin(true);
    // Print working directory and env for debug
    println!(
        "[TEST DEBUG] CWD: {}",
        env::current_dir().unwrap().display()
    );
    for (k, v) in env::vars() {
        println!("[TEST DEBUG] ENV {}={}", k, v);
    }
    fs::write(
        "package.json",
        r#"{"name":"boltpm-demo","version":"1.0.0"}"#,
    )
    .unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "boltpm", "--", "install"])
        .output()
        .expect("Failed to run boltpm install");
    assert!(
        output.status.success(),
        "CLI install failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let pre = fs::read_to_string(workspace_path(".boltpm/plugins_output/preinstall"))
        .expect("preinstall output missing");
    let post = fs::read_to_string(workspace_path(".boltpm/plugins_output/postinstall"))
        .expect("postinstall output missing");
    assert!(pre.contains("plugin ran for preinstall"));
    assert!(post.contains("plugin ran for postinstall"));
}

#[test]
#[ignore]
#[serial]
fn test_plugin_lifecycle_failure() {
    clean_plugins_dir();
    cleanup_plugin_output();
    setup_test_plugin(false);
    // Print working directory and env for debug
    println!(
        "[TEST DEBUG] CWD: {}",
        env::current_dir().unwrap().display()
    );
    for (k, v) in env::vars() {
        println!("[TEST DEBUG] ENV {}={}", k, v);
    }
    // List plugins directory contents for verification
    let plugins_dir = workspace_path(".boltpm/plugins");
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        println!("[TEST DEBUG] Plugins directory contents before CLI run:");
        for entry in entries.flatten() {
            println!("[TEST DEBUG] - {:?}", entry.path());
        }
    } else {
        println!(
            "[TEST DEBUG] Plugins directory does not exist: {}",
            plugins_dir.display()
        );
    }
    fs::write(
        "package.json",
        r#"{"name":"boltpm-demo","version":"1.0.0"}"#,
    )
    .unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "boltpm", "--", "install"])
        .output()
        .expect("Failed to run boltpm install");
    // Should fail due to plugin error
    assert!(
        !output.status.success(),
        "CLI install should fail due to plugin error"
    );
    let pre = fs::read_to_string(workspace_path(".boltpm/plugins_output/preinstall"));
    assert!(
        pre.is_err() || !pre.unwrap().contains("plugin ran for preinstall"),
        "preinstall output should not exist or be incomplete"
    );
}
