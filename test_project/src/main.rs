fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod e2e {
    use std::process::{Command, Stdio, Child};
    use std::{thread, time};
    use std::net::TcpListener;
    use std::io::{Read, Write};
    use std::fs::File;
    use tempfile::NamedTempFile;
    use std::{fs, path::Path};
    use std::path::PathBuf;

    fn port_in_use(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_err()
    }

    fn start_registry() -> Option<std::process::Child> {
        // Check if port 4000 is already in use
        if port_in_use(4000) {
            panic!("Port 4000 already in use — cannot launch registry.");
        }
        // Redirect registry output to a temp file
        let mut log_file = NamedTempFile::new().expect("Failed to create temp log file");
        let log_path = log_file.path().to_owned();
        let child = std::process::Command::new("cargo")
            .args(["run", "--bin", "boltpm-registry"])
            .current_dir("../registry")
            .stdout(File::create(&log_path).unwrap())
            .stderr(File::create(&log_path).unwrap())
            .spawn()
            .ok();
        // Poll the registry endpoint until it responds or timeout
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60);
        while start.elapsed() < timeout {
            if let Ok(resp) = reqwest::blocking::get("http://localhost:4000/v1/index") {
                if resp.status().is_success() { return child; }
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        // If failed, print captured logs
        let mut logs = String::new();
        File::open(&log_path).unwrap().read_to_string(&mut logs).ok();
        eprintln!("Registry did not start in time. Captured logs:\n{}", logs);
        child
    }

    fn copy_plugin_to_project() -> std::io::Result<()> {
        let plugin_src = Path::new("../target/debug/libsample_plugin.dylib");
        let plugin_dest_dir = Path::new("temp_project/.boltpm/plugins");
        let plugin_dest = plugin_dest_dir.join("libsample_plugin.dylib");
        fs::create_dir_all(plugin_dest_dir)?;
        fs::copy(plugin_src, plugin_dest)?;
        Ok(())
    }

    #[test]
    fn test_end_to_end() {
        // Start registry
        let mut registry = start_registry();
        // Publish a package using HTTP multipart
        let pkg_name = "e2etestpkg";
        let version = "1.0.0";
        let desc = "E2E test package";
        let tarball_bytes = b"test tarball contents";
        let form = reqwest::blocking::multipart::Form::new()
            .text("version", version)
            .text("description", desc)
            .part("tarball", reqwest::blocking::multipart::Part::bytes(tarball_bytes.to_vec()).file_name("package.tgz"));
        let publish_url = format!("http://localhost:4000/v1/{}/", pkg_name);
        let resp = reqwest::blocking::Client::new().put(&publish_url).multipart(form).send();
        assert!(resp.is_ok(), "Publish failed: {:?}", resp);
        // Yank the package
        let yank_url = format!("http://localhost:4000/v1/{}/{}/yank", pkg_name, version);
        let resp = reqwest::blocking::Client::new().post(&yank_url).send();
        assert!(resp.is_ok(), "Yank failed: {:?}", resp);
        // Unyank the package
        let unyank_url = format!("http://localhost:4000/v1/{}/{}/unyank", pkg_name, version);
        let resp = reqwest::blocking::Client::new().post(&unyank_url).send();
        assert!(resp.is_ok(), "Unyank failed: {:?}", resp);
        // Deprecate the package
        let deprecate_url = format!("http://localhost:4000/v1/{}/{}/deprecate", pkg_name, version);
        let body = serde_json::json!({ "message": "Deprecated for test" });
        let resp = reqwest::blocking::Client::new()
            .post(&deprecate_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&body).unwrap())
            .send();
        assert!(resp.is_ok(), "Deprecate failed: {:?}", resp);
        // Search for the package
        let search_url = format!("http://localhost:4000/v1/search?q={}", pkg_name);
        let resp = reqwest::blocking::get(&search_url);
        assert!(resp.is_ok(), "Search failed: {:?}", resp);
        let text = resp.unwrap().text().unwrap();
        println!("Search response: {}", text);
        assert!(text.contains(pkg_name), "Search did not return package");
        // Create temp_project/package.json before CLI install
        let temp_project_dir = "temp_project";
        let temp_package_json = format!("{}/package.json", temp_project_dir);
        std::fs::create_dir_all(temp_project_dir).expect("Failed to create temp_project dir");
        std::fs::write(&temp_package_json, r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#).expect("Failed to write temp package.json");
        // Copy plugin binary into temp_project/.boltpm/plugins
        copy_plugin_to_project().expect("Failed to copy plugin binary");
        // Install the package using CLI from temp_project
        let output = Command::new("cargo")
            .arg("run")
            .arg("--manifest-path")
            .arg("../../cli/Cargo.toml")
            .arg("--bin")
            .arg("boltpm")
            .arg("--")
            .arg("install")
            .arg(pkg_name)
            .current_dir(temp_project_dir)
            .output()
            .expect("Failed to run CLI install");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("[E2E] CLI install stdout:\n{}", stdout);
        println!("[E2E] CLI install stderr:\n{}", stderr);
        // Print directory listings for debugging
        fn print_dir_contents(path: &str) {
            println!("[E2E] Listing directory: {}", path);
            match std::fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            println!("  {}", entry.path().display());
                        }
                    }
                },
                Err(e) => println!("  [E2E] Failed to read dir: {}", e),
            }
        }
        print_dir_contents(temp_project_dir);
        print_dir_contents("../.boltpm");
        print_dir_contents("../.boltpm/plugins_output");
        // Check for plugin output file immediately after CLI install
        let output_file = PathBuf::from(temp_project_dir).join(".boltpm/plugins_output/PLUGIN_TEST");
        println!("[Test] Checking for plugin output at: {}", output_file.display());
        if output_file.exists() {
            println!("[Test] Plugin output file found!");
        } else {
            println!("[Test] Plugin output file NOT found.");
            let plugins_output_dir = PathBuf::from(temp_project_dir).join(".boltpm/plugins_output");
            if let Ok(entries) = std::fs::read_dir(&plugins_output_dir) {
                println!("[Test] Listing contents of plugins_output:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!(" - {}", entry.path().display());
                    }
                }
            } else {
                println!("[Test] plugins_output directory not found or inaccessible.");
            }
        }
        // Plugin verification: check for plugin output file
        let plugin_output = fs::read_to_string(output_file);
        assert!(plugin_output.is_ok(), "Plugin output file not found");
        assert!(plugin_output.unwrap().contains("plugin ran"), "Plugin did not run as expected");
        // Migration script verification
        let scripts = ["npm_to_boltpm.js", "yarn_to_boltpm.js", "pnpm_to_boltpm.js"];
        let workspace_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        for script in scripts.iter() {
            let script_path = format!("{}/../scripts/{}", workspace_root, script);
            let status = Command::new("node")
                .arg(&script_path)
                .current_dir("..")
                .status()
                .expect("Failed to run migration script");
            assert!(status.success(), "Migration script {} failed", script);
            let lock = fs::read_to_string("../bolt.lock");
            assert!(lock.is_ok(), "bolt.lock not created by {}", script);
            assert!(lock.unwrap().contains("bolt"), "bolt.lock missing expected content after {}", script);
        }
        // Clean up temp_project directory
        std::fs::remove_dir_all(temp_project_dir).ok();
        // Optionally, check for plugin execution (look for known output)
        // Clean up
        if let Some(mut child) = registry.take() {
            let _ = child.kill();
        }
    }
}
